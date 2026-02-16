import { Cpu, MemoryStick, Monitor, Server } from 'lucide-react';
import type { SystemInfo, CpuInfo, MemoryInfo } from '../../types';

interface SystemInfoCardProps {
  systemInfo: SystemInfo | null;
  cpuInfo: CpuInfo | null;
  memoryInfo: MemoryInfo | null;
  className?: string;
}

export default function SystemInfoCard({ 
  systemInfo, 
  cpuInfo, 
  memoryInfo,
  className = '' 
}: SystemInfoCardProps) {
  const formatMemory = (bytes: number): string => {
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  };

  return (
    <div className={`card p-6 ${className}`}>
      <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-4 flex items-center gap-2">
        <Monitor className="w-5 h-5 text-[var(--color-primary)]" />
        系统信息
      </h3>

      <div className="space-y-4">
        {systemInfo && (
          <div className="p-4 rounded-xl bg-[var(--bg-secondary)] border border-[var(--card-border)]">
            <div className="flex items-center gap-2 mb-3">
              <Server className="w-4 h-4 text-[var(--color-primary)]" />
              <span className="text-sm font-medium text-[var(--text-secondary)]">操作系统</span>
            </div>
            <div className="grid grid-cols-2 gap-3 text-sm">
              <div>
                <span className="text-[var(--text-tertiary)]">名称</span>
                <p className="text-[var(--text-primary)] mt-1">{systemInfo.os_name}</p>
              </div>
              <div>
                <span className="text-[var(--text-tertiary)]">版本</span>
                <p className="text-[var(--text-primary)] mt-1">{systemInfo.os_version}</p>
              </div>
              <div>
                <span className="text-[var(--text-tertiary)]">架构</span>
                <p className="text-[var(--text-primary)] mt-1">{systemInfo.os_arch}</p>
              </div>
              <div>
                <span className="text-[var(--text-tertiary)]">主机名</span>
                <p className="text-[var(--text-primary)] mt-1">{systemInfo.hostname}</p>
              </div>
            </div>
          </div>
        )}

        {cpuInfo && (
          <div className="p-4 rounded-xl bg-[var(--bg-secondary)] border border-[var(--card-border)]">
            <div className="flex items-center gap-2 mb-3">
              <Cpu className="w-4 h-4 text-[var(--color-primary)]" />
              <span className="text-sm font-medium text-[var(--text-secondary)]">处理器</span>
            </div>
            <div className="space-y-2">
              <p className="text-[var(--text-primary)] text-sm">{cpuInfo.name}</p>
              <div className="flex gap-4 text-xs">
                <span className="text-[var(--text-tertiary)]">
                  <span className="text-[var(--color-primary)] font-medium">{cpuInfo.cores}</span> 核心
                </span>
                <span className="text-[var(--text-tertiary)]">
                  使用率 <span className="text-[var(--color-primary)] font-medium">{cpuInfo.usage.toFixed(1)}%</span>
                </span>
              </div>
              <div className="h-2 bg-[var(--bg-tertiary)] rounded-full overflow-hidden mt-2">
                <div 
                  className="h-full rounded-full transition-all duration-500"
                  style={{ 
                    width: `${cpuInfo.usage}%`,
                    background: cpuInfo.usage > 80 
                      ? 'linear-gradient(90deg, #ef4444, #f87171)'
                      : cpuInfo.usage > 60
                      ? 'linear-gradient(90deg, #f59e0b, #fbbf24)'
                      : 'linear-gradient(90deg, #6366f1, #818cf8)'
                  }}
                />
              </div>
            </div>
          </div>
        )}

        {memoryInfo && (
          <div className="p-4 rounded-xl bg-[var(--bg-secondary)] border border-[var(--card-border)]">
            <div className="flex items-center gap-2 mb-3">
              <MemoryStick className="w-4 h-4 text-[var(--color-primary)]" />
              <span className="text-sm font-medium text-[var(--text-secondary)]">内存</span>
            </div>
            <div className="space-y-3">
              <div className="flex justify-between text-sm">
                <span className="text-[var(--text-tertiary)]">总计</span>
                <span className="text-[var(--text-primary)]">{formatMemory(memoryInfo.total)}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-[var(--text-tertiary)]">已使用</span>
                <span className="text-[var(--text-primary)]">{formatMemory(memoryInfo.used)}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-[var(--text-tertiary)]">可用</span>
                <span className="text-[var(--text-primary)]">{formatMemory(memoryInfo.free)}</span>
              </div>
              <div className="h-2 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
                <div 
                  className="h-full rounded-full transition-all duration-500"
                  style={{ 
                    width: `${memoryInfo.usage_percent}%`,
                    background: memoryInfo.usage_percent > 80 
                      ? 'linear-gradient(90deg, #ef4444, #f87171)'
                      : memoryInfo.usage_percent > 60
                      ? 'linear-gradient(90deg, #f59e0b, #fbbf24)'
                      : 'linear-gradient(90deg, #10b981, #34d399)'
                  }}
                />
              </div>
              <p className="text-xs text-[var(--text-tertiary)] text-right">
                使用率 {memoryInfo.usage_percent.toFixed(1)}%
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
