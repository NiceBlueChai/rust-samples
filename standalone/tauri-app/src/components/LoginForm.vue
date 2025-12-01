<template>
  <div class="login-container">
    <el-card class="login-card">
      <template #header>
        <div class="card-header">
          <span>禅道系统配置</span>
        </div>
      </template>
      
      <el-form :model="config" :rules="rules" ref="configForm" label-width="100px">
        <el-form-item label="基地址" prop="baseUrl">
          <el-input 
            v-model="config.baseUrl" 
            placeholder="请输入禅道服务器地址，如：http://192.168.181.130:81"
          />
        </el-form-item>
        
        <el-form-item label="账号" prop="account">
          <el-input v-model="config.account" placeholder="请输入您的禅道账号" />
        </el-form-item>
        
        <el-form-item label="密码" prop="password">
          <el-input 
            v-model="config.password" 
            type="password" 
            placeholder="请输入您的禅道密码" 
            show-password
          />
        </el-form-item>
        
        <el-form-item>
          <el-button type="primary" @click="handleLogin" :loading="loading">
            <el-icon><User /></el-icon>
            登录
          </el-button>
          <el-button @click="handleSaveConfig">
            <el-icon><Document /></el-icon>
            保存配置
          </el-button>
        </el-form-item>
      </el-form>
      
      <el-alert 
        title="安全提示" 
        type="info" 
        description="配置信息将安全保存在您的本地设备中，不会发送到其他服务器。建议使用 HTTPS 地址确保通信安全。"
        show-icon 
        :closable="false"
        style="margin-top: 16px;"
      />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, defineEmits, watch, ref } from 'vue';
import { User, Document } from '@element-plus/icons-vue';
import type { ZentaoConfig } from '../types';
import type { FormInstance, FormRules } from 'element-plus';

interface Props {
  loading: boolean;
  initialConfig?: ZentaoConfig;
}

interface Emits {
  (e: 'login', config: ZentaoConfig): void;
  (e: 'save-config', config: ZentaoConfig): void;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  initialConfig: () => ({
    baseUrl: '',
    account: '',
    password: ''
  })
});

const emit = defineEmits<Emits>();

const configForm = ref<FormInstance>();

// 表单验证规则
const rules = reactive<FormRules<ZentaoConfig>>({
  baseUrl: [
    { required: true, message: '请输入禅道服务器地址', trigger: 'blur' },
    { 
      pattern: /^https?:\/\/.+/, 
      message: '请输入有效的服务器地址（以 http:// 或 https:// 开头）', 
      trigger: 'blur' 
    }
  ],
  account: [
    { required: true, message: '请输入账号', trigger: 'blur' },
    { min: 2, message: '账号长度至少2个字符', trigger: 'blur' }
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 3, message: '密码长度至少3个字符', trigger: 'blur' }
  ]
});

// 配置信息
const config = reactive<ZentaoConfig>({
  baseUrl: props.initialConfig?.baseUrl || '',
  account: props.initialConfig?.account || '',
  password: props.initialConfig?.password || ''
});

// 监听初始配置变化，更新表单
watch(() => props.initialConfig, (newConfig) => {
  if (newConfig) {
    config.baseUrl = newConfig.baseUrl;
    config.account = newConfig.account;
    config.password = newConfig.password;
  }
}, { immediate: true });

const handleLogin = async () => {
  if (!configForm.value) return;
  
  const isValid = await configForm.value.validate().catch(() => false);
  if (isValid) {
    emit('login', { ...config });
  }
};

const handleSaveConfig = async () => {
  if (!configForm.value) return;
  
  const isValid = await configForm.value.validate().catch(() => false);
  if (isValid) {
    emit('save-config', { ...config });
  }
};
</script>

<style scoped>
.login-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 400px;
}

.login-card {
  width: 500px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
