<template>
  <div class="product-list-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>产品列表 ({{ products.length }})</span>
          <el-button type="primary" @click="handleRefresh" :loading="loading">
            <el-icon><Refresh /></el-icon>
            刷新列表
          </el-button>
        </div>
      </template>

      <el-table 
        :data="products" 
        v-loading="loading"
        stripe
        style="width: 100%"
        empty-text="暂无数据"
      >
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="name" label="产品名称" min-width="200" />
        <el-table-column prop="code" label="产品代号" width="120" />
        <el-table-column prop="type" label="产品类型" width="100">
          <template #default="{ row }">
            <el-tag :type="row.type === 'normal' ? 'success' : row.type === 'branch' ? 'warning' : 'info'">
              {{ getProductTypeText(row.type) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="acl" label="访问控制" width="100">
          <template #default="{ row }">
            <el-tag :type="row.acl === 'open' ? 'success' : 'warning'">
              {{ getAclText(row.acl) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="createdDate" label="创建时间" width="180">
          <template #default="{ row }">
            {{ formatDate(row.createdDate) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button 
              type="primary" 
              size="small" 
              @click="handleViewDetail(row.id)"
              :loading="detailLoading"
            >
              详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, ref } from 'vue';
import { ElMessageBox } from 'element-plus';
import { Refresh } from '@element-plus/icons-vue';
import type { Product } from '../types';
import { ZentaoApiService } from '../services/api';

interface Props {
  products: Product[];
  loading: boolean;
}

interface Emits {
  (e: 'refresh'): void;
}

defineProps<Props>();
const emit = defineEmits<Emits>();

const detailLoading = ref(false);

const handleRefresh = () => {
  emit('refresh');
};

// 获取产品类型显示文本
function getProductTypeText(type: string) {
  const typeMap: { [key: string]: string } = {
    'normal': '正常',
    'branch': '多分支',
    'platform': '多平台'
  };
  return typeMap[type] || type;
}

// 获取访问控制显示文本
function getAclText(acl: string) {
  const aclMap: { [key: string]: string } = {
    'open': '公开',
    'private': '私有'
  };
  return aclMap[acl] || acl;
}

// 格式化日期
function formatDate(dateStr: string) {
  if (!dateStr) return '未知';
  try {
    return new Date(dateStr).toLocaleString();
  } catch {
    return dateStr;
  }
}

// 查看产品详情 - 调用详情API
async function handleViewDetail(productId: number) {
  detailLoading.value = true;
  try {
    // 调用产品详情API: GET /products/:id - 使用保存的配置
    const product = await ZentaoApiService.getProductDetail(productId);
    
    const whitelist = product.whitelist && product.whitelist.length > 0 
      ? product.whitelist.map(user => user.realname).join('、')
      : '无';

    ElMessageBox.alert(`
      <div style="text-align: left;">
        <h4 style="margin-bottom: 8px; color: #409eff;">基本信息</h4>
        <p><strong>产品ID：</strong>${product.id}</p>
        <p><strong>产品名称：</strong>${product.name}</p>
        <p><strong>产品代号：</strong>${product.code || '无'}</p>
        <p><strong>产品类型：</strong>${getProductTypeText(product.type)}</p>
        <p><strong>访问控制：</strong>${getAclText(product.acl)}</p>
        <p><strong>产品描述：</strong>${product.desc || '无描述'}</p>
        <p><strong>所属项目集：</strong>${product.program}</p>
        <p><strong>所属产品线：</strong>${product.line}</p>
        
        <div style="margin-top: 15px;">
          <h4 style="margin-bottom: 8px; color: #e6a23c;">负责人信息</h4>
          <p><strong>产品负责人：</strong>${product.PO ? product.PO.realname : '未指定'}</p>
          <p><strong>测试负责人：</strong>${product.QD ? product.QD.realname : '未指定'}</p>
          <p><strong>发布负责人：</strong>${product.RD ? product.RD.realname : '未指定'}</p>
          <p><strong>白名单：</strong>${whitelist}</p>
        </div>

        <div style="margin-top: 15px;">
          <h4 style="margin-bottom: 8px; color: #909399;">创建信息</h4>
          <p><strong>创建人：</strong>${product.createdBy ? product.createdBy.realname : '未知'}</p>
          <p><strong>创建时间：</strong>${formatDate(product.createdDate)}</p>
        </div>
      </div>
    `, `产品详情 - ${product.name}`, {
      confirmButtonText: '确定',
      dangerouslyUseHTMLString: true,
      customStyle: {
        width: '500px'
      }
    });
  } catch (error) {
    ElMessageBox.alert(`获取产品详情失败: ${error}`, '错误', {
      type: 'error'
    });
  } finally {
    detailLoading.value = false;
  }
}
</script>

<style scoped>
.product-list-container {
  max-width: 1400px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
