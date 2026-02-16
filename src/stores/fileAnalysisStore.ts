import { create } from 'zustand';
import { UnlistenFn } from '@tauri-apps/api/event';
import { largeFileScanService } from '../services/largeFileScanService';
import type { LargeFile, JunkScanResult, JunkFileItem, LargeFileFilter, LargeFileAnalysisResult, LargeFileScanProgress } from '../types';

interface FileAnalysisState {
  activeTab: 'large' | 'junk';
  selectedDisk: string;
  largeFiles: LargeFile[];
  filteredLargeFiles: LargeFile[];
  junkResults: JunkScanResult[];
  selectedLargeFiles: Set<string>;
  selectedJunkFiles: Set<string>;
  isScanning: boolean;
  scanProgress: LargeFileScanProgress | null;
  filter: LargeFileFilter;
  searchQuery: string;
  error: string | null;
  scanResult: LargeFileAnalysisResult | null;
  scanId: string | null;
  progressListener: UnlistenFn | null;
  completeListener: UnlistenFn | null;
  isListenersSetup: boolean;
  
  actions: {
    setActiveTab: (tab: 'large' | 'junk') => void;
    setSelectedDisk: (disk: string) => void;
    setLargeFiles: (files: LargeFile[]) => void;
    setJunkResults: (results: JunkScanResult[]) => void;
    toggleLargeFileSelection: (path: string) => void;
    toggleJunkFileSelection: (id: string) => void;
    selectAllLargeFiles: () => void;
    deselectAllLargeFiles: () => void;
    selectAllJunkFiles: () => void;
    deselectAllJunkFiles: () => void;
    setScanning: (scanning: boolean) => void;
    setScanProgress: (progress: LargeFileScanProgress | null) => void;
    setFilter: (filter: LargeFileFilter) => void;
    setSearchQuery: (query: string) => void;
    applyFilters: () => void;
    setError: (error: string | null) => void;
    setupListeners: () => Promise<void>;
    scanLargeFiles: (disk: string, minSize: number, unit: 'MB' | 'GB') => Promise<void>;
    pauseScan: () => Promise<void>;
    resumeScan: () => Promise<void>;
    cancelScan: () => Promise<void>;
    scanJunkFiles: () => Promise<void>;
    reset: () => void;
  };
}

const initialFilter: LargeFileFilter = {
  minSize: 500,
  unit: 'MB',
  fileTypes: [],
};

const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

class TauriEnvironmentError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'TauriEnvironmentError';
  }
}

function ensureTauriEnvironment(): void {
  if (!isTauri) {
    throw new TauriEnvironmentError(
      '此功能需要在 Tauri 应用环境中运行。请在桌面应用程序中使用此功能。'
    );
  }
}

export const useFileAnalysisStore = create<FileAnalysisState>((set, get) => ({
  activeTab: 'large',
  selectedDisk: 'C:\\',
  largeFiles: [],
  filteredLargeFiles: [],
  junkResults: [],
  selectedLargeFiles: new Set(),
  selectedJunkFiles: new Set(),
  isScanning: false,
  scanProgress: null,
  filter: initialFilter,
  searchQuery: '',
  error: null,
  scanResult: null,
  scanId: null,
  progressListener: null,
  completeListener: null,
  isListenersSetup: false,

  actions: {
    setActiveTab: (tab) => set({ activeTab: tab }),
    
    setSelectedDisk: (disk) => set({ selectedDisk: disk }),
    
    setLargeFiles: (files) => {
      set({ largeFiles: files });
      get().actions.applyFilters();
    },
    
    setJunkResults: (results) => set({ junkResults: results }),
    
    toggleLargeFileSelection: (path) => {
      const { selectedLargeFiles } = get();
      const newSet = new Set(selectedLargeFiles);
      if (newSet.has(path)) {
        newSet.delete(path);
      } else {
        newSet.add(path);
      }
      set({ selectedLargeFiles: newSet });
    },
    
    toggleJunkFileSelection: (id) => {
      const { selectedJunkFiles } = get();
      const newSet = new Set(selectedJunkFiles);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      set({ selectedJunkFiles: newSet });
    },
    
    selectAllLargeFiles: () => {
      const { filteredLargeFiles } = get();
      set({ selectedLargeFiles: new Set(filteredLargeFiles.map(f => f.path)) });
    },
    
    deselectAllLargeFiles: () => set({ selectedLargeFiles: new Set() }),
    
    selectAllJunkFiles: () => {
      const { junkResults } = get();
      const allIds = junkResults.flatMap((r: JunkScanResult) => r.items.map((i: JunkFileItem) => i.id));
      set({ selectedJunkFiles: new Set(allIds) });
    },
    
    deselectAllJunkFiles: () => set({ selectedJunkFiles: new Set() }),
    
    setScanning: (scanning) => set({ isScanning: scanning }),
    
    setScanProgress: (progress) => set({ scanProgress: progress }),
    
    setFilter: (filter) => {
      set({ filter });
      get().actions.applyFilters();
    },
    
    setSearchQuery: (query) => {
      set({ searchQuery: query });
      get().actions.applyFilters();
    },
    
    applyFilters: () => {
      const { largeFiles, searchQuery } = get();
      let filtered = [...largeFiles];
      
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase();
        filtered = filtered.filter(file => 
          file.name.toLowerCase().includes(query) ||
          file.path.toLowerCase().includes(query) ||
          file.extension.toLowerCase().includes(query) ||
          file.file_type.toLowerCase().includes(query)
        );
      }
      
      set({ filteredLargeFiles: filtered });
    },
    
    setError: (error) => set({ error }),
    
    setupListeners: async () => {
      const { isListenersSetup, progressListener, completeListener } = get();
      
      if (isListenersSetup) return;
      
      if (progressListener) {
        progressListener();
      }
      if (completeListener) {
        completeListener();
      }

      const unlistenProgress = await largeFileScanService.onProgress((progress) => {
        const currentScanId = get().scanId;
        if (!currentScanId || progress.scan_id === currentScanId) {
          // isScanning should be true for both scanning and paused states
          const isActive = progress.status === 'scanning' || progress.status === 'paused';
          set({ scanProgress: progress, isScanning: isActive });
        }
      });

      const unlistenComplete = await largeFileScanService.onComplete((result) => {
        const currentScanId = get().scanId;
        if (!currentScanId || result.scan_id === currentScanId) {
          set({ 
            largeFiles: result.files,
            filteredLargeFiles: result.files,
            scanResult: result,
            isScanning: false,
            scanProgress: null,
          });
        }
      });

      set({ 
        progressListener: unlistenProgress,
        completeListener: unlistenComplete,
        isListenersSetup: true,
      });
    },
    
    scanLargeFiles: async (disk: string, minSize: number, unit: 'MB' | 'GB') => {
      ensureTauriEnvironment();
      
      const { isListenersSetup, progressListener, completeListener } = get();
      
      if (!isListenersSetup) {
        if (progressListener) {
          progressListener();
        }
        if (completeListener) {
          completeListener();
        }

        const unlistenProgress = await largeFileScanService.onProgress((progress) => {
          const currentScanId = get().scanId;
          if (!currentScanId || progress.scan_id === currentScanId) {
            // isScanning should be true for both scanning and paused states
            const isActive = progress.status === 'scanning' || progress.status === 'paused';
            set({ scanProgress: progress, isScanning: isActive });
          }
        });

        const unlistenComplete = await largeFileScanService.onComplete((result) => {
          const currentScanId = get().scanId;
          if (!currentScanId || result.scan_id === currentScanId) {
            set({ 
              largeFiles: result.files,
              filteredLargeFiles: result.files,
              scanResult: result,
              isScanning: false,
              scanProgress: null,
            });
          }
        });

        set({ 
          progressListener: unlistenProgress,
          completeListener: unlistenComplete,
          isListenersSetup: true,
        });
      }
      
      set({ 
        isScanning: true, 
        scanProgress: null, 
        error: null, 
        selectedDisk: disk,
        largeFiles: [],
        filteredLargeFiles: [],
        scanResult: null,
      });
      
      try {
        const minBytes = unit === 'GB' ? minSize * 1024 * 1024 * 1024 : minSize * 1024 * 1024;
        
        const scanId = await largeFileScanService.start(disk, minBytes);
        set({ scanId });
      } catch (error) {
        set({ 
          isScanning: false, 
          error: error instanceof Error ? error.message : '扫描大文件失败' 
        });
      }
    },
    
    pauseScan: async () => {
      const { scanId, scanProgress } = get();
      if (!scanId || !scanProgress) return;
      
      try {
        await largeFileScanService.pause(scanId);
        // Optimistically update local state for immediate UI feedback
        set({ 
          scanProgress: { ...scanProgress, status: 'paused' }
        });
      } catch (error) {
        set({ error: error instanceof Error ? error.message : '暂停扫描失败' });
      }
    },
    
    resumeScan: async () => {
      const { scanId, scanProgress } = get();
      if (!scanId || !scanProgress) return;
      
      try {
        await largeFileScanService.resume(scanId);
        // Optimistically update local state for immediate UI feedback
        set({ 
          scanProgress: { ...scanProgress, status: 'scanning' }
        });
      } catch (error) {
        set({ error: error instanceof Error ? error.message : '恢复扫描失败' });
      }
    },
    
    cancelScan: async () => {
      const { scanId } = get();
      if (!scanId) return;
      
      try {
        await largeFileScanService.cancel(scanId);
        set({ 
          isScanning: false, 
          scanProgress: null,
          scanId: null,
        });
      } catch (error) {
        set({ error: error instanceof Error ? error.message : '取消扫描失败' });
      }
    },
    
    scanJunkFiles: async () => {
      ensureTauriEnvironment();
      set({ isScanning: true, scanProgress: null, error: null });
      
      try {
        const { fileAnalyzerService } = await import('../services/fileAnalyzerService');
        const results = await fileAnalyzerService.scanJunkFiles();
        
        set({ 
          junkResults: results, 
          isScanning: false, 
        });
      } catch (error) {
        set({ 
          isScanning: false, 
          error: error instanceof Error ? error.message : '扫描零碎文件失败' 
        });
      }
    },
    
    reset: () => {
      const { scanId } = get();
      if (scanId) {
        largeFileScanService.clearResult(scanId).catch(() => {});
      }
      
      set({
        largeFiles: [],
        filteredLargeFiles: [],
        junkResults: [],
        selectedLargeFiles: new Set(),
        selectedJunkFiles: new Set(),
        isScanning: false,
        scanProgress: null,
        error: null,
        scanResult: null,
        searchQuery: '',
        scanId: null,
      });
    },
  },
}));

export const useFileAnalysisActions = () => useFileAnalysisStore((state) => state.actions);
