export type AppType = 'wechat' | 'dingtalk' | 'qq' | 'wework';

export type CleanCategory = 
  | 'chat_images' 
  | 'video_files' 
  | 'document_files' 
  | 'install_packages' 
  | 'cache_data' 
  | 'voice_files' 
  | 'emoji_cache'
  | 'temp_files'
  | 'thumb_cache';

export type AppCacheScanStatus = 'idle' | 'scanning' | 'paused' | 'completed' | 'cancelled' | 'error';

export interface AppCacheFile {
  id: string;
  path: string;
  name: string;
  size: number;
  category: CleanCategory;
  app: AppType;
  chatObject: string;
  createdAt: number;
  modifiedAt: number;
  selected: boolean;
  isEncrypted: boolean;
  originalFormat: string | null;
}

export interface AppCacheScanProgress {
  scanId: string;
  status: AppCacheScanStatus;
  currentPath: string;
  scannedFiles: number;
  scannedSize: number;
  totalFiles: number;
  totalSize: number;
  percent: number;
  speed: number;
  currentApp: string;
  incremental: boolean;
  skippedFiles: number;
}

export interface AppCacheScanResult {
  scanId: string;
  files: AppCacheFile[];
  totalFiles: number;
  totalSize: number;
  durationMs: number;
  status: AppCacheScanStatus;
  incremental: boolean;
  skippedFiles: number;
}

export interface AppCacheScanOptions {
  apps: string[];
  categories: string[];
  incremental?: boolean;
  forceRescan?: boolean;
}

export interface AppConfig {
  paths: Record<string, string>;
}

export const EVENT_APP_CACHE_PROGRESS = 'app_cache:progress';
export const EVENT_APP_CACHE_COMPLETE = 'app_cache:complete';

export const APP_DISPLAY_NAMES: Record<AppType, string> = {
  wechat: '微信',
  dingtalk: '钉钉',
  qq: 'QQ',
  wework: '企业微信',
};

export const APP_PATH_HINTS: Record<AppType, string> = {
  wechat: '微信文件存储路径，通常包含 wxid_ 开头的用户文件夹',
  dingtalk: '钉钉文件存储路径（开发中）',
  qq: 'QQ文件存储路径（开发中）',
  wework: '企业微信文件存储路径（开发中）',
};

export const APP_ENABLED_STATUS: Record<AppType, boolean> = {
  wechat: true,
  dingtalk: false,
  qq: false,
  wework: false,
};

export const CLEAN_CATEGORY_DISPLAY_NAMES: Record<CleanCategory, string> = {
  chat_images: '聊天图片',
  video_files: '视频文件',
  document_files: '文档文件',
  install_packages: '安装包',
  cache_data: '缓存数据',
  voice_files: '语音文件',
  emoji_cache: '表情缓存',
  temp_files: '临时文件',
  thumb_cache: '缩略图缓存',
};

export const CLEAN_CATEGORY_DESCRIPTIONS: Record<CleanCategory, string> = {
  chat_images: '聊天过程中发送和接收的图片，包括加密的.dat文件',
  video_files: '聊天过程中发送和接收的视频文件',
  document_files: '聊天过程中接收的文档、压缩包等文件',
  install_packages: '接收的安装包文件（apk、exe等）',
  cache_data: 'HTTP资源缓存、小程序图标缓存等',
  voice_files: '聊天语音消息文件',
  emoji_cache: '表情包缓存文件',
  temp_files: '临时文件，可安全清理',
  thumb_cache: '图片和视频的缩略图缓存',
};
