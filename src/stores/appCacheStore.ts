import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppType,
  CleanCategory,
  AppCacheScanStatus,
  AppCacheScanProgress,
  AppCacheScanResult,
  CleanResult,
} from '../types';
import { appCacheService } from '../services/appCacheService';
import { cleanService } from '../services/cleanService';

interface AppCacheState {
  status: AppCacheScanStatus;
  scanId: string | null;
  progress: AppCacheScanProgress | null;
  result: AppCacheScanResult | null;
  error: string | null;
  selectedApps: AppType[];
  selectedCategories: CleanCategory[];
  selectedFiles: Set<string>;
  expandedCategories: Set<CleanCategory>;
  cleanResult: CleanResult | null;
  progressListener: UnlistenFn | null;
  completeListener: UnlistenFn | null;
  isListenersSetup: boolean;
  configuredApps: Set<string>;
  configLoaded: boolean;

  actions: {
    setSelectedApps: (apps: AppType[]) => void;
    toggleApp: (app: AppType) => void;
    setSelectedCategories: (categories: CleanCategory[]) => void;
    toggleCategory: (category: CleanCategory) => void;
    setupListeners: () => Promise<void>;
    loadConfig: () => Promise<void>;
    startScan: () => Promise<void>;
    pauseScan: () => Promise<void>;
    resumeScan: () => Promise<void>;
    cancelScan: () => Promise<void>;
    resetScan: () => void;
    toggleFileSelection: (fileId: string) => void;
    toggleCategorySelection: (category: CleanCategory) => void;
    selectAllFiles: () => void;
    deselectAllFiles: () => void;
    toggleCategoryExpand: (category: CleanCategory) => void;
    expandAllCategories: () => void;
    collapseAllCategories: () => void;
    deleteSelectedFiles: () => Promise<CleanResult | null>;
    clearError: () => void;
    getSelectedCount: () => number;
    isAppConfigured: (app: AppType) => boolean;
    hasAnyConfiguredApp: () => boolean;
  };
}

export const useAppCacheStore = create<AppCacheState>()(
  devtools(
    (set, get) => ({
      status: 'idle',
      scanId: null,
      progress: null,
      result: null,
      error: null,
      selectedApps: [],
      selectedCategories: [],
      selectedFiles: new Set<string>(),
      expandedCategories: new Set<CleanCategory>(),
      cleanResult: null,
      progressListener: null,
      completeListener: null,
      isListenersSetup: false,
      configuredApps: new Set<string>(),
      configLoaded: false,

      actions: {
        setSelectedApps: (apps) => set({ selectedApps: apps }),

        toggleApp: (app) => {
          const { selectedApps } = get();
          const newApps = selectedApps.includes(app)
            ? selectedApps.filter((a) => a !== app)
            : [...selectedApps, app];
          set({ selectedApps: newApps });
        },

        setSelectedCategories: (categories) => set({ selectedCategories: categories }),

        toggleCategory: (category) => {
          const { selectedCategories } = get();
          const newCategories = selectedCategories.includes(category)
            ? selectedCategories.filter((c) => c !== category)
            : [...selectedCategories, category];
          set({ selectedCategories: newCategories });
        },

        loadConfig: async () => {
          try {
            const configuredApps = await appCacheService.getConfiguredApps();
            set({ 
              configuredApps: new Set(configuredApps),
              configLoaded: true,
            });
          } catch (e) {
            console.error('Failed to load config:', e);
            set({ configLoaded: true });
          }
        },

        setupListeners: async () => {
          const { isListenersSetup, progressListener, completeListener } = get();

          if (isListenersSetup) {
            console.log('[AppCacheStore] Listeners already setup');
            return;
          }

          if (progressListener) {
            progressListener();
          }
          if (completeListener) {
            completeListener();
          }

          console.log('[AppCacheStore] Setting up event listeners');

          const unlistenProgress = await appCacheService.onProgress((progress) => {
            const currentScanId = get().scanId;
            if (!currentScanId || progress.scanId === currentScanId) {
              set({ progress, status: progress.status });
            }
          });

          const unlistenComplete = await appCacheService.onComplete((result) => {
            const currentScanId = get().scanId;
            if (!currentScanId || result.scanId === currentScanId) {
              set({
                result,
                status: 'completed',
                progress: null,
              });
            }
          });

          set({
            progressListener: unlistenProgress,
            completeListener: unlistenComplete,
            isListenersSetup: true,
          });
          
          console.log('[AppCacheStore] Event listeners setup complete');
        },

        startScan: async () => {
          const { selectedApps, selectedCategories, progressListener, completeListener, isListenersSetup } = get();

          console.log('[AppCacheStore] startScan called with apps:', selectedApps, 'categories:', selectedCategories);

          if (!isListenersSetup) {
            if (progressListener) {
              progressListener();
            }
            if (completeListener) {
              completeListener();
            }

            const unlistenProgress = await appCacheService.onProgress((progress) => {
              const currentScanId = get().scanId;
              if (!currentScanId || progress.scanId === currentScanId) {
                set({ progress, status: progress.status });
              }
            });

            const unlistenComplete = await appCacheService.onComplete((result) => {
              const currentScanId = get().scanId;
              if (!currentScanId || result.scanId === currentScanId) {
                set({
                  result,
                  status: 'completed',
                  progress: null,
                });
              }
            });

            set({
              progressListener: unlistenProgress,
              completeListener: unlistenComplete,
              isListenersSetup: true,
            });
          }

          const initialProgress: AppCacheScanProgress = {
            scanId: '',
            status: 'scanning',
            currentPath: '',
            scannedFiles: 0,
            scannedSize: 0,
            totalFiles: 0,
            totalSize: 0,
            percent: 0,
            speed: 0,
            currentApp: '',
            incremental: false,
            skippedFiles: 0,
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
            const categories = selectedCategories.length > 0 ? selectedCategories : [];
            console.log('[AppCacheStore] Calling startScan service with apps:', selectedApps, 'categories:', categories);
            const scanId = await appCacheService.startScan(selectedApps, categories);
            console.log('[AppCacheStore] Scan started with scanId:', scanId);
            set({ scanId });
          } catch (error) {
            console.error('[AppCacheStore] Scan error:', error);
            set({
              status: 'error',
              error: error instanceof Error ? error.message : String(error),
            });
          }
        },

        pauseScan: async () => {
          const { scanId } = get();
          if (!scanId) return;

          try {
            await appCacheService.pauseScan(scanId);
            set({ status: 'paused' });
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
          }
        },

        resumeScan: async () => {
          const { scanId } = get();
          if (!scanId) return;

          try {
            await appCacheService.resumeScan(scanId);
            set({ status: 'scanning' });
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
          }
        },

        cancelScan: async () => {
          const { scanId } = get();
          if (!scanId) return;

          try {
            await appCacheService.cancelScan(scanId);
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
            appCacheService.clearResult(scanId).catch(() => {});
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
          });
        },

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

        toggleCategorySelection: (category: CleanCategory) => {
          const { result, selectedFiles } = get();
          if (!result) return;

          const categoryFiles = result.files.filter(f => f.category === category);
          const categoryPaths = categoryFiles.map(f => f.path);
          const allSelected = categoryPaths.every(p => selectedFiles.has(p));

          const newSelected = new Set(selectedFiles);
          if (allSelected) {
            categoryPaths.forEach(p => newSelected.delete(p));
          } else {
            categoryPaths.forEach(p => newSelected.add(p));
          }
          set({ selectedFiles: newSelected });
        },

        selectAllFiles: () => {
          const { result } = get();
          if (!result) return;

          const allPaths = result.files.map(f => f.path);
          set({ selectedFiles: new Set(allPaths) });
        },

        deselectAllFiles: () => {
          set({ selectedFiles: new Set() });
        },

        toggleCategoryExpand: (category: CleanCategory) => {
          const { expandedCategories } = get();
          const newExpanded = new Set(expandedCategories);
          if (newExpanded.has(category)) {
            newExpanded.delete(category);
          } else {
            newExpanded.add(category);
          }
          set({ expandedCategories: newExpanded });
        },

        expandAllCategories: () => {
          const { result } = get();
          if (!result) return;
          
          const categories = [...new Set(result.files.map(f => f.category))];
          set({ expandedCategories: new Set(categories) });
        },

        collapseAllCategories: () => {
          set({ expandedCategories: new Set() });
        },

        deleteSelectedFiles: async () => {
          const { selectedFiles } = get();
          if (selectedFiles.size === 0) return null;

          const filePaths = Array.from(selectedFiles);

          try {
            await cleanService.execute(filePaths, {
              move_to_recycle_bin: true,
              secure_delete: false,
              secure_pass_count: 3,
            });

            const { result } = get();
            if (result) {
              const cleanedPaths = new Set(filePaths);
              const updatedFiles = result.files.filter(f => !cleanedPaths.has(f.path));

              set({
                result: {
                  ...result,
                  files: updatedFiles,
                  totalFiles: updatedFiles.length,
                  totalSize: updatedFiles.reduce((sum, f) => sum + f.size, 0),
                },
                selectedFiles: new Set(),
              });
            }

            return null;
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
            return null;
          }
        },

        clearError: () => set({ error: null }),

        getSelectedCount: () => get().selectedFiles.size,

        isAppConfigured: (app: AppType) => {
          const { configuredApps } = get();
          return configuredApps.has(app);
        },

        hasAnyConfiguredApp: () => {
          const { configuredApps } = get();
          return configuredApps.size > 0;
        },
      },
    }),
    { name: 'app-cache-store' }
  )
);

export const useAppCacheActions = () => useAppCacheStore((state) => state.actions);
