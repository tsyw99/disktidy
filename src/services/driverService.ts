import { invoke } from '@tauri-apps/api/core';
import type { DriverInfo } from '../types/system';

export const driverService = {
  getList: (): Promise<DriverInfo[]> =>
    invoke<DriverInfo[]>('driver_get_list'),

  delete: (infName: string): Promise<void> =>
    invoke('driver_delete', { infName }),
};
