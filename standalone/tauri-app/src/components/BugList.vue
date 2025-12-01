<template>
  <div class="bug-list-container">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>Bug列表 ({{ bugs.length }})</span>
          <div class="header-actions">
            <el-button @click="columnSettingsVisible = true">
              <el-icon><Setting /></el-icon>
              列设置
            </el-button>
            <el-button type="primary" @click="handleRefresh" :loading="loading">
              <el-icon><Refresh /></el-icon>
              刷新列表
            </el-button>
          </div>
        </div>
      </template>

      <!-- 产品选择器 -->
      <div class="filter-section">
        <el-row :gutter="20" style="margin-bottom: 20px;">
          <el-col :span="16">
            <el-select
              v-model="selectedProducts"
              multiple
              placeholder="请选择产品（支持多选）"
              style="width: 100%"
              @change="handleProductChange"
              clearable
              collapse-tags
              collapse-tags-tooltip
            >
              <el-option
                v-for="product in products"
                :key="product.id"
                :label="product.name"
                :value="product.id"
              >
                <span>{{ product.name }}</span>
                <span style="float: right; color: #8492a6; font-size: 13px">
                  ID: {{ product.id }}
                </span>
              </el-option>
            </el-select>
          </el-col>
          <el-col :span="4">
            <el-button type="success" @click="handleSelectAll">
              全选产品
            </el-button>
          </el-col>
          <el-col :span="4">
            <el-button @click="handleClearSelection">
              清空选择
            </el-button>
          </el-col>
        </el-row>
        
        <!-- 筛选和排序控件 -->
        <el-row :gutter="20" style="margin-bottom: 20px;">
          <!-- 状态筛选 -->
          <el-col :span="6">
            <el-select
              v-model="filterStatus"
              multiple
              placeholder="筛选状态（可多选）"
              clearable
              collapse-tags
              @change="applyFilters"
            >
              <el-option label="激活" value="active" />
              <el-option label="已解决" value="resolved" />
              <el-option label="已关闭" value="closed" />
              <el-option label="测试中" value="testing" />
            </el-select>
          </el-col>
          
          <!-- 严重程度筛选 -->
          <el-col :span="6">
            <el-select
              v-model="filterSeverity"
              multiple
              placeholder="筛选严重程度（可多选）"
              clearable
              collapse-tags
              @change="applyFilters"
            >
              <el-option label="致命" :value="1" />
              <el-option label="严重" :value="2" />
              <el-option label="一般" :value="3" />
              <el-option label="轻微" :value="4" />
            </el-select>
          </el-col>
          
          <!-- 指派人筛选 -->
          <el-col :span="6">
            <el-select
              v-model="filterAssignedTo"
              multiple
              placeholder="筛选指派人（可多选）"
              clearable
              filterable
              collapse-tags
              @change="applyFilters"
            >
              <el-option label="未指派" value="unassigned" />
              <el-option
                v-for="user in uniqueAssignedUsers"
                :key="user"
                :label="user"
                :value="user"
              />
            </el-select>
          </el-col>
          
          <!-- 排序选择器 -->
          <el-col :span="6">
            <el-input
              v-model="searchKeyword"
              placeholder="搜索标题、ID、创建者..."
              clearable
              @input="handleSearchChange"
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
          </el-col>
        </el-row>
        
        <!-- 分组和搜索控件 -->
        <el-row :gutter="20" style="margin-bottom: 20px;">
          <!-- 分组选择器 -->
          <el-col :span="6">
            <el-select v-model="groupBy" placeholder="分组方式" @change="applyGrouping">
              <el-option label="不分组" value="" />
              <el-option label="按状态分组" value="status" />
              <el-option label="按严重程度分组" value="severity" />
              <el-option label="按优先级分组" value="pri" />
              <el-option label="按指派人分组" value="assignedTo" />
              <el-option label="按产品分组" value="product" />
              <el-option label="按Bug类型分组" value="bug_type" />
            </el-select>
          </el-col>
          
          <!-- 搜索框 -->
          <el-col :span="10">
            <span><!-- 已移动到上面的行 --></span>
          </el-col>
          
          <!-- 操作按钮 -->
          <el-col :span="8">
            <el-button @click="clearAllFilters">
              <el-icon><Refresh /></el-icon>
              清除筛选
            </el-button>
            <el-button type="primary" @click="exportBugs" :disabled="filteredBugs.length === 0">
              <el-icon><Download /></el-icon>
              导出 ({{ filteredBugs.length }})
            </el-button>
          </el-col>
        </el-row>
        
        <!-- 选中的产品信息 -->
        <div v-if="selectedProducts.length > 0" class="selected-products">
          <el-tag
            v-for="productId in selectedProducts"
            :key="productId"
            closable
            @close="handleRemoveProduct(productId)"
            style="margin-right: 8px; margin-bottom: 8px;"
          >
            {{ getProductName(productId) }}
          </el-tag>
        </div>
        
        <!-- 筛选结果统计 -->
        <div v-if="bugs.length > 0" class="filter-stats">
          <el-tag type="info" size="small">
            显示 {{ filteredBugs.length }} / {{ bugs.length }} 个Bug
          </el-tag>
          <el-tag v-if="searchKeyword" type="warning" size="small" style="margin-left: 8px;">
            搜索: "{{ searchKeyword }}"
          </el-tag>
          <el-tag v-if="filterStatus.length > 0" type="primary" size="small" style="margin-left: 8px;">
            状态: {{ getStatusDisplayText(filterStatus) }}
          </el-tag>
          <el-tag v-if="filterSeverity.length > 0" type="danger" size="small" style="margin-left: 8px;">
            严重程度: {{ getSeverityFilterDisplayText(filterSeverity) }}
          </el-tag>
          <el-tag v-if="filterAssignedTo.length > 0" type="warning" size="small" style="margin-left: 8px;">
            指派人: {{ getAssignedToDisplayText(filterAssignedTo) }}
          </el-tag>
          <el-tag v-if="groupBy" type="success" size="small" style="margin-left: 8px;">
            分组: {{ getGroupByDisplayText(groupBy) }}
          </el-tag>
        </div>
      </div>

      <!-- Bug表格 - 支持分组显示 -->
      <div v-if="groupBy && filteredBugs.length > 0">
        <!-- 分组显示 -->
        <div v-for="(group, groupName) in groupedBugs" :key="groupName" class="bug-group">
          <div class="group-header" @click="toggleGroupCollapse(String(groupName))">
            <div class="group-header-content">
              <el-icon class="collapse-icon">
                <ArrowDown v-if="!isGroupCollapsed(String(groupName))" />
                <ArrowRight v-else />
              </el-icon>
              <el-tag :type="getGroupTagType(groupBy, String(groupName))" size="large">
                {{ getGroupDisplayName(groupBy, String(groupName)) }}
                <span class="group-count">({{ group.length }})</span>
              </el-tag>
            </div>
          </div>
          <el-collapse-transition>
            <el-table 
              v-show="!isGroupCollapsed(String(groupName))"
              :data="group" 
              stripe
              style="width: 100%; margin-bottom: 20px;"
              :empty-text="`该分组暂无数据`"
              :default-sort="{ prop: 'id', order: 'ascending' }"
            >
            <!-- 动态列 - 同原来的表格列配置 -->
            <template v-for="column in visibleColumns" :key="column.key">
              <!-- ID列 -->
              <el-table-column 
                v-if="column.key === 'id'"
                prop="id" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
                sortable
              />
              
              <!-- 所属产品列 -->
              <el-table-column 
                v-else-if="column.key === 'product'"
                prop="product" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ getProductName(row.product) }}
                </template>
              </el-table-column>
              
              <!-- 标题列 -->
              <el-table-column 
                v-else-if="column.key === 'title'"
                prop="title" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
                show-overflow-tooltip
                sortable
              />
              
              <!-- 状态列 -->
              <el-table-column 
                v-else-if="column.key === 'status'"
                prop="status" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
                sortable
                :sort-method="(a: Bug, b: Bug) => a.status.name.localeCompare(b.status.name)"
              >
                <template #default="{ row }">
                  <el-tag :type="getStatusTagType(row.status.code)">
                    {{ row.status.name }}
                  </el-tag>
                </template>
              </el-table-column>
              
              <!-- 严重程度列 -->
              <el-table-column 
                v-else-if="column.key === 'severity'"
                prop="severity" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
                sortable
              >
                <template #default="{ row }">
                  <el-tag :type="getSeverityTagType(row.severity)">
                    {{ getSeverityText(row.severity) }}
                  </el-tag>
                </template>
              </el-table-column>
              
              <!-- 优先级列 -->
              <el-table-column 
                v-else-if="column.key === 'pri'"
                prop="pri" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
                sortable
              >
                <template #default="{ row }">
                  {{ getPriorityText(row.pri) }}
                </template>
              </el-table-column>
              
              <!-- Bug类型列 -->
              <el-table-column 
                v-else-if="column.key === 'bug_type'"
                prop="bug_type" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ getBugTypeText(row.bug_type) }}
                </template>
              </el-table-column>
              
              <!-- 指派给列 -->
              <el-table-column 
                v-else-if="column.key === 'assigned_to'"
                prop="assigned_to" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ row.assignedTo ? row.assignedTo.realname : '未指派' }}
                </template>
              </el-table-column>
              
              <!-- 创建者列 -->
              <el-table-column 
                v-else-if="column.key === 'opened_by'"
                prop="opened_by" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ row.openedBy ? row.openedBy.realname : '未知' }}
                </template>
              </el-table-column>
              
              <!-- 创建时间列 -->
              <el-table-column 
                v-else-if="column.key === 'opened_date'"
                prop="opened_date" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ row.openedDateFormatted }}
                </template>
              </el-table-column>
              
              <!-- 指派时间列 -->
              <el-table-column 
                v-else-if="column.key === 'assigned_date'"
                prop="assigned_date" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ row.assignedDateFormatted || '未指派' }}
                </template>
              </el-table-column>
              
              <!-- 解决者列 -->
              <el-table-column 
                v-else-if="column.key === 'resolved_by'"
                prop="resolved_by" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ row.resolvedBy ? row.resolvedBy.realname : '未解决' }}
                </template>
              </el-table-column>
              
              <!-- 解决时间列 -->
              <el-table-column 
                v-else-if="column.key === 'resolved_date'"
                prop="resolved_date" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ row.resolvedDateFormatted || '未解决' }}
                </template>
              </el-table-column>
              
              <!-- 操作系统列 -->
              <el-table-column 
                v-else-if="column.key === 'os'"
                prop="os" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ row.os || '未指定' }}
                </template>
              </el-table-column>
              
              <!-- 浏览器列 -->
              <el-table-column 
                v-else-if="column.key === 'browser'"
                prop="browser" 
                :label="column.label" 
                :width="column.width"
                :min-width="column.minWidth"
                :fixed="column.fixed"
              >
                <template #default="{ row }">
                  {{ row.browser || '未指定' }}
                </template>
              </el-table-column>
            </template>
            
            <!-- 操作列（固定显示） -->
            <el-table-column label="操作" width="120" fixed="right">
              <template #default="{ row }">
                <el-button type="primary" size="small" @click="handleViewDetail(row.id)">
                  详情
                </el-button>
              </template>
            </el-table-column>
          </el-table>
          </el-collapse-transition>
        </div>
      </div>
      
      <!-- 普通表格显示（不分组） -->
      <el-table 
        v-else
        :data="filteredBugs" 
        v-loading="loading"
        stripe
        style="width: 100%"
        empty-text="请先选择产品"
        :default-sort="{ prop: 'id', order: 'ascending' }"
      >
        <!-- 动态列 -->
        <template v-for="column in visibleColumns" :key="column.key">
          <!-- ID列 -->
          <el-table-column 
            v-if="column.key === 'id'"
            prop="id" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
            sortable
          />
          
          <!-- 所属产品列 -->
          <el-table-column 
            v-else-if="column.key === 'product'"
            prop="product" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
          >
            <template #default="{ row }">
              {{ getProductName(row.product) }}
            </template>
          </el-table-column>
          
          <!-- 标题列 -->
          <el-table-column 
            v-else-if="column.key === 'title'"
            prop="title" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
            show-overflow-tooltip
            sortable
          />
          
          <!-- 状态列 -->
          <el-table-column 
            v-else-if="column.key === 'status'"
            prop="status" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
            sortable
            :sort-method="(a: Bug, b: Bug) => a.status.name.localeCompare(b.status.name)"
          >
            <template #default="{ row }">
              <el-tag :type="getStatusTagType(row.status.code)">
                {{ row.status.name }}
              </el-tag>
            </template>
          </el-table-column>
          
          <!-- 严重程度列 -->
          <el-table-column 
            v-else-if="column.key === 'severity'"
            prop="severity" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
            sortable
          >
            <template #default="{ row }">
              <el-tag :type="getSeverityTagType(row.severity)">
                {{ getSeverityText(row.severity) }}
              </el-tag>
            </template>
          </el-table-column>
          
          <!-- 优先级列 -->
          <el-table-column 
            v-else-if="column.key === 'pri'"
            prop="pri" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
            sortable
          >
            <template #default="{ row }">
              {{ getPriorityText(row.pri) }}
            </template>
          </el-table-column>
          
          <!-- Bug类型列 -->
          <el-table-column 
            v-else-if="column.key === 'bug_type'"
            prop="bug_type" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
          >
            <template #default="{ row }">
              {{ getBugTypeText(row.bug_type) }}
            </template>
          </el-table-column>
          
          <!-- 指派给列 -->
          <el-table-column 
            v-else-if="column.key === 'assigned_to'"
            prop="assigned_to" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
            sortable
            :sort-method="(a: Bug, b: Bug) => (a.assignedTo?.realname || '').localeCompare(b.assignedTo?.realname || '')"
          >
            <template #default="{ row }">
              {{ row.assignedTo ? row.assignedTo.realname : '未指派' }}
            </template>
          </el-table-column>
          
          <!-- 创建者列 -->
          <el-table-column 
            v-else-if="column.key === 'opened_by'"
            prop="opened_by" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
            sortable
            :sort-method="(a: Bug, b: Bug) => (a.openedBy?.realname || '').localeCompare(b.openedBy?.realname || '')"
          >
            <template #default="{ row }">
              {{ row.openedBy ? row.openedBy.realname : '未知' }}
            </template>
          </el-table-column>
          
          <!-- 创建时间列 -->
          <el-table-column 
            v-else-if="column.key === 'opened_date'"
            prop="opened_date" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
            sortable
            :sort-method="(a: Bug, b: Bug) => new Date(a.openedDate || 0).getTime() - new Date(b.openedDate || 0).getTime()"
          >
            <template #default="{ row }">
              {{ row.openedDateFormatted }}
            </template>
          </el-table-column>
          
          <!-- 指派时间列 -->
          <el-table-column 
            v-else-if="column.key === 'assigned_date'"
            prop="assigned_date" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
            sortable
            :sort-method="(a: Bug, b: Bug) => new Date(a.assignedDate || 0).getTime() - new Date(b.assignedDate || 0).getTime()"
          >
            <template #default="{ row }">
              {{ row.assignedDateFormatted || '未指派' }}
            </template>
          </el-table-column>
          
          <!-- 解决者列 -->
          <el-table-column 
            v-else-if="column.key === 'resolved_by'"
            prop="resolved_by" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
          >
            <template #default="{ row }">
              {{ row.resolvedBy ? row.resolvedBy.realname : '未解决' }}
            </template>
          </el-table-column>
          
          <!-- 解决时间列 -->
          <el-table-column 
            v-else-if="column.key === 'resolved_date'"
            prop="resolved_date" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
          >
            <template #default="{ row }">
              {{ row.resolvedDateFormatted || '未解决' }}
            </template>
          </el-table-column>
          
          <!-- 操作系统列 -->
          <el-table-column 
            v-else-if="column.key === 'os'"
            prop="os" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
          >
            <template #default="{ row }">
              {{ row.os || '未指定' }}
            </template>
          </el-table-column>
          
          <!-- 浏览器列 -->
          <el-table-column 
            v-else-if="column.key === 'browser'"
            prop="browser" 
            :label="column.label" 
            :width="column.width"
            :min-width="column.minWidth"
            :fixed="column.fixed"
          >
            <template #default="{ row }">
              {{ row.browser || '未指定' }}
            </template>
          </el-table-column>
        </template>
        
        <!-- 操作列（固定显示） -->
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" size="small" @click="handleViewDetail(row.id)">
              详情
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- Bug详情对话框 -->
    <BugDetail 
      v-model="detailDialogVisible"
      :bug-id="currentBugId"
      :products="products"
    />

    <!-- 列设置对话框 -->
    <el-dialog 
      v-model="columnSettingsVisible" 
      title="列设置" 
      width="50%"
    >
      <div class="column-settings">
        <p style="margin-bottom: 16px; color: #666;">选择要显示的列：</p>
        <div class="column-list">
          <div 
            v-for="column in columns" 
            :key="column.key" 
            class="column-item"
          >
            <el-checkbox 
              v-model="column.visible" 
              :label="column.label"
              @change="updateVisibleColumns"
            />
          </div>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="resetColumns">重置列设置</el-button>
        <el-button @click="resetAllSettings" type="warning">重置所有设置</el-button>
        <el-button type="primary" @click="saveColumnSettings">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, ref, watch, computed } from 'vue';
import { Refresh, Setting, Search, Download, ArrowDown, ArrowRight } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';
import type { Bug, Product } from '../types';
import BugDetail from './BugDetail.vue';

interface Props {
  bugs: Bug[];
  products: Product[];
  loading: boolean;
}

interface Emits {
  (e: 'refresh', productIds: number[]): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// 监听bugs数据变化
watch(() => props.bugs, (newBugs) => {
  console.log('BugList组件接收到bugs数据:', newBugs);
  console.log('bugs数量:', newBugs.length);
  if (newBugs.length > 0) {
    console.log('第一个Bug的opened_date:', newBugs[0].openedBy, '类型:', typeof newBugs[0].openedBy);
    console.log('第一个Bug完整数据:', JSON.stringify(newBugs[0], null, 2));
  }
}, { immediate: true });

// 选中的产品ID列表
const selectedProducts = ref<number[]>([]);

// 列配置
interface ColumnConfig {
  key: string;
  label: string;
  width?: string;
  minWidth?: string;
  fixed?: string;
  visible: boolean;
}

const defaultColumns: ColumnConfig[] = [
  { key: 'id', label: 'ID', width: '80px', visible: true },
  { key: 'product', label: '所属产品', width: '150px', visible: true },
  { key: 'title', label: '标题', minWidth: '200px', visible: true },
  { key: 'status', label: '状态', width: '100px', visible: true },
  { key: 'severity', label: '严重程度', width: '120px', visible: true },
  { key: 'pri', label: '优先级', width: '100px', visible: true },
  { key: 'bug_type', label: 'Bug类型', width: '120px', visible: false },
  { key: 'assigned_to', label: '指派给', width: '120px', visible: true },
  { key: 'opened_by', label: '创建者', width: '120px', visible: true },
  { key: 'opened_date', label: '创建时间', width: '180px', visible: true },
  { key: 'assigned_date', label: '指派时间', width: '180px', visible: false },
  { key: 'resolved_by', label: '解决者', width: '120px', visible: false },
  { key: 'resolved_date', label: '解决时间', width: '180px', visible: false },
  { key: 'os', label: '操作系统', width: '100px', visible: false },
  { key: 'browser', label: '浏览器', width: '100px', visible: false },
];

const columns = ref<ColumnConfig[]>([...defaultColumns]);
const columnSettingsVisible = ref(false);

// 获取可见的列
const visibleColumns = computed(() => columns.value.filter(col => col.visible));

// Bug详情相关
const detailDialogVisible = ref(false);
const currentBugId = ref<number | null>(null);

// 查看Bug详情
const handleViewDetail = (bugId: number) => {
  currentBugId.value = bugId;
  detailDialogVisible.value = true;
};

const handleRefresh = () => {
  emit('refresh', selectedProducts.value);
};

// 产品选择变化时自动刷新Bug列表
const handleProductChange = () => {
  saveFilterState(); // 保存状态
  if (selectedProducts.value.length > 0) {
    handleRefresh();
  }
};

// 全选产品
const handleSelectAll = () => {
  selectedProducts.value = props.products.map(p => p.id);
  saveFilterState(); // 保存状态
  handleRefresh();
};

// 清空选择
const handleClearSelection = () => {
  selectedProducts.value = [];
  saveFilterState(); // 保存状态
};

// 移除单个产品
const handleRemoveProduct = (productId: number) => {
  selectedProducts.value = selectedProducts.value.filter(id => id !== productId);
  saveFilterState(); // 保存状态
  if (selectedProducts.value.length > 0) {
    handleRefresh();
  }
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

// 列设置相关方法
const updateVisibleColumns = () => {
  // 更新可见列时触发重新计算
  visibleColumns.value;
};

const resetColumns = () => {
  columns.value.forEach((column: ColumnConfig) => {
    column.visible = true;
  });
  updateVisibleColumns();
};

// 重置所有设置
const resetAllSettings = () => {
  // 重置列设置
  columns.value.forEach((column: ColumnConfig) => {
    const defaultColumn = defaultColumns.find(col => col.key === column.key);
    if (defaultColumn) {
      column.visible = defaultColumn.visible;
    }
  });
  
  // 重置筛选状态
  selectedProducts.value = [];
  filterStatus.value = [];
  filterSeverity.value = [];
  filterAssignedTo.value = [];
  filterOpenedBy.value = '';
  filterProduct.value = '';
  dateRange.value = null;
  searchKeyword.value = '';
  groupBy.value = '';
  
  // 清除本地存储
  localStorage.removeItem('bugListColumnSettings');
  localStorage.removeItem('bugListFilterState');
  
  updateVisibleColumns();
  ElMessage.success('已重置所有设置');
};

const saveColumnSettings = () => {
  // 保存列设置到本地存储
  const columnSettings = columns.value.map((col: ColumnConfig) => ({
    key: col.key,
    visible: col.visible
  }));
  localStorage.setItem('bugListColumnSettings', JSON.stringify(columnSettings));
  columnSettingsVisible.value = false;
  ElMessage.success('列设置保存成功');
};

// 加载列设置
const loadColumnSettings = () => {
  const savedSettings = localStorage.getItem('bugListColumnSettings');
  if (savedSettings) {
    try {
      const settings = JSON.parse(savedSettings);
      settings.forEach((setting: any) => {
        const column = columns.value.find((col: ColumnConfig) => col.key === setting.key);
        if (column) {
          column.visible = setting.visible;
        }
      });
    } catch (error) {
      console.error('加载列设置失败:', error);
    }
  }
};

// 保存筛选状态
const saveFilterState = () => {
  const filterState = {
    selectedProducts: selectedProducts.value,
    filterStatus: filterStatus.value,
    filterSeverity: filterSeverity.value,
    filterAssignedTo: filterAssignedTo.value,
    groupBy: groupBy.value,
    searchKeyword: searchKeyword.value
  };
  localStorage.setItem('bugListFilterState', JSON.stringify(filterState));
};

// 加载筛选状态
const loadFilterState = () => {
  const savedState = localStorage.getItem('bugListFilterState');
  if (savedState) {
    try {
      const state = JSON.parse(savedState);
      selectedProducts.value = state.selectedProducts || [];
      filterStatus.value = state.filterStatus || [];
      filterSeverity.value = state.filterSeverity || [];
      filterAssignedTo.value = state.filterAssignedTo || [];
      groupBy.value = state.groupBy || '';
      searchKeyword.value = state.searchKeyword || '';
      
      // 如果有选中的产品，自动刷新数据
      if (selectedProducts.value.length > 0) {
        // 使用 nextTick 确保 DOM 更新后再刷新
        setTimeout(() => {
          handleRefresh();
        }, 100);
      }
    } catch (error) {
      console.error('加载筛选状态失败:', error);
    }
  }
};

// 搜索关键字防抖保存
let searchTimeout: number | null = null;
const handleSearchChange = () => {
  if (searchTimeout) {
    clearTimeout(searchTimeout);
  }
  searchTimeout = setTimeout(() => {
    saveFilterState();
  }, 500); // 500ms 防抖
};

// 筛选、排序、分组相关状态
const filterStatus = ref<string[]>([]);
const filterSeverity = ref<number[]>([]);
const filterAssignedTo = ref<string[]>([]);
const filterOpenedBy = ref('');
const filterProduct = ref('');
const dateRange = ref<[Date, Date] | null>(null);
const groupBy = ref('');
const searchKeyword = ref('');

// 分组折叠状态
const collapsedGroups = ref<Set<string>>(new Set());

// 获取所有指派用户
const uniqueAssignedUsers = computed(() => {
  const users = new Set<string>();
  props.bugs.forEach(bug => {
    if (bug.assignedTo?.realname) {
      users.add(bug.assignedTo.realname);
    }
  });
  return Array.from(users);
});

// 筛选后的Bug列表
const filteredBugs = computed(() => {
  let result = [...props.bugs];
  
  // 状态筛选 - 多选支持
  if (filterStatus.value.length > 0) {
    result = result.filter(bug => filterStatus.value.includes(bug.status.code));
  }
  
  // 严重程度筛选 - 多选支持
  if (filterSeverity.value.length > 0) {
    result = result.filter(bug => filterSeverity.value.includes(bug.severity));
  }
  
  // 指派人筛选 - 多选支持
  if (filterAssignedTo.value.length > 0) {
    result = result.filter(bug => {
      // 检查是否包含"未指派"选项
      const hasUnassigned = filterAssignedTo.value.includes('unassigned');
      const isUnassigned = !bug.assignedTo || !bug.assignedTo.realname;
      
      // 检查是否包含具体的用户名
      const hasSpecificUser = bug.assignedTo?.realname && filterAssignedTo.value.includes(bug.assignedTo.realname);
      
      // 满足其中任一条件即可
      return (hasUnassigned && isUnassigned) || hasSpecificUser;
    });
  }
  
  // 创建人筛选
  if (filterOpenedBy.value) {
    result = result.filter(bug => bug.openedBy?.realname === filterOpenedBy.value);
  }
  
  // 产品筛选
  if (filterProduct.value) {
    result = result.filter(bug => bug.product.toString() === filterProduct.value);
  }
  
  // 日期范围筛选
  if (dateRange.value && dateRange.value.length === 2) {
    const [startDate, endDate] = dateRange.value;
    result = result.filter(bug => {
      if (!bug.openedDate) return false;
      const bugDate = new Date(bug.openedDate);
      return bugDate >= startDate && bugDate <= endDate;
    });
  }
  
  // 搜索关键字筛选
  if (searchKeyword.value.trim()) {
    const keyword = searchKeyword.value.toLowerCase().trim();
    result = result.filter(bug => 
      bug.title.toLowerCase().includes(keyword) ||
      bug.id.toString().includes(keyword) ||
      (bug.assignedTo?.realname?.toLowerCase().includes(keyword)) ||
      (bug.openedBy?.realname?.toLowerCase().includes(keyword))
    );
  }
  
  return result;
});

// 分组后的Bug列表
const groupedBugs = computed(() => {
  if (!groupBy.value) return {};
  
  const groups: { [key: string]: Bug[] } = {};
  
  filteredBugs.value.forEach(bug => {
    let groupKey: string;
    
    switch (groupBy.value) {
      case 'status':
        groupKey = bug.status.name;
        break;
      case 'severity':
        groupKey = getSeverityText(bug.severity);
        break;
      case 'assignedTo':
        groupKey = bug.assignedTo?.realname || '未指派';
        break;
      case 'opened_by':
        groupKey = bug.openedBy?.realname || '未知';
        break;
      case 'product':
        groupKey = getProductName(bug.product);
        break;
      case 'pri':
        groupKey = getPriorityText(bug.pri);
        break;
      case 'bug_type':
        groupKey = getBugTypeText(bug.bug_type);
        break;
      default:
        groupKey = '其他';
    }
    
    if (!groups[groupKey]) {
      groups[groupKey] = [];
    }
    groups[groupKey].push(bug);
  });
  
  return groups;
});

// 筛选、排序、分组方法
const applyFilters = () => {
  // 筛选逻辑已在computed中处理
  saveFilterState(); // 保存状态
  console.log('应用筛选条件');
};

const applyGrouping = () => {
  // groupBy.value 已经通过 v-model 自动更新
  saveFilterState(); // 保存状态
  console.log('分组方式已更改为:', groupBy.value);
  // 清空折叠状态
  collapsedGroups.value.clear();
};

// 切换分组折叠状态
const toggleGroupCollapse = (groupName: string) => {
  if (collapsedGroups.value.has(groupName)) {
    collapsedGroups.value.delete(groupName);
  } else {
    collapsedGroups.value.add(groupName);
  }
};

// 判断分组是否折叠
const isGroupCollapsed = (groupName: string) => {
  return collapsedGroups.value.has(groupName);
};

const clearAllFilters = () => {
  filterStatus.value = [];
  filterSeverity.value = [];
  filterAssignedTo.value = [];
  filterOpenedBy.value = '';
  filterProduct.value = '';
  dateRange.value = null;
  searchKeyword.value = '';
  groupBy.value = '';
  saveFilterState(); // 保存状态
  ElMessage.success('已清除所有筛选条件');
};

const exportBugs = () => {
  const data = filteredBugs.value.map(bug => ({
    ID: bug.id,
    标题: bug.title,
    状态: bug.status.name,
    严重程度: getSeverityText(bug.severity),
    优先级: getPriorityText(bug.pri),
    指派给: bug.assignedTo?.realname || '未指派',
    创建者: bug.openedBy?.realname || '未知',
    创建时间: bug.openedDateFormatted,
    产品: getProductName(bug.product)
  }));
  
  const csv = [
    Object.keys(data[0]).join(','),
    ...data.map(row => Object.values(row).join(','))
  ].join('\n');
  
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = `bugs_export_${new Date().toISOString().split('T')[0]}.csv`;
  link.click();
  
  ElMessage.success(`已导出 ${data.length} 条Bug记录`);
};

// 分组相关辅助方法
const getGroupTagType = (groupField: string, groupName: string) => {
  switch (groupField) {
    case 'status':
      if (groupName.includes('激活')) return 'danger';
      if (groupName.includes('已解决')) return 'success';
      if (groupName.includes('已关闭')) return 'info';
      return 'warning';
    case 'severity':
      if (groupName.includes('致命')) return 'danger';
      if (groupName.includes('严重')) return 'warning';
      if (groupName.includes('一般')) return 'info';
      return 'success';
    default:
      return 'primary';
  }
};

const getGroupDisplayName = (_groupField: string, groupName: string) => {
  return groupName;
};

// 获取状态显示文本
const getStatusDisplayText = (statusCodes: string[]) => {
  if (statusCodes.length === 0) return '';
  const statusMap: { [key: string]: string } = {
    'active': '激活',
    'resolved': '已解决',
    'closed': '已关闭',
    'testing': '测试中'
  };
  return statusCodes.map(code => statusMap[code] || code).join(', ');
};

// 获取严重程度筛选显示文本
const getSeverityFilterDisplayText = (severityFilters: number[]) => {
  if (severityFilters.length === 0) return '';
  return severityFilters.map(severity => getSeverityText(severity)).join(', ');
};

// 获取指派人筛选显示文本
const getAssignedToDisplayText = (assignedToFilters: string[]) => {
  if (assignedToFilters.length === 0) return '';
  return assignedToFilters.map(filter => {
    if (filter === 'unassigned') return '未指派';
    return filter;
  }).join(', ');
};

// 获取分组显示文本
const getGroupByDisplayText = (groupField: string) => {
  const groupMap: { [key: string]: string } = {
    'status': '状态',
    'severity': '严重程度',
    'assignedTo': '指派人',
    'opened_by': '创建者',
    'product': '产品',
    'pri': '优先级',
    'bug_type': 'Bug类型'
  };
  return groupMap[groupField] || groupField;
};

// 初始化时加载列设置和筛选状态
loadColumnSettings();
loadFilterState();
</script>

<style scoped>
.bug-list-container {
  max-width: 1400px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.filter-section {
  margin-bottom: 20px;
}

.selected-products {
  margin-top: 10px;
  padding: 10px 0;
  border-top: 1px solid #e4e7ed;
}

.column-settings {
  padding: 10px 0;
}

.column-list {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  max-height: 400px;
  overflow-y: auto;
}

.column-item {
  padding: 8px 12px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  background-color: #fafafa;
  transition: background-color 0.2s;
}

.column-item:hover {
  background-color: #f0f2f5;
}

/* 新增的筛选、排序、分组样式 */
.filter-controls {
  margin-bottom: 16px;
  padding: 16px;
  background: #f5f7fa;
  border-radius: 8px;
  border: 1px solid #e4e7ed;
}

.filter-row {
  margin-bottom: 12px;
}

.filter-actions {
  margin-top: 12px;
}

.filter-stats {
  margin-bottom: 16px;
}

.filter-stats .el-tag {
  margin-right: 8px;
  margin-bottom: 4px;
}

.bug-group {
  margin-bottom: 24px;
}

.group-header {
  margin-bottom: 12px;
  padding: 8px 12px;
  background: linear-gradient(to right, #f0f9ff, #e0f2fe);
  border-radius: 6px;
  border-left: 4px solid #0ea5e9;
  cursor: pointer;
  user-select: none;
  transition: all 0.2s ease;
}

.group-header:hover {
  background: linear-gradient(to right, #e0f7fa, #b3e5fc);
  transform: translateX(2px);
}

.group-header-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.collapse-icon {
  transition: transform 0.2s ease;
  color: #0ea5e9;
  font-size: 16px;
}

.group-count {
  font-weight: bold;
  margin-left: 4px;
}

.search-export-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.export-section {
  display: flex;
  gap: 8px;
}

.stats-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.stat-card {
  text-align: center;
  padding: 12px;
  background: white;
  border-radius: 6px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  border: 1px solid #e4e7ed;
}

.stat-number {
  font-size: 24px;
  font-weight: bold;
  color: #409eff;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 12px;
  color: #909399;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
