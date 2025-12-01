<template>
  <div class="user-info-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>个人信息</span>
          <el-button type="primary" @click="handleRefresh" :loading="loading">
            <el-icon><Refresh /></el-icon>
            刷新
          </el-button>
        </div>
      </template>

      <div v-if="userInfo" class="user-profile">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-descriptions title="基本信息" :column="1" border>
              <el-descriptions-item label="用户ID">{{ userInfo.id }}</el-descriptions-item>
              <el-descriptions-item label="账号">{{ userInfo.account }}</el-descriptions-item>
              <el-descriptions-item label="真实姓名">{{ userInfo.realname }}</el-descriptions-item>
              <el-descriptions-item label="昵称">{{ userInfo.nickname || '未设置' }}</el-descriptions-item>
              <el-descriptions-item label="用户类型">
                <el-tag :type="userInfo.user_type === 'inside' ? 'success' : 'warning'">
                  {{ userInfo.user_type === 'inside' ? '内部用户' : '外部用户' }}
                </el-tag>
              </el-descriptions-item>
              <el-descriptions-item label="部门ID">{{ userInfo.dept }}</el-descriptions-item>
              <el-descriptions-item label="管理员">
                <el-tag :type="userInfo.admin ? 'success' : 'info'">
                  {{ userInfo.admin ? '是' : '否' }}
                </el-tag>
              </el-descriptions-item>
            </el-descriptions>
          </el-col>
          <el-col :span="12">
            <el-descriptions title="联系信息" :column="1" border>
              <el-descriptions-item label="手机号码">{{ userInfo.mobile || '未设置' }}</el-descriptions-item>
              <el-descriptions-item label="电话号码">{{ userInfo.phone || '未设置' }}</el-descriptions-item>
              <el-descriptions-item label="微信号">{{ userInfo.weixin || '未设置' }}</el-descriptions-item>
              <el-descriptions-item label="住址">{{ userInfo.address || '未设置' }}</el-descriptions-item>
              <el-descriptions-item label="性别">
                {{ userInfo.gender === 'f' ? '女' : userInfo.gender === 'm' ? '男' : '未设置' }}
              </el-descriptions-item>
              <el-descriptions-item label="生日">{{ userInfo.birthday || '未设置' }}</el-descriptions-item>
              <el-descriptions-item label="加入日期">{{ userInfo.join || '未设置' }}</el-descriptions-item>
            </el-descriptions>
          </el-col>
        </el-row>
      </div>
      
      <div v-else class="no-user-info">
        <el-empty description="暂无用户信息">
          <el-button type="primary" @click="handleRefresh" :loading="loading">
            获取用户信息
          </el-button>
        </el-empty>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue';
import type { UserInfo } from '../types';

interface Props {
  userInfo: UserInfo | null;
  loading: boolean;
}

interface Emits {
  (e: 'refresh'): void;
}

defineProps<Props>();
const emit = defineEmits<Emits>();

const handleRefresh = () => {
  emit('refresh');
};
</script>

<style scoped>
.user-info-container {
  max-width: 1200px;
  margin: 0 auto;
}

.user-profile {
  margin-top: 20px;
}

.no-user-info {
  text-align: center;
  padding: 40px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
