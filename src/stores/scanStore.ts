import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { UnlistenFn } from '@tauri-apps/api/event';
import type { ScanMode, ScanStatus, ScanProgress, ScanResult, ScanOptions, CleanResult } from '../types';
import { scanService } from '../services/scanService';
import { useSettingsStore } from './settingsStore';

interface ScanState {
  scanMode: ScanMode;
  status: ScanStatus;
  scanId: string | null;
  selectedDisk: string;
  selectedScanPath: string | null;
  progress: ScanProgress | null;
  result: ScanResult | null;
  cleanResult: CleanResult | null;
  error: string | null;
  progressListener: UnlistenFn | null;
  completeListener: UnlistenFn | null;
  isListenersSetup: boolean;
  selectedFiles: Set<string>;
  expandedCategories: Set<string>;
  deleteProgress: {
    isDeleting: boolean;
    current: number;
    total: number;
    percent: number;
    currentFile: string;
  } | null;
  
  actions: {
    setScanMode: (mode: ScanMode) => void;
    setSelectedDisk: (disk: string) => void;
    setSelectedScanPath: (path: string | null) => void;
    setupListeners: () => Promise<void>;
    startScan: () => Promise<void>;
    pauseScan: () => Promise<void>;
    resumeScan: () => Promise<void>;
    cancelScan: () => Promise<void>;
    resetScan: () => void;
    deleteScannedFiles: () => Promise<CleanResult | null>;
    deleteSelectedFiles: () => Promise<CleanResult | null>;
    clearError: () => void;
    toggleFileSelection: (filePath: string) => void;
    toggleCategorySelection: (categoryName: string) => void;
    selectAllFiles: () => void;
    deselectAllFiles: () => void;
    toggleCategoryExpand: (categoryName: string) => void;
    expandAllCategories: () => void;
    collapseAllCategories: () => void;
    getSelectedCount: () => number;
    getSelectedSize: () => number;
  };
}

const defaultScanOptions: ScanOptions = {
  paths: [],
  mode: 'quick',
  include_hidden: false,
  include_system: false,
  exclude_paths: [],
};

export const useScanStore = create<ScanState>()(
  devtools(
    (set, get) => ({
      scanMode: 'quick',
      status: 'idle',
      scanId: null,
      selectedDisk: 'C:\\',
      selectedScanPath: null,
      progress: null,
      result: null,
      cleanResult: null,
      error: null,
      progressListener: null,
      completeListener: null,
      isListenersSetup: false,
      selectedFiles: new Set<string>(),
      expandedCategories: new Set<string>(),
      deleteProgress: null,

      actions: {
        setScanMode: (mode) => {
          const { status, scanMode } = get();
          if (status === 'idle' || status === 'completed' || status === 'cancelled' || status === 'failed') {
            if (mode !== scanMode) {
              set({ scanMode: mode });
            }
          }
        },

        setSelectedDisk: (disk) => {
          set({ selectedDisk: disk });
        },

        setSelectedScanPath: (path) => {
          set({ selectedScanPath: path });
        },

        setupListeners: async () => {
          const { isListenersSetup, progressListener, completeListener } = get();
          
          if (isListenersSetup) return;
          
          if (progressListener) {
            progressListener();
          }
          if (completeListener) {
            completeListener();
          }

          const unlistenProgress = await scanService.onProgress((progress) => {
            const currentScanId = get().scanId;
            if (!currentScanId || progress.scan_id === currentScanId) {
              const progressStatus = progress.status;
              if (progressStatus === 'idle') {
                return;
              }
              set({ progress, status: progressStatus });
            }
          });

          const unlistenComplete = await scanService.onComplete((result) => {
            const currentScanId = get().scanId;
            if (!currentScanId || result.scan_id === currentScanId) {
              set({ 
                result, 
                status: 'completed', 
              });
            }
          });

          set({ 
            progressListener: unlistenProgress,
            completeListener: unlistenComplete,
            isListenersSetup: true,
          });
        },

        startScan: async () => {
          const { scanMode, selectedDisk, selectedScanPath } = get();

          const initialProgress: ScanProgress = {
            scan_id: '',
            status: 'scanning',
            current_path: '',
            scanned_files: 0,
            scanned_size: 0,
            total_files: 0,
            total_size: 0,
            percent: 0,
            speed: 0,
          };

          set({ 
            status: 'scanning', 
            error: null, 
            result: null, 
            progress: initialProgress, 
            cleanResult: null,
            selectedFiles: new Set(),
            expandedCategories: new Set(),
          });

          try {
            let scanPaths: string[];
            let options: ScanOptions = { ...defaultScanOptions };
            
            // 获取用户设置的扫描配置
            const scanSettings = useSettingsStore.getState().scanSettings;

            if (scanMode === 'quick') {
              scanPaths = [];
              options = {
                ...defaultScanOptions,
                mode: 'quick',
                include_hidden: scanSettings.includeHidden,
                include_system: scanSettings.includeSystem,
                exclude_paths: scanSettings.excludePaths,
              };
            } else {
              scanPaths = selectedScanPath ? [selectedScanPath] : [selectedDisk];
              options = {
                ...defaultScanOptions,
                mode: 'full',
                include_hidden: scanSettings.includeHidden,
                include_system: scanSettings.includeSystem,
                exclude_paths: scanSettings.excludePaths,
              };
            }

            const scanId = await scanService.start(scanPaths, scanMode, options);
            set({ scanId });
          } catch (error) {
            set({ 
              status: 'failed', 
              error: error instanceof Error ? error.message : String(error) 
            });
          }
        },

        pauseScan: async () => {
          const { scanId } = get();
          if (!scanId) return;

          try {
            await scanService.pause(scanId);
            set({ status: 'paused' });
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
          }
        },

        resumeScan: async () => {
          const { scanId } = get();
          if (!scanId) return;

          try {
            await scanService.resume(scanId);
            set({ status: 'scanning' });
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
          }
        },

        cancelScan: async () => {
          const { scanId } = get();
          if (!scanId) return;

          try {
            await scanService.cancel(scanId);

            set({ 
              status: 'cancelled', 
              scanId: null,
              progress: null,
            });
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
          }
        },

        resetScan: () => {
          const { scanId } = get();
          
          if (scanId) {
            scanService.clearResult(scanId).catch(() => {});
          }

          set({
            status: 'idle',
            scanId: null,
            progress: null,
            result: null,
            cleanResult: null,
            error: null,
            selectedFiles: new Set(),
            expandedCategories: new Set(),
            deleteProgress: null,
          });
        },

        deleteScannedFiles: async () => {
          const { scanId } = get();
          if (!scanId) return null;

          try {
            const cleanResult = await scanService.deleteFiles(scanId, true);
            set({ cleanResult });
            return cleanResult;
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
            return null;
          }
        },

        deleteSelectedFiles: async () => {
          const { scanId, selectedFiles } = get();
          if (!scanId || selectedFiles.size === 0) return null;

          const filePaths = Array.from(selectedFiles);
          const total = filePaths.length;
          
          set({ 
            deleteProgress: { 
              isDeleting: true, 
              current: 0, 
              total, 
              percent: 0,
              currentFile: ''
            } 
          });

          try {
            const cleanResult = await scanService.deleteSelectedFiles(scanId, filePaths, true);
            
            const { result } = get();
            if (result) {
              const cleanedPaths = new Set(filePaths);
              const updatedCategories = result.categories
                .map(category => ({
                  ...category,
                  files: category.files.filter(f => !cleanedPaths.has(f.path)),
                }))
                .map(category => ({
                  ...category,
                  file_count: category.files.length,
                  total_size: category.files.reduce((sum, f) => sum + f.size, 0),
                }))
                .filter(category => category.files.length > 0);
              
              set({
                result: {
                  ...result,
                  categories: updatedCategories,
                  total_files: updatedCategories.reduce((sum, c) => sum + c.file_count, 0),
                  total_size: updatedCategories.reduce((sum, c) => sum + c.total_size, 0),
                },
                selectedFiles: new Set(),
                deleteProgress: null,
                cleanResult,
              });
            }
            
            return cleanResult;
          } catch (error) {
            set({ 
              error: error instanceof Error ? error.message : String(error),
              deleteProgress: null,
            });
            return null;
          }
        },

        clearError: () => set({ error: null }),

        toggleFileSelection: (filePath: string) => {
          const { selectedFiles } = get();
          const newSelected = new Set(selectedFiles);
          if (newSelected.has(filePath)) {
            newSelected.delete(filePath);
          } else {
            newSelected.add(filePath);
          }
          set({ selectedFiles: newSelected });
        },

        toggleCategorySelection: (categoryName: string) => {
          const { result, selectedFiles } = get();
          if (!result) return;
          
          const category = result.categories.find(c => c.name === categoryName);
          if (!category) return;
          
          const categoryFilePaths = category.files.map(f => f.path);
          const allSelected = categoryFilePaths.every(p => selectedFiles.has(p));
          
          const newSelected = new Set(selectedFiles);
          if (allSelected) {
            categoryFilePaths.forEach(p => newSelected.delete(p));
          } else {
            categoryFilePaths.forEach(p => newSelected.add(p));
          }
          set({ selectedFiles: newSelected });
        },

        selectAllFiles: () => {
          const { result } = get();
          if (!result) return;
          
          const allPaths = result.categories.flatMap(c => c.files.map(f => f.path));
          set({ selectedFiles: new Set(allPaths) });
        },

        deselectAllFiles: () => {
          set({ selectedFiles: new Set() });
        },

        toggleCategoryExpand: (categoryName: string) => {
          const { expandedCategories } = get();
          const newExpanded = new Set(expandedCategories);
          if (newExpanded.has(categoryName)) {
            newExpanded.delete(categoryName);
          } else {
            newExpanded.add(categoryName);
          }
          set({ expandedCategories: newExpanded });
        },

        expandAllCategories: () => {
          const { result } = get();
          if (!result) return;
          const allCategoryNames = result.categories.map(c => c.name);
          set({ expandedCategories: new Set(allCategoryNames) });
        },

        collapseAllCategories: () => {
          set({ expandedCategories: new Set() });
        },

        getSelectedCount: () => get().selectedFiles.size,

        getSelectedSize: () => {
          const { result, selectedFiles } = get();
          if (!result) return 0;
          
          let totalSize = 0;
          for (const category of result.categories) {
            for (const file of category.files) {
              if (selectedFiles.has(file.path)) {
                totalSize += file.size;
              }
            }
          }
          return totalSize;
        },
      },
    }),
    { name: 'scan-store' }
  )
);

export const useScanActions = () => useScanStore((state) => state.actions);
