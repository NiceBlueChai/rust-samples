<template>
  <el-dialog
    v-model="visible"
    title="Bug详情"
    width="1200px"
    :close-on-click-modal="false"
    @closed="handleClosed"
    top="5vh"
  >
    <div v-if="loading" class="dialog-loading">
      <el-icon class="is-loading"><Loading /></el-icon>
      <span>加载中...</span>
    </div>
    <div v-else-if="bugDetail" class="bug-detail">
      <el-descriptions :column="2" border>
        <el-descriptions-item label="Bug ID">{{ bugDetail.id }}</el-descriptions-item>
        <el-descriptions-item label="所属产品">{{ getProductName(bugDetail.product) }}</el-descriptions-item>
        <el-descriptions-item label="标题" :span="2">{{ bugDetail.title }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag :type="getStatusTagType(bugDetail.status.code)">
            {{ bugDetail.status.name }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="严重程度">
          <el-tag :type="getSeverityTagType(bugDetail.severity)">
            {{ getSeverityText(bugDetail.severity) }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="优先级">{{ getPriorityText(bugDetail.pri) }}</el-descriptions-item>
        <el-descriptions-item label="Bug类型">{{ getBugTypeText(bugDetail.bug_type) }}</el-descriptions-item>
        <el-descriptions-item label="创建者">{{ bugDetail.openedBy ? bugDetail.openedBy.realname : '未知' }}</el-descriptions-item>
        <el-descriptions-item label="创建时间">{{ bugDetail.openedDateFormatted }}</el-descriptions-item>
        <el-descriptions-item label="指派给">{{ bugDetail.assignedTo ? bugDetail.assignedTo.realname : '未指派' }}</el-descriptions-item>
        <el-descriptions-item label="指派时间">{{ bugDetail.assignedDateFormatted || '未指派' }}</el-descriptions-item>
        <el-descriptions-item label="操作系统">{{ bugDetail.os || '未指定' }}</el-descriptions-item>
        <el-descriptions-item label="浏览器">{{ bugDetail.browser || '未指定' }}</el-descriptions-item>
        <el-descriptions-item label="关键字" :span="2">{{ bugDetail.keywords || '无' }}</el-descriptions-item>
        <el-descriptions-item label="重现步骤" :span="2">
          <div v-html="processedSteps" ref="stepsContainer" class="steps-content"></div>
        </el-descriptions-item>
      </el-descriptions>
    </div>
    <template #footer>
      <el-button @click="handleClose">关闭</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed, nextTick } from 'vue';
import { Loading } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import type { Bug, Product } from '../types';
import { ZentaoApiService } from '../services/api';

interface Props {
  modelValue: boolean;
  bugId: number | null;
  products: Product[];
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const visible = ref(false);
const loading = ref(false);
const bugDetail = ref<Bug | null>(null);
const stepsContainer = ref<HTMLElement | null>(null);
const imageLoadingMap = ref<Map<string, boolean>>(new Map());

// 计算属性：处理过的steps内容
const processedSteps = computed(() => {
  if (!bugDetail.value?.stepsProcessed) return '无';
  return bugDetail.value.stepsProcessed;
});

// 监听props.modelValue变化
watch(() => props.modelValue, (newValue) => {
  visible.value = newValue;
  if (newValue && props.bugId) {
    loadBugDetail();
  }
});

// 监听visible变化，同步到父组件
watch(visible, (newValue) => {
  emit('update:modelValue', newValue);
});

// 监听bugDetail变化，处理图片懒加载
watch(bugDetail, async (newValue) => {
  if (newValue && newValue.stepsProcessed) {
    await nextTick();
    setupImageLazyLoad();
  }
});

// 加载Bug详情
const loadBugDetail = async () => {
  if (!props.bugId) return;
  
  loading.value = true;
  try {
    const detail = await ZentaoApiService.getBugDetail(props.bugId);
    bugDetail.value = detail;
  } catch (error) {
    ElMessage.error(`获取Bug详情失败: ${error}`);
    handleClose();
  } finally {
    loading.value = false;
  }
};

// 设置图片懒加载
const setupImageLazyLoad = () => {
  if (!stepsContainer.value) return;
  
  const images = stepsContainer.value.querySelectorAll('img[data-lazy-load="true"]');
  images.forEach((img: Element) => {
    const htmlImg = img as HTMLImageElement;
    const imagePath = htmlImg.getAttribute('data-image-path');
    
    if (imagePath && !htmlImg.src.startsWith('data:')) {
      // 创建加载按钮
      const loadButton = document.createElement('button');
      loadButton.textContent = '点击加载图片';
      loadButton.style.cssText = `
        background: #409EFF;
        color: white;
        border: none;
        padding: 8px 16px;
        border-radius: 4px;
        cursor: pointer;
        font-size: 14px;
        margin: 8px 0;
        display: block;
      `;
      
      loadButton.addEventListener('click', () => loadImage(imagePath, htmlImg, loadButton));
      
      // 隐藏原图片，显示加载按钮
      htmlImg.style.display = 'none';
      htmlImg.parentNode?.insertBefore(loadButton, htmlImg);
    }
  });
};

// 加载单个图片
const loadImage = async (imagePath: string, imgElement: HTMLImageElement, buttonElement: HTMLButtonElement) => {
  if (imageLoadingMap.value.get(imagePath)) return; // 防止重复加载
  
  imageLoadingMap.value.set(imagePath, true);
  buttonElement.textContent = '加载中...';
  buttonElement.disabled = true;
  
  try {
    const dataUrl = await ZentaoApiService.getImage(imagePath);
    imgElement.src = dataUrl;
    imgElement.style.display = 'inline';
    imgElement.style.maxWidth = '100%';
    imgElement.style.height = 'auto';
    
    // 添加点击放大功能
    imgElement.addEventListener('click', () => zoomImage(imgElement));
    
    buttonElement.remove(); // 移除加载按钮
  } catch (error) {
    ElMessage.error(`加载图片失败: ${error}`);
    buttonElement.textContent = '加载失败，点击重试';
    buttonElement.disabled = false;
    imageLoadingMap.value.set(imagePath, false);
  }
};

// 图片放大功能
const zoomImage = (imgElement: HTMLImageElement) => {
  // 创建遮罩层
  const overlay = document.createElement('div');
  overlay.className = 'image-overlay';
  
  // 克隆图片
  const clonedImg = imgElement.cloneNode(true) as HTMLImageElement;
  clonedImg.className = 'zoomed';
  
  // 点击关闭
  const closeZoom = () => {
    overlay.remove();
    clonedImg.remove();
  };
  
  overlay.addEventListener('click', closeZoom);
  clonedImg.addEventListener('click', closeZoom);
  
  // 添加到页面
  document.body.appendChild(overlay);
  document.body.appendChild(clonedImg);
  
  // ESC键关闭
  const handleEsc = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      closeZoom();
      document.removeEventListener('keydown', handleEsc);
    }
  };
  document.addEventListener('keydown', handleEsc);
};

// 关闭对话框
const handleClose = () => {
  visible.value = false;
};

// 对话框完全关闭后的回调
const handleClosed = () => {
  bugDetail.value = null;
  loading.value = false;
};

// 根据产品ID获取产品名称
const getProductName = (productId: number) => {
  const product = props.products.find(p => p.id === productId);
  return product ? product.name : `产品${productId}`;
};

// 获取状态标签类型
function getStatusTagType(status: string) {
  const typeMap: { [key: string]: string } = {
    'active': 'danger',
    'resolved': 'success',
    'closed': 'info',
    'testing': 'warning'
  };
  return typeMap[status] || 'info';
}

// 获取严重程度显示文本
function getSeverityText(severity: number) {
  const severityMap: { [key: number]: string } = {
    1: '致命',
    2: '严重',
    3: '一般',
    4: '轻微'
  };
  return severityMap[severity] || `等级${severity}`;
}

// 获取严重程度标签类型
function getSeverityTagType(severity: number) {
  const typeMap: { [key: number]: string } = {
    1: 'danger',
    2: 'warning',
    3: 'info',
    4: 'success'
  };
  return typeMap[severity] || 'info';
}

// 获取优先级显示文本
function getPriorityText(priority: number) {
  const priorityMap: { [key: number]: string } = {
    1: '高',
    2: '中',
    3: '低',
    4: '不指定'
  };
  return priorityMap[priority] || `P${priority}`;
}

// 获取Bug类型显示文本
function getBugTypeText(type: string) {
  const typeMap: { [key: string]: string } = {
    'codeerror': '代码错误',
    'config': '配置相关',
    'install': '安装部署',
    'security': '安全相关',
    'performance': '性能问题',
    'standard': '标准规范',
    'automation': '测试脚本',
    'designdefect': '设计缺陷',
    'others': '其他'
  };
  return typeMap[type] || type || '未指定';
}
</script>

<style scoped>
.dialog-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px;
  color: #606266;
}

.bug-detail {
  max-height: 70vh;
  overflow-y: auto;
}

.steps-content {
  line-height: 1.6;
  max-width: 100%;
}

.steps-content img {
  max-width: 100%;
  height: auto;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  margin: 8px 0;
  cursor: zoom-in;
  transition: transform 0.2s ease;
}

.steps-content img:hover {
  transform: scale(1.02);
}

.steps-content img.zoomed {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%) scale(1);
  max-width: 90vw;
  max-height: 90vh;
  z-index: 3000;
  cursor: zoom-out;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  border-radius: 8px;
}

.image-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.8);
  z-index: 2999;
  cursor: zoom-out;
}

.steps-content button {
  transition: all 0.3s ease;
}

.steps-content button:hover {
  background: #337ecc !important;
  transform: translateY(-1px);
}
</style>
