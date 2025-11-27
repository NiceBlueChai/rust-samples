// 用户相关的组合式函数
import { ref } from 'vue';
import { ZentaoApiService } from '../services/api';
import type { UserInfo } from '../types';

export function useUser() {
  const userInfo = ref<UserInfo | null>(null);
  const loading = ref(false);

  const getUserInfo = async () => {
    loading.value = true;
    try {
      const result = await ZentaoApiService.getUserInfo();
      userInfo.value = result;
      console.log('用户信息:', userInfo.value);
    } catch (error) {
      console.error('获取用户信息失败:', error);
      // 不显示错误消息，因为这不是关键功能
    } finally {
      loading.value = false;
    }
  };

  return {
    userInfo,
    loading,
    getUserInfo
  };
}
