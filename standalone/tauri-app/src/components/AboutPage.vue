<template>
  <div class="about-page">
    <el-card class="about-card">
      <template #header>
        <div class="card-header">
          <el-icon class="header-icon"><InfoFilled /></el-icon>
          <span>关于禅道Bug管理</span>
        </div>
      </template>
      
      <div class="about-content">
        <!-- 应用信息 -->
        <div class="app-info">
          <div class="app-logo">
            <el-icon class="logo-icon"><Tools /></el-icon>
          </div>
          <h2 class="app-name">禅道Bug管理工具</h2>
          <p class="app-version">版本 1.0.0</p>
          <p class="app-description">
            一个基于Tauri和Vue 3构建的现代化禅道Bug管理客户端，提供高效的Bug查看、筛选、排序和管理功能。
          </p>
        </div>

        <el-divider />

        <!-- 作者信息 -->
        <div class="author-info">
          <h3>
            <el-icon><User /></el-icon>
            开发者信息
          </h3>
          <div class="author-details">
            <div class="info-item">
              <el-icon class="info-icon"><User /></el-icon>
              <span class="info-label">作者：</span>
              <span class="info-value">NiceBlueChai</span>
            </div>
            <div class="info-item">
              <el-icon class="info-icon"><Message /></el-icon>
              <span class="info-label">邮箱：</span>
              <el-link 
                :href="`mailto:bluechai@qq.com`" 
                type="primary" 
                class="info-value"
              >
                bluechai@qq.com
              </el-link>
            </div>
            <div class="info-item">
              <el-icon class="info-icon"><Calendar /></el-icon>
              <span class="info-label">开发时间：</span>
              <span class="info-value">2025年7月</span>
            </div>
          </div>
        </div>

        <el-divider />

        <!-- 技术栈 -->
        <div class="tech-stack">
          <h3>
            <el-icon><SetUp /></el-icon>
            技术栈
          </h3>
          <div class="tech-tags">
            <el-tag type="primary" class="tech-tag">Tauri</el-tag>
            <el-tag type="success" class="tech-tag">Vue 3</el-tag>
            <el-tag type="warning" class="tech-tag">TypeScript</el-tag>
            <el-tag type="info" class="tech-tag">Element Plus</el-tag>
            <el-tag type="danger" class="tech-tag">Rust</el-tag>
            <el-tag class="tech-tag">Vite</el-tag>
          </div>
        </div>

        <el-divider />

        <!-- 功能特性 -->
        <div class="features">
          <h3>
            <el-icon><Star /></el-icon>
            主要功能
          </h3>
          <div class="feature-list">
            <div class="feature-item">
              <el-icon class="feature-icon" color="#409eff"><Check /></el-icon>
              <span>禅道系统登录认证</span>
            </div>
            <div class="feature-item">
              <el-icon class="feature-icon" color="#409eff"><Check /></el-icon>
              <span>Token自动保存与加密存储</span>
            </div>
            <div class="feature-item">
              <el-icon class="feature-icon" color="#409eff"><Check /></el-icon>
              <span>产品列表管理</span>
            </div>
            <div class="feature-item">
              <el-icon class="feature-icon" color="#409eff"><Check /></el-icon>
              <span>Bug列表查看与详情</span>
            </div>
            <div class="feature-item">
              <el-icon class="feature-icon" color="#409eff"><Check /></el-icon>
              <span>自定义列配置</span>
            </div>
            <div class="feature-item">
              <el-icon class="feature-icon" color="#409eff"><Check /></el-icon>
              <span>高级筛选与排序</span>
            </div>
            <div class="feature-item">
              <el-icon class="feature-icon" color="#409eff"><Check /></el-icon>
              <span>分组查看与折叠</span>
            </div>
            <div class="feature-item">
              <el-icon class="feature-icon" color="#409eff"><Check /></el-icon>
              <span>中国时区时间格式化</span>
            </div>
          </div>
        </div>

        <el-divider />

        <!-- 版权信息 -->
        <div class="copyright">
          <p class="copyright-text">
            <el-icon><InfoFilled /></el-icon>
            2025 NiceBlueChai. 本软件基于开源协议开发，仅供学习和交流使用。
          </p>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import {
  InfoFilled,
  Tools,
  User,
  Message,
  Calendar,
  SetUp,
  Star,
  Check
} from '@element-plus/icons-vue';
</script>

<style scoped>
.about-page {
  max-width: 800px;
  margin: 0 auto;
  padding: 20px;
}

.about-card {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  border-radius: 12px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.header-icon {
  font-size: 20px;
  color: #409eff;
}

.about-content {
  padding: 0;
}

/* 应用信息 */
.app-info {
  text-align: center;
  margin-bottom: 20px;
}

.app-logo {
  margin-bottom: 16px;
}

.logo-icon {
  font-size: 64px;
  color: #409eff;
  background: linear-gradient(135deg, #409eff 0%, #67c23a 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.app-name {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  margin: 0 0 8px 0;
}

.app-version {
  font-size: 16px;
  color: #909399;
  margin: 0 0 16px 0;
}

.app-description {
  font-size: 14px;
  color: #606266;
  line-height: 1.6;
  max-width: 600px;
  margin: 0 auto;
}

/* 作者信息 */
.author-info h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 16px 0;
}

.author-details {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.info-icon {
  font-size: 16px;
  color: #909399;
  width: 16px;
}

.info-label {
  font-weight: 500;
  color: #606266;
  min-width: 60px;
}

.info-value {
  color: #303133;
}

/* 技术栈 */
.tech-stack h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 16px 0;
}

.tech-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.tech-tag {
  font-size: 12px;
  padding: 4px 12px;
  border-radius: 16px;
}

/* 功能特性 */
.features h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 16px 0;
}

.feature-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 12px;
}

.feature-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #606266;
  padding: 4px 0;
}

.feature-icon {
  font-size: 16px;
}

/* 版权信息 */
.copyright {
  text-align: center;
  margin-top: 20px;
}

.copyright-text {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  font-size: 12px;
  color: #909399;
  margin: 0;
  line-height: 1.5;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .about-page {
    padding: 16px;
  }
  
  .app-name {
    font-size: 24px;
  }
  
  .logo-icon {
    font-size: 48px;
  }
  
  .feature-list {
    grid-template-columns: 1fr;
  }
  
  .author-details {
    align-items: flex-start;
  }
  
  .info-item {
    flex-wrap: wrap;
  }
  
  .tech-tags {
    justify-content: center;
  }
}
</style>
