import type { JunkCategory } from './fileAnalyzer';
import type { CleanMode } from './cleaner';

export type SettingsScanMode = 'Quick' | 'Full' | 'Custom';

export interface AppSettings {
  auto_scan: boolean;
  scan_on_startup: boolean;
  default_scan_mode: SettingsScanMode;
  default_clean_mode: CleanMode;
  confirm_before_clean: boolean;
  show_notifications: boolean;
  language: string;
  theme: string;
}

export interface CleanRule {
  id: string;
  name: string;
  description: string;
  pattern: string;
  category: JunkCategory;
  enabled: boolean;
}

export interface SettingsUpdate {
  auto_scan?: boolean;
  scan_on_startup?: boolean;
  default_scan_mode?: SettingsScanMode;
  default_clean_mode?: CleanMode;
  confirm_before_clean?: boolean;
  show_notifications?: boolean;
  language?: string;
  theme?: string;
}

export interface CleanRuleInput {
  name: string;
  description: string;
  pattern: string;
  category: JunkCategory;
}

export const DEFAULT_SETTINGS: AppSettings = {
  auto_scan: false,
  scan_on_startup: false,
  default_scan_mode: 'Quick',
  default_clean_mode: 'MoveToTrash',
  confirm_before_clean: true,
  show_notifications: true,
  language: 'zh-CN',
  theme: 'dark',
};
