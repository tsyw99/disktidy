import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import type { SystemInfo, DiskInfo, CpuInfo, MemoryInfo } from '../types';
import { systemService } from '../services/systemService';

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
              error: error instanceof Error ? error.message : String(error), 
              isLoading: false 
            });
          }
        },

        fetchDiskList: async () => {
          try {
            const disks = await systemService.getDiskList();
            set({ diskList: disks });
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
          }
        },

        fetchCpuInfo: async () => {
          try {
            const cpu = await systemService.getCpuInfo();
            set({ cpuInfo: cpu });
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
          }
        },

        fetchMemoryInfo: async () => {
          try {
            const memory = await systemService.getMemoryInfo();
            set({ memoryInfo: memory });
          } catch (error) {
            set({ error: error instanceof Error ? error.message : String(error) });
          }
        },

        refreshAll: async () => {
          const { actions } = get();
          set({ isLoading: true });
          await Promise.all([
            actions.fetchSystemInfo(),
            actions.fetchDiskList(),
            actions.fetchCpuInfo(),
            actions.fetchMemoryInfo(),
          ]);
          set({ isLoading: false, lastUpdated: Date.now() });
        },

        clearError: () => set({ error: null }),
      },
    }),
    { name: 'system-store' }
  )
);

export const useSystemActions = () => useSystemStore((state) => state.actions);
