// 产品相关的组合式函数
import { ref } from 'vue';
import { ElMessage } from 'element-plus';
import { ZentaoApiService } from '../services/api';
import type { Product } from '../types';

export function useProducts() {
  const products = ref<Product[]>([]);
  const loading = ref(false);

  const getProducts = async () => {
    loading.value = true;
    try {
      const result = await ZentaoApiService.getProducts();
      products.value = result;
      ElMessage.success(`获取到 ${products.value.length} 个产品`);
    } catch (error) {
      // 如果token过期，需要重新登录
      if (error === 'Token expired' || error === 'Unauthorized') {
        throw new Error('LOGIN_REQUIRED');
      } else {
        ElMessage.error(`获取产品列表失败: ${error}`);
      }
    } finally {
      loading.value = false;
    }
  };

  return {
    products,
    loading,
    getProducts
  };
}
