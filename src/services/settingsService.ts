import { invoke } from '@tauri-apps/api/core';
import type {
  AppSettings,
  CleanRule,
  SettingsUpdate,
  CleanRuleInput,
} from '../types';

export const settingsService = {
  get: (): Promise<AppSettings> =>
    invoke<AppSettings>('settings_get'),

  update: (settings: AppSettings): Promise<AppSettings> =>
    invoke<AppSettings>('settings_update', { settings }),

  updatePartial: (updates: SettingsUpdate): Promise<AppSettings> =>
    invoke<AppSettings>('settings_update_partial', { updates }),

  reset: (): Promise<AppSettings> =>
    invoke<AppSettings>('settings_reset'),

  export: (): Promise<string> =>
    invoke<string>('settings_export'),

  import: (json: string): Promise<void> =>
    invoke<void>('settings_import', { json }),

  getRules: (): Promise<CleanRule[]> =>
    invoke<CleanRule[]>('rule_list'),

  getRule: (ruleId: string): Promise<CleanRule> =>
    invoke<CleanRule>('rule_get', { ruleId }),

  addRule: (rule: CleanRuleInput): Promise<CleanRule> =>
    invoke<CleanRule>('rule_add', { rule }),

  updateRule: (rule: CleanRule): Promise<CleanRule> =>
    invoke<CleanRule>('rule_update', { rule }),

  removeRule: (ruleId: string): Promise<boolean> =>
    invoke<boolean>('rule_remove', { ruleId }),

  toggleRule: (ruleId: string): Promise<CleanRule> =>
    invoke<CleanRule>('rule_toggle', { ruleId }),

  testRule: (path: string): Promise<CleanRule[]> =>
    invoke<CleanRule[]>('rule_test', { path }),

  getDefaultRules: (): Promise<CleanRule[]> =>
    invoke<CleanRule[]>('rule_get_defaults'),
};
