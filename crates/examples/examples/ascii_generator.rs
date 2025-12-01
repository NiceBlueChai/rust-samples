use clap::Parser;
use image::{DynamicImage, GenericImageView, ImageError};
use std::path::PathBuf;

/// 命令行参数
#[derive(Debug, Parser)]
#[command(author, version, about = "Simple ASCII art generator", long_about = None)]
struct Args {
    /// 输入图片路径
    #[arg(short, long)]
    input: PathBuf,

    /// 输出宽度（字符数）
    #[arg(short, long, default_value_t = 80)]
    width: u32,

    /// 是否反转亮度（亮色用密集字符）
    #[arg(long, default_value_t = false)]
    invert: bool,
}

/// 灰度值（0-255）到 ASCII 字符的映射表
///
/// - 下标 0 表示最暗（黑）
/// - 下标末尾表示最亮（白）
///
/// 你可以根据喜好调整这个字符串。
const ASCII_CHARS: &str = " .:-=+*#%@";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 读取图片
    let img = image::open(&args.input).map_err(|e| {
        eprintln!("Failed to open image {:?}: {e}", args.input);
        e
    })?;

    // 生成 ASCII
    let ascii = image_to_ascii(&img, args.width, args.invert)?;

    // 输出到终端
    println!("{ascii}");

    Ok(())
}

/// 将一张图片转换为 ASCII 字符画
///
/// - `img`: 输入图片
/// - `target_width`: 目标字符宽度
/// - `invert`: 是否反转亮度映射
fn image_to_ascii(
    img: &DynamicImage,
    target_width: u32,
    invert: bool,
) -> Result<String, ImageError> {
    // 原始尺寸
    let (w, h) = img.dimensions();

    if target_width == 0 {
        return Ok(String::new());
    }

    // 终端里一个字符的宽高比一般接近 1:2（字符更高）
    // 为了让输出不变形，我们缩放时把高度按这个比例调整一下。
    let aspect_ratio = h as f32 / w as f32;
    let char_aspect = 2.0; // 终端字符高度约为宽度的 2 倍
    let target_height = (target_width as f32 * aspect_ratio / char_aspect).round() as u32;

    // 将图片缩放到目标大小，并转为灰度图
    let resized = img
        .resize_exact(target_width, target_height, image::imageops::FilterType::Triangle)
        .grayscale();

    let (rw, rh) = resized.dimensions();
    let mut result = String::new();

    for y in 0..rh {
        for x in 0..rw {
            let pixel = resized.get_pixel(x, y);
            let luma = pixel[0]; // 灰度图每个像素只有一个通道
            let ch = luma_to_char(luma, invert);
            result.push(ch);
        }
        result.push('\n');
    }

    Ok(result)
}

/// 将 0-255 灰度值映射为 ASCII 字符
fn luma_to_char(luma: u8, invert: bool) -> char {
    let chars: Vec<char> = ASCII_CHARS.chars().collect();
    let n = chars.len() as u8;

    // 计算索引：luma 越大，索引越大
    let mut idx = (luma as u16 * (n as u16 - 1) / 255) as u8;

    if invert {
        idx = (n - 1) - idx;
    }

    chars[idx as usize]
}