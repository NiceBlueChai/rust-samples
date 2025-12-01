<script setup lang="ts">
import { ref, onMounted } from "vue";
import { ElMessage } from 'element-plus';
import { 
  User, 
  Box, 
  Warning, 
  SwitchButton,
  Tools,
  InfoFilled
} from '@element-plus/icons-vue';

// 导入新的模块化组件
import LoginForm from './components/LoginForm.vue';
import UserInfo from './components/UserInfo.vue';
import ProductList from './components/ProductList.vue';
import BugList from './components/BugList.vue';
import AboutPage from './components/AboutPage.vue';

// 导入组合式函数
import { useUser } from './composables/useUser';
import { useProducts } from './composables/useProducts';
import { useBugs } from './composables/useBugs';

import type { ZentaoConfig } from './types';

// 应用状态
const isLoggedIn = ref(false);
const loading = ref(false);
const currentView = ref<'user' | 'bugs' | 'products' | 'about'>('user');
const baseUrl = ref(''); // 移除硬编码，从配置中获取
const initializing = ref(true);
const savedConfig = ref<ZentaoConfig | null>(null);

// 使用组合式函数
const { userInfo, loading: userLoading, getUserInfo } = useUser();
const { products, loading: productsLoading, getProducts } = useProducts();
const { bugs, loading: bugsLoading, getBugs } = useBugs();

// 应用初始化
const initializeApp = async () => {
  try {
    initializing.value = true;
    const { ZentaoApiService } = await import('./services/api');
    
    // 尝试初始化应用（加载保存的token）
    const hasValidToken = await ZentaoApiService.initializeApp();
    
    // 加载保存的配置
    try {
      const config = await ZentaoApiService.loadConfig();
      if (config) {
        savedConfig.value = config;
        baseUrl.value = config.baseUrl;
        console.log('加载到保存的配置:', config);
      }
    } catch (error) {
      console.error('加载配置失败:', error);
    }
    
    if (hasValidToken) {
      isLoggedIn.value = true;
      // 自动获取用户信息
      await handleGetUserInfo();
      ElMessage.success('自动登录成功');
    } else {
      console.log('没有找到有效的保存token，需要重新登录');
    }
  } catch (error) {
    console.error('应用初始化失败:', error);
    // 初始化失败不显示错误消息，只是需要用户重新登录
  } finally {
    initializing.value = false;
  }
};

// 组件挂载时初始化应用
onMounted(() => {
  initializeApp();
});

// 登录处理
const handleLogin = async (config: ZentaoConfig) => {
  loading.value = true;
  try {
    const { ZentaoApiService } = await import('./services/api');
    const success = await ZentaoApiService.login(config.baseUrl, config.account, config.password);
    
    if (success) {
      isLoggedIn.value = true;
      baseUrl.value = config.baseUrl;
      currentView.value = 'user';
      ElMessage.success('登录成功');
      // 登录成功后获取用户信息
      await handleGetUserInfo();
    }
  } catch (error) {
    ElMessage.error(`登录失败: ${error}`);
  } finally {
    loading.value = false;
  }
};

// 获取用户信息
const handleGetUserInfo = async () => {
  try {
    await getUserInfo();
  } catch (error: any) {
    if (error?.message === 'LOGIN_REQUIRED') {
      isLoggedIn.value = false;
      ElMessage.error('登录已过期，请重新登录');
    }
  }
};

// 获取产品列表
const handleGetProducts = async () => {
  try {
    await getProducts();
  } catch (error: any) {
    if (error?.message === 'LOGIN_REQUIRED') {
      isLoggedIn.value = false;
      ElMessage.error('登录已过期，请重新登录');
    }
  }
};

// 获取Bug列表
const handleGetBugs = async (productIds: number[]) => {
  console.log('App.vue handleGetBugs调用，产品IDs:', productIds);
  if (productIds.length === 0) {
    ElMessage.warning('请先选择产品');
    return;
  }
  
  try {
    await getBugs(productIds);
    console.log('获取Bug后的数据:', bugs.value);
  } catch (error: any) {
    console.error('获取Bug列表错误:', error);
    if (error?.message === 'LOGIN_REQUIRED') {
      isLoggedIn.value = false;
      ElMessage.error('登录已过期，请重新登录');
    }
  }
};

// 切换到产品列表页面时，确保产品数据已加载
const switchToProductList = async () => {
  currentView.value = 'products';
  // 如果还没有产品数据，先加载
  if (products.value.length === 0) {
    await handleGetProducts();
  }
};

// 切换到Bug列表页面时，确保产品数据已加载
const switchToBugList = async () => {
  currentView.value = 'bugs';
  // 如果还没有产品数据，先加载
  if (products.value.length === 0) {
    await handleGetProducts();
  }
};

// 退出登录
const handleLogout = async () => {
  try {
    const { ZentaoApiService } = await import('./services/api');
    await ZentaoApiService.logout();
    isLoggedIn.value = false;
    currentView.value = 'user';
    ElMessage.success('已退出登录');
  } catch (error) {
    ElMessage.error(`退出登录失败: ${error}`);
  }
};

// 保存配置
const handleSaveConfig = async (config: ZentaoConfig) => {
  try {
    const { ZentaoApiService } = await import('./services/api');
    await ZentaoApiService.saveConfig(config);
    ElMessage.success('配置已保存');
  } catch (error) {
    ElMessage.error(`保存配置失败: ${error}`);
  }
};
</script>

<template>
  <div id="app">
    <!-- 应用初始化加载 -->
    <div v-if="initializing" class="initializing">
      <el-container>
        <el-main class="loading-container">
          <div class="loading-content">
            <el-icon class="loading-icon is-loading"><Tools /></el-icon>
            <h2>正在初始化应用...</h2>
            <p>检查登录状态</p>
          </div>
        </el-main>
      </el-container>
    </div>

    <!-- 主应用界面 -->
    <el-container v-else>
      <el-header class="header">
        <div class="header-content">
          <div class="logo">
            <el-icon><Tools /></el-icon>
            <h1>禅道Bug管理</h1>
          </div>
          <div class="actions" v-if="isLoggedIn">
            <el-button type="primary" @click="currentView = 'user'" :loading="loading">
              <el-icon><User /></el-icon>
              用户信息
            </el-button>
            <el-button type="success" @click="switchToProductList" :loading="loading">
              <el-icon><Box /></el-icon>
              产品列表
            </el-button>
            <el-button type="warning" @click="switchToBugList" :loading="loading">
              <el-icon><Warning /></el-icon>
              Bug列表
            </el-button>
            <el-button type="info" @click="currentView = 'about'" :loading="loading">
              <el-icon><InfoFilled /></el-icon>
              关于
            </el-button>
            <el-button type="danger" @click="handleLogout">
              <el-icon><SwitchButton /></el-icon>
              退出
            </el-button>
          </div>
        </div>
      </el-header>

      <el-main class="main-content">
        <!-- 登录表单 -->
        <LoginForm 
          v-if="!isLoggedIn" 
          @login="handleLogin"
          @save-config="handleSaveConfig"
          :loading="loading"
          :initial-config="savedConfig || undefined"
        />

        <!-- 用户信息 -->
        <UserInfo 
          v-if="isLoggedIn && currentView === 'user'" 
          :user-info="userInfo"
          :loading="userLoading"
          @refresh="handleGetUserInfo"
        />

        <!-- 产品列表 -->
        <ProductList 
          v-if="isLoggedIn && currentView === 'products'" 
          :products="products"
          :loading="productsLoading"
          @refresh="handleGetProducts"
        />

        <!-- Bug列表 -->
        <BugList 
          v-if="isLoggedIn && currentView === 'bugs'" 
          :bugs="bugs"
          :products="products"
          :loading="bugsLoading"
          @refresh="handleGetBugs"
        />

        <!-- 关于页面 -->
        <AboutPage 
          v-if="isLoggedIn && currentView === 'about'"
        />
      </el-main>
    </el-container>
  </div>
</template>

<style scoped>
.header {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  height: 60px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 100%;
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 20px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
}

.logo .el-icon {
  font-size: 28px;
}

.actions {
  display: flex;
  gap: 12px;
}

.main-content {
  background: #f5f7fa;
  min-height: calc(100vh - 60px);
  padding: 20px;
  margin-top: 60px; /* 为固定头部留出空间 */
}

/* 应用初始化加载样式 */
.initializing {
  width: 100%;
  height: 100vh;
  background: #f5f7fa;
}

.loading-container {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.loading-content {
  text-align: center;
  color: #606266;
}

.loading-icon {
  font-size: 48px;
  color: #409eff;
  margin-bottom: 16px;
  animation: rotate 2s linear infinite;
}

@keyframes rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.loading-content h2 {
  margin: 16px 0 8px 0;
  font-size: 24px;
  color: #303133;
}

.loading-content p {
  margin: 0;
  font-size: 14px;
  color: #909399;
}

#app {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
  color: #2c3e50;
  background: #f5f7fa;
  min-height: 100vh;
}

/* Element Plus 容器样式调整 */
.el-container {
  min-height: 100vh;
}

.el-header {
  padding: 0;
}

/* Element Plus 按钮样式调整 */
.el-button {
  border-radius: 6px;
  font-weight: 500;
}

.el-button--primary {
  background: #409eff;
  border-color: #409eff;
}

.el-button--success {
  background: #67c23a;
  border-color: #67c23a;
}

.el-button--warning {
  background: #e6a23c;
  border-color: #e6a23c;
}

.el-button--danger {
  background: #f56c6c;
  border-color: #f56c6c;
}
</style>
