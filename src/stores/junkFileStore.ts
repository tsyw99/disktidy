/**
 * 零碎文件扫描状态管理
 */

import { create } from 'zustand';
import { junkFileScanService } from '../services/junkFileScanService';
import type {
  JunkScanResult,
  JunkScanOptions,
  JunkFileScanProgress,
} from '../types/fileAnalyzer';

let junkListenersInitialized = false;
let junkListenersInitPromise: Promise<void> | null = null;

interface JunkFileState {
  isScanning: boolean;
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

async function ensureJunkListenersInitialized(): Promise<void> {
  if (junkListenersInitialized) {
    return;
  }
  
  if (junkListenersInitPromise) {
    return junkListenersInitPromise;
  }

  junkListenersInitPromise = (async () => {
    await junkFileScanService.onProgress((progress) => {
      console.log('[JunkFileStore] Received progress:', JSON.stringify(progress, null, 2));
      const state = useJunkFileStore.getState();
      const { scanId } = state;
      
      if (!scanId || progress.scanId === scanId) {
        const progressStatus = progress.status;
        if (progressStatus === 'idle') {
          return;
        }
        useJunkFileStore.setState({
          scanProgress: progress,
          isScanning: progressStatus === 'scanning' || progressStatus === 'paused',
        });
      }
    });

    await junkFileScanService.onComplete((result) => {
      console.log('[JunkFileStore] Received complete:', result);
      const state = useJunkFileStore.getState();
      const { scanId } = state;
      if (scanId) {
        useJunkFileStore.setState({
          results: result,
          isScanning: false,
          scanProgress: null,
        });
      }
    });
    
    junkListenersInitialized = true;
  })();

  return junkListenersInitPromise;
}

export const useJunkFileStore = create<JunkFileState>((set, get) => ({
  isScanning: false,
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
    await ensureJunkListenersInitialized();
  },

  startScan: async () => {
    await ensureJunkListenersInitialized();

    const { selectedDisk, options } = get();

    const initialProgress: JunkFileScanProgress = {
      scanId: '',
      currentPath: '',
      scannedFiles: 0,
      foundFiles: 0,
      scannedSize: 0,
      totalSize: 0,
      percent: 0,
      currentPhase: '',
      status: 'scanning',
      speed: 0,
    };

    set({
      isScanning: true,
      scanProgress: initialProgress,
      error: null,
      results: [],
      selectedFiles: new Set(),
      expandedTypes: new Set(),
      scanId: null,
    });

    try {
      const scanId = await junkFileScanService.start({
        ...options,
        scan_paths: [selectedDisk],
      });

      if (!scanId) {
        throw new Error('后端未返回 scanId');
      }
      
      set({ scanId });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : '扫描失败';
      set({
        error: errorMessage,
        isScanning: false,
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
    } catch {
      // 静默处理
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
    } catch {
      // 静默处理
    }
  },

  cancelScan: async () => {
    const { scanId } = get();
    if (!scanId) return;

    try {
      await junkFileScanService.cancel(scanId);
      set({
        isScanning: false,
        scanProgress: null,
        scanId: null,
        results: [],
        selectedFiles: new Set(),
      });
    } catch {
      // 静默处理
    }
  },

  reset: () => {
    set({
      isScanning: false,
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
  },
}));
