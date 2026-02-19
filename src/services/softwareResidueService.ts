import { invoke } from '@tauri-apps/api/core';
import type {
  ResidueScanResult,
  ResidueScanProgress,
  ResidueScanOptions,
  DeleteResidueResult,
} from '../types/softwareResidue';

export const softwareResidueService = {
  startScan: (options: ResidueScanOptions): Promise<ResidueScanResult[]> =>
    invoke<ResidueScanResult[]>('residue_scan_start', { options }),

  pauseScan: (): Promise<void> =>
    invoke('residue_scan_pause'),

  resumeScan: (): Promise<void> =>
    invoke('residue_scan_resume'),

  cancelScan: (): Promise<void> =>
    invoke('residue_scan_cancel'),

  getProgress: (): Promise<ResidueScanProgress | null> =>
    invoke<ResidueScanProgress | null>('residue_scan_progress'),

  getResult: (): Promise<ResidueScanResult[]> =>
    invoke<ResidueScanResult[]>('residue_scan_result'),

  deleteItems: (itemIds: string[], moveToRecycleBin: boolean = true): Promise<DeleteResidueResult> =>
    invoke<DeleteResidueResult>('residue_delete_items', { itemIds, moveToRecycleBin }),
};
