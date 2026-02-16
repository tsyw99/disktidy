export interface ErrorResponse {
  code: number;
  message: string;
  details?: string;
}

export enum ErrorCode {
  SystemInfoFailed = 1001,
  DiskInfoFailed = 1002,
  ScanNotFound = 2001,
  ScanAlreadyRunning = 2002,
  ScanPathInvalid = 2003,
  ScanPermissionDenied = 2004,
  CleanFileNotFound = 3001,
  CleanPermissionDenied = 3002,
  CleanFileInUse = 3003,
  CleanProtectedFile = 3004,
  SettingsLoadFailed = 4001,
  SettingsSaveFailed = 4002,
  IoError = 5001,
  Unknown = 9999,
}

export const EVENT_CLEAN_PROGRESS = 'clean:progress';
export const EVENT_CLEAN_COMPLETE = 'clean:complete';
export const EVENT_ERROR = 'app:error';
