/**
 * 文件分析页面
 * 整合大文件管理和零碎文件扫描功能
 */

import { useEffect, useState, useMemo, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  HardDrive,
  Search,
  Settings,
  Play,
  Pause,
  X,
  Loader2,
  FileText,
  Package,
  Database,
  Trash2,
  CheckSquare,
  Square,
  AlertCircle,
  RotateCcw,
  Sparkles,
  FolderOpen,
  FileX,
  FolderX,
  Link2Off,
  Clock,
  Download,
  FileStack,
  CheckCircle2,
  ExternalLink,
} from 'lucide-react';
import { ProgressBar, SegmentedControl, CategorizedFileList } from '../components/common';
import type { BaseCategoryInfo, BaseFileInfo, FileRowProps } from '../components/common';
import { useLargeFileStore, useJunkFileStore } from '../stores';
import { useSystemStore } from '../stores/systemStore';
import { useSelectedSize, useSelectedSizeFromResults } from '../hooks';
import { formatBytes, formatDate } from '../utils/format';
import { openFileLocation } from '../utils/shell';
import { fileAnalyzerService } from '../services/fileAnalyzerService';
import { FILE_EXTENSION_GROUPS, FILE_TYPE_COLORS } from '../utils/constants';
import type { LargeFile, LargeFileFilter } from '../types/largeFile';
import type { JunkScanResult, JunkFileType } from '../types/fileAnalyzer';

type AnalysisMode = 'largeFile' | 'junkFile';

const junkFileTypeConfig: Record<JunkFileType, { name: string; icon: React.ReactNode; color: string }> = {
  empty_folders: { name: '空文件夹', icon: <FolderX className="w-5 h-5" />, color: 'text-blue-400' },
  invalid_shortcuts: { name: '无效快捷方式', icon: <Link2Off className="w-5 h-5" />, color: 'text-orange-400' },
  old_logs: { name: '过期日志', icon: <Clock className="w-5 h-5" />, color: 'text-yellow-400' },
  old_installers: { name: '旧安装包', icon: <Package className="w-5 h-5" />, color: 'text-purple-400' },
  invalid_downloads: { name: '无效下载', icon: <Download className="w-5 h-5" />, color: 'text-red-400' },
  small_files: { name: '零散小文件', icon: <FileStack className="w-5 h-5" />, color: 'text-cyan-400' },
  orphaned_files: { name: '孤立文件', icon: <FileX className="w-5 h-5" />, color: 'text-gray-400' },
};

const LargeFileIcon = ({ extension }: { extension: string }) => {
  const ext = extension.toLowerCase();
  
  if (FILE_EXTENSION_GROUPS.video.includes(ext as typeof FILE_EXTENSION_GROUPS.video[number])) {
    return <FileText className={`w-5 h-5 ${FILE_TYPE_COLORS.video}`} />;
  }
  if (FILE_EXTENSION_GROUPS.archive.includes(ext as typeof FILE_EXTENSION_GROUPS.archive[number])) {
    return <Package className={`w-5 h-5 ${FILE_TYPE_COLORS.archive}`} />;
  }
  if (FILE_EXTENSION_GROUPS.diskImage.includes(ext as typeof FILE_EXTENSION_GROUPS.diskImage[number])) {
    return <Database className={`w-5 h-5 ${FILE_TYPE_COLORS.diskImage}`} />;
  }
  if (FILE_EXTENSION_GROUPS.executable.includes(ext as typeof FILE_EXTENSION_GROUPS.executable[number])) {
    return <Package className={`w-5 h-5 ${FILE_TYPE_COLORS.executable}`} />;
  }
  if (FILE_EXTENSION_GROUPS.audio.includes(ext as typeof FILE_EXTENSION_GROUPS.audio[number])) {
    return <FileText className={`w-5 h-5 ${FILE_TYPE_COLORS.audio}`} />;
  }
  if (FILE_EXTENSION_GROUPS.image.includes(ext as typeof FILE_EXTENSION_GROUPS.image[number])) {
    return <FileText className={`w-5 h-5 ${FILE_TYPE_COLORS.image}`} />;
  }
  if (FILE_EXTENSION_GROUPS.document.includes(ext as typeof FILE_EXTENSION_GROUPS.document[number])) {
    return <FileText className={`w-5 h-5 ${FILE_TYPE_COLORS.document}`} />;
  }
  return <FileText className={`w-5 h-5 ${FILE_TYPE_COLORS.default}`} />;
};

const DiskSelector = ({
  selectedDisk,
  onSelect,
  disabled,
}: {
  selectedDisk: string;
  onSelect: (disk: string) => void;
  disabled: boolean;
}) => {
  const { diskList, actions: systemActions } = useSystemStore();

  useEffect(() => {
    if (diskList.length === 0) {
      systemActions.fetchDiskList();
    }
  }, []);

  return (
    <div className="space-y-3">
      {diskList.map((disk) => {
        const isSelected = selectedDisk === disk.mount_point;
        const usedPercent =
          disk.total_size > 0
            ? ((disk.total_size - disk.free_size) / disk.total_size) * 100
            : 0;

        return (
          <motion.button
            key={disk.mount_point}
            whileHover={{ scale: disabled ? 1 : 1.01 }}
            whileTap={{ scale: disabled ? 1 : 0.99 }}
            onClick={() => !disabled && onSelect(disk.mount_point)}
            disabled={disabled}
            className={`w-full flex items-center gap-4 p-4 rounded-xl border transition-all duration-200 ${
              isSelected
                ? 'border-[var(--color-primary)] bg-[var(--color-primary)]/5'
                : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50'
            } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
          >
            <div
              className={`w-12 h-12 rounded-xl flex items-center justify-center ${
                isSelected
                  ? 'bg-gradient-to-br from-[#6366f1] to-[#8b5cf6]'
                  : 'bg-[var(--bg-tertiary)]'
              }`}
            >
              <HardDrive
                className={`w-6 h-6 ${isSelected ? 'text-white' : 'text-[var(--color-primary)]'}`}
              />
            </div>
            <div className="flex-1 text-left">
              <div className="flex items-center justify-between mb-1">
                <p className="text-sm font-semibold text-[var(--text-primary)]">
                  {disk.name || disk.mount_point}
                </p>
                <span className="text-xs text-[var(--text-tertiary)]">
                  {formatBytes(disk.total_size)}
                </span>
              </div>
              <div className="flex items-center gap-3">
                <div className="flex-1 h-2 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
                  <div
                    className={`h-full rounded-full transition-all duration-300 ${
                      usedPercent > 90
                        ? 'bg-red-500'
                        : usedPercent > 70
                          ? 'bg-amber-500'
                          : 'bg-emerald-500'
                    }`}
                    style={{ width: `${Math.min(usedPercent, 100)}%` }}
                  />
                </div>
                <span className="text-xs text-[var(--text-tertiary)] whitespace-nowrap">
                  可用 {formatBytes(disk.free_size)}
                </span>
              </div>
            </div>
            {isSelected && (
              <div className="w-3 h-3 rounded-full bg-[var(--color-primary)]" />
            )}
          </motion.button>
        );
      })}
    </div>
  );
};

const LargeFileFilterModal = ({
  isOpen,
  onClose,
  filter,
  onApply,
}: {
  isOpen: boolean;
  onClose: () => void;
  filter: LargeFileFilter;
  onApply: (filter: LargeFileFilter) => void;
}) => {
  const [localFilter, setLocalFilter] = useState(filter);

  if (!isOpen) return null;

  const minSizeValue = localFilter.unit === 'MB' ? 100 : 1;
  const maxSizeValue = localFilter.unit === 'MB' ? 10240 : 100;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.95 }}
        className="bg-[var(--bg-secondary)] rounded-2xl p-6 w-full max-w-md"
      >
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-lg font-semibold text-[var(--text-primary)]">
            扫描设置
          </h3>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-[var(--bg-tertiary)] transition-colors"
          >
            <X className="w-5 h-5 text-[var(--text-secondary)]" />
          </button>
        </div>

        <div className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-[var(--text-secondary)] mb-3">
              最小文件大小
            </label>
            <div className="flex gap-3 mb-4">
              <input
                type="number"
                value={localFilter.minSize}
                onChange={(e) => {
                  const value = parseInt(e.target.value) || 500;
                  setLocalFilter({ ...localFilter, minSize: Math.max(minSizeValue, value) });
                }}
                className="flex-1 px-4 py-2.5 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-color)] text-[var(--text-primary)] text-center font-medium"
                min={minSizeValue}
              />
              <select
                value={localFilter.unit}
                onChange={(e) => {
                  const newUnit = e.target.value as 'MB' | 'GB';
                  const newSize =
                    newUnit === 'GB'
                      ? Math.max(1, Math.round(localFilter.minSize / 1024))
                      : Math.max(100, localFilter.minSize * 1024);
                  setLocalFilter({
                    ...localFilter,
                    unit: newUnit,
                    minSize: newUnit === 'GB' ? Math.min(newSize, 100) : Math.min(newSize, 10240),
                  });
                }}
                className="px-4 py-2.5 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-color)] text-[var(--text-primary)]"
              >
                <option value="MB">MB</option>
                <option value="GB">GB</option>
              </select>
            </div>

            <input
              type="range"
              min={minSizeValue}
              max={maxSizeValue}
              value={localFilter.minSize}
              onChange={(e) =>
                setLocalFilter({ ...localFilter, minSize: parseInt(e.target.value) })
              }
              className="w-full h-2 bg-[var(--bg-tertiary)] rounded-lg appearance-none cursor-pointer accent-[var(--color-primary)]"
            />
            <div className="flex justify-between text-xs text-[var(--text-tertiary)] mt-2">
              <span>{minSizeValue} {localFilter.unit}</span>
              <span>{maxSizeValue} {localFilter.unit}</span>
            </div>

            <p className="text-xs text-[var(--text-tertiary)] mt-3">
              将扫描大于等于 {localFilter.minSize} {localFilter.unit} 的文件
            </p>
          </div>
        </div>

        <div className="flex gap-3 mt-8">
          <button
            onClick={onClose}
            className="flex-1 px-4 py-2.5 rounded-lg border border-[var(--border-color)] text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] transition-colors"
          >
            取消
          </button>
          <button
            onClick={() => {
              onApply(localFilter);
              onClose();
            }}
            className="flex-1 px-4 py-2.5 rounded-lg bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white font-medium hover:shadow-lg hover:shadow-[#6366f1]/25 transition-all"
          >
            应用设置
          </button>
        </div>
      </motion.div>
    </div>
  );
};

const LargeFileScanProgress = ({
  progress,
  status,
}: {
  progress: { scannedFiles: number; foundFiles: number; percent: number; speed: number; currentPath: string; scannedSize?: number } | null;
  status: string;
}) => {
  const isScanning = status?.toLowerCase() === 'scanning';
  const isPaused = status?.toLowerCase() === 'paused';
  const percent = progress?.percent ?? 0;
  const scannedSize = progress?.scannedSize ?? 0;
  const speed = progress?.speed ?? 0;

  const scannedSizeGB = (scannedSize / (1024 * 1024 * 1024)).toFixed(2);

  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-[var(--text-primary)]">扫描进度：</span>
            {isPaused && (
              <span className="flex items-center gap-1 text-xs text-amber-500">
                <Pause className="w-3 h-3" />
                已暂停
              </span>
            )}
          </div>
          <span className="text-sm text-[var(--text-secondary)]">{scannedSizeGB} GB</span>
        </div>

        <div className="relative">
          <ProgressBar
            percent={percent}
            height={12}
            showShimmer={isScanning}
            indeterminate={isScanning && percent === 0}
            gradient={isPaused 
              ? { from: '#f59e0b', to: '#d97706' }
              : { from: '#6366f1', to: '#8b5cf6' }
            }
          />
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
            <HardDrive className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">{scannedSizeGB}</p>
          <p className="text-sm text-[var(--text-tertiary)]">已扫描 (GB)</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="text-center"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <FileText className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {(progress?.scannedFiles ?? 0).toLocaleString()}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">已扫描文件</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="text-center"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <Database className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {(progress?.foundFiles ?? 0).toLocaleString()}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">发现大文件</p>
        </motion.div>
      </div>

      {progress?.currentPath && isScanning && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-center"
        >
          <p className="text-xs text-[var(--text-tertiary)] mb-1">正在扫描</p>
          <p className="text-sm text-[var(--text-secondary)] truncate font-mono max-w-md mx-auto">
            {progress.currentPath}
          </p>
        </motion.div>
      )}

      <div className="flex items-center justify-center gap-2 text-sm text-[var(--text-secondary)]">
        {isScanning ? (
          <Loader2 className="w-4 h-4 text-[var(--color-primary)] animate-spin" />
        ) : (
          <span className="text-lg">⚡</span>
        )}
        <span>扫描速度：{speed > 0 ? `${formatBytes(speed)}/s` : '计算中...'}</span>
      </div>
    </div>
  );
};

const JunkFileScanProgress = ({
  progress,
  status,
}: {
  progress: { scannedFiles: number; foundFiles: number; percent: number; speed?: number; currentPath: string; scannedSize?: number; currentPhase: string } | null;
  status: string;
}) => {
  const isScanning = status?.toLowerCase() === 'scanning';
  const isPaused = status?.toLowerCase() === 'paused';
  const percent = progress?.percent ?? 0;
  const scannedSize = progress?.scannedSize ?? 0;
  const speed = progress?.speed ?? 0;
  const scannedFiles = progress?.scannedFiles ?? 0;
  const foundFiles = progress?.foundFiles ?? 0;
  const currentPath = progress?.currentPath ?? '';
  const currentPhase = progress?.currentPhase ?? '';

  return (
    <div className="space-y-6">
      <div>
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-[var(--text-primary)]">扫描进度：</span>
            {isPaused && (
              <span className="flex items-center gap-1 text-xs text-amber-500">
                <Pause className="w-3 h-3" />
                已暂停
              </span>
            )}
          </div>
          <span className="text-sm text-[var(--text-secondary)]">{percent.toFixed(0)}%</span>
        </div>

        <div className="relative">
          <ProgressBar
            percent={percent}
            height={12}
            showShimmer={isScanning}
            indeterminate={isScanning && percent === 0}
            gradient={isPaused 
              ? { from: '#f59e0b', to: '#d97706' }
              : { from: '#6366f1', to: '#8b5cf6' }
            }
          />
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
            {scannedFiles.toLocaleString()}
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
            <HardDrive className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">{formatBytes(scannedSize)}</p>
          <p className="text-sm text-[var(--text-tertiary)]">已扫描大小</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="text-center"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <FileX className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {foundFiles.toLocaleString()}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">发现零碎文件</p>
        </motion.div>
      </div>

      {currentPath && isScanning && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          className="text-center"
        >
          <p className="text-xs text-[var(--text-tertiary)] mb-1">
            {currentPhase || '正在扫描'}
          </p>
          <p className="text-sm text-[var(--text-secondary)] truncate font-mono max-w-md mx-auto">
            {currentPath}
          </p>
        </motion.div>
      )}

      <div className="flex items-center justify-center gap-2 text-sm text-[var(--text-secondary)]">
        {isScanning ? (
          <Loader2 className="w-4 h-4 text-[var(--color-primary)] animate-spin" />
        ) : (
          <span className="text-lg">⚡</span>
        )}
        <span>扫描速度：{speed > 0 ? `${formatBytes(speed)}/s` : '计算中...'}</span>
      </div>
    </div>
  );
};

function LargeFileRow({
  file,
  isSelected,
  isLast,
  onToggleSelection,
}: {
  file: LargeFile;
  isSelected: boolean;
  isLast: boolean;
  onToggleSelection: () => void;
}) {
  const handleOpenLocation = async (e: React.MouseEvent) => {
    e.stopPropagation();
    await openFileLocation(file.path);
  };

  return (
    <div
      className={`flex items-center gap-3 px-4 py-3 hover:bg-[var(--bg-tertiary)] transition-colors cursor-pointer ${
        !isLast ? 'border-b border-[var(--border-color)]/50' : ''
      }`}
      onClick={onToggleSelection}
    >
      <button className="flex-shrink-0">
        {isSelected ? (
          <CheckSquare className="w-4 h-4 text-[var(--color-primary)]" />
        ) : (
          <Square className="w-4 h-4 text-[var(--text-tertiary)]" />
        )}
      </button>

      <LargeFileIcon extension={file.extension} />

      <div className="flex-1 min-w-0">
        <p className="text-sm text-[var(--text-primary)] truncate" title={file.name}>
          {file.name}
        </p>
        <p className="text-xs text-[var(--text-tertiary)] truncate" title={file.path}>
          {file.path}
        </p>
        <div className="flex items-center gap-3 mt-1">
          <span className="text-xs text-[var(--text-tertiary)]">
            {formatBytes(file.size)}
          </span>
          <span className="text-xs text-[var(--text-tertiary)]">
            {formatDate(file.modified_time)}
          </span>
        </div>
      </div>

      <motion.button
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.9 }}
        onClick={handleOpenLocation}
        className="flex-shrink-0 p-1.5 rounded-lg text-[var(--text-tertiary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-secondary)] transition-colors"
        title="打开文件所在位置"
      >
        <ExternalLink className="w-4 h-4" />
      </motion.button>
    </div>
  );
}

const LargeFileList = ({
  files,
  selectedFiles,
  onToggleSelection,
  onSelectAll,
  onDeselectAll,
  onDelete,
}: {
  files: LargeFile[];
  selectedFiles: Set<string>;
  onToggleSelection: (path: string) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onDelete?: () => void;
}) => {
  const allSelected = files.length > 0 && files.every((f) => selectedFiles.has(f.path));
  const selectedCount = selectedFiles.size;
  const selectedSize = useSelectedSize(files, selectedFiles);

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.3 }}
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.1 }}
        className="rounded-xl border-2 border-[#10b981] bg-[#10b981]/5 p-6"
      >
        <div className="flex items-start gap-4">
          <div className="flex-shrink-0">
            <div className="w-12 h-12 rounded-full bg-[#10b981] flex items-center justify-center">
              <CheckCircle2 className="w-7 h-7 text-white" />
            </div>
          </div>

          <div className="flex-1 min-w-0">
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
              扫描完成
            </h3>
            <p className="text-sm text-[var(--text-secondary)] leading-relaxed">
              共发现 <span className="font-medium text-[var(--text-primary)]">{files.length.toLocaleString()}</span> 个大文件，
              占用空间 <span className="font-medium text-[#10b981]">{formatBytes(files.reduce((sum, f) => sum + f.size, 0))}</span>
            </p>
          </div>
        </div>
      </motion.div>

      <div className="grid grid-cols-3 gap-4 mt-6">
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="text-center p-4 rounded-lg bg-[var(--bg-secondary)]"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <FileText className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {files.length.toLocaleString()}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">文件总数</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="text-center p-4 rounded-lg bg-[var(--bg-secondary)]"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <Database className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[#10b981]">
            {formatBytes(files.reduce((sum, f) => sum + f.size, 0))}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">占用空间</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="text-center p-4 rounded-lg bg-[var(--bg-secondary)]"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <span className="text-lg">📊</span>
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {files.length > 0 ? formatBytes(files.reduce((sum, f) => sum + f.size, 0) / files.length) : '0 B'}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">平均大小</p>
        </motion.div>
      </div>

      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.5 }}
        className="mt-6"
      >
        <div className="flex items-center justify-between p-3 rounded-lg bg-[var(--bg-secondary)] mb-3">
          <div className="flex items-center gap-3">
            <button
              onClick={() => {
                if (allSelected) {
                  onDeselectAll();
                } else {
                  onSelectAll();
                }
              }}
              className="flex items-center gap-2"
            >
              {allSelected ? (
                <CheckSquare className="w-5 h-5 text-[var(--color-primary)]" />
              ) : selectedCount > 0 ? (
                <div className="w-5 h-5 rounded border-2 border-[var(--color-primary)] bg-[var(--color-primary)]/20 flex items-center justify-center">
                  <div className="w-2 h-2 rounded-sm bg-[var(--color-primary)]" />
                </div>
              ) : (
                <Square className="w-5 h-5 text-[var(--text-tertiary)]" />
              )}
              <span className="text-sm text-[var(--text-primary)]">
                {allSelected ? '取消全选' : '全选'}
              </span>
            </button>
          </div>

          {selectedCount > 0 && (
            <div className="flex items-center gap-4">
              <span className="text-sm text-[var(--text-secondary)]">
                已选择 <span className="font-medium text-[var(--text-primary)]">{selectedCount}</span> 个文件
              </span>
              <span className="text-sm font-medium text-[#10b981]">
                {formatBytes(selectedSize)}
              </span>
            </div>
          )}
        </div>

        <div className="rounded-lg border border-[var(--border-color)] overflow-hidden">
          <div className="max-h-[400px] overflow-y-auto scrollbar-thin">
            {files.map((file, index) => (
              <LargeFileRow
                key={file.path}
                file={file}
                isSelected={selectedFiles.has(file.path)}
                isLast={index === files.length - 1}
                onToggleSelection={() => onToggleSelection(file.path)}
              />
            ))}
          </div>
        </div>
      </motion.div>

      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
        className="flex justify-center gap-4 mt-8"
      >
        <motion.button
          onClick={onDelete}
          disabled={selectedCount === 0}
          whileHover={{ scale: selectedCount > 0 ? 1.02 : 1 }}
          whileTap={{ scale: selectedCount > 0 ? 0.98 : 1 }}
          className="flex items-center gap-2 px-6 py-3 rounded-lg bg-gradient-to-r from-red-500 to-red-600 text-white font-medium shadow-lg shadow-red-500/25 hover:shadow-red-500/40 transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Trash2 className="w-5 h-5" />
          {selectedCount > 0 
            ? `删除选中的 ${selectedCount} 个文件` 
            : '请先选择要删除的文件'}
        </motion.button>
      </motion.div>
    </motion.div>
  );
};

interface JunkFileInfo extends BaseFileInfo {
  file_type: JunkFileType;
  description: string;
  safe_to_delete: boolean;
  risk_level: string;
}

interface JunkCategoryInfo extends BaseCategoryInfo<JunkFileInfo> {
  fileType: JunkFileType;
}

function JunkFileRow({ file, isSelected, isLast, onToggleSelection, onOpenLocation }: FileRowProps<JunkFileInfo>) {
  return (
    <div
      className={`flex items-center gap-3 px-4 py-3 hover:bg-[var(--bg-tertiary)] transition-colors cursor-pointer ${
        !isLast ? 'border-b border-[var(--border-color)]/50' : ''
      }`}
      onClick={onToggleSelection}
    >
      <button className="flex-shrink-0">
        {isSelected ? (
          <CheckSquare className="w-4 h-4 text-[var(--color-primary)]" />
        ) : (
          <Square className="w-4 h-4 text-[var(--text-tertiary)]" />
        )}
      </button>

      <div className="flex-1 min-w-0">
        <p className="text-sm text-[var(--text-primary)] truncate" title={file.path.split(/[/\\]/).pop() ?? file.path}>
          {file.path.split(/[/\\]/).pop() ?? file.path}
        </p>
        <p className="text-xs text-[var(--text-tertiary)] truncate" title={file.path}>
          {file.path}
        </p>
        <div className="flex items-center gap-3 mt-1">
          <span className="text-xs text-[var(--text-tertiary)]">
            {formatBytes(file.size)}
          </span>
        </div>
      </div>

      {onOpenLocation && (
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={(e) => {
            e.stopPropagation();
            onOpenLocation(file);
          }}
          className="flex-shrink-0 p-1.5 rounded-lg text-[var(--text-tertiary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-secondary)] transition-colors"
          title="打开文件所在位置"
        >
          <ExternalLink className="w-4 h-4" />
        </motion.button>
      )}
    </div>
  );
}

const JunkFileList = ({
  results,
  scanId,
  selectedFiles,
  expandedTypes,
  onToggleSelection,
  onToggleTypeSelection,
  onSelectAll,
  onDeselectAll,
  onToggleExpand,
  onExpandAll,
  onCollapseAll,
  onDelete,
}: {
  results: JunkScanResult[];
  scanId: string | null;
  selectedFiles: Set<string>;
  expandedTypes: Set<string>;
  onToggleSelection: (path: string) => void;
  onToggleTypeSelection: (fileType: string) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onToggleExpand: (fileType: string) => void;
  onExpandAll: () => void;
  onCollapseAll: () => void;
  onDelete?: () => void;
}) => {
  const totalCount = results.reduce((sum, r) => sum + r.count, 0);
  const totalSize = results.reduce((sum, r) => sum + r.total_size, 0);
  const selectedCount = selectedFiles.size;
  const selectedSize = useSelectedSizeFromResults(results, selectedFiles);

  const categories: JunkCategoryInfo[] = useMemo(() => {
    return results.map(result => {
      const config = junkFileTypeConfig[result.file_type] || { 
        name: result.file_type, 
        icon: <FileX className="w-5 h-5" />, 
        color: 'text-gray-400' 
      };
      
      const files: JunkFileInfo[] = result.items.map(item => ({
        path: item.path,
        name: item.path.split(/[/\\]/).pop() ?? item.path,
        size: item.size,
        modified_time: item.modified_time,
        file_type: item.file_type,
        description: item.description,
        safe_to_delete: item.safe_to_delete,
        risk_level: item.risk_level,
      }));
      
      return {
        key: result.file_type,
        fileType: result.file_type,
        displayName: config.name,
        files,
        fileCount: result.count,
        totalSize: result.total_size,
        hasMore: result.count > result.items.length,
        icon: config.icon,
        iconColor: config.color,
      };
    });
  }, [results]);

  const handleOpenLocation = useCallback(async (file: JunkFileInfo) => {
    await openFileLocation(file.path);
  }, []);

  const handleLoadMore = useCallback(async (
    categoryKey: string,
    offset: number,
    limit: number
  ) => {
    if (!scanId) return null;
    
    try {
      const response = await fileAnalyzerService.getJunkCategoryFiles(
        scanId,
        categoryKey,
        offset,
        limit
      );
      
      if (response) {
        const files: JunkFileInfo[] = response.files.map(item => ({
          path: item.path,
          name: item.path.split(/[/\\]/).pop() ?? item.path,
          size: item.size,
          modified_time: item.modified_time,
          file_type: item.file_type,
          description: item.description,
          safe_to_delete: item.safe_to_delete,
          risk_level: item.risk_level,
        }));
        
        return {
          files,
          hasMore: response.has_more,
        };
      }
    } catch {
      // 静默处理错误
    }
    return null;
  }, [scanId]);

  const renderFileRow = useCallback((props: FileRowProps<JunkFileInfo>) => {
    return (
      <JunkFileRow
        key={props.file.path}
        file={props.file}
        isSelected={props.isSelected}
        isLast={props.isLast}
        onToggleSelection={props.onToggleSelection}
        onOpenLocation={props.onOpenLocation}
      />
    );
  }, []);

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.3 }}
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.1 }}
        className="rounded-xl border-2 border-[#10b981] bg-[#10b981]/5 p-6"
      >
        <div className="flex items-start gap-4">
          <div className="flex-shrink-0">
            <div className="w-12 h-12 rounded-full bg-[#10b981] flex items-center justify-center">
              <CheckCircle2 className="w-7 h-7 text-white" />
            </div>
          </div>

          <div className="flex-1 min-w-0">
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
              扫描完成
            </h3>
            <p className="text-sm text-[var(--text-secondary)] leading-relaxed">
              共发现 <span className="font-medium text-[var(--text-primary)]">{totalCount.toLocaleString()}</span> 个零碎文件，
              可清理空间 <span className="font-medium text-[#10b981]">{formatBytes(totalSize)}</span>
            </p>
          </div>
        </div>
      </motion.div>

      <div className="grid grid-cols-3 gap-4 mt-6">
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="text-center p-4 rounded-lg bg-[var(--bg-secondary)]"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <FileText className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {totalCount.toLocaleString()}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">文件总数</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="text-center p-4 rounded-lg bg-[var(--bg-secondary)]"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <FolderOpen className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {results.length}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">分类数量</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="text-center p-4 rounded-lg bg-[var(--bg-secondary)]"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <span className="text-lg">🗑️</span>
          </div>
          <p className="text-2xl font-semibold text-[#10b981]">
            {formatBytes(totalSize)}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">可清理空间</p>
        </motion.div>
      </div>

      {results.length > 0 && (
        <CategorizedFileList<JunkFileInfo, JunkCategoryInfo>
          categories={categories}
          selectedFiles={selectedFiles}
          expandedCategories={expandedTypes}
          getCategoryKey={(c) => c.key}
          getFileKey={(f) => f.path}
          onToggleFileSelection={onToggleSelection}
          onToggleCategorySelection={onToggleTypeSelection}
          onToggleCategoryExpand={onToggleExpand}
          onSelectAll={onSelectAll}
          onDeselectAll={onDeselectAll}
          onExpandAll={onExpandAll}
          onCollapseAll={onCollapseAll}
          onLoadMore={handleLoadMore}
          onOpenLocation={handleOpenLocation}
          totalCount={totalCount}
          totalSize={totalSize}
          selectedCount={selectedCount}
          selectedSize={selectedSize}
          renderFileRow={renderFileRow}
          title="文件分类"
        />
      )}

      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
        className="flex justify-center gap-4 mt-8"
      >
        <motion.button
          onClick={onDelete}
          disabled={selectedCount === 0}
          whileHover={{ scale: selectedCount > 0 ? 1.02 : 1 }}
          whileTap={{ scale: selectedCount > 0 ? 0.98 : 1 }}
          className="flex items-center gap-2 px-6 py-3 rounded-lg bg-gradient-to-r from-red-500 to-red-600 text-white font-medium shadow-lg shadow-red-500/25 hover:shadow-red-500/40 transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Trash2 className="w-5 h-5" />
          {selectedCount > 0 
            ? `删除选中的 ${selectedCount} 个文件` 
            : '请先选择要删除的文件'}
        </motion.button>
      </motion.div>
    </motion.div>
  );
};

const DeleteConfirmModal = ({
  isOpen,
  selectedCount,
  selectedSize,
  onConfirm,
  onCancel,
}: {
  isOpen: boolean;
  selectedCount: number;
  selectedSize: number;
  onConfirm: () => void;
  onCancel: () => void;
}) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.95 }}
        className="bg-[var(--bg-secondary)] rounded-2xl p-6 w-full max-w-md"
      >
        <div className="flex items-center gap-4 mb-6">
          <div className="w-12 h-12 rounded-xl bg-red-500/20 flex items-center justify-center">
            <Trash2 className="w-6 h-6 text-red-500" />
          </div>
          <div>
            <h3 className="text-lg font-semibold text-[var(--text-primary)]">确认删除</h3>
            <p className="text-sm text-[var(--text-secondary)]">此操作不可撤销</p>
          </div>
        </div>

        <p className="text-[var(--text-secondary)] mb-6">
          您即将删除 <span className="font-semibold text-[var(--text-primary)]">{selectedCount}</span> 个文件，
          释放空间 <span className="font-semibold text-[#10b981]">{formatBytes(selectedSize)}</span>。
          删除的文件将被移至回收站。
        </p>

        <div className="flex gap-3">
          <button
            onClick={onCancel}
            className="flex-1 px-4 py-2.5 rounded-lg border border-[var(--border-color)] text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] transition-colors"
          >
            取消
          </button>
          <button
            onClick={onConfirm}
            className="flex-1 px-4 py-2.5 rounded-lg bg-red-500 text-white font-medium hover:bg-red-600 transition-colors"
          >
            确认删除
          </button>
        </div>
      </motion.div>
    </div>
  );
};

export default function FileAnalysisPage() {
  const [analysisMode, setAnalysisMode] = useState<AnalysisMode>('largeFile');
  const [showFilterModal, setShowFilterModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);

  const largeFileStore = useLargeFileStore();
  const junkFileStore = useJunkFileStore();

  const {
    isScanning: isLargeFileScanning,
    scanProgress: largeFileProgress,
    files: largeFiles,
    filteredFiles,
    selectedFiles: largeFileSelected,
    filter,
    selectedDisk: largeFileDisk,
    error: largeFileError,
    setSelectedDisk: setLargeFileDisk,
    setFilter,
    toggleFileSelection: toggleLargeFileSelection,
    selectAll: selectAllLargeFiles,
    deselectAll: deselectAllLargeFiles,
    removeFiles: removeLargeFiles,
    startScan: startLargeFileScan,
    pauseScan: pauseLargeFileScan,
    resumeScan: resumeLargeFileScan,
    cancelScan: cancelLargeFileScan,
    initListeners: initLargeFileListeners,
    cleanup: cleanupLargeFile,
  } = largeFileStore;

  const {
    isScanning: isJunkScanning,
    isCompleted: isJunkCompleted,
    scanProgress: junkProgress,
    scanId: junkScanId,
    results: junkResults,
    selectedFiles: junkSelected,
    expandedTypes,
    selectedDisk: junkDisk,
    error: junkError,
    setSelectedDisk: setJunkDisk,
    toggleFileSelection: toggleJunkSelection,
    toggleTypeSelection,
    selectAll: selectAllJunk,
    deselectAll: deselectAllJunk,
    toggleTypeExpand,
    expandAll,
    collapseAll,
    removeFiles: removeJunkFiles,
    startScan: startJunkScan,
    cancelScan: cancelJunkScan,
    initListeners: initJunkListeners,
    cleanup: cleanupJunk,
  } = junkFileStore;

  const isScanning = analysisMode === 'largeFile' ? isLargeFileScanning : isJunkScanning;
  const error = analysisMode === 'largeFile' ? largeFileError : junkError;
  const selectedDisk = analysisMode === 'largeFile' ? largeFileDisk : junkDisk;
  const setSelectedDisk = analysisMode === 'largeFile' ? setLargeFileDisk : setJunkDisk;

  useEffect(() => {
    initLargeFileListeners();
    initJunkListeners();
    return () => {
      cleanupLargeFile();
      cleanupJunk();
    };
  }, []);

  const largeFilePageState = useMemo(() => {
    if (largeFileError) return 'error';
    if (isLargeFileScanning && !largeFileProgress) return 'initializing';
    const status = largeFileProgress?.status?.toLowerCase();
    if (status === 'scanning') return 'scanning';
    if (status === 'paused') return 'paused';
    if (status === 'completed') return 'completed';
    if (largeFiles.length > 0) return 'completed';
    return 'idle';
  }, [isLargeFileScanning, largeFileProgress, largeFiles.length, largeFileError]);

  const junkPageState = useMemo(() => {
    if (junkError) return 'error';
    if (isJunkScanning && !junkProgress) return 'initializing';
    if (isJunkCompleted) return 'completed'; // 优先检查完成标志
    const status = junkProgress?.status?.toLowerCase();
    if (status === 'scanning') return 'scanning';
    if (status === 'paused') return 'paused';
    if (status === 'completed') return 'completed';
    if (status === 'error') return 'error';
    if (junkResults.length > 0) return 'completed';
    return 'idle';
  }, [isJunkScanning, isJunkCompleted, junkProgress, junkResults.length, junkError]);

  const pageState = analysisMode === 'largeFile' ? largeFilePageState : junkPageState;

  const handleDelete = async () => {
    setShowDeleteModal(false);

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      
      if (analysisMode === 'largeFile') {
        await invoke('move_files_to_recycle_bin', {
          paths: Array.from(largeFileSelected),
        });
        removeLargeFiles(Array.from(largeFileSelected));
      } else {
        await invoke('move_files_to_recycle_bin', {
          paths: Array.from(junkSelected),
        });
        removeJunkFiles(Array.from(junkSelected));
      }
    } catch (error) {
      console.error('删除失败:', error);
      alert('删除失败: ' + (error instanceof Error ? error.message : '未知错误'));
    }
  };

  const largeFileSelectedSize = useSelectedSize(filteredFiles, largeFileSelected);
  const junkSelectedSize = useSelectedSizeFromResults(junkResults, junkSelected);

  const selectedCount = analysisMode === 'largeFile' ? largeFileSelected.size : junkSelected.size;
  const selectedSize = analysisMode === 'largeFile' ? largeFileSelectedSize : junkSelectedSize;

  const canStartScan = pageState === 'idle' || pageState === 'completed';

  const handleModeChange = (mode: AnalysisMode) => {
    if (canStartScan) {
      setAnalysisMode(mode);
    }
  };

  const renderLargeFileContent = () => {
    if (pageState === 'idle') {
      return (
        <motion.div
          key="idle"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -20 }}
          className="panel p-8 min-h-[400px] flex flex-col items-center justify-center"
        >
          <div className="relative mb-6">
            <div className="w-24 h-24 rounded-2xl bg-gradient-to-br from-[#6366f1]/20 to-[#8b5cf6]/20 flex items-center justify-center">
              <Database className="w-12 h-12 text-[#6366f1]" />
            </div>
            <motion.div
              className="absolute -top-1 -right-1 w-8 h-8 rounded-full bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] flex items-center justify-center"
              animate={{ scale: [1, 1.1, 1] }}
              transition={{ duration: 2, repeat: Infinity, ease: 'easeInOut' }}
            >
              <Sparkles className="w-4 h-4 text-white" />
            </motion.div>
          </div>

          <h3 className="text-xl font-semibold text-[var(--text-primary)] mb-2">
            大文件管理
          </h3>
          <p className="text-[var(--text-secondary)] text-sm text-center max-w-md mb-6">
            扫描磁盘找出占用空间较大的文件，帮助您释放磁盘空间
          </p>

          <div className="w-full max-w-md mb-6">
            <label className="block text-sm font-medium text-[var(--text-secondary)] mb-3">
              选择扫描磁盘
            </label>
            <DiskSelector
              selectedDisk={selectedDisk}
              onSelect={setSelectedDisk}
              disabled={isScanning}
            />
          </div>

          <div className="text-sm text-[var(--text-tertiary)] mb-6">
            当前设置：扫描 {filter.minSize}
            {filter.unit} 及以上的文件
          </div>

          <motion.button
            onClick={startLargeFileScan}
            disabled={!selectedDisk}
            whileHover={{ scale: !selectedDisk ? 1 : 1.02 }}
            whileTap={{ scale: !selectedDisk ? 1 : 0.98 }}
            className="flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white font-medium disabled:opacity-50 hover:shadow-lg hover:shadow-[#6366f1]/25 transition-all"
          >
            <Search className="w-5 h-5" />
            开始扫描
          </motion.button>
        </motion.div>
      );
    }

    if (pageState === 'scanning' || pageState === 'paused' || pageState === 'initializing') {
      return (
        <motion.div
          key="scanning"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="panel p-6 space-y-6"
        >
          <LargeFileScanProgress
            progress={largeFileProgress}
            status={largeFileProgress?.status ?? 'scanning'}
          />
          
          <div className="flex items-center justify-center gap-3 pt-4 border-t border-[var(--border-color)]">
            {(pageState === 'scanning' || pageState === 'initializing') && largeFileProgress?.status?.toLowerCase() !== 'paused' && (
              <motion.button
                onClick={pauseLargeFileScan}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-amber-500 text-white text-sm font-medium transition-all duration-200 hover:bg-amber-600"
              >
                <Pause className="w-4 h-4" />
                暂停扫描
              </motion.button>
            )}
            {largeFileProgress?.status?.toLowerCase() === 'paused' && (
              <motion.button
                onClick={resumeLargeFileScan}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-[#6366f1]/25"
              >
                <Play className="w-4 h-4" />
                继续扫描
              </motion.button>
            )}
            <motion.button
              onClick={cancelLargeFileScan}
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-500 text-white text-sm font-medium transition-all duration-200 hover:bg-red-600"
            >
              <X className="w-4 h-4" />
              取消扫描
            </motion.button>
          </div>
        </motion.div>
      );
    }

    if (pageState === 'completed') {
      return (
        <LargeFileList
          files={filteredFiles}
          selectedFiles={largeFileSelected}
          onToggleSelection={toggleLargeFileSelection}
          onSelectAll={selectAllLargeFiles}
          onDeselectAll={deselectAllLargeFiles}
          onDelete={() => setShowDeleteModal(true)}
        />
      );
    }

    return null;
  };

  const renderJunkFileContent = () => {
    if (pageState === 'idle') {
      return (
        <motion.div
          key="junk-idle"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -20 }}
          className="panel p-8 min-h-[400px] flex flex-col items-center justify-center"
        >
          <div className="relative mb-6">
            <div className="w-24 h-24 rounded-2xl bg-gradient-to-br from-[#f59e0b]/20 to-[#ef4444]/20 flex items-center justify-center">
              <FileX className="w-12 h-12 text-[#f59e0b]" />
            </div>
            <motion.div
              className="absolute -top-1 -right-1 w-8 h-8 rounded-full bg-gradient-to-r from-[#f59e0b] to-[#ef4444] flex items-center justify-center"
              animate={{ scale: [1, 1.1, 1] }}
              transition={{ duration: 2, repeat: Infinity, ease: 'easeInOut' }}
            >
              <Sparkles className="w-4 h-4 text-white" />
            </motion.div>
          </div>

          <h3 className="text-xl font-semibold text-[var(--text-primary)] mb-2">
            零碎文件扫描
          </h3>
          <p className="text-[var(--text-secondary)] text-sm text-center max-w-md mb-6">
            扫描磁盘中的空文件夹、无效快捷方式、过期日志等零碎文件
          </p>

          <div className="w-full max-w-md mb-6">
            <label className="block text-sm font-medium text-[var(--text-secondary)] mb-3">
              选择扫描磁盘
            </label>
            <DiskSelector
              selectedDisk={selectedDisk}
              onSelect={setSelectedDisk}
              disabled={isScanning}
            />
          </div>

          <div className="grid grid-cols-2 md:grid-cols-4 gap-3 w-full max-w-2xl mb-6">
            {[
              { name: '空文件夹', icon: '📁' },
              { name: '无效快捷方式', icon: '🔗' },
              { name: '过期日志', icon: '📝' },
              { name: '旧安装包', icon: '📦' },
            ].map((item) => (
              <div key={item.name} className="flex items-center gap-2 p-3 rounded-lg bg-[var(--bg-tertiary)]">
                <span className="text-lg">{item.icon}</span>
                <span className="text-xs text-[var(--text-secondary)]">{item.name}</span>
              </div>
            ))}
          </div>

          <motion.button
            onClick={startJunkScan}
            disabled={!selectedDisk}
            whileHover={{ scale: !selectedDisk ? 1 : 1.02 }}
            whileTap={{ scale: !selectedDisk ? 1 : 0.98 }}
            className="flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-[#f59e0b] to-[#ef4444] text-white font-medium disabled:opacity-50 hover:shadow-lg hover:shadow-[#f59e0b]/25 transition-all"
          >
            <Search className="w-5 h-5" />
            开始扫描
          </motion.button>
        </motion.div>
      );
    }

    if (pageState === 'scanning' || pageState === 'initializing') {
      return (
        <motion.div
          key="junk-scanning"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="panel p-6 space-y-6"
        >
          <JunkFileScanProgress 
            progress={junkProgress} 
            status={junkProgress?.status ?? 'scanning'}
          />
          
          <div className="flex items-center justify-center gap-3 pt-4 border-t border-[var(--border-color)]">
            <motion.button
              onClick={cancelJunkScan}
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-500 text-white text-sm font-medium transition-all duration-200 hover:bg-red-600"
            >
              <X className="w-4 h-4" />
              取消扫描
            </motion.button>
          </div>
        </motion.div>
      );
    }

    if (pageState === 'completed') {
      return (
        <JunkFileList
          results={junkResults}
          scanId={junkScanId}
          selectedFiles={junkSelected}
          expandedTypes={expandedTypes}
          onToggleSelection={toggleJunkSelection}
          onToggleTypeSelection={toggleTypeSelection}
          onSelectAll={selectAllJunk}
          onDeselectAll={deselectAllJunk}
          onToggleExpand={toggleTypeExpand}
          onExpandAll={expandAll}
          onCollapseAll={collapseAll}
          onDelete={() => setShowDeleteModal(true)}
        />
      );
    }

    return null;
  };

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-[var(--text-primary)]">
              文件分析
            </h1>
            <p className="text-[var(--text-secondary)] text-sm mt-1">
              {analysisMode === 'largeFile' ? '扫描并管理占用磁盘空间较大的文件' : '扫描磁盘中的零碎文件'}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <SegmentedControl
              options={[
                { value: 'largeFile', label: '大文件管理', icon: <Database className="w-4 h-4" />, disabled: !canStartScan },
                { value: 'junkFile', label: '零碎文件', icon: <FileX className="w-4 h-4" />, disabled: !canStartScan },
              ]}
              value={analysisMode}
              onChange={(mode) => handleModeChange(mode as AnalysisMode)}
              color={analysisMode === 'largeFile' ? 'linear-gradient(to right, #6366f1, #8b5cf6)' : 'linear-gradient(to right, #f59e0b, #ef4444)'}
            />
            {analysisMode === 'largeFile' && (pageState === 'idle' || pageState === 'completed') && (
              <motion.button
                onClick={() => setShowFilterModal(true)}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-secondary)] hover:border-[var(--color-primary)]/50 transition-all"
              >
                <Settings className="w-4 h-4" />
                设置
              </motion.button>
            )}
          </div>
        </div>

        <AnimatePresence>
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="flex items-center gap-3 p-4 rounded-xl bg-red-500/10 border border-red-500/20"
            >
              <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0" />
              <div className="flex-1">
                <p className="text-sm text-red-400">{error}</p>
                <p className="text-xs text-red-400/70 mt-1">请检查磁盘权限或稍后重试</p>
              </div>
              <motion.button
                onClick={analysisMode === 'largeFile' ? startLargeFileScan : startJunkScan}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-red-500 text-white text-sm font-medium"
              >
                <RotateCcw className="w-4 h-4" />
                重试
              </motion.button>
            </motion.div>
          )}
        </AnimatePresence>

        <AnimatePresence mode="wait">
          {analysisMode === 'largeFile' ? renderLargeFileContent() : renderJunkFileContent()}
        </AnimatePresence>

        <LargeFileFilterModal
          isOpen={showFilterModal}
          onClose={() => setShowFilterModal(false)}
          filter={filter}
          onApply={setFilter}
        />

        <DeleteConfirmModal
          isOpen={showDeleteModal}
          selectedCount={selectedCount}
          selectedSize={selectedSize}
          onConfirm={handleDelete}
          onCancel={() => setShowDeleteModal(false)}
        />
      </div>
    </div>
  );
}
