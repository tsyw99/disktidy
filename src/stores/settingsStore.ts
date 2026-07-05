import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

export interface ScanSettings {
  excludePaths: string[];
  includeHidden: boolean;
  includeSystem: boolean;
}

export interface AiSettings {
  provider: string;
  apiKey: string;
  model: string;
  baseUrl: string;
  maxTokens: number;
  temperature: number;
}

interface SettingsState {
  scanSettings: ScanSettings;
  aiSettings: AiSettings;

  actions: {
    setScanSettings: (settings: Partial<ScanSettings>) => void;
    addExcludePath: (path: string) => void;
    removeExcludePath: (path: string) => void;
    resetScanSettings: () => void;
    setAiSettings: (settings: Partial<AiSettings>) => void;
    resetAiSettings: () => void;
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

const defaultAiSettings: AiSettings = {
  provider: 'deepseek',
  apiKey: '',
  model: 'deepseek-chat',
  baseUrl: '',
  maxTokens: 4096,
  temperature: 0.7,
};

export const useSettingsStore = create<SettingsState>()(
  devtools(
    persist(
      (set, get) => ({
        scanSettings: { ...defaultScanSettings },
        aiSettings: { ...defaultAiSettings },

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

          setAiSettings: (settings) => {
            set((state) => ({
              aiSettings: { ...state.aiSettings, ...settings },
            }));
          },

          resetAiSettings: () => {
            set({ aiSettings: { ...defaultAiSettings } });
          },
        },
      }),
      {
        name: 'settings-store',
        partialize: (state) => ({
          scanSettings: state.scanSettings,
          aiSettings: state.aiSettings,
        }),
      }
    ),
    { name: 'settings-store' }
  )
);

export const useSettingsActions = () => useSettingsStore((state) => state.actions);
