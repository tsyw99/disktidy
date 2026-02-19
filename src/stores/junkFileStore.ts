/**
 * 零碎文件扫描状态管理
 * 
 * 优化点：
 * 1. 监听器初始化非阻塞：后台异步初始化，不等待完成
 * 2. 错误处理增强：捕获并显示扫描过程中的错误
 * 3. 进度实时更新：正确处理进度百分比
 */

import { create } from 'zustand';
import { junkFileScanService } from '../services/junkFileScanService';
import type {
  JunkScanResult,
  JunkScanOptions,
  JunkFileScanProgress,
} from '../types/fileAnalyzer';

// 监听器状态跟踪
let junkListenersInitialized = false;
let junkListenersInitializing = false;
let progressListener: (() => void) | null = null;
let completeListener: (() => void) | null = null;

interface JunkFileState {
  isScanning: boolean;
  isCompleted: boolean; // 新增：跟踪扫描是否完成
  scanProgress: JunkFileScanProgress | null;
  scanId: string | null;
  error: string | null;
  results: JunkScanResult[];
  selectedFiles: Set<string>;
  options: JunkScanOptions;
  selectedDisk: string;
  expandedTypes: Set<string>;
  
  setSelectedDisk: (disk: string) => void;
  setOptions: (options: Partial<JunkScanOptions>) => void;
  toggleFileSelection: (path: string) => void;
  toggleTypeSelection: (fileType: string) => void;
  selectAll: () => void;
  deselectAll: () => void;
  toggleTypeExpand: (fileType: string) => void;
  expandAll: () => void;
  collapseAll: () => void;
  removeFiles: (paths: string[]) => void;
  startScan: () => Promise<void>;
  pauseScan: () => Promise<void>;
  resumeScan: () => Promise<void>;
  cancelScan: () => Promise<void>;
  reset: () => void;
  initListeners: () => Promise<void>;
  cleanup: () => void;
}

const defaultOptions: JunkScanOptions = {
  scan_paths: [],
  include_empty_folders: true,
  include_invalid_shortcuts: true,
  include_old_logs: true,
  include_old_installers: true,
  include_invalid_downloads: true,
  include_small_files: false,
  small_file_max_size: 1024 * 100,
  log_max_age_days: 30,
  installer_max_age_days: 90,
  exclude_paths: [],
  include_hidden: false,
  include_system: false,
};

/**
 * 初始化事件监听器（非阻塞版本）
 * 快速返回，在后台完成监听器注册
 */
async function initListenersAsync(): Promise<void> {
  if (junkListenersInitialized || junkListenersInitializing) {
    return;
  }
  
  junkListenersInitializing = true;
  
  try {
    // 先清理旧的监听器
    if (progressListener) {
      progressListener();
      progressListener = null;
    }
    if (completeListener) {
      completeListener();
      completeListener = null;
    }

    // 注册进度监听器
    progressListener = await junkFileScanService.onProgress((progress) => {
      const state = useJunkFileStore.getState();
      const { scanId } = state;
      
      // 验证扫描ID匹配
      if (!scanId || progress.scanId === scanId) {
        const progressStatus = progress.status;
        
        // 忽略空闲状态
        if (progressStatus === 'idle') {
          return;
        }
        
        const isCompleted = progressStatus === 'completed';
        const isError = progressStatus === 'error';
        
        // 更新状态 - 注意：完成时不清理 scanId，由 completeListener 处理
        useJunkFileStore.setState({
          scanProgress: isCompleted || isError ? null : progress,
          isScanning: progressStatus === 'scanning' || progressStatus === 'paused',
          error: isError ? '扫描过程中发生错误' : null,
        });
      }
    });

    // 注册完成监听器
    completeListener = await junkFileScanService.onComplete((result) => {
      const state = useJunkFileStore.getState();
      const currentScanId = state.scanId;

      // 创建完成状态的进度对象
      const completedProgress: JunkFileScanProgress = {
        scanId: currentScanId || '',
        currentPath: '',
        scannedFiles: 0,
        foundFiles: result.reduce((sum, r) => sum + r.count, 0),
        scannedSize: 0,
        totalSize: result.reduce((sum, r) => sum + r.total_size, 0),
        percent: 100,
        currentPhase: '扫描完成',
        status: 'completed',
        speed: 0,
      };
      
      // 保存结果并更新状态 - 保留 scanId 用于分页加载
      useJunkFileStore.setState({
        results: result,
        isScanning: false,
        isCompleted: true,
        scanProgress: completedProgress,
        // 注意：不清空 scanId，保留它用于分页加载
        error: null,
      });
    });
    
    junkListenersInitialized = true;
    junkListenersInitializing = false;
  } catch (error) {
    console.error('[JunkFileStore] Failed to initialize listeners:', error);
    junkListenersInitializing = false;
    throw error;
  }
}

export const useJunkFileStore = create<JunkFileState>((set, get) => ({
  isScanning: false,
  isCompleted: false,
  scanProgress: null,
  scanId: null,
  error: null,
  results: [],
  selectedFiles: new Set(),
  options: defaultOptions,
  selectedDisk: 'C:\\',
  expandedTypes: new Set(),

  setSelectedDisk: (disk) => set({ selectedDisk: disk }),

  setOptions: (newOptions) => {
    const { options } = get();
    set({ options: { ...options, ...newOptions } });
  },

  toggleFileSelection: (path) => {
    const { selectedFiles } = get();
    const newSet = new Set(selectedFiles);
    if (newSet.has(path)) {
      newSet.delete(path);
    } else {
      newSet.add(path);
    }
    set({ selectedFiles: newSet });
  },

  toggleTypeSelection: (fileType) => {
    const { results, selectedFiles } = get();
    const typeResult = results.find(r => r.file_type === fileType);
    if (!typeResult) return;

    const newSet = new Set(selectedFiles);
    const allSelected = typeResult.items.every(item => newSet.has(item.path));
    
    if (allSelected) {
      typeResult.items.forEach(item => newSet.delete(item.path));
    } else {
      typeResult.items.forEach(item => newSet.add(item.path));
    }
    set({ selectedFiles: newSet });
  },

  selectAll: () => {
    const { results } = get();
    const allPaths = new Set<string>();
    results.forEach(result => {
      result.items.forEach(item => allPaths.add(item.path));
    });
    set({ selectedFiles: allPaths });
  },

  deselectAll: () => set({ selectedFiles: new Set() }),

  toggleTypeExpand: (fileType) => {
    const { expandedTypes } = get();
    const newSet = new Set(expandedTypes);
    if (newSet.has(fileType)) {
      newSet.delete(fileType);
    } else {
      newSet.add(fileType);
    }
    set({ expandedTypes: newSet });
  },

  expandAll: () => {
    const { results } = get();
    const allTypes = new Set(results.map(r => r.file_type));
    set({ expandedTypes: allTypes });
  },

  collapseAll: () => set({ expandedTypes: new Set() }),

  removeFiles: (paths) => {
    const { results, selectedFiles } = get();
    const pathSet = new Set(paths);
    
    const newResults = results.map(result => ({
      ...result,
      items: result.items.filter(item => !pathSet.has(item.path)),
      count: result.items.filter(item => !pathSet.has(item.path)).length,
      total_size: result.items
        .filter(item => !pathSet.has(item.path))
        .reduce((sum, item) => sum + item.size, 0),
    })).filter(result => result.items.length > 0);
    
    const newSelectedFiles = new Set(selectedFiles);
    paths.forEach(p => newSelectedFiles.delete(p));
    
    set({
      results: newResults,
      selectedFiles: newSelectedFiles,
    });
  },

  initListeners: async () => {
    // 非阻塞初始化：立即返回，后台完成监听器注册
    initListenersAsync().catch(err => {
      console.error('[JunkFileStore] Listener initialization failed:', err);
    });
  },

  startScan: async () => {
    // 确保监听器已初始化
    if (!junkListenersInitialized) {
      await initListenersAsync();
    }

    const { selectedDisk, options } = get();

    // 设置初始状态（快速响应UI）
    const initialProgress: JunkFileScanProgress = {
      scanId: '',
      currentPath: '',
      scannedFiles: 0,
      foundFiles: 0,
      scannedSize: 0,
      totalSize: 0,
      percent: 0,
      currentPhase: '正在启动扫描...',
      status: 'scanning',
      speed: 0,
    };

    set({
      isScanning: true,
      isCompleted: false,
      scanProgress: initialProgress,
      error: null,
      results: [],
      selectedFiles: new Set(),
      expandedTypes: new Set(),
      scanId: null,
    });

    try {
      const scanOptions = {
        ...options,
        scan_paths: [selectedDisk],
      };

      const scanId = await junkFileScanService.start(scanOptions);

      if (!scanId) {
        throw new Error('后端未返回扫描ID');
      }
      
      set({ scanId });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '扫描启动失败';
      console.error('[JunkFileStore] Scan start failed:', errorMessage);
      set({
        error: errorMessage,
        isScanning: false,
        isCompleted: false,
        scanId: null,
        scanProgress: null,
      });
    }
  },

  pauseScan: async () => {
    const { scanId, scanProgress } = get();
    if (!scanId) return;

    try {
      await junkFileScanService.pause(scanId);
      if (scanProgress) {
        set({
          scanProgress: { ...scanProgress, status: 'paused' },
        });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '暂停扫描失败';
      console.error('[JunkFileStore] Pause failed:', errorMessage);
      set({ error: errorMessage });
    }
  },

  resumeScan: async () => {
    const { scanId, scanProgress } = get();
    if (!scanId) return;

    try {
      await junkFileScanService.resume(scanId);
      if (scanProgress) {
        set({
          scanProgress: { ...scanProgress, status: 'scanning' },
        });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '恢复扫描失败';
      console.error('[JunkFileStore] Resume failed:', errorMessage);
      set({ error: errorMessage });
    }
  },

  cancelScan: async () => {
    const { scanId } = get();
    if (!scanId) return;

    try {
      await junkFileScanService.cancel(scanId);
      set({
        isScanning: false,
        isCompleted: false,
        scanProgress: null,
        scanId: null,
        results: [],
        selectedFiles: new Set(),
        error: null,
      });
    } catch (error) {
      console.error('[JunkFileStore] Cancel failed:', error);
      // 即使取消失败，也重置状态
      set({
        isScanning: false,
        isCompleted: false,
        scanProgress: null,
        scanId: null,
        error: '取消扫描失败',
      });
    }
  },

  reset: () => {
    set({
      isScanning: false,
      isCompleted: false,
      scanProgress: null,
      scanId: null,
      error: null,
      results: [],
      selectedFiles: new Set(),
      expandedTypes: new Set(),
    });
  },

  cleanup: () => {
    const { scanId } = get();
    if (scanId) {
      junkFileScanService.clearResult(scanId).catch(() => {});
    }
    // 清理监听器
    if (progressListener) {
      progressListener();
      progressListener = null;
    }
    if (completeListener) {
      completeListener();
      completeListener = null;
    }
    junkListenersInitialized = false;
    junkListenersInitializing = false;
  },
}));
