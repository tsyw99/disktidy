/**
 * 大文件管理状态管理
 * 
 * 使用 Tauri 事件系统接收实时进度更新
 */

import { create } from 'zustand';
import { largeFileService } from '../services/largeFileService';
import type {
  LargeFile,
  LargeFileScanProgress,
  LargeFileFilter,
} from '../types/largeFile';
import { DEFAULT_LARGE_FILE_FILTER } from '../types/largeFile';

let listenersInitialized = false;
let listenersInitPromise: Promise<void> | null = null;

interface LargeFileState {
  isScanning: boolean;
  scanProgress: LargeFileScanProgress | null;
  scanId: string | null;
  error: string | null;
  files: LargeFile[];
  filteredFiles: LargeFile[];
  selectedFiles: Set<string>;
  filter: LargeFileFilter;
  searchQuery: string;
  selectedDisk: string;
  setSelectedDisk: (disk: string) => void;
  setFilter: (filter: LargeFileFilter) => void;
  setSearchQuery: (query: string) => void;
  toggleFileSelection: (path: string) => void;
  selectAll: () => void;
  deselectAll: () => void;
  applyFilters: () => void;
  removeFiles: (paths: string[]) => void;
  startScan: () => Promise<void>;
  pauseScan: () => Promise<void>;
  resumeScan: () => Promise<void>;
  cancelScan: () => Promise<void>;
  initListeners: () => Promise<void>;
  cleanup: () => void;
}

// 初始化事件监听器（只执行一次）
async function ensureListenersInitialized(): Promise<void> {
  if (listenersInitialized) {
    return;
  }
  
  if (listenersInitPromise) {
    return listenersInitPromise;
  }

  listenersInitPromise = (async () => {
    await largeFileService.onProgress((progress) => {
      const state = useLargeFileStore.getState();
      const { scanId } = state;
      
      if (!scanId || progress.scanId === scanId) {
        const progressStatus = progress.status;
        if (progressStatus === 'idle') {
          return;
        }
        useLargeFileStore.setState({
          scanProgress: progress,
          isScanning: progressStatus === 'scanning' || progressStatus === 'paused',
        });
      }
    });

    await largeFileService.onComplete((result) => {
      const state = useLargeFileStore.getState();
      const { scanId } = state;
      if (scanId && result.scan_id === scanId) {
        useLargeFileStore.setState({
          files: result.files,
          filteredFiles: result.files,
          isScanning: false,
          scanProgress: null,
        });
      }
    });
    
    listenersInitialized = true;
  })();

  return listenersInitPromise;
}

export const useLargeFileStore = create<LargeFileState>((set, get) => ({
  isScanning: false,
  scanProgress: null,
  scanId: null,
  error: null,
  files: [],
  filteredFiles: [],
  selectedFiles: new Set(),
  filter: DEFAULT_LARGE_FILE_FILTER,
  searchQuery: '',
  selectedDisk: 'C:\\',

  setSelectedDisk: (disk) => set({ selectedDisk: disk }),

  setFilter: (filter) => {
    set({ filter });
    get().applyFilters();
  },

  setSearchQuery: (query) => {
    set({ searchQuery: query });
    get().applyFilters();
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

  selectAll: () => {
    const { filteredFiles } = get();
    set({ selectedFiles: new Set(filteredFiles.map((f) => f.path)) });
  },

  deselectAll: () => set({ selectedFiles: new Set() }),

  applyFilters: () => {
    const { files, searchQuery, filter } = get();
    let filtered = [...files];

    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase();
      filtered = filtered.filter(
        (file) =>
          file.name.toLowerCase().includes(query) ||
          file.path.toLowerCase().includes(query) ||
          file.extension.toLowerCase().includes(query)
      );
    }

    if (filter.fileTypes && filter.fileTypes.length > 0) {
      filtered = filtered.filter((file) =>
        filter.fileTypes!.includes(file.extension.toLowerCase())
      );
    }

    set({ filteredFiles: filtered });
  },

  removeFiles: (paths: string[]) => {
    const { files, filteredFiles, selectedFiles } = get();
    const pathSet = new Set(paths);
    const newFiles = files.filter((f) => !pathSet.has(f.path));
    const newFilteredFiles = filteredFiles.filter((f) => !pathSet.has(f.path));
    const newSelectedFiles = new Set(selectedFiles);
    paths.forEach((p) => newSelectedFiles.delete(p));
    set({
      files: newFiles,
      filteredFiles: newFilteredFiles,
      selectedFiles: newSelectedFiles,
    });
  },

  initListeners: async () => {
    await ensureListenersInitialized();
  },

  startScan: async () => {
    await ensureListenersInitialized();

    const { selectedDisk, filter } = get();

    const initialProgress: LargeFileScanProgress = {
      scanId: '',
      status: 'scanning',
      currentPath: '',
      scannedFiles: 0,
      foundFiles: 0,
      scannedSize: 0,
      totalSize: 0,
      percent: 0,
      speed: 0,
    };

    set({
      isScanning: true,
      scanProgress: initialProgress,
      error: null,
      files: [],
      filteredFiles: [],
      selectedFiles: new Set(),
      scanId: null,
    });

    try {
      const minSizeBytes =
        filter.unit === 'GB'
          ? filter.minSize * 1024 * 1024 * 1024
          : filter.minSize * 1024 * 1024;

      const scanId = await largeFileService.startScan({
        path: selectedDisk,
        minSizeBytes,
        excludePaths: [],
        includeHidden: false,
        includeSystem: false,
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
      await largeFileService.pauseScan(scanId);
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
      await largeFileService.resumeScan(scanId);
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
      await largeFileService.cancelScan(scanId);
      set({
        isScanning: false,
        scanProgress: null,
        scanId: null,
        files: [],
        filteredFiles: [],
        selectedFiles: new Set(),
      });
    } catch {
      // 静默处理
    }
  },

  cleanup: () => {
    const { scanId } = get();
    if (scanId) {
      largeFileService.clearResult(scanId).catch(() => {});
    }
  },
}));
