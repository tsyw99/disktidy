export interface SystemInfo {
  os_name: string;
  os_version: string;
  os_arch: string;
  hostname: string;
  cpu_info: CpuInfo;
  memory_info: MemoryInfo;
}

export interface CpuInfo {
  name: string;
  cores: number;
  usage: number;
}

export interface MemoryInfo {
  total: number;
  used: number;
  free: number;
  usage_percent: number;
}

export interface DiskInfo {
  name: string;
  mount_point: string;
  file_system: string;
  total_size: number;
  used_size: number;
  free_size: number;
  usage_percent: number;
  volume_name: string;
}

export type DriverStatus = 'Running' | 'Stopped' | 'Error' | 'Unknown';

export interface DriverInfo {
  id: string;
  name: string;
  version: string;
  date: string;
  provider: string;
  status: DriverStatus;
  driver_type: string;
  device_name: string;
  inf_name: string;
  signed: boolean;
}
