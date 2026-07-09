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

// 存储每个提供商的API Key
let providerApiKeys: Record<string, string> = {
  'deepseek': '',
  'glm': '',
  'kimi': '',
  'openai_compatible': '',
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
            set((state) => {
              const currentAiSettings = state.aiSettings;
              let updatedSettings = { ...currentAiSettings, ...settings };
              
              // 如果切换了提供商，需要保存当前提供商的API Key，并加载新提供商的API Key
              if (settings.provider && settings.provider !== currentAiSettings.provider) {
                // 保存当前提供商的API Key
                providerApiKeys[currentAiSettings.provider] = currentAiSettings.apiKey || '';
                
                // 切换到新提供商的API Key
                const newProvider = settings.provider;
                const newProviderApiKey = providerApiKeys[newProvider] || '';
                
                updatedSettings = {
                  ...updatedSettings,
                  provider: newProvider,
                  apiKey: newProviderApiKey
                };
                
                // 如果设置了新的API Key值，则更新对应提供商的Key
                if (settings.apiKey !== undefined) {
                  providerApiKeys[newProvider] = settings.apiKey;
                  updatedSettings.apiKey = settings.apiKey;
                }
              } else if (settings.apiKey !== undefined && currentAiSettings.provider) {
                // 如果没有切换提供商但设置了API Key，则更新当前提供商的API Key
                providerApiKeys[currentAiSettings.provider] = settings.apiKey;
                updatedSettings.apiKey = settings.apiKey;
              }
              
              return {
                aiSettings: updatedSettings
              };
            });
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
