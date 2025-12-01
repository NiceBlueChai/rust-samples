// API服务层 - 封装所有与后端的交互
import { invoke } from "@tauri-apps/api/core";
import type { Bug, UserInfo, Product, ZentaoConfig } from '../types';

export class ZentaoApiService {
  // 应用初始化
  static async initializeApp(): Promise<boolean> {
    return await invoke('initialize_app');
  }

  // 登录相关
  static async login(baseUrl: string, account: string, password: string): Promise<boolean> {
    return await invoke('login_zentao', { baseUrl, account, password });
  }

  static async logout(): Promise<void> {
    return await invoke('logout_zentao');
  }

  static async checkLoginStatus(): Promise<boolean> {
    return await invoke('check_login_status');
  }

  // 用户信息
  static async getUserInfo(): Promise<UserInfo> {
    return await invoke('get_user_info');
  }

  // 产品相关
  static async getProducts(): Promise<Product[]> {
    return await invoke('get_products');
  }

  static async getProductDetail(productId: number): Promise<Product> {
    return await invoke('get_product_detail', { productId });
  }

  // Bug相关
  static async getBugs(productIds: number[]): Promise<Bug[]> {
    return await invoke('get_bugs_by_product', { productIds });
  }

  static async getBugDetail(bugId: number): Promise<Bug> {
    return await invoke('get_bug_detail', { bugId });
  }

  // 图片相关
  static async getImage(imagePath: string): Promise<string> {
    return await invoke('get_image', { imagePath });
  }

  // 配置相关
  static async saveConfig(config: ZentaoConfig): Promise<void> {
    return await invoke('save_config', { config });
  }

  static async loadConfig(): Promise<ZentaoConfig | null> {
    return await invoke('load_config');
  }
}
