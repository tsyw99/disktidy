export const APP_NAME = 'DiskTidy';
export const APP_VERSION = '1.0.1 测试版';

export const SCAN_MODES = {
  QUICK: 'quick',
  FULL: 'full',
  CUSTOM: 'custom',
} as const;

export const FILE_CATEGORIES = {
  SYSTEM_TEMP: 'system_temp',
  BROWSER_CACHE: 'browser_cache',
  APP_CACHE: 'app_cache',
  LOG_FILES: 'log_files',
  RECYCLE_BIN: 'recycle_bin',
  LARGE_FILES: 'large_files',
  DUPLICATES: 'duplicates',
} as const;

export const CATEGORY_DISPLAY_NAMES: Record<string, string> = {
  system_temp: '系统临时文件',
  browser_cache: '浏览器缓存',
  app_cache: '应用程序缓存',
  log_files: '日志文件',
  recycle_bin: '回收站',
  large_files: '大文件',
  duplicates: '重复文件',
};

export const PROTECTED_PATHS = [
  'C:\\Windows',
  'C:\\Program Files',
  'C:\\Program Files (x86)',
  'C:\\ProgramData',
];

export const DEFAULT_EXCLUDE_PATHS = [
  'C:\\$Recycle.Bin',
  'C:\\System Volume Information',
];

export const ENABLED_APPS: string[] = ['wechat'];

export const APP_CONFIGS = [
  { id: 'wechat', name: '微信', color: '#07C160', bgColor: 'bg-[#07C160]/10' },
  { id: 'dingtalk', name: '钉钉', color: '#0089FF', bgColor: 'bg-[#0089FF]/10' },
  { id: 'qq', name: 'QQ', color: '#12B7F5', bgColor: 'bg-[#12B7F5]/10' },
  { id: 'wework', name: '企业微信', color: '#2B7EFF', bgColor: 'bg-[#2B7EFF]/10' },
] as const;

export const CATEGORY_CONFIGS = [
  { id: 'chat_images', name: '聊天图片', description: '聊天过程中发送和接收的图片，包括加密的.dat文件' },
  { id: 'video_files', name: '视频文件', description: '聊天过程中发送和接收的视频文件' },
  { id: 'document_files', name: '文档文件', description: '聊天过程中接收的文档、压缩包等文件' },
  { id: 'install_packages', name: '安装包', description: '接收的安装包文件（apk、exe等）' },
  { id: 'cache_data', name: '缓存数据', description: 'HTTP资源缓存、小程序图标缓存等' },
  { id: 'emoji_cache', name: '表情缓存', description: '表情包缓存文件' },
  { id: 'temp_files', name: '临时文件', description: '临时文件，可安全清理' },
  { id: 'thumb_cache', name: '缩略图缓存', description: '图片和视频的缩略图缓存' },
] as const;

export const FILE_EXTENSION_GROUPS = {
  video: ['.mp4', '.avi', '.mkv', '.mov', '.wmv', '.flv', '.webm'],
  archive: ['.zip', '.rar', '.7z', '.tar', '.gz', '.bz2'],
  diskImage: ['.iso', '.img'],
  executable: ['.exe', '.msi', '.pkg', '.deb', '.rpm'],
  audio: ['.mp3', '.wav', '.flac', '.aac', '.ogg'],
  image: ['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.webp', '.svg'],
  document: ['.pdf', '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx'],
} as const;

export const FILE_TYPE_COLORS = {
  video: 'text-purple-400',
  archive: 'text-yellow-400',
  diskImage: 'text-orange-400',
  executable: 'text-green-400',
  audio: 'text-pink-400',
  image: 'text-cyan-400',
  document: 'text-blue-400',
  default: 'text-gray-400',
} as const;
