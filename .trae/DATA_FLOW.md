# DiskTidy 数据流设计文档

## 1. 数据流概述

### 1.1 设计原则

- **单向数据流**：数据从后端流向前端，状态更新单向传递
- **响应式更新**：状态变化自动触发UI更新
- **异步处理**：耗时操作采用异步模式，避免阻塞
- **事件驱动**：进度通知使用事件订阅模式

### 1.2 数据流层次架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        数据流层次架构                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    用户交互层 (UI)                        │   │
│  │                    用户操作 / 事件监听                     │   │
│  └────────────────────────────┬────────────────────────────┘   │
│                               │                                 │
│                               ▼                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    React组件层                            │   │
│  │              Props / Local State / Hooks                  │   │
│  └────────────────────────────┬────────────────────────────┘   │
│                               │                                 │
│                               ▼                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Zustand Store层                         │   │
│  │         Global State / Actions / Selectors                │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │   │
│  │  │systemStore│ │ scanStore│ │cleanStore│ │settingsStore│   │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │   │
│  └────────────────────────────┬────────────────────────────┘   │
│                               │                                 │
│                               ▼                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     Service层                             │   │
│  │              Tauri API 封装 / 数据转换                     │   │
│  │         invoke() / listen() / emit()                      │   │
│  └────────────────────────────┬────────────────────────────┘   │
│                               │                                 │
│                               ▼                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Tauri IPC层                            │   │
│  │              进程间通信桥梁 (JSON序列化)                    │   │
│  └────────────────────────────┬────────────────────────────┘   │
│                               │                                 │
│                               ▼                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  Tauri Commands层                         │   │
│  │              命令处理 / 参数验证 / 错误处理                  │   │
│  └────────────────────────────┬────────────────────────────┘   │
│                               │                                 │
│                               ▼                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    业务模块层                             │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │   │
│  │  │SystemInfo│ │DiskScanner│ │FileAnalyzer│ │ Cleaner │    │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │   │
│  └────────────────────────────┬────────────────────────────┘   │
│                               │                                 │
│                               ▼                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    系统调用层                             │   │
│  │           Windows API / File System / Registry            │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. 前端数据流设计

### 2.1 Zustand Store 架构

#### 2.1.1 Store 划分策略

| Store名称 | 职责 | 持久化 | 说明 |
|----------|------|--------|------|
| systemStore | 系统信息状态 | 否 | 实时数据，无需持久化 |
| scanStore | 扫描任务状态 | 部分 | 扫描历史需要持久化 |
| cleanStore | 清理操作状态 | 部分 | 清理历史需要持久化 |
| settingsStore | 应用设置状态 | 是 | 用户配置需要持久化 |
| uiStore | UI状态 | 否 | 临时状态 |

#### 2.1.2 SystemStore 设计

```typescript
// stores/systemStore.ts
import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import type { SystemInfo, DiskInfo, CpuInfo, MemoryInfo } from '../types/system';
import * as systemService from '../services/systemService';

interface SystemState {
  systemInfo: SystemInfo | null;
  diskList: DiskInfo[];
  cpuInfo: CpuInfo | null;
  memoryInfo: MemoryInfo | null;
  isLoading: boolean;
  error: string | null;
  lastUpdated: number | null;
  
  actions: {
    fetchSystemInfo: () => Promise<void>;
    fetchDiskList: () => Promise<void>;
    fetchCpuInfo: () => Promise<void>;
    fetchMemoryInfo: () => Promise<void>;
    refreshAll: () => Promise<void>;
    clearError: () => void;
  };
}

export const useSystemStore = create<SystemState>()(
  devtools(
    (set, get) => ({
      systemInfo: null,
      diskList: [],
      cpuInfo: null,
      memoryInfo: null,
      isLoading: false,
      error: null,
      lastUpdated: null,

      actions: {
        fetchSystemInfo: async () => {
          set({ isLoading: true, error: null });
          try {
            const info = await systemService.getSystemInfo();
            set({ 
              systemInfo: info, 
              isLoading: false,
              lastUpdated: Date.now()
            });
          } catch (error) {
            set({ 
              error: handleTauriError(error).message, 
              isLoading: false 
            });
          }
        },

        fetchDiskList: async () => {
          try {
            const disks = await systemService.getDiskList();
            set({ diskList: disks });
          } catch (error) {
            set({ error: handleTauriError(error).message });
          }
        },

        fetchCpuInfo: async () => {
          try {
            const cpu = await systemService.getCpuInfo();
            set({ cpuInfo: cpu });
          } catch (error) {
            set({ error: handleTauriError(error).message });
          }
        },

        fetchMemoryInfo: async () => {
          try {
            const memory = await systemService.getMemoryInfo();
            set({ memoryInfo: memory });
          } catch (error) {
            set({ error: handleTauriError(error).message });
          }
        },

        refreshAll: async () => {
          const { actions } = get();
          await Promise.all([
            actions.fetchSystemInfo(),
            actions.fetchDiskList(),
            actions.fetchCpuInfo(),
            actions.fetchMemoryInfo(),
          ]);
        },

        clearError: () => set({ error: null }),
      },
    }),
    { name: 'system-store' }
  )
);

export const useSystemActions = () => useSystemStore((state) => state.actions);
```

#### 2.1.3 ScanStore 设计

```typescript
// stores/scanStore.ts
import { create } from 'zustand';
import { persist, devtools } from 'zustand/middleware';
import type { ScanProgress, ScanResult, ScanOptions, ScanHistory } from '../types/scan';
import * as scanService from '../services/scanService';

interface ScanState {
  currentScan: {
    scanId: string | null;
    progress: ScanProgress | null;
    result: ScanResult | null;
    status: 'idle' | 'scanning' | 'paused' | 'completed' | 'error';
  };
  scanHistory: ScanHistory[];
  isLoading: boolean;
  error: string | null;
  
  actions: {
    startScan: (options: ScanOptions) => Promise<string>;
    pauseScan: () => Promise<void>;
    resumeScan: () => Promise<void>;
    cancelScan: () => Promise<void>;
    updateProgress: (progress: ScanProgress) => void;
    completeScan: (result: ScanResult) => void;
    clearCurrentScan: () => void;
    loadScanHistory: () => Promise<void>;
    deleteHistoryItem: (scanId: string) => void;
  };
}

export const useScanStore = create<ScanState>()(
  devtools(
    persist(
      (set, get) => ({
        currentScan: {
          scanId: null,
          progress: null,
          result: null,
          status: 'idle',
        },
        scanHistory: [],
        isLoading: false,
        error: null,

        actions: {
          startScan: async (options: ScanOptions) => {
            set({ 
              isLoading: true, 
              error: null,
              currentScan: {
                scanId: null,
                progress: null,
                result: null,
                status: 'scanning',
              }
            });
            
            try {
              const scanId = await scanService.startScan(options);
              set((state) => ({
                currentScan: { ...state.currentScan, scanId },
                isLoading: false,
              }));
              return scanId;
            } catch (error) {
              set((state) => ({
                currentScan: { ...state.currentScan, status: 'error' },
                error: handleTauriError(error).message,
                isLoading: false,
              }));
              throw error;
            }
          },

          pauseScan: async () => {
            const { currentScan } = get();
            if (!currentScan.scanId) return;
            
            try {
              await scanService.pauseScan(currentScan.scanId);
              set((state) => ({
                currentScan: { ...state.currentScan, status: 'paused' }
              }));
            } catch (error) {
              set({ error: handleTauriError(error).message });
            }
          },

          resumeScan: async () => {
            const { currentScan } = get();
            if (!currentScan.scanId) return;
            
            try {
              await scanService.resumeScan(currentScan.scanId);
              set((state) => ({
                currentScan: { ...state.currentScan, status: 'scanning' }
              }));
            } catch (error) {
              set({ error: handleTauriError(error).message });
            }
          },

          cancelScan: async () => {
            const { currentScan } = get();
            if (!currentScan.scanId) return;
            
            try {
              await scanService.cancelScan(currentScan.scanId);
              set((state) => ({
                currentScan: { ...state.currentScan, status: 'idle' }
              }));
            } catch (error) {
              set({ error: handleTauriError(error).message });
            }
          },

          updateProgress: (progress: ScanProgress) => {
            set((state) => ({
              currentScan: { ...state.currentScan, progress }
            }));
          },

          completeScan: (result: ScanResult) => {
            set((state) => ({
              currentScan: {
                ...state.currentScan,
                result,
                status: 'completed'
              },
              scanHistory: [
                {
                  scanId: result.scanId,
                  startTime: result.startTime,
                  endTime: result.endTime,
                  totalFiles: result.totalFiles,
                  totalSize: result.totalSize,
                },
                ...state.scanHistory,
              ].slice(0, 50),
            }));
          },

          clearCurrentScan: () => {
            set({
              currentScan: {
                scanId: null,
                progress: null,
                result: null,
                status: 'idle',
              }
            });
          },

          loadScanHistory: async () => {
            try {
              const history = await scanService.getScanHistory();
              set({ scanHistory: history });
            } catch (error) {
              set({ error: handleTauriError(error).message });
            }
          },

          deleteHistoryItem: (scanId: string) => {
            set((state) => ({
              scanHistory: state.scanHistory.filter(h => h.scanId !== scanId)
            }));
          },
        },
      }),
      {
        name: 'scan-store',
        partialize: (state) => ({
          scanHistory: state.scanHistory,
        }),
      }
    ),
    { name: 'scan-store' }
  )
);

export const useScanActions = () => useScanStore((state) => state.actions);
```

#### 2.1.4 CleanStore 设计

```typescript
// stores/cleanStore.ts
import { create } from 'zustand';
import { persist, devtools } from 'zustand/middleware';
import type { CleanOptions, CleanProgress, CleanReport, CleanHistory } from '../types/clean';
import * as cleanService from '../services/cleanService';

interface CleanState {
  selectedFiles: string[];
  cleanOptions: CleanOptions;
  currentClean: {
    cleanId: string | null;
    progress: CleanProgress | null;
    report: CleanReport | null;
    status: 'idle' | 'cleaning' | 'completed' | 'error';
  };
  cleanHistory: CleanHistory[];
  isLoading: boolean;
  error: string | null;
  
  actions: {
    setSelectedFiles: (files: string[]) => void;
    addSelectedFile: (file: string) => void;
    removeSelectedFile: (file: string) => void;
    toggleSelectAll: (files: string[]) => void;
    setCleanOptions: (options: Partial<CleanOptions>) => void;
    startClean: () => Promise<void>;
    updateProgress: (progress: CleanProgress) => void;
    completeClean: (report: CleanReport) => void;
    clearCurrentClean: () => void;
    loadCleanHistory: () => Promise<void>;
  };
}

const defaultCleanOptions: CleanOptions = {
  moveToRecycleBin: true,
  secureDelete: false,
  securePassCount: 3,
  createBackup: false,
};

export const useCleanStore = create<CleanState>()(
  devtools(
    persist(
      (set, get) => ({
        selectedFiles: [],
        cleanOptions: defaultCleanOptions,
        currentClean: {
          cleanId: null,
          progress: null,
          report: null,
          status: 'idle',
        },
        cleanHistory: [],
        isLoading: false,
        error: null,

        actions: {
          setSelectedFiles: (files) => set({ selectedFiles: files }),

          addSelectedFile: (file) => {
            set((state) => ({
              selectedFiles: state.selectedFiles.includes(file)
                ? state.selectedFiles
                : [...state.selectedFiles, file]
            }));
          },

          removeSelectedFile: (file) => {
            set((state) => ({
              selectedFiles: state.selectedFiles.filter(f => f !== file)
            }));
          },

          toggleSelectAll: (files) => {
            const { selectedFiles } = get();
            if (selectedFiles.length === files.length) {
              set({ selectedFiles: [] });
            } else {
              set({ selectedFiles: [...files] });
            }
          },

          setCleanOptions: (options) => {
            set((state) => ({
              cleanOptions: { ...state.cleanOptions, ...options }
            }));
          },

          startClean: async () => {
            const { selectedFiles, cleanOptions } = get();
            if (selectedFiles.length === 0) return;

            set({
              isLoading: true,
              error: null,
              currentClean: {
                cleanId: null,
                progress: null,
                report: null,
                status: 'cleaning',
              }
            });

            try {
              const cleanId = await cleanService.startClean({
                files: selectedFiles,
                options: cleanOptions,
              });
              set((state) => ({
                currentClean: { ...state.currentClean, cleanId },
                isLoading: false,
              }));
            } catch (error) {
              set((state) => ({
                currentClean: { ...state.currentClean, status: 'error' },
                error: handleTauriError(error).message,
                isLoading: false,
              }));
            }
          },

          updateProgress: (progress) => {
            set((state) => ({
              currentClean: { ...state.currentClean, progress }
            }));
          },

          completeClean: (report) => {
            set((state) => ({
              currentClean: {
                ...state.currentClean,
                report,
                status: 'completed'
              },
              selectedFiles: [],
              cleanHistory: [
                {
                  cleanId: report.cleanId,
                  startTime: report.startTime,
                  endTime: report.endTime,
                  totalFiles: report.totalFiles,
                  totalSize: report.totalSize,
                  successCount: report.successCount,
                  failedCount: report.failedCount,
                },
                ...state.cleanHistory,
              ].slice(0, 50),
            }));
          },

          clearCurrentClean: () => {
            set({
              currentClean: {
                cleanId: null,
                progress: null,
                report: null,
                status: 'idle',
              }
            });
          },

          loadCleanHistory: async () => {
            try {
              const history = await cleanService.getCleanHistory();
              set({ cleanHistory: history });
            } catch (error) {
              set({ error: handleTauriError(error).message });
            }
          },
        },
      }),
      {
        name: 'clean-store',
        partialize: (state) => ({
          cleanHistory: state.cleanHistory,
          cleanOptions: state.cleanOptions,
        }),
      }
    ),
    { name: 'clean-store' }
  )
);

export const useCleanActions = () => useCleanStore((state) => state.actions);
```

#### 2.1.5 SettingsStore 设计

```typescript
// stores/settingsStore.ts
import { create } from 'zustand';
import { persist, devtools } from 'zustand/middleware';
import type { AppSettings, CleanRule, ScheduleTask } from '../types/settings';

interface SettingsState {
  general: AppSettings;
  cleanRules: CleanRule[];
  scheduleTasks: ScheduleTask[];
  isLoading: boolean;
  error: string | null;
  
  actions: {
    updateGeneralSettings: (settings: Partial<AppSettings>) => void;
    addCleanRule: (rule: CleanRule) => void;
    updateCleanRule: (id: string, rule: Partial<CleanRule>) => void;
    removeCleanRule: (id: string) => void;
    addScheduleTask: (task: ScheduleTask) => Promise<void>;
    updateScheduleTask: (id: string, task: Partial<ScheduleTask>) => Promise<void>;
    removeScheduleTask: (id: string) => Promise<void>;
    loadSettings: () => Promise<void>;
    resetSettings: () => void;
  };
}

const defaultGeneralSettings: AppSettings = {
  theme: 'dark',
  language: 'zh-CN',
  autoStart: false,
  minimizeToTray: true,
  showNotifications: true,
  largeFileThreshold: 100,
  duplicateMinSize: 1,
  scanExcludePaths: [],
  protectedPaths: [],
};

export const useSettingsStore = create<SettingsState>()(
  devtools(
    persist(
      (set, get) => ({
        general: defaultGeneralSettings,
        cleanRules: [],
        scheduleTasks: [],
        isLoading: false,
        error: null,

        actions: {
          updateGeneralSettings: (settings) => {
            set((state) => ({
              general: { ...state.general, ...settings }
            }));
          },

          addCleanRule: (rule) => {
            set((state) => ({
              cleanRules: [...state.cleanRules, rule]
            }));
          },

          updateCleanRule: (id, rule) => {
            set((state) => ({
              cleanRules: state.cleanRules.map(r =>
                r.id === id ? { ...r, ...rule } : r
              )
            }));
          },

          removeCleanRule: (id) => {
            set((state) => ({
              cleanRules: state.cleanRules.filter(r => r.id !== id)
            }));
          },

          addScheduleTask: async (task) => {
            try {
              await scheduleService.createTask(task);
              set((state) => ({
                scheduleTasks: [...state.scheduleTasks, task]
              }));
            } catch (error) {
              set({ error: handleTauriError(error).message });
            }
          },

          updateScheduleTask: async (id, task) => {
            try {
              await scheduleService.updateTask(id, task);
              set((state) => ({
                scheduleTasks: state.scheduleTasks.map(t =>
                  t.id === id ? { ...t, ...task } : t
                )
              }));
            } catch (error) {
              set({ error: handleTauriError(error).message });
            }
          },

          removeScheduleTask: async (id) => {
            try {
              await scheduleService.deleteTask(id);
              set((state) => ({
                scheduleTasks: state.scheduleTasks.filter(t => t.id !== id)
              }));
            } catch (error) {
              set({ error: handleTauriError(error).message });
            }
          },

          loadSettings: async () => {
            set({ isLoading: true });
            try {
              const settings = await settingsService.loadSettings();
              set({
                general: settings.general,
                cleanRules: settings.cleanRules,
                scheduleTasks: settings.scheduleTasks,
                isLoading: false,
              });
            } catch (error) {
              set({ 
                error: handleTauriError(error).message,
                isLoading: false 
              });
            }
          },

          resetSettings: () => {
            set({
              general: defaultGeneralSettings,
              cleanRules: [],
              scheduleTasks: [],
            });
          },
        },
      }),
      {
        name: 'settings-store',
      }
    ),
    { name: 'settings-store' }
  )
);

export const useSettingsActions = () => useSettingsStore((state) => state.actions);
```

### 2.2 状态更新机制

#### 2.2.1 同步更新流程

```
┌─────────────────────────────────────────────────────────────────┐
│                     同步状态更新流程                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   用户操作                                                       │
│      │                                                          │
│      ▼                                                          │
│   ┌──────────┐                                                  │
│   │ 组件事件  │                                                  │
│   │ Handler  │                                                  │
│   └────┬─────┘                                                  │
│        │                                                        │
│        ▼                                                        │
│   ┌──────────┐     ┌──────────┐                                 │
│   │ 调用Store│────>│ 直接更新  │                                 │
│   │ Action   │     │  State   │                                 │
│   └──────────┘     └────┬─────┘                                 │
│                         │                                       │
│                         ▼                                       │
│                    ┌──────────┐                                 │
│                    │ 触发组件  │                                 │
│                    │ 重渲染    │                                 │
│                    └──────────┘                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

```typescript
// 同步更新示例
const handleToggleSelectAll = () => {
  const { actions, selectedFiles } = useCleanStore.getState();
  const allFiles = useScanStore.getState().currentScan.result?.files || [];
  actions.toggleSelectAll(allFiles.map(f => f.path));
};
```

#### 2.2.2 异步更新流程

```
┌─────────────────────────────────────────────────────────────────┐
│                     异步状态更新流程                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   用户操作                                                       │
│      │                                                          │
│      ▼                                                          │
│   ┌──────────┐                                                  │
│   │ 组件事件  │                                                  │
│   │ Handler  │                                                  │
│   └────┬─────┘                                                  │
│        │                                                        │
│        ▼                                                        │
│   ┌──────────┐     ┌──────────┐                                 │
│   │ 调用Store│────>│ 设置Loading│                                │
│   │ Action   │     │  = true   │                                 │
│   └──────────┘     └────┬─────┘                                 │
│                         │                                       │
│                         ▼                                       │
│                    ┌──────────┐                                 │
│                    │ 调用Service│                                │
│                    │ (Tauri)  │                                 │
│                    └────┬─────┘                                 │
│                         │                                       │
│                         ▼                                       │
│                    ┌──────────┐                                 │
│                    │ await响应 │                                 │
│                    └────┬─────┘                                 │
│                         │                                       │
│            ┌────────────┼────────────┐                          │
│            │            │            │                          │
│            ▼            ▼            ▼                          │
│       ┌────────┐   ┌────────┐   ┌────────┐                     │
│       │ 成功   │   │ 失败   │   │ 进度   │                     │
│       │更新数据│   │设置Error│   │更新进度│                     │
│       └────┬───┘   └────┬───┘   └────┬───┘                     │
│            │            │            │                          │
│            └────────────┼────────────┘                          │
│                         │                                       │
│                         ▼                                       │
│                    ┌──────────┐                                 │
│                    │ 设置Loading│                                │
│                    │  = false  │                                 │
│                    └────┬─────┘                                 │
│                         │                                       │
│                         ▼                                       │
│                    ┌──────────┐                                 │
│                    │ 触发组件  │                                 │
│                    │ 重渲染    │                                 │
│                    └──────────┘                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

```typescript
// 异步更新示例
const handleStartScan = async () => {
  const { actions } = useScanStore.getState();
  
  try {
    const scanId = await actions.startScan({
      path: selectedPath,
      mode: 'quick',
    });
    
    // 监听进度事件
    const unlisten = await listen<ScanProgress>('scan-progress', (event) => {
      if (event.payload.scanId === scanId) {
        actions.updateProgress(event.payload);
      }
    });
    
    // 监听完成事件
    await listen<ScanResult>('scan-complete', (event) => {
      if (event.payload.scanId === scanId) {
        actions.completeScan(event.payload);
        unlisten();
      }
    });
    
  } catch (error) {
    console.error('扫描启动失败:', error);
  }
};
```

### 2.3 状态持久化策略

#### 2.3.1 持久化状态分类

| 状态类型 | 持久化方式 | 存储位置 | 说明 |
|---------|-----------|---------|------|
| 用户设置 | localStorage | 浏览器存储 | 主题、语言等偏好设置 |
| 扫描历史 | localStorage | 浏览器存储 | 最近50条扫描记录 |
| 清理历史 | localStorage | 浏览器存储 | 最近50条清理记录 |
| 清理选项 | localStorage | 浏览器存储 | 用户选择的清理选项 |
| 清理规则 | 后端存储 | 配置文件 | 自定义清理规则 |
| 定时任务 | 后端存储 | 配置文件 | 定时扫描/清理任务 |

#### 2.3.2 持久化实现

```typescript
// 使用Zustand persist中间件
import { persist } from 'zustand/middleware';

// 部分状态持久化
persist(
  (set, get) => ({
    // store definition
  }),
  {
    name: 'store-name',
    partialize: (state) => ({
      // 只持久化需要的字段
      scanHistory: state.scanHistory,
      cleanHistory: state.cleanHistory,
    }),
    storage: createJSONStorage(() => localStorage),
    version: 1,
    migrate: (persistedState, version) => {
      // 版本迁移逻辑
      if (version === 0) {
        return {
          ...persistedState,
          // 迁移旧版本数据
        };
      }
      return persistedState;
    },
  }
)
```

---

## 3. 后端数据流设计

### 3.1 数据输入处理

#### 3.1.1 参数验证流程

```
┌─────────────────────────────────────────────────────────────────┐
│                     参数验证流程                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   前端请求                                                       │
│      │                                                          │
│      ▼                                                          │
│   ┌──────────────────────────────────────────┐                  │
│   │           Tauri Command 入口              │                  │
│   └────────────────────┬─────────────────────┘                  │
│                        │                                        │
│                        ▼                                        │
│   ┌──────────────────────────────────────────┐                  │
│   │           参数类型验证 (serde)            │                  │
│   │     - 类型匹配检查                        │                  │
│   │     - 必填字段检查                        │                  │
│   └────────────────────┬─────────────────────┘                  │
│                        │                                        │
│            ┌───────────┴───────────┐                            │
│            │                       │                            │
│            ▼                       ▼                            │
│       验证通过                 验证失败                          │
│            │                       │                            │
│            ▼                       ▼                            │
│   ┌──────────────────┐    ┌──────────────────┐                  │
│   │   业务参数验证    │    │   返回错误响应    │                  │
│   │ - 路径有效性     │    │   E006: 参数错误  │                  │
│   │ - 权限检查      │    └──────────────────┘                  │
│   │ - 范围检查      │                                          │
│   └────────┬─────────┘                                          │
│            │                                                    │
│            ▼                                                    │
│   ┌──────────────────┐                                          │
│   │   调用业务模块    │                                          │
│   └──────────────────┘                                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 3.1.2 参数验证实现

```rust
// commands/scan.rs
use serde::Deserialize;
use std::path::PathBuf;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct StartScanRequest {
    #[validate(length(min = 1, message = "路径不能为空"))]
    pub path: String,
    
    #[validate(range(min = 0, max = 100, message = "深度范围无效"))]
    pub max_depth: Option<usize>,
    
    pub mode: ScanMode,
    pub exclude_paths: Vec<String>,
}

#[tauri::command]
pub async fn start_scan(
    request: StartScanRequest,
    app: AppHandle,
) -> Result<String, String> {
    // 自动验证（通过Validate派生宏）
    request.validate()
        .map_err(|e| format!("E006: {}", e))?;
    
    // 业务验证
    let path = PathBuf::from(&request.path);
    if !path.exists() {
        return Err("E006: 指定路径不存在".to_string());
    }
    if !path.is_dir() {
        return Err("E006: 指定路径不是目录".to_string());
    }
    
    // 检查访问权限
    if let Err(e) = std::fs::read_dir(&path) {
        return Err(format!("E002: 无法访问目录 - {}", e));
    }
    
    // 调用业务模块
    let scanner = DiskScanner::new(ScanOptions::from(request));
    let scan_id = scanner.scan_id.clone();
    
    // 异步执行扫描
    tokio::spawn(async move {
        let result = scanner.start_scan().await;
        match result {
            Ok(r) => {
                app.emit("scan-complete", r).ok();
            }
            Err(e) => {
                app.emit("scan-error", e.to_string()).ok();
            }
        }
    });
    
    Ok(scan_id)
}
```

### 3.2 业务数据处理

#### 3.2.1 扫描数据处理流程

```
┌─────────────────────────────────────────────────────────────────┐
│                     扫描数据处理流程                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                    文件系统遍历                           │  │
│   │  ┌─────────┐    ┌─────────┐    ┌─────────┐              │  │
│   │  │ WalkDir │───>│ 过滤器  │───>│ 批量收集 │              │  │
│   │  └─────────┘    └─────────┘    └─────────┘              │  │
│   └──────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                    文件信息提取                           │  │
│   │  ┌─────────┐    ┌─────────┐    ┌─────────┐              │  │
│   │  │ 元数据  │───>│ 大小计算 │───>│ 类型识别 │              │  │
│   │  └─────────┘    └─────────┘    └─────────┘              │  │
│   └──────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                    分类处理                               │  │
│   │  ┌─────────┐    ┌─────────┐    ┌─────────┐              │  │
│   │  │垃圾文件 │    │ 大文件  │    │重复文件 │              │  │
│   │  │ 识别    │    │  筛选   │    │ 检测    │              │  │
│   │  └─────────┘    └─────────┘    └─────────┘              │  │
│   └──────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│   ┌──────────────────────────────────────────────────────────┐  │
│   │                    结果聚合                               │  │
│   │  ┌─────────┐    ┌─────────┐    ┌─────────┐              │  │
│   │  │ 统计计算 │───>│ 分类汇总 │───>│ 报告生成 │              │  │
│   │  └─────────┘    └─────────┘    └─────────┘              │  │
│   └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 3.2.2 扫描器实现

```rust
// modules/disk_scan/scanner.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use walkdir::WalkDir;

pub struct DiskScanner {
    pub scan_id: String,
    options: ScanOptions,
    progress: Arc<RwLock<ScanProgress>>,
    cancel_flag: Arc<RwLock<bool>>,
    pause_flag: Arc<RwLock<bool>>,
    result: Arc<RwLock<ScanResult>>,
}

impl DiskScanner {
    pub async fn start_scan(&self) -> Result<ScanResult, Error> {
        let start_time = std::time::Instant::now();
        let mut files: Vec<FileInfo> = Vec::new();
        let mut categories: HashMap<FileCategory, CategoryStats> = HashMap::new();
        
        let walker = WalkDir::new(&self.options.path)
            .max_depth(self.options.max_depth.unwrap_or(usize::MAX))
            .into_iter()
            .filter_entry(|e| !self.should_skip(e.path()));
        
        for entry in walker {
            // 检查取消标志
            if *self.cancel_flag.read().await {
                self.update_status(ScanStatus::Cancelled).await;
                break;
            }
            
            // 检查暂停标志
            while *self.pause_flag.read().await {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        if let Ok(file_info) = self.process_file(&entry).await {
                            // 更新分类统计
                            self.update_category(&mut categories, &file_info);
                            files.push(file_info);
                        }
                    }
                    
                    // 更新进度
                    self.update_progress(&entry, start_time).await?;
                }
                Err(e) => {
                    // 记录错误但继续扫描
                    tracing::warn!("扫描错误: {}", e);
                    continue;
                }
            }
        }
        
        // 生成最终结果
        let result = ScanResult {
            scan_id: self.scan_id.clone(),
            start_time: start_time.elapsed().as_millis() as u64,
            end_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            total_files: files.len() as u64,
            total_size: files.iter().map(|f| f.size).sum(),
            files,
            categories: categories.into_values().collect(),
        };
        
        self.update_status(ScanStatus::Completed).await;
        Ok(result)
    }
    
    async fn process_file(&self, entry: &walkdir::DirEntry) -> Result<FileInfo, Error> {
        let path = entry.path();
        let metadata = std::fs::metadata(path)?;
        
        Ok(FileInfo {
            path: path.to_string_lossy().to_string(),
            name: entry.file_name().to_string_lossy().to_string(),
            size: metadata.len(),
            modified_time: metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_millis() as u64,
            accessed_time: metadata.accessed()?.duration_since(std::time::UNIX_EPOCH)?.as_millis() as u64,
            created_time: metadata.created()?.duration_since(std::time::UNIX_EPOCH)?.as_millis() as u64,
            is_directory: false,
            extension: path.extension().map(|e| e.to_string_lossy().to_string()),
            category: self.detect_category(path),
        })
    }
    
    fn detect_category(&self, path: &std::path::Path) -> FileCategory {
        let ext = path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        
        // 垃圾文件检测
        if self.is_temp_file(path, &ext) {
            return FileCategory::TempFile;
        }
        if self.is_cache_file(path, &ext) {
            return FileCategory::Cache;
        }
        if self.is_log_file(&ext) {
            return FileCategory::Log;
        }
        
        // 大文件检测
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > self.options.large_file_threshold {
                return FileCategory::LargeFile;
            }
        }
        
        FileCategory::Other
    }
}
```

### 3.3 数据输出处理

#### 3.3.1 响应格式统一

```rust
// models/response.rs
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: current_timestamp(),
        }
    }
    
    pub fn error(code: &str, message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                details: None,
            }),
            timestamp: current_timestamp(),
        }
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
```

#### 3.3.2 分页数据结构

```rust
// models/pagination.rs
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub has_more: bool,
}

impl<T> PagedResult<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, page_size: u32) -> Self {
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
            has_more: page < total_pages,
        }
    }
}

// 使用示例
#[tauri::command]
pub async fn get_scan_files(
    scan_id: String,
    page: u32,
    page_size: u32,
    category: Option<FileCategory>,
) -> Result<PagedResult<FileInfo>, String> {
    let scan_result = SCAN_RESULTS.read().await
        .get(&scan_id)
        .cloned()
        .ok_or("E006: 扫描结果不存在")?;
    
    let filtered: Vec<FileInfo> = scan_result.files
        .into_iter()
        .filter(|f| category.as_ref().map_or(true, |c| &f.category == c))
        .collect();
    
    let total = filtered.len() as u64;
    let start = (page * page_size) as usize;
    let end = std::cmp::min(start + page_size as usize, filtered.len());
    
    let items = filtered[start..end].to_vec();
    
    Ok(PagedResult::new(items, total, page, page_size))
}
```

---

## 4. IPC数据流设计

### 4.1 请求-响应模式

#### 4.1.1 同步调用设计

```
┌─────────────────────────────────────────────────────────────────┐
│                     同步调用时序图                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   React组件          Service层           Tauri IPC      Rust命令 │
│       │                 │                   │              │    │
│       │   invoke()      │                   │              │    │
│       │────────────────>│                   │              │    │
│       │                 │    序列化参数      │              │    │
│       │                 │──────────────────>│              │    │
│       │                 │                   │   调用命令    │    │
│       │                 │                   │─────────────>│    │
│       │                 │                   │              │    │
│       │                 │                   │              │ 处理│
│       │                 │                   │              │    │
│       │                 │                   │   返回结果    │    │
│       │                 │                   │<─────────────│    │
│       │                 │    反序列化结果    │              │    │
│       │                 │<──────────────────│              │    │
│       │   Promise<T>    │                   │              │    │
│       │<────────────────│                   │              │    │
│       │                 │                   │              │    │
│       │   更新UI        │                   │              │    │
│       │                 │                   │              │    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

```typescript
// services/systemService.ts
import { invoke } from '@tauri-apps/api/core';

export const systemService = {
  getSystemInfo: async (): Promise<SystemInfo> => {
    return invoke<SystemInfo>('get_system_info');
  },
  
  getDiskList: async (): Promise<DiskInfo[]> => {
    return invoke<DiskInfo[]>('get_disk_list');
  },
  
  getCpuInfo: async (): Promise<CpuInfo> => {
    return invoke<CpuInfo>('get_cpu_info');
  },
  
  getMemoryInfo: async (): Promise<MemoryInfo> => {
    return invoke<MemoryInfo>('get_memory_info');
  },
};
```

#### 4.1.2 异步调用设计

```
┌─────────────────────────────────────────────────────────────────┐
│                     异步调用时序图                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   React组件          Service层           Tauri IPC      Rust命令 │
│       │                 │                   │              │    │
│       │ startScan()     │                   │              │    │
│       │────────────────>│                   │              │    │
│       │                 │ invoke(start_scan)│              │    │
│       │                 │──────────────────>│              │    │
│       │                 │                   │─────────────>│    │
│       │                 │                   │              │    │
│       │                 │                   │   返回scanId │    │
│       │                 │<──────────────────│<─────────────│    │
│       │   return scanId │                   │              │    │
│       │<────────────────│                   │              │    │
│       │                 │                   │              │    │
│       │ listen(progress)│                   │   后台执行    │    │
│       │────────────────>│                   │              │    │
│       │                 │                   │              │    │
│       │                 │                   │ emit(progress)│   │
│       │                 │<──────────────────│<─────────────│    │
│       │ 更新进度UI      │                   │              │    │
│       │<────────────────│                   │              │    │
│       │                 │                   │              │    │
│       │                 │                   │     ...重复   │    │
│       │                 │                   │              │    │
│       │                 │                   │ emit(complete)│   │
│       │                 │<──────────────────│<─────────────│    │
│       │ 显示结果        │                   │              │    │
│       │<────────────────│                   │              │    │
│       │                 │                   │              │    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

```typescript
// services/scanService.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

export const scanService = {
  startScan: async (options: ScanOptions): Promise<string> => {
    return invoke<string>('start_scan', { request: options });
  },
  
  pauseScan: async (scanId: string): Promise<void> => {
    return invoke('pause_scan', { scanId });
  },
  
  resumeScan: async (scanId: string): Promise<void> => {
    return invoke('resume_scan', { scanId });
  },
  
  cancelScan: async (scanId: string): Promise<void> => {
    return invoke('cancel_scan', { scanId });
  },
  
  getScanProgress: async (scanId: string): Promise<ScanProgress> => {
    return invoke<ScanProgress>('get_scan_progress', { scanId });
  },
  
  getScanFiles: async (
    scanId: string, 
    page: number, 
    pageSize: number,
    category?: FileCategory
  ): Promise<PagedResult<FileInfo>> => {
    return invoke('get_scan_files', { scanId, page, pageSize, category });
  },
};

// 事件监听封装
export function subscribeScanProgress(
  scanId: string,
  onProgress: (progress: ScanProgress) => void,
  onComplete: (result: ScanResult) => void,
  onError: (error: string) => void
): Promise<() => void> {
  return new Promise(async (resolve) => {
    const unlisteners: UnlistenFn[] = [];
    
    unlisteners.push(
      await listen<ScanProgress>('scan-progress', (event) => {
        if (event.payload.scanId === scanId) {
          onProgress(event.payload);
        }
      })
    );
    
    unlisteners.push(
      await listen<ScanResult>('scan-complete', (event) => {
        if (event.payload.scanId === scanId) {
          onComplete(event.payload);
          unlisteners.forEach(u => u());
        }
      })
    );
    
    unlisteners.push(
      await listen<string>('scan-error', (event) => {
        onError(event.payload);
        unlisteners.forEach(u => u());
      })
    );
    
    resolve(() => unlisteners.forEach(u => u()));
  });
}
```

### 4.2 事件订阅模式

#### 4.2.1 事件定义清单

| 事件名称 | 方向 | 数据类型 | 说明 |
|---------|------|---------|------|
| scan-progress | 后端→前端 | ScanProgress | 扫描进度更新 |
| scan-complete | 后端→前端 | ScanResult | 扫描完成 |
| scan-error | 后端→前端 | String | 扫描错误 |
| clean-progress | 后端→前端 | CleanProgress | 清理进度更新 |
| clean-complete | 后端→前端 | CleanReport | 清理完成 |
| clean-error | 后端→前端 | String | 清理错误 |
| schedule-trigger | 后端→前端 | ScheduleEvent | 定时任务触发 |
| system-alert | 后端→前端 | SystemAlert | 系统警告 |

#### 4.2.2 事件发射实现

```rust
// modules/disk_scan/scanner.rs
use tauri::AppHandle;
use tauri::Emitter;

impl DiskScanner {
    pub async fn start_scan_with_events(
        &self,
        app: AppHandle,
    ) -> Result<ScanResult, Error> {
        let start_time = std::time::Instant::now();
        let mut last_emit = start_time;
        let emit_interval = std::time::Duration::from_millis(100); // 100ms节流
        
        // ... 扫描逻辑
        
        for entry in walker {
            // ... 处理文件
            
            // 节流发射进度事件
            let now = std::time::Instant::now();
            if now.duration_since(last_emit) >= emit_interval {
                let progress = self.progress.read().await.clone();
                app.emit("scan-progress", &progress).ok();
                last_emit = now;
            }
        }
        
        // 发射完成事件
        let result = self.generate_result().await;
        app.emit("scan-complete", &result).ok();
        
        Ok(result)
    }
}
```

### 4.3 数据传输优化

#### 4.3.1 大数据分块传输

```rust
// commands/scan.rs
#[tauri::command]
pub async fn get_scan_files_chunked(
    scan_id: String,
    offset: usize,
    limit: usize,
) -> Result<FileChunk, String> {
    let results = SCAN_RESULTS.read().await;
    let scan_result = results.get(&scan_id)
        .ok_or("扫描结果不存在")?;
    
    let total = scan_result.files.len();
    let end = std::cmp::min(offset + limit, total);
    
    Ok(FileChunk {
        items: scan_result.files[offset..end].to_vec(),
        offset,
        total: total as u64,
        has_more: end < total,
    })
}
```

```typescript
// hooks/useScanFiles.ts
import { useState, useEffect, useCallback } from 'react';

interface UseScanFilesOptions {
  scanId: string;
  pageSize?: number;
  category?: FileCategory;
}

export function useScanFiles({ scanId, pageSize = 100, category }: UseScanFilesOptions) {
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const [page, setPage] = useState(0);
  
  const loadMore = useCallback(async () => {
    if (loading || !hasMore) return;
    
    setLoading(true);
    try {
      const result = await scanService.getScanFiles(scanId, page, pageSize, category);
      setFiles(prev => [...prev, ...result.items]);
      setHasMore(result.hasMore);
      setPage(prev => prev + 1);
    } catch (error) {
      console.error('加载文件失败:', error);
    } finally {
      setLoading(false);
    }
  }, [scanId, page, pageSize, category, loading, hasMore]);
  
  return { files, loading, hasMore, loadMore };
}
```

#### 4.3.2 事件节流策略

```rust
// utils/throttle.rs
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct ThrottledEmitter {
    last_emit: Arc<Mutex<Instant>>,
    interval: Duration,
}

impl ThrottledEmitter {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            last_emit: Arc::new(Mutex::new(Instant::now())),
            interval: Duration::from_millis(interval_ms),
        }
    }
    
    pub async fn emit<T: serde::Serialize + Clone>(
        &self,
        app: &AppHandle,
        event: &str,
        payload: T,
    ) -> bool {
        let mut last = self.last_emit.lock().await;
        let now = Instant::now();
        
        if now.duration_since(*last) >= self.interval {
            app.emit(event, payload).ok();
            *last = now;
            true
        } else {
            false
        }
    }
}
```

---

## 5. 核心业务数据流

### 5.1 系统信息获取数据流

```
┌─────────────────────────────────────────────────────────────────┐
│                  系统信息获取数据流                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   用户打开应用                                                   │
│        │                                                        │
│        ▼                                                        │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    SystemPage组件                       │    │
│   │  useEffect(() => {                                      │    │
│   │    actions.fetchSystemInfo();                           │    │
│   │    actions.fetchDiskList();                             │    │
│   │  }, []);                                                │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    systemStore                          │    │
│   │  set({ isLoading: true });                              │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    systemService                        │    │
│   │  invoke('get_system_info')                              │    │
│   │  invoke('get_disk_list')                                │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    Tauri IPC                            │    │
│   │  JSON序列化 → 传递到Rust进程                             │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    Rust Commands                        │    │
│   │  get_system_info() -> SystemInfoModule::get_info()      │    │
│   │  get_disk_list() -> DiskModule::get_list()              │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    Windows API                          │    │
│   │  GetVersionEx() / GlobalMemoryStatusEx()                │    │
│   │  GetLogicalDriveStrings() / GetDiskFreeSpaceEx()        │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    数据返回                              │    │
│   │  SystemInfo { osName, cpuInfo, memoryInfo... }          │    │
│   │  Vec<DiskInfo> [{ drive, totalSize, usedSize... }]      │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    systemStore                          │    │
│   │  set({ systemInfo, diskList, isLoading: false })        │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    UI组件更新                            │    │
│   │  <SystemInfoCard info={systemInfo} />                   │    │
│   │  <DiskList disks={diskList} />                          │    │
│   │  <DiskChart data={diskList} />                          │    │
│   └────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 磁盘扫描数据流

```
┌─────────────────────────────────────────────────────────────────┐
│                    磁盘扫描数据流                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   用户点击"开始扫描"按钮                                         │
│        │                                                        │
│        ▼                                                        │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    ScanPage组件                         │    │
│   │  const handleStartScan = async () => {                  │    │
│   │    const scanId = await actions.startScan(options);     │    │
│   │    subscribeProgress(scanId);                           │    │
│   │  };                                                     │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    scanStore                            │    │
│   │  status: 'scanning', scanId: null                       │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    scanService.startScan()              │    │
│   │  invoke('start_scan', { path, mode })                   │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    Rust: start_scan                     │    │
│   │  1. 验证参数                                            │    │
│   │  2. 创建DiskScanner实例                                 │    │
│   │  3. 返回scanId                                          │    │
│   │  4. tokio::spawn(异步扫描)                              │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    前端接收scanId                        │    │
│   │  scanStore: { scanId: 'xxx', status: 'scanning' }       │    │
│   │  开始监听事件: listen('scan-progress', ...)             │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    后台扫描线程                          │    │
│   │  for entry in WalkDir::new(path) {                      │    │
│   │    // 处理文件                                          │    │
│   │    // 每100ms发射进度事件                                │    │
│   │    app.emit('scan-progress', progress);                 │    │
│   │  }                                                      │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    前端接收进度                          │    │
│   │  scanStore: { progress: {...}, status: 'scanning' }     │    │
│   │  UI: <ProgressBar percent={progress.percent} />         │    │
│   │      <Stats files={progress.scannedFiles} />            │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    扫描完成                              │    │
│   │  app.emit('scan-complete', result);                     │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    前端处理完成                          │    │
│   │  scanStore: {                                          │    │
│   │    status: 'completed',                                │    │
│   │    result: ScanResult                                  │    │
│   │  }                                                      │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    显示结果                              │    │
│   │  <ScanResultSummary result={result} />                  │    │
│   │  <CategoryList categories={result.categories} />        │    │
│   │  <FileList files={result.files} />                      │    │
│   └────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 5.3 文件清理数据流

```
┌─────────────────────────────────────────────────────────────────┐
│                    文件清理数据流                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   用户选择文件并点击"清理"按钮                                    │
│        │                                                        │
│        ▼                                                        │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    CleanPage组件                        │    │
│   │  const handleClean = async () => {                      │    │
│   │    // 显示确认对话框                                     │    │
│   │    const confirmed = await showConfirmDialog();         │    │
│   │    if (confirmed) {                                     │    │
│   │      await actions.startClean();                        │    │
│   │    }                                                    │    │
│   │  };                                                     │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    确认对话框                            │    │
│   │  ┌──────────────────────────────────────────────────┐  │    │
│   │  │  确认清理                                          │  │    │
│   │  │  ───────────────────────────────────────────────  │  │    │
│   │  │  将清理 156 个文件，共 2.3 GB                      │  │    │
│   │  │  ───────────────────────────────────────────────  │  │    │
│   │  │  [ ] 移至回收站（推荐）                            │  │    │
│   │  │  [ ] 永久删除                                     │  │    │
│   │  │  ───────────────────────────────────────────────  │  │    │
│   │  │        [取消]  [确认清理]                          │  │    │
│   │  └──────────────────────────────────────────────────┘  │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    cleanStore                           │    │
│   │  status: 'cleaning', cleanId: null                      │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    cleanService.startClean()            │    │
│   │  invoke('clean_files', { files, options })              │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    Rust: clean_files                    │    │
│   │  1. 验证文件列表                                        │    │
│   │  2. 安全检查（保护目录/扩展名）                          │    │
│   │  3. 创建Cleaner实例                                     │    │
│   │  4. 异步执行清理                                        │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    后台清理线程                          │    │
│   │  for file in files {                                    │    │
│   │    // 安全检查                                          │    │
│   │    if is_protected(file) { continue; }                  │    │
│   │    // 执行删除                                          │    │
│   │    if options.recycle_bin {                             │    │
│   │      move_to_recycle_bin(file);                         │    │
│   │    } else {                                             │    │
│   │      std::fs::remove_file(file);                        │    │
│   │    }                                                    │    │
│   │    // 发射进度                                          │    │
│   │    app.emit('clean-progress', progress);                │    │
│   │  }                                                      │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    清理完成                              │    │
│   │  app.emit('clean-complete', report);                    │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    cleanStore                           │    │
│   │  status: 'completed',                                   │    │
│   │  report: CleanReport {                                  │    │
│   │    totalFiles: 156,                                     │    │
│   │    successCount: 152,                                   │    │
│   │    failedCount: 4,                                      │    │
│   │    totalSize: 2.3GB                                     │    │
│   │  }                                                      │    │
│   └────────────────────────┬───────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌────────────────────────────────────────────────────────┐    │
│   │                    显示清理报告                          │    │
│   │  ┌──────────────────────────────────────────────────┐  │    │
│   │  │  清理完成                                          │  │    │
│   │  │  ───────────────────────────────────────────────  │  │    │
│   │  │  ✓ 成功清理 152 个文件                             │  │    │
│   │  │  ✗ 失败 4 个文件（权限不足）                        │  │    │
│   │  │  ───────────────────────────────────────────────  │  │    │
│   │  │  释放空间: 2.3 GB                                  │  │    │
│   │  │  ───────────────────────────────────────────────  │  │    │
│   │  │        [查看详情]  [完成]                          │  │    │
│   │  └──────────────────────────────────────────────────┘  │    │
│   └────────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 6. 数据模型定义

### 6.1 前端数据模型

```typescript
// types/system.ts
export interface SystemInfo {
  osName: string;
  osVersion: string;
  osBuild: string;
  architecture: string;
  computerName: string;
  cpuInfo: CpuInfo;
  memoryInfo: MemoryInfo;
}

export interface CpuInfo {
  name: string;
  cores: number;
  logicalProcessors: number;
  usagePercent: number;
}

export interface MemoryInfo {
  total: number;
  available: number;
  used: number;
  usagePercent: number;
}

export interface DiskInfo {
  letter: string;
  label: string;
  fileSystem: string;
  totalSize: number;
  usedSize: number;
  freeSize: number;
  usagePercent: number;
  isSystem: boolean;
  isRemovable: boolean;
}

// types/scan.ts
export interface ScanOptions {
  path: string;
  mode: 'quick' | 'deep';
  maxDepth?: number;
  excludePaths: string[];
  largeFileThreshold?: number;
}

export interface ScanProgress {
  scanId: string;
  status: ScanStatus;
  currentPath: string;
  scannedFiles: number;
  scannedDirs: number;
  scannedSize: number;
  totalFiles: number;
  totalSize: number;
  percent: number;
  speed: number;
  elapsedTime: number;
  estimatedTime: number;
}

export type ScanStatus = 'idle' | 'scanning' | 'paused' | 'completed' | 'cancelled' | 'error';

export interface ScanResult {
  scanId: string;
  startTime: number;
  endTime: number;
  totalFiles: number;
  totalSize: number;
  categories: CategoryResult[];
  files: FileInfo[];
}

export interface CategoryResult {
  category: FileCategory;
  fileCount: number;
  totalSize: number;
  description: string;
}

export type FileCategory = 
  | 'tempFile'
  | 'cache'
  | 'log'
  | 'recycleBin'
  | 'browserCache'
  | 'appCache'
  | 'largeFile'
  | 'duplicate'
  | 'other';

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  modifiedTime: number;
  accessedTime: number;
  createdTime: number;
  isDirectory: boolean;
  extension?: string;
  category: FileCategory;
  isSafeToDelete: boolean;
}

// types/clean.ts
export interface CleanOptions {
  moveToRecycleBin: boolean;
  secureDelete: boolean;
  securePassCount: number;
  createBackup: boolean;
}

export interface CleanProgress {
  cleanId: string;
  status: CleanStatus;
  totalFiles: number;
  cleanedFiles: number;
  failedFiles: number;
  totalSize: number;
  cleanedSize: number;
  percent: number;
  currentFile: string;
}

export type CleanStatus = 'idle' | 'cleaning' | 'completed' | 'error';

export interface CleanReport {
  cleanId: string;
  startTime: number;
  endTime: number;
  totalFiles: number;
  totalSize: number;
  successCount: number;
  failedCount: number;
  failedFiles: FailedFileInfo[];
}

export interface FailedFileInfo {
  path: string;
  error: string;
}

// types/settings.ts
export interface AppSettings {
  theme: 'light' | 'dark' | 'system';
  language: string;
  autoStart: boolean;
  minimizeToTray: boolean;
  showNotifications: boolean;
  largeFileThreshold: number;
  duplicateMinSize: number;
  scanExcludePaths: string[];
  protectedPaths: string[];
}

export interface CleanRule {
  id: string;
  name: string;
  type: 'include' | 'exclude';
  targets: CleanRuleTarget[];
  enabled: boolean;
}

export interface CleanRuleTarget {
  path: string;
  fileTypes: string[];
  minSize?: number;
  maxSize?: number;
  olderThan?: number;
}

export interface ScheduleTask {
  id: string;
  name: string;
  type: 'scan' | 'clean';
  schedule: ScheduleConfig;
  enabled: boolean;
  lastRun?: number;
  nextRun?: number;
}

export interface ScheduleConfig {
  type: 'daily' | 'weekly' | 'monthly';
  time: string;
  dayOfWeek?: number;
  dayOfMonth?: number;
}
```

### 6.2 后端数据模型

```rust
// models/system.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub os_build: String,
    pub architecture: String,
    pub computer_name: String,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub name: String,
    pub cores: u32,
    pub logical_processors: u32,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total: u64,
    pub available: u64,
    pub used: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub letter: String,
    pub label: String,
    pub file_system: String,
    pub total_size: u64,
    pub used_size: u64,
    pub free_size: u64,
    pub usage_percent: f32,
    pub is_system: bool,
    pub is_removable: bool,
}

// models/scan.rs
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanOptions {
    pub path: String,
    pub mode: ScanMode,
    pub max_depth: Option<usize>,
    pub exclude_paths: Vec<String>,
    pub large_file_threshold: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanMode {
    Quick,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub scan_id: String,
    pub status: ScanStatus,
    pub current_path: String,
    pub scanned_files: u64,
    pub scanned_dirs: u64,
    pub scanned_size: u64,
    pub total_files: u64,
    pub total_size: u64,
    pub percent: f32,
    pub speed: u64,
    pub elapsed_time: u64,
    pub estimated_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanStatus {
    Idle,
    Scanning,
    Paused,
    Completed,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub scan_id: String,
    pub start_time: u64,
    pub end_time: u64,
    pub total_files: u64,
    pub total_size: u64,
    pub categories: Vec<CategoryResult>,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResult {
    pub category: FileCategory,
    pub file_count: u64,
    pub total_size: u64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileCategory {
    TempFile,
    Cache,
    Log,
    RecycleBin,
    BrowserCache,
    AppCache,
    LargeFile,
    Duplicate,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified_time: u64,
    pub accessed_time: u64,
    pub created_time: u64,
    pub is_directory: bool,
    pub extension: Option<String>,
    pub category: FileCategory,
    pub is_safe_to_delete: bool,
}

// models/clean.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanOptions {
    pub move_to_recycle_bin: bool,
    pub secure_delete: bool,
    pub secure_pass_count: u8,
    pub create_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanProgress {
    pub clean_id: String,
    pub status: CleanStatus,
    pub total_files: u64,
    pub cleaned_files: u64,
    pub failed_files: u64,
    pub total_size: u64,
    pub cleaned_size: u64,
    pub percent: f32,
    pub current_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanStatus {
    Idle,
    Cleaning,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub clean_id: String,
    pub start_time: u64,
    pub end_time: u64,
    pub total_files: u64,
    pub total_size: u64,
    pub success_count: u64,
    pub failed_count: u64,
    pub failed_files: Vec<FailedFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedFileInfo {
    pub path: String,
    pub error: String,
}
```

---

## 7. 数据流验证清单

| 验收项 | 状态 | 说明 |
|-------|------|------|
| Store结构完整 | ✅ | 5个Store定义完整 |
| 状态更新机制 | ✅ | 同步/异步流程清晰 |
| 状态持久化 | ✅ | 策略明确，实现方案完整 |
| 参数验证 | ✅ | 前后端验证机制完善 |
| IPC通信规范 | ✅ | 请求-响应和事件模式完整 |
| 事件定义 | ✅ | 8个事件定义清晰 |
| 核心业务流程 | ✅ | 3个核心流程图完整 |
| 数据模型 | ✅ | 前后端模型对应完整 |
| 分页传输 | ✅ | 大数据传输方案完整 |
| 事件节流 | ✅ | 性能优化方案完整 |

---

## 8. 附录

### 8.1 状态管理最佳实践

1. **单一数据源**：每个状态只在一个Store中管理
2. **不可变更新**：使用展开运算符或immer进行状态更新
3. **选择性订阅**：使用选择器避免不必要的重渲染
4. **异步处理**：使用async/await处理异步操作
5. **错误处理**：统一的错误处理机制

### 8.2 性能优化建议

1. **使用选择器**：避免在组件中直接访问整个Store
2. **批量更新**：合并多个状态更新
3. **延迟加载**：按需加载数据
4. **虚拟列表**：大列表使用虚拟滚动
5. **事件节流**：控制事件发射频率

### 8.3 调试工具

- **Redux DevTools**：通过Zustand devtools中间件支持
- **Tauri DevTools**：使用Tauri内置调试工具
- **Console Logging**：关键数据流节点添加日志
