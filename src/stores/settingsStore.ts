import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

export interface ScanSettings {
  excludePaths: string[];
  includeHidden: boolean;
  includeSystem: boolean;
}

interface SettingsState {
  scanSettings: ScanSettings;
  
  actions: {
    setScanSettings: (settings: Partial<ScanSettings>) => void;
    addExcludePath: (path: string) => void;
    removeExcludePath: (path: string) => void;
    resetScanSettings: () => void;
  };
}

const defaultScanSettings: ScanSettings = {
  excludePaths: [
    'C:\\Windows\\System32',
    'C:\\Program Files',
    'C:\\Program Files (x86)',
    'C:\\$Recycle.Bin',
  ],
  includeHidden: false,
  includeSystem: false,
};

export const useSettingsStore = create<SettingsState>()(
  devtools(
    persist(
      (set, get) => ({
        scanSettings: { ...defaultScanSettings },

        actions: {
          setScanSettings: (settings) => {
            set((state) => ({
              scanSettings: { ...state.scanSettings, ...settings },
            }));
          },

          addExcludePath: (path) => {
            const { excludePaths } = get().scanSettings;
            if (!excludePaths.includes(path)) {
              set((state) => ({
                scanSettings: {
                  ...state.scanSettings,
                  excludePaths: [...excludePaths, path],
                },
              }));
            }
          },

          removeExcludePath: (path) => {
            const { excludePaths } = get().scanSettings;
            set((state) => ({
              scanSettings: {
                ...state.scanSettings,
                excludePaths: excludePaths.filter((p) => p !== path),
              },
            }));
          },

          resetScanSettings: () => {
            set({ scanSettings: { ...defaultScanSettings } });
          },
        },
      }),
      {
        name: 'settings-store',
        partialize: (state) => ({ scanSettings: state.scanSettings }),
      }
    ),
    { name: 'settings-store' }
  )
);

export const useSettingsActions = () => useSettingsStore((state) => state.actions);
