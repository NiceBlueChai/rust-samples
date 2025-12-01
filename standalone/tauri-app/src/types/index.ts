// 禅道API相关类型定义

export interface BugStatus {
  code: string;
  name: string;
}

export interface Bug {
  id: number;
  product: number;
  branch: number;
  module: number;
  project: number;
  execution: number;
  to_task: number;
  to_story: number;
  title: string;
  keywords: string;
  severity: number;
  pri: number;
  bug_type: string;
  os: string;
  browser: string;
  steps: string;
  stepsProcessed?: string; // 处理过图片路径的steps
  task?: number;
  story?: number;
  openedBy?: User;
  openedDate: string;
  openedDateFormatted: string; // 格式化后的创建时间
  deadline?: string;
  assignedTo?: User;
  assignedDate?: string;
  assignedDateFormatted?: string; // 格式化后的分配时间
  resolvedBy?: User;
  resolvedDate?: string;
  resolvedDateFormatted?: string; // 格式化后的解决时间
  resolved_build?: string;
  closed_by?: User;
  closed_date?: string;
  closedDateFormatted?: string; // 格式化后的关闭时间
  status: BugStatus;
}

export interface BugPaginatedResponse {
  page: number;
  total: number;
  limit: number;
  bugs: Bug[];
}

export interface UserInfo {
  id: number;
  user_type: string; // 'inside' | 'outside'
  dept: number;
  account: string;
  realname: string;
  nickname?: string;
  avatar?: string;
  birthday?: string;
  gender?: string; // 'f' | 'm'
  mobile?: string;
  phone?: string;
  weixin?: string;
  address?: string;
  join?: string;
  admin: boolean;
}

export interface User {
  id: number;
  account: string;
  avatar: string;
  realname: string;
}

export interface StoryStats {
  [key: string]: string | number;
}

export interface RequirementStats {
  [key: string]: string | number;
}

export interface Product {
  id: number;
  program: number;
  name: string;
  code: string;  
  line: number;
  PO?: User;
  QD?: User;
  RD?: User;
  type: string; // 'normal' | 'branch' | 'platform'
  desc: string;
  acl: string; // 'open' | 'private'
  whitelist?: User[];
  createdBy: User;
  createdDate: string;
}

export interface ProductListResponse {
  page: number;
  total: number;
  limit: number;
  products: Product[];
}

export interface LoginRequest {
  account: string;
  password: string;
}

export interface LoginResponse {
  token: string;
}

export interface ZentaoConfig {
  baseUrl: string;
  account: string;
  password: string;
}
