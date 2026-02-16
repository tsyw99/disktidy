import { HardDrive } from 'lucide-react';
import DiskUsageChart from './DiskUsageChart';
import { formatBytes } from '../../utils/format';
import type { DiskInfo } from '../../types';

interface DiskCardProps {
  disk: DiskInfo;
  className?: string;
}

export default function DiskCard({ disk, className = '' }: DiskCardProps) {
  const getUsageColor = (percent: number): string => {
    if (percent < 50) return '#10b981';
    if (percent < 75) return '#f59e0b';
    return '#ef4444';
  };

  return (
    <div className={`card p-5 ${className}`}>
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center">
            <HardDrive className="w-5 h-5 text-[var(--color-primary)]" />
          </div>
          <div>
            <h3 className="font-semibold text-[var(--text-primary)]">{disk.name}</h3>
          </div>
        </div>
        <span 
          className="text-sm font-medium px-2 py-1 rounded-md"
          style={{ 
            backgroundColor: `${getUsageColor(disk.usage_percent)}20`,
            color: getUsageColor(disk.usage_percent)
          }}
        >
          {disk.usage_percent.toFixed(1)}%
        </span>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex-shrink-0">
          <DiskUsageChart 
            used={disk.used_size} 
            total={disk.total_size}
            color={getUsageColor(disk.usage_percent)}
          />
        </div>
        
        <div className="flex-1 space-y-2">
          <div className="flex justify-between text-sm">
            <span className="text-[var(--text-tertiary)]">已使用</span>
            <span className="text-[var(--text-primary)] font-medium">{formatBytes(disk.used_size)}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-[var(--text-tertiary)]">可用空间</span>
            <span className="text-[var(--text-primary)] font-medium">{formatBytes(disk.free_size)}</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-[var(--text-tertiary)]">总容量</span>
            <span className="text-[var(--text-primary)] font-medium">{formatBytes(disk.total_size)}</span>
          </div>
        </div>
      </div>

      <div className="mt-4 h-2 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
        <div 
          className="h-full rounded-full transition-all duration-500 ease-out"
          style={{ 
            width: `${disk.usage_percent}%`,
            background: `linear-gradient(90deg, ${getUsageColor(disk.usage_percent)}, ${getUsageColor(disk.usage_percent)}80)`
          }}
        />
      </div>
    </div>
  );
}
