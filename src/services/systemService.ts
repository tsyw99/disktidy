import { invoke } from '@tauri-apps/api/core';
import type { SystemInfo, DiskInfo, CpuInfo, MemoryInfo } from '../types';

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

export const systemService = {
  getSystemInfo: async (): Promise<SystemInfo> => {
    ensureTauriEnvironment();
    return invoke<SystemInfo>('system_get_info');
  },
  
  getDiskList: async (): Promise<DiskInfo[]> => {
    ensureTauriEnvironment();
    return invoke<DiskInfo[]>('system_get_disks');
  },

  getCpuInfo: async (): Promise<CpuInfo> => {
    ensureTauriEnvironment();
    return invoke<CpuInfo>('system_get_cpu_info');
  },

  getMemoryInfo: async (): Promise<MemoryInfo> => {
    ensureTauriEnvironment();
    return invoke<MemoryInfo>('system_get_memory_info');
  },
};
