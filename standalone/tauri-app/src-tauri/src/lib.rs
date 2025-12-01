use serde::{Deserialize, Serialize, Deserializer};
use serde_json::Value;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc, FixedOffset};
use std::fs;
use std::path::PathBuf;
use base64::{Engine as _, engine::general_purpose};
use tracing::{info, warn, error, debug};

// Unicode 解码函数：处理 JSON 中的 Unicode 转义序列
fn decode_unicode_escapes(input: &str) -> String {
    // 使用正则表达式处理 Unicode 转义序列
    use regex::Regex;
    
    let re = match Regex::new(r"\\u([0-9a-fA-F]{4})") {
        Ok(regex) => regex,
        Err(e) => {
            error!("创建Unicode解码正则表达式失败: {}", e);
            return input.to_string();
        }
    };
    
    let result = re.replace_all(input, |caps: &regex::Captures| {
        let hex_str = &caps[1];
        if let Ok(code_point) = u32::from_str_radix(hex_str, 16) {
            if let Some(unicode_char) = std::char::from_u32(code_point) {
                unicode_char.to_string()
            } else {
                caps[0].to_string() // 保留原始字符
            }
        } else {
            caps[0].to_string() // 保留原始字符
        }
    });
    
    result.to_string()
}

// 自定义反序列化函数来处理status字段
fn deserialize_status<'de, D>(deserializer: D) -> Result<BugStatus, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(BugStatus {
            code: s.clone(),
            name: s,
        }),
        Value::Object(obj) => {
            let code = obj.get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let name = obj.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&code)
                .to_string();
            Ok(BugStatus { code, name })
        }
        _ => Ok(BugStatus {
            code: "unknown".to_string(),
            name: "未知".to_string(),
        }),
    }
}

// 时间格式化函数：将ISO 8601时间转换为中国时间显示
fn format_datetime_to_china(datetime_str: &str) -> String {
    if datetime_str.is_empty() {
        return "未知".to_string();
    }
    
    match DateTime::parse_from_rfc3339(datetime_str) {
        Ok(dt) => {
            // 转换为中国时区 (UTC+8)
            let china_offset = FixedOffset::east_opt(8 * 3600).unwrap();
            let china_time = dt.with_timezone(&china_offset);
            let formatted = china_time.format("%Y-%m-%d %H:%M:%S").to_string();
            debug!("时间格式化: {} -> {}", datetime_str, formatted);
            formatted
        }
        Err(_) => {
            // 如果解析失败，尝试直接解析UTC时间
            match datetime_str.parse::<DateTime<Utc>>() {
                Ok(utc_dt) => {
                    let china_offset = FixedOffset::east_opt(8 * 3600).unwrap();
                    let china_time = utc_dt.with_timezone(&china_offset);
                    let formatted = china_time.format("%Y-%m-%d %H:%M:%S").to_string();
                    debug!("时间格式化(UTC): {} -> {}", datetime_str, formatted);
                    formatted
                }
                Err(e) => {
                    error!("时间解析失败: {} - {}", datetime_str, e);
                    datetime_str.to_string() // 如果都失败，返回原字符串
                }
            }
        }
    }
}

// 处理HTML中的图片路径，提取图片路径信息以便前端按需加载
fn process_html_images(html_content: &str, _base_url: &str) -> String {
    use regex::Regex;
    
    debug!("开始处理HTML图片，内容长度: {}", html_content.len());
    debug!("HTML内容预览: {}", if html_content.len() > 200 { &html_content[..200] } else { html_content });
    
    // 创建正则表达式来匹配img标签，处理HTML实体编码
    // 匹配格式：<img ... alt="index.php?m=file&amp;f=read&amp;t=ext&amp;fileID=id" ... />
    let re = match Regex::new(r#"<img\s+[^>]*alt="([^"]*index\.php\?m=file[^"]*)"[^>]*/?>"#) {
        Ok(regex) => regex,
        Err(e) => {
            error!("创建正则表达式失败: {}", e);
            return html_content.to_string();
        }
    };
    
    let result = re.replace_all(html_content, |caps: &regex::Captures| {
        let original_tag = &caps[0];
        let alt_content = &caps[1];
        
        debug!("找到图片标签: {}", original_tag);
        debug!("Alt内容: {}", alt_content);
        
        // HTML实体解码：将&amp;转换为&
        let decoded_alt = alt_content.replace("&amp;", "&");
        
        // 使用正则表达式添加data-image-path属性
        let attr_re = match Regex::new(r#"<img(\s+[^>]*?)/?>"#) {
            Ok(regex) => regex,
            Err(_) => return original_tag.to_string(),
        };
        
        let updated_tag = attr_re.replace(original_tag, |img_caps: &regex::Captures| {
            let attributes = &img_caps[1];
            format!(r#"<img{} data-image-path="{}" data-lazy-load="true" />"#, attributes, decoded_alt)
        });
        
        debug!("图片标记添加: {}", decoded_alt);
        updated_tag.to_string()
    });
    
    if result != html_content {
        info!("HTML处理完成，发现并处理了图片");
    } else {
        debug!("HTML处理完成，未发现图片");
    }
    
    result.to_string()
}

// 全局状态管理
struct AppState {
    token: Option<String>,
    token_expire_time: u64,
    config: Option<ZentaoConfig>,
}

static APP_STATE: Mutex<AppState> = Mutex::new(AppState {
    token: None,
    token_expire_time: 0,
    config: None,
});

// Token持久化数据结构
#[derive(Debug, Serialize, Deserialize)]
struct TokenData {
    token: String,
    expire_time: u64,
    encrypted: bool,
}

// 简单的XOR加密/解密函数
fn xor_encrypt_decrypt(data: &str, key: &str) -> String {
    let key_bytes = key.as_bytes();
    let data_bytes = data.as_bytes();
    let key_len = key_bytes.len();
    
    let encrypted: Vec<u8> = data_bytes
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ key_bytes[i % key_len])
        .collect();
    
    general_purpose::STANDARD.encode(&encrypted)
}

fn xor_decrypt(encrypted_data: &str, key: &str) -> Result<String, String> {
    match general_purpose::STANDARD.decode(encrypted_data) {
        Ok(decoded) => {
            let key_bytes = key.as_bytes();
            let key_len = key_bytes.len();
            
            let decrypted: Vec<u8> = decoded
                .iter()
                .enumerate()
                .map(|(i, &b)| b ^ key_bytes[i % key_len])
                .collect();
            
            match String::from_utf8(decrypted) {
                Ok(s) => Ok(s),
                Err(e) => Err(format!("UTF8解码失败: {}", e)),
            }
        }
        Err(e) => Err(format!("Base64解码失败: {}", e)),
    }
}

// 获取token存储文件路径
fn get_token_file_path() -> Result<PathBuf, String> {
    match dirs::data_dir() {
        Some(mut path) => {
            path.push("zentao_app");
            path.push("zentao_token.json");
            Ok(path)
        }
        None => Err("无法获取用户数据目录".to_string()),
    }
}

// 获取配置存储文件路径
fn get_config_file_path() -> Result<PathBuf, String> {
    match dirs::data_dir() {
        Some(mut path) => {
            path.push("zentao_app");
            path.push("zentao_config.json");
            Ok(path)
        }
        None => Err("无法获取用户数据目录".to_string()),
    }
}

// 保存token到本地文件
fn save_token_to_file(token: &str, expire_time: u64) -> Result<(), String> {
    let file_path = get_token_file_path()?;
    
    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("创建目录失败: {}", e));
        }
    }
    
    // 使用简单的加密密钥（实际项目中应该使用更安全的方式）
    let encryption_key = "zentao_app_key_2025";
    let encrypted_token = xor_encrypt_decrypt(token, encryption_key);
    
    let token_data = TokenData {
        token: encrypted_token,
        expire_time,
        encrypted: true,
    };
    
    match serde_json::to_string_pretty(&token_data) {
        Ok(json_str) => {
            match fs::write(&file_path, json_str) {
                Ok(_) => {
                    debug!("Token已保存到: {:?}", file_path);
                    Ok(())
                }
                Err(e) => Err(format!("写入文件失败: {}", e)),
            }
        }
        Err(e) => Err(format!("序列化Token数据失败: {}", e)),
    }
}

// 从本地文件加载token
fn load_token_from_file() -> Result<Option<(String, u64)>, String> {
    let file_path = get_token_file_path()?;
    
    if !file_path.exists() {
        debug!("Token文件不存在: {:?}", file_path);
        return Ok(None);
    }
    
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            match serde_json::from_str::<TokenData>(&content) {
                Ok(token_data) => {
                    let current_time = get_current_timestamp();
                    
                    // 检查token是否过期
                    if current_time >= token_data.expire_time {
                        warn!("保存的token已过期");
                        // 删除过期的token文件
                        let _ = fs::remove_file(&file_path);
                        return Ok(None);
                    }
                    
                    // 解密token
                    if token_data.encrypted {
                        let encryption_key = "zentao_app_key_2025";
                        match xor_decrypt(&token_data.token, encryption_key) {
                            Ok(decrypted_token) => {
                                debug!("成功加载并解密token，剩余时间: {}秒", token_data.expire_time - current_time);
                                Ok(Some((decrypted_token, token_data.expire_time)))
                            }
                            Err(e) => {
                                error!("解密token失败: {}", e);
                                // 删除损坏的token文件
                                let _ = fs::remove_file(&file_path);
                                Ok(None)
                            }
                        }
                    } else {
                        // 兼容未加密的旧格式
                        Ok(Some((token_data.token, token_data.expire_time)))
                    }
                }
                Err(e) => {
                    error!("解析token文件失败: {}", e);
                    // 删除损坏的token文件
                    let _ = fs::remove_file(&file_path);
                    Ok(None)
                }
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(format!("读取token文件失败: {}", e))
            }
        }
    }
}

// 删除保存的token文件
fn remove_token_file() -> Result<(), String> {
    let file_path = get_token_file_path()?;
    
    if file_path.exists() {
        match fs::remove_file(&file_path) {
            Ok(_) => {
                debug!("Token文件已删除: {:?}", file_path);
                Ok(())
            }
            Err(e) => Err(format!("删除token文件失败: {}", e)),
        }
    } else {
        Ok(())
    }
}

// 初始化应用状态（从本地文件加载token和配置）
fn initialize_app_state() -> Result<(), String> {
    let mut state = APP_STATE.lock().unwrap();
    
    // 加载token
    match load_token_from_file() {
        Ok(Some((token, expire_time))) => {
            state.token = Some(token);
            state.token_expire_time = expire_time;
            info!("应用启动时成功加载保存的token");
        }
        Ok(None) => {
            debug!("应用启动时没有找到有效的保存token");
        }
        Err(e) => {
            warn!("加载保存的token失败: {}", e);
        }
    }
    
    // 加载配置
    match load_config_from_file() {
        Ok(Some(config)) => {
            state.config = Some(config);
            info!("应用启动时成功加载保存的配置");
        }
        Ok(None) => {
            debug!("应用启动时没有找到保存的配置");
        }
        Err(e) => {
            warn!("加载保存的配置失败: {}", e);
        }
    }
    
    Ok(())
}

// 保存配置到本地文件
fn save_config_to_file(config: &ZentaoConfig) -> Result<(), String> {
    let file_path = get_config_file_path()?;
    
    // 确保目录存在
    if let Some(parent) = file_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return Err(format!("创建目录失败: {}", e));
        }
    }
    
    match serde_json::to_string_pretty(config) {
        Ok(json_str) => {
            match fs::write(&file_path, json_str) {
                Ok(_) => {
                    debug!("配置已保存到: {:?}", file_path);
                    Ok(())
                }
                Err(e) => Err(format!("写入配置文件失败: {}", e)),
            }
        }
        Err(e) => Err(format!("序列化配置数据失败: {}", e)),
    }
}

// 从本地文件加载配置
fn load_config_from_file() -> Result<Option<ZentaoConfig>, String> {
    let file_path = get_config_file_path()?;
    
    if !file_path.exists() {
        debug!("配置文件不存在: {:?}", file_path);
        return Ok(None);
    }
    
    match fs::read_to_string(&file_path) {
        Ok(content) => {
            match serde_json::from_str::<ZentaoConfig>(&content) {
                Ok(config) => {
                    debug!("成功加载配置文件: {:?}", file_path);
                    Ok(Some(config))
                }
                Err(e) => {
                    error!("解析配置文件失败: {}", e);
                    // 删除损坏的配置文件
                    let _ = fs::remove_file(&file_path);
                    Ok(None)
                }
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                Ok(None)
            } else {
                Err(format!("读取配置文件失败: {}", e))
            }
        }
    }
}

// 定义禅道API的数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ZentaoConfig {
    #[serde(rename = "baseUrl")]
    base_url: String,
    account: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    account: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BugStatus {
    code: String,
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct Bug {
    id: i32,
    product: i32,
    branch: i32,
    module: i32,
    project: i32,
    execution: i32,
    #[serde(rename = "toTask")]
    to_task: i32,
    #[serde(rename = "toStory")]
    to_story: i32,
    title: String,
    keywords: String,
    severity: i32,
    pri: i32,
    #[serde(rename = "type")]
    bug_type: String,
    #[serde(default)]
    os: String,
    #[serde(default)]
    browser: String,
    steps: String,
    #[serde(default)]
    task: Option<i32>,
    #[serde(default)]
    story: Option<i32>,
    #[serde(rename = "openedBy")]
    opened_by: Option<User>,
    #[serde(rename = "openedDate")]
    opened_date: String,
    #[serde(default)]
    deadline: Option<String>,
    #[serde(rename = "assignedTo")]
    assigned_to: Option<User>,
    #[serde(rename = "assignedDate")]
    #[serde(default)]
    assigned_date: Option<String>,
    #[serde(rename = "resolvedBy")]
    #[serde(default)]
    resolved_by: Option<User>,
    #[serde(rename = "resolvedDate")]
    #[serde(default)]
    resolved_date: Option<String>,
    #[serde(rename = "closedBy")]
    #[serde(default)]
    closed_by: Option<User>,
    #[serde(rename = "closedDate")]
    #[serde(default)]
    closed_date: Option<String>,
    #[serde(deserialize_with = "deserialize_status")]
    status: BugStatus,
    // 用于存储基础URL，不参与反序列化，在获取Bug时手动设置
    #[serde(skip)]
    pub base_url: Option<String>,
}

// 为Bug实现自定义序列化，添加格式化后的时间字段
impl Serialize for Bug {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Bug", 24)?;
        
        state.serialize_field("id", &self.id)?;
        state.serialize_field("product", &self.product)?;
        state.serialize_field("branch", &self.branch)?;
        state.serialize_field("module", &self.module)?;
        state.serialize_field("project", &self.project)?;
        state.serialize_field("execution", &self.execution)?;
        state.serialize_field("toTask", &self.to_task)?;
        state.serialize_field("toStory", &self.to_story)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("keywords", &self.keywords)?;
        state.serialize_field("severity", &self.severity)?;
        state.serialize_field("pri", &self.pri)?;
        state.serialize_field("type", &self.bug_type)?;
        state.serialize_field("os", &self.os)?;
        state.serialize_field("browser", &self.browser)?;
        state.serialize_field("steps", &self.steps)?;
        // 添加处理过图片路径的steps字段
        let processed_steps = if let Some(ref base_url) = self.base_url {
            process_html_images(&self.steps, base_url)
        } else {
            self.steps.clone()
        };
        state.serialize_field("stepsProcessed", &processed_steps)?;
        state.serialize_field("task", &self.task)?;
        state.serialize_field("story", &self.story)?;
        state.serialize_field("openedBy", &self.opened_by)?;
        state.serialize_field("openedDate", &self.opened_date)?;
        state.serialize_field("openedDateFormatted", &format_datetime_to_china(&self.opened_date))?;
        state.serialize_field("deadline", &self.deadline)?;
        state.serialize_field("assignedTo", &self.assigned_to)?;
        state.serialize_field("assignedDate", &self.assigned_date)?;
        state.serialize_field("assignedDateFormatted", &self.assigned_date.as_ref().map(|d| format_datetime_to_china(d)))?;
        state.serialize_field("resolvedBy", &self.resolved_by)?;
        state.serialize_field("resolvedDate", &self.resolved_date)?;
        state.serialize_field("resolvedDateFormatted", &self.resolved_date.as_ref().map(|d| format_datetime_to_china(d)))?;
        state.serialize_field("closedBy", &self.closed_by)?;
        state.serialize_field("closedDate", &self.closed_date)?;
        state.serialize_field("closedDateFormatted", &self.closed_date.as_ref().map(|d| format_datetime_to_china(d)))?;
        state.serialize_field("status", &self.status)?;
        
        state.end()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BugListResponse {
    page: i32,
    total: i32,
    limit: i32,
    bugs: Vec<Bug>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    id: i32,
    #[serde(rename = "type")]
    user_type: String,
    dept: i32,
    account: String,
    realname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    birthday: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mobile: Option<String>,  
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weixin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    join: Option<String>,
    admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    id: i32,
    account: String,
    avatar: String,
    realname: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    id: i32,
    program: i32,
    name: String,
    code: String,
    line: i32,
    #[serde(rename = "PO")]
    #[serde(skip_serializing_if = "Option::is_none")]
    po: Option<User>,
    #[serde(rename = "QD")]
    #[serde(skip_serializing_if = "Option::is_none")]
    qd: Option<User>,
    #[serde(rename = "RD")]
    #[serde(skip_serializing_if = "Option::is_none")]
    rd: Option<User>,
    #[serde(rename = "type")]
    product_type: String, // normal | branch | platform
    desc: String,
    acl: String, // open | private
    #[serde(skip_serializing_if = "Option::is_none")]
    whitelist: Option<Vec<User>>,
    #[serde(rename = "createdBy")]
    created_by: Option<User>,
    #[serde(rename = "createdDate")]
    created_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductListResponse {
    page: i32,
    total: i32,
    limit: i32,
    products: Vec<Product>,
}

// 获取当前时间戳
fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// 设置 tracing - 同时输出到控制台和文件
fn setup_tracing() {
    use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
    use tracing_appender::rolling;
    
    // 获取日志目录
    let log_dir = match dirs::data_dir() {
        Some(mut path) => {
            path.push("zentao_app");
            path.push("logs");
            if let Err(_) = fs::create_dir_all(&path) {
                eprintln!("创建日志目录失败，将只输出到控制台");
                setup_simple_tracing();
                return;
            }
            path
        }
        None => {
            eprintln!("无法获取用户数据目录，将只输出到控制台");
            setup_simple_tracing();
            return;
        }
    };
    
    // 创建文件输出器 - 每天轮转
    let file_appender = rolling::daily(&log_dir, "app.log");
    
    // 环境过滤器
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "debug".into());
    
    // 组合订阅器 - 控制台和文件分别配置
    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .with_ansi(true) // 控制台支持颜色
        )
        .with(
            fmt::Layer::new()
                .with_writer(file_appender)
                .with_ansi(false) // 文件不需要颜色代码
        )
        .init();
    
    println!("Tracing 初始化成功，日志文件目录: {:?}", log_dir);
}

// 简单 tracing 设置（仅控制台）
fn setup_simple_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into())
        )
        .init();
}

// 检查token是否有效
fn is_token_valid() -> bool {
    let state = APP_STATE.lock().unwrap();
    state.token.is_some() && get_current_timestamp() < state.token_expire_time
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 保存配置
#[tauri::command]
fn save_config(config: ZentaoConfig) -> Result<String, String> {
    debug!("保存配置: {:?}", config);
    
    {
        let mut state = APP_STATE.lock().unwrap();
        state.config = Some(config.clone());
    }
    
    // 将配置保存到文件
    save_config_to_file(&config)?;
    
    Ok("配置保存成功".to_string())
}

// 加载配置
#[tauri::command]
fn load_config() -> Result<Option<ZentaoConfig>, String> {
    // 首先尝试从文件加载
    match load_config_from_file()? {
        Some(config) => {
            // 同时更新内存中的状态
            let mut state = APP_STATE.lock().unwrap();
            state.config = Some(config.clone());
            Ok(Some(config))
        }
        None => {
            // 如果文件中没有，再尝试从内存中读取
            let state = APP_STATE.lock().unwrap();
            Ok(state.config.clone())
        }
    }
}

// 初始化应用（加载保存的token）
#[tauri::command]
fn initialize_app() -> Result<bool, String> {
    match initialize_app_state() {
        Ok(_) => Ok(is_token_valid()),
        Err(e) => Err(e),
    }
}

// 检查登录状态
#[tauri::command]
fn check_login_status() -> Result<bool, String> {
    Ok(is_token_valid())
}

// 登录到禅道系统
#[tauri::command]
async fn login_zentao(base_url: String, account: String, password: String) -> Result<bool, String> {
    // 如果已有有效token，直接返回成功
    if is_token_valid() {
        return Ok(true);
    }

    // 自动拼接API路径
    let api_url = if base_url.ends_with("/api.php/v1") {
        base_url
    } else {
        format!("{}/api.php/v1", base_url.trim_end_matches('/'))
    };

    let client = reqwest::Client::new();
    let login_url = format!("{}/tokens", api_url);
    
    let login_request = LoginRequest {
        account,
        password,
    };
    
    info!("尝试登录到: {}", login_url);
    
    match client
        .post(&login_url)
        .header("Content-Type", "application/json")
        .json(&login_request)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            debug!("登录响应状态: {}", status);
            
            if status.is_success() {
                match response.text().await {
                    Ok(text) => {
                        debug!("登录响应: 成功获取token (长度: {})", text.len());
                        
                        // 解码 Unicode 转义序列
                        let decoded_text = decode_unicode_escapes(&text);
                        
                        match serde_json::from_str::<Value>(&decoded_text) {
                            Ok(json) => {
                                if let Some(token) = json.get("token") {
                                    if let Some(token_str) = token.as_str() {
                                        // 保存token和过期时间（24小时）
                                        let expire_time = get_current_timestamp() + (24 * 60 * 60);
                                        let mut state = APP_STATE.lock().unwrap();
                                        state.token = Some(token_str.to_string());
                                        state.token_expire_time = expire_time;
                                        
                                        // 保存token到本地文件
                                        if let Err(e) = save_token_to_file(token_str, expire_time) {
                                            warn!("保存token到文件失败: {}", e);
                                            // 即使保存失败，登录仍然成功，只是下次需要重新登录
                                        }
                                        
                                        Ok(true)
                                    } else {
                                        Err("token字段不是字符串类型".to_string())
                                    }
                                } else {
                                    Err(format!("响应中没有找到token字段: {}", text))
                                }
                            }
                            Err(e) => Err(format!("解析JSON失败: {} - 响应内容: {}", e, text)),
                        }
                    }
                    Err(e) => Err(format!("读取响应内容失败: {}", e)),
                }
            } else {
                match response.text().await {
                    Ok(error_text) => {
                        // 解码错误信息中的 Unicode 转义序列
                        let decoded_error = decode_unicode_escapes(&error_text);
                        Err(format!("登录失败，状态码: {} - 错误信息: {}", status, decoded_error))
                    },
                    Err(_) => Err(format!("登录失败，状态码: {}", status)),
                }
            }
        }
        Err(e) => Err(format!("请求失败: {}", e)),
    }
}

// 退出登录
#[tauri::command]
fn logout_zentao() -> Result<(), String> {
    let mut state = APP_STATE.lock().unwrap();
    state.token = None;
    state.token_expire_time = 0;
    
    // 删除保存的token文件
    if let Err(e) = remove_token_file() {
        warn!("删除token文件失败: {}", e);
        // 即使删除失败，登出仍然成功
    }
    
    Ok(())
}

// 获取用户信息
#[tauri::command]
async fn get_user_info() -> Result<UserInfo, String> {
    // 检查token是否有效
    if !is_token_valid() {
        return Err("Token expired".to_string());
    }

    let (token, base_url) = {
        let state = APP_STATE.lock().unwrap();
        let token = state.token.clone().unwrap();
        let config = state.config.as_ref().ok_or("配置未找到")?;
        (token, config.base_url.clone())
    };

    // 自动拼接API路径
    let api_url = if base_url.ends_with("/api.php/v1") {
        base_url
    } else {
        format!("{}/api.php/v1", base_url.trim_end_matches('/'))
    };

    let client = reqwest::Client::new();
    let user_url = format!("{}/user", api_url);
    
    debug!("获取用户信息: {}", user_url);
    
    match client
        .get(&user_url)
        .header("Token", &token)
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            debug!("用户信息响应状态: {}", status);
            
            if status.is_success() {
                match response.text().await {
                    Ok(text) => {
                        debug!("用户信息响应: 成功获取 (长度: {})", text.len());
                        
                        // 解码 Unicode 转义序列
                        let decoded_text = decode_unicode_escapes(&text);
                        
                        match serde_json::from_str::<Value>(&decoded_text) {
                            Ok(json) => {
                                // 根据实际API响应结构解析用户信息
                                // 禅道API返回的是 { "profile": {...} } 格式
                                if let Some(profile) = json.get("profile") {
                                    // 尝试解析为UserInfo结构
                                    match serde_json::from_value::<UserInfo>(profile.clone()) {
                                        Ok(user_info) => Ok(user_info),
                                        Err(e) => {
                                            let preview = if text.len() > 100 {
                                                format!("{}...", &text[..100])
                                            } else {
                                                text.clone()
                                            };
                                            Err(format!("解析UserInfo失败: {} - 响应预览: {}", e, preview))
                                        }
                                    }
                                } else {
                                    let preview = if text.len() > 100 {
                                        format!("{}...", &text[..100])
                                    } else {
                                        text.clone()
                                    };
                                    Err(format!("响应中没有找到profile字段 - 响应预览: {}", preview))
                                }
                            }
                            Err(e) => Err(format!("解析用户信息JSON失败: {} - 响应内容: {}", e, text)),
                        }
                    }
                    Err(e) => Err(format!("读取用户信息响应失败: {}", e)),
                }
            } else {
                // 如果是401错误，说明token过期
                if status == 401 {
                    let mut state = APP_STATE.lock().unwrap();
                    state.token = None;
                    state.token_expire_time = 0;
                    return Err("Unauthorized".to_string());
                }
                
                match response.text().await {
                    Ok(error_text) => {
                        // 解码错误信息中的 Unicode 转义序列
                        let decoded_error = decode_unicode_escapes(&error_text);
                        Err(format!("获取用户信息失败，状态码: {} - 错误信息: {}", status, decoded_error))
                    },
                    Err(_) => Err(format!("获取用户信息失败，状态码: {}", status)),
                }
            }
        }
        Err(e) => Err(format!("请求失败: {}", e)),
    }
}

// 根据产品ID获取Bug列表
#[tauri::command]
async fn get_bugs_by_product(product_ids: Vec<i32>) -> Result<Vec<Bug>, String> {
    // 检查token是否有效
    if !is_token_valid() {
        return Err("Token expired".to_string());
    }

    let (token, base_url) = {
        let state = APP_STATE.lock().unwrap();
        let token = state.token.clone().unwrap();
        let config = state.config.as_ref().ok_or("配置未找到")?;
        (token, config.base_url.clone())
    };

    // 自动拼接API路径
    let api_url = if base_url.ends_with("/api.php/v1") {
        base_url.clone()
    } else {
        format!("{}/api.php/v1", base_url.trim_end_matches('/'))
    };

    let client = reqwest::Client::new();
    
    // 合并多个产品的Bug列表
    let mut all_bugs = Vec::new();
    
    for product_id in product_ids {
        let bugs_url = format!("{}/products/{}/bugs", api_url, product_id);
        
        debug!("获取产品{}的Bug列表: {}", product_id, bugs_url);
        
        match client
            .get(&bugs_url)
            .header("Token", &token)
            .header("Content-Type", "application/json")
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                debug!("产品{}Bug列表响应状态: {}", product_id, status);
                
                if status.is_success() {
                    match response.text().await {
                        Ok(text) => {
                            let preview = if text.len() > 200 {
                                format!("{}...", &text[..200])
                            } else {
                                text.clone()
                            };
                            debug!("产品{}Bug列表响应 (长度: {}): {}", product_id, text.len(), preview);
                            
                            // 解码 Unicode 转义序列
                            let decoded_text = decode_unicode_escapes(&text);
                            
                            match serde_json::from_str::<BugListResponse>(&decoded_text) {
                                Ok(mut bug_response) => {
                                    debug!("产品{}解析到{}个Bug", product_id, bug_response.bugs.len());
                                    // 调试第一个Bug的数据
                                    if let Some(first_bug) = bug_response.bugs.first() {
                                        debug!("第一个Bug数据: id={}, opened_date={}", first_bug.id, first_bug.opened_date);
                                    }
                                    // 为每个Bug设置base_url
                                    for bug in &mut bug_response.bugs {
                                        bug.base_url = Some(api_url.clone());
                                    }
                                    all_bugs.extend(bug_response.bugs);
                                }
                                Err(e) => {
                                    let preview = if text.len() > 100 {
                                        format!("{}...", &text[..100])
                                    } else {
                                        text.clone()
                                    };
                                    error!("解析产品{}Bug列表JSON失败: {} - 响应预览: {}", product_id, e, preview);
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            error!("读取产品{}Bug列表响应失败: {}", product_id, e);
                            continue;
                        }
                    }
                } else if status == 401 {
                    let mut state = APP_STATE.lock().unwrap();
                    state.token = None;
                    state.token_expire_time = 0;
                    return Err("Unauthorized".to_string());
                } else {
                    warn!("获取产品{}Bug列表失败，状态码: {}", product_id, status);
                    continue;
                }
            }
            Err(e) => {
                error!("请求产品{}Bug列表失败: {}", product_id, e);
                continue;
            }
        }
    }
    
    Ok(all_bugs)
}

// 获取Bug详情
#[tauri::command]
async fn get_bug_detail(bug_id: i32) -> Result<Bug, String> {
    info!("开始获取Bug详情 - ID: {}", bug_id);
    
    // 检查token是否有效
    if !is_token_valid() {
        warn!("Token无效或已过期，无法获取Bug详情");
        return Err("Token expired".to_string());
    }

    let (token, base_url) = {
        let state = APP_STATE.lock().unwrap();
        let token = state.token.clone().unwrap();
        let config = state.config.as_ref().ok_or("配置未找到")?;
        (token, config.base_url.clone())
    };

    // 自动拼接API路径
    let api_url = if base_url.ends_with("/api.php/v1") {
        base_url.clone()
    } else {
        format!("{}/api.php/v1", base_url.trim_end_matches('/'))
    };

    let client = reqwest::Client::new();
    let bug_url = format!("{}/bugs/{}", api_url, bug_id);
    
    debug!("发送请求获取Bug详情: {}", bug_url);
    
    match client
        .get(&bug_url)
        .header("Token", &token)
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            debug!("Bug详情响应状态: {}", status);
            
            if status.is_success() {
                match response.text().await {
                    Ok(text) => {
                        // 只打印响应长度和前100个字符，避免控制台输出过长
                        let preview= text.clone();
                        debug!("Bug详情响应 (长度: {}): {}", text.len(), preview);
                        
                        // 解码 Unicode 转义序列
                        let decoded_text = decode_unicode_escapes(&text);
                        
                        match serde_json::from_str::<Bug>(&decoded_text) {
                            Ok(mut bug) => {
                                // 设置base_url用于图片路径处理
                                bug.base_url = Some(api_url.clone());
                                info!("成功解析Bug详情，ID: {}", bug.id);
                                Ok(bug)
                            }
                            Err(e) => {
                                let preview = if text.len() > 100 {
                                    format!("{}...", &text[..100])
                                } else {
                                    text.clone()
                                };
                                error!("解析Bug详情JSON失败: {} - 响应预览: {}", e, preview);
                                Err(format!("解析Bug详情JSON失败: {} - 响应预览: {}", e, preview))
                            }
                        }
                    }
                    Err(e) => {
                        error!("读取Bug详情响应失败: {}", e);
                        Err(format!("读取Bug详情响应失败: {}", e))
                    }
                }
            } else if status == 401 {
                let mut state = APP_STATE.lock().unwrap();
                state.token = None;
                state.token_expire_time = 0;
                Err("Unauthorized".to_string())
            } else {
                match response.text().await {
                    Ok(error_text) => {
                        // 解码错误信息中的 Unicode 转义序列
                        let decoded_error = decode_unicode_escapes(&error_text);
                        Err(format!("获取Bug详情失败，状态码: {} - 错误信息: {}", status, decoded_error))
                    },
                    Err(_) => Err(format!("获取Bug详情失败，状态码: {}", status)),
                }
            }
        }
        Err(e) => {
            error!("请求Bug详情失败: {}", e);
            Err(format!("请求失败: {}", e))
        }
    }
}

// 获取产品列表
#[tauri::command]
async fn get_products() -> Result<Vec<Product>, String> {
    // 检查token是否有效
    if !is_token_valid() {
        return Err("Token expired".to_string());
    }

    let (token, base_url) = {
        let state = APP_STATE.lock().unwrap();
        let token = state.token.clone().unwrap();
        let config = state.config.as_ref().ok_or("配置未找到")?;
        (token, config.base_url.clone())
    };

    // 自动拼接API路径
    let api_url = if base_url.ends_with("/api.php/v1") {
        base_url
    } else {
        format!("{}/api.php/v1", base_url.trim_end_matches('/'))
    };

    let client = reqwest::Client::new();
    let products_url = format!("{}/products", api_url);
    
    debug!("获取产品列表: {}", products_url);
    
    match client
        .get(&products_url)
        .header("Token", &token)
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            debug!("产品列表响应状态: {}", status);
            
            if status.is_success() {
                match response.text().await {
                    Ok(text) => {
                        let preview = if text.len() > 300 {
                            format!("{}...", &text[..300])
                        } else {
                            text.clone()
                        };
                        debug!("产品列表响应 (长度: {}): {}", text.len(), preview);
                        
                        // 解码 Unicode 转义序列
                        let decoded_text = decode_unicode_escapes(&text);
                        
                        match serde_json::from_str::<Value>(&decoded_text) {
                            Ok(json) => {
                                // 检查是否是分页响应格式 {"page":1,"total":2,"limit":100,"products":[]}
                                if let (Some(page), Some(total), Some(limit), Some(products)) = (
                                    json.get("page"),
                                    json.get("total"), 
                                    json.get("limit"),
                                    json.get("products")
                                ) {
                                    debug!("分页响应 - 页码: {}, 总数: {}, 限制: {}", page, total, limit);
                                    // 解析products数组
                                    match serde_json::from_value::<Vec<Product>>(products.clone()) {
                                        Ok(product_list) => {
                                            debug!("成功解析 {} 个产品", product_list.len());
                                            Ok(product_list)
                                        },
                                        Err(e) => {
                                            let preview = if text.len() > 100 {
                                                format!("{}...", &text[..100])
                                            } else {
                                                text.clone()
                                            };
                                            Err(format!("解析产品列表失败: {} - 响应预览: {}", e, preview))
                                        }
                                    }
                                } else if let Some(products) = json.get("products") {
                                    // 兼容旧格式，直接有products字段但没有分页信息
                                    match serde_json::from_value::<Vec<Product>>(products.clone()) {
                                        Ok(product_list) => Ok(product_list),
                                        Err(e) => Err(format!("解析产品列表失败: {} - 响应内容: {}", e, text)),
                                    }
                                } else if json.is_array() {
                                    // 如果直接是数组格式
                                    match serde_json::from_value::<Vec<Product>>(json) {
                                        Ok(product_list) => Ok(product_list),
                                        Err(e) => Err(format!("解析产品数组失败: {} - 响应内容: {}", e, text)),
                                    }
                                } else {
                                    let preview = if text.len() > 100 {
                                        format!("{}...", &text[..100])
                                    } else {
                                        text.clone()
                                    };
                                    Err(format!("响应格式不正确，期望分页格式 - 响应预览: {}", preview))
                                }
                            }
                            Err(e) => Err(format!("解析产品列表JSON失败: {} - 响应内容: {}", e, text)),
                        }
                    }
                    Err(e) => Err(format!("读取产品列表响应失败: {}", e)),
                }
            } else {
                // 如果是401错误，说明token过期
                if status == 401 {
                    let mut state = APP_STATE.lock().unwrap();
                    state.token = None;
                    state.token_expire_time = 0;
                    return Err("Unauthorized".to_string());
                }
                
                match response.text().await {
                    Ok(error_text) => {
                        // 解码错误信息中的 Unicode 转义序列
                        let decoded_error = decode_unicode_escapes(&error_text);
                        Err(format!("获取产品列表失败，状态码: {} - 错误信息: {}", status, decoded_error))
                    },
                    Err(_) => Err(format!("获取产品列表失败，状态码: {}", status)),
                }
            }
        }
        Err(e) => Err(format!("请求失败: {}", e)),
    }
}

// 获取图片数据（Base64编码）
#[tauri::command]
async fn get_image(image_path: String) -> Result<String, String> {
    // 检查token是否有效
    if !is_token_valid() {
        return Err("Token expired".to_string());
    }

    let (token, base_url) = {
        let state = APP_STATE.lock().unwrap();
        let token = state.token.clone().unwrap();
        let config = state.config.as_ref().ok_or("配置未找到")?;
        (token, config.base_url.clone())
    };

    // 自动拼接API路径，然后提取根URL
    let api_url = if base_url.ends_with("/api.php/v1") {
        base_url
    } else {
        format!("{}/api.php/v1", base_url.trim_end_matches('/'))
    };

    // 从API URL中提取根URL
    // api_url 格式: http://192.168.181.130:81/api.php/v1
    // 需要提取: http://192.168.181.130:81/
    let root_url = if let Some(pos) = api_url.find("/api.php") {
        &api_url[..pos]
    } else {
        api_url.trim_end_matches('/')
    };

    let client = reqwest::Client::new();
    let image_url = format!("{}/{}", root_url, image_path);
    
    debug!("获取图片: {} (从根URL: {})", image_url, root_url);
    
    match client
        .get(&image_url)
        .header("Token", &token)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            debug!("图片响应状态: {}", status);
            
            if status.is_success() {
                match response.bytes().await {
                    Ok(bytes) => {
                        // 将图片数据编码为Base64
                        let base64_data = general_purpose::STANDARD.encode(&bytes);
                        
                        // 根据文件扩展名确定MIME类型
                        let mime_type = if image_path.to_lowercase().ends_with(".png") {
                            "image/png"
                        } else if image_path.to_lowercase().ends_with(".jpg") || image_path.to_lowercase().ends_with(".jpeg") {
                            "image/jpeg"
                        } else if image_path.to_lowercase().ends_with(".gif") {
                            "image/gif"
                        } else if image_path.to_lowercase().ends_with(".webp") {
                            "image/webp"
                        } else {
                            "image/png" // 默认为PNG
                        };
                        
                        let data_url = format!("data:{};base64,{}", mime_type, base64_data);
                        info!("成功获取图片，大小: {} bytes", bytes.len());
                        
                        Ok(data_url)
                    }
                    Err(e) => {
                        error!("读取图片数据失败: {}", e);
                        Err(format!("读取图片数据失败: {}", e))
                    }
                }
            } else if status == 401 {
                let mut state = APP_STATE.lock().unwrap();
                state.token = None;
                state.token_expire_time = 0;
                Err("Unauthorized".to_string())
            } else {
                match response.text().await {
                    Ok(error_text) => {
                        // 解码错误信息中的 Unicode 转义序列
                        let decoded_error = decode_unicode_escapes(&error_text);
                        Err(format!("获取图片失败，状态码: {} - 错误信息: {}", status, decoded_error))
                    },
                    Err(_) => Err(format!("获取图片失败，状态码: {}", status)),
                }
            }
        }
        Err(e) => {
            error!("请求图片失败: {}", e);
            Err(format!("请求失败: {}", e))
        }
    }
}

// 获取产品详情
#[tauri::command]
async fn get_product_detail(product_id: i32) -> Result<Product, String> {
    // 检查token是否有效
    if !is_token_valid() {
        return Err("Token expired".to_string());
    }

    let (token, base_url) = {
        let state = APP_STATE.lock().unwrap();
        let token = state.token.clone().unwrap();
        let config = state.config.as_ref().ok_or("配置未找到")?;
        (token, config.base_url.clone())
    };

    // 自动拼接API路径
    let api_url = if base_url.ends_with("/api.php/v1") {
        base_url
    } else {
        format!("{}/api.php/v1", base_url.trim_end_matches('/'))
    };

    let client = reqwest::Client::new();
    let product_url = format!("{}/products/{}", api_url, product_id);
    
    debug!("获取产品详情: {}", product_url);
    
    match client
        .get(&product_url)
        .header("Token", &token)
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            debug!("产品详情响应状态: {}", status);
            
            if status.is_success() {
                match response.text().await {
                    Ok(text) => {
                        debug!("产品详情响应: 成功获取 (长度: {})", text.len());
                        
                        // 解码 Unicode 转义序列
                        let decoded_text = decode_unicode_escapes(&text);
                        debug!("产品详情响应: {}", decoded_text);
                        
                        match serde_json::from_str::<Value>(&decoded_text) {
                            Ok(json) => {
                                // 直接解析产品对象
                                match serde_json::from_value::<Product>(json) {
                                    Ok(product) => {
                                        debug!("成功解析产品详情: {}", product.name);
                                        Ok(product)
                                    },
                                    Err(e) => Err(format!("解析产品详情失败: {} - 响应内容: {}", e, text)),
                                }
                            }
                            Err(e) => Err(format!("解析产品详情JSON失败: {} - 响应内容: {}", e, text)),
                        }
                    }
                    Err(e) => Err(format!("读取产品详情响应失败: {}", e)),
                }
            } else {
                // 如果是401错误，说明token过期
                if status == 401 {
                    let mut state = APP_STATE.lock().unwrap();
                    state.token = None;
                    state.token_expire_time = 0;
                    return Err("Unauthorized".to_string());
                }
                
                match response.text().await {
                    Ok(error_text) => {
                        // 解码错误信息中的 Unicode 转义序列
                        let decoded_error = decode_unicode_escapes(&error_text);
                        Err(format!("获取产品详情失败，状态码: {} - 错误信息: {}", status, decoded_error))
                    },
                    Err(_) => Err(format!("获取产品详情失败，状态码: {}", status)),
                }
            }
        }
        Err(e) => Err(format!("请求失败: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化 tracing 订阅器 - 同时输出到控制台和文件
    setup_tracing();
    
    info!("禅道应用启动");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            initialize_app,
            login_zentao, 
            logout_zentao,
            get_bugs_by_product,
            get_bug_detail,
            get_user_info,
            get_products,
            get_product_detail,
            get_image,
            save_config, 
            load_config, 
            check_login_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
