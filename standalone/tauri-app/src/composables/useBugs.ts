// Bug相关的组合式函数
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import { ZentaoApiService } from '../services/api';
import type { Bug } from '../types';

export function useBugs() {
  const bugs = ref<Bug[]>([]);
  const loading = ref(false);

  const getBugs = async (productIds: number[]) => {
    loading.value = true;
    try {
      const result = await ZentaoApiService.getBugs(productIds);
      bugs.value = result;
      if (result.length > 0) {
        ElMessage.success(`获取到 ${bugs.value.length} 个Bug`);
      } else {
        ElMessage.info('暂无Bug数据');
      }
    } catch (error) {
      // 如果token过期，需要重新登录
      if (error === 'Token expired' || error === 'Unauthorized') {
        throw new Error('LOGIN_REQUIRED');
      } else {
        ElMessage.error(`获取Bug列表失败: ${error}`);
      }
    } finally {
      loading.value = false;
    }
  };

  return {
    bugs,
    loading,
    getBugs
  };
}
