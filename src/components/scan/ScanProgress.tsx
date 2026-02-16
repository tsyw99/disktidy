import { motion } from 'framer-motion';
import { FileText, FolderOpen, CheckCircle2, Pause, Loader2 } from 'lucide-react';
import type { ScanProgress as ScanProgressType, ScanStatus } from '../../types';
import { formatBytes } from '../../utils/format';
import { ProgressBar } from '../common';

interface ScanProgressProps {
  progress: ScanProgressType | null;
  status: ScanStatus;
}

export default function ScanProgress({ progress, status }: ScanProgressProps) {
  const percent = progress?.percent ?? 0;
  const isScanning = status === 'scanning';
  const isPaused = status === 'paused';
  const isCompleted = status === 'completed';

  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-[var(--text-primary)]">
              扫描进度：
            </span>
            {isPaused && (
              <span className="flex items-center gap-1 text-xs text-amber-500">
                <Pause className="w-3 h-3" />
                已暂停
              </span>
            )}
          </div>
          <span className="text-sm text-[var(--text-secondary)]">
            {percent.toFixed(0)}%
          </span>
        </div>

        <div className="relative">
          <ProgressBar
            percent={percent}
            height={12}
            showShimmer={isScanning}
            gradient={isPaused 
              ? { from: '#f59e0b', to: '#d97706' }
              : { from: '#3b82f6', to: '#10b981' }
            }
          />

          {isCompleted && (
            <motion.div
              initial={{ scale: 0, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              transition={{ delay: 0.3, type: 'spring', stiffness: 300 }}
              className="absolute right-2 top-1/2 -translate-y-1/2"
            >
              <CheckCircle2 className="w-4 h-4 text-white" />
            </motion.div>
          )}
        </div>
      </div>

      <div className="grid grid-cols-3 gap-6">
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="text-center"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <FileText className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {(progress?.scanned_files ?? 0).toLocaleString()}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">已扫描文件</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="text-center"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <FolderOpen className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {formatBytes(progress?.scanned_size ?? 0)}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">已扫描大小</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="text-center"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            {isScanning ? (
              <Loader2 className="w-5 h-5 text-[var(--color-primary)] animate-spin" />
            ) : (
              <span className="text-lg">⚡</span>
            )}
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {progress?.speed ? formatBytes(progress.speed) : '0 B'}/s
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">扫描速度</p>
        </motion.div>
      </div>

      {progress?.current_path && isScanning && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-center"
        >
          <p className="text-xs text-[var(--text-tertiary)] mb-1">正在扫描</p>
          <p className="text-sm text-[var(--text-secondary)] truncate font-mono max-w-md mx-auto">
            {progress.current_path}
          </p>
        </motion.div>
      )}
    </div>
  );
}
