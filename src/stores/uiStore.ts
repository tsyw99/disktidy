import { create } from 'zustand';
import { persist, devtools } from 'zustand/middleware';
import type { PageType } from '../types';

export type Theme = 'light' | 'dark';
export type SystemViewMode = 'cards' | 'chart';

export interface UIState {
  currentPage: PageType;
  sidebarCollapsed: boolean;
  theme: Theme;
  isWorking: boolean;
  systemViewMode: SystemViewMode;

  actions: {
    setCurrentPage: (page: PageType) => void;
    toggleSidebar: () => void;
    setTheme: (theme: Theme) => void;
    toggleTheme: () => void;
    setIsWorking: (isWorking: boolean) => void;
    setSystemViewMode: (mode: SystemViewMode) => void;
  };
}

export const useUIStore = create<UIState>()(
  devtools(
    persist(
      (set) => ({
        currentPage: 'system',
        sidebarCollapsed: false,
        theme: 'light', // 默认浅色主题
        isWorking: false,
        systemViewMode: 'chart', // 默认图表视图

        actions: {
          setCurrentPage: (page) => set({ currentPage: page }),
          toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
          setTheme: (theme) => set({ theme }),
          toggleTheme: () => set((state) => ({
            theme: state.theme === 'light' ? 'dark' : 'light'
          })),
          setIsWorking: (isWorking) => set({ isWorking }),
          setSystemViewMode: (mode) => set({ systemViewMode: mode }),
        },
      }),
      {
        name: 'disktidy-ui-storage',
        partialize: (state) => ({
          theme: state.theme,
          currentPage: state.currentPage,
          systemViewMode: state.systemViewMode,
        }),
      }
    ),
    { name: 'ui-store' }
  )
);

export const useUIActions = () => useUIStore((state) => state.actions);
