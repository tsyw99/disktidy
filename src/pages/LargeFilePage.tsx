/**
 * 大文件管理页面
 * 参考 ScanPage 的交互设计
 */

import { useEffect, useState, useMemo } from 'react';
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
} from 'lucide-react';
import { ProgressBar } from '../components/common';
import { useLargeFileStore } from '../stores/largeFileStore';
import { useSystemStore } from '../stores/systemStore';
import { formatBytes, formatDate } from '../utils/format';
import { openFileLocation } from '../utils/shell';
import type { LargeFile, LargeFileFilter } from '../types/largeFile';

// 文件图标组件
const FileIcon = ({ extension }: { extension: string }) => {
  const ext = extension.toLowerCase();
  if (['.mp4', '.avi', '.mkv', '.mov', '.wmv', '.flv', '.webm'].includes(ext)) {
    return <FileText className="w-5 h-5 text-purple-400" />;
  }
  if (['.zip', '.rar', '.7z', '.tar', '.gz', '.bz2'].includes(ext)) {
    return <Package className="w-5 h-5 text-yellow-400" />;
  }
  if (['.iso', '.img'].includes(ext)) {
    return <Database className="w-5 h-5 text-orange-400" />;
  }
  if (['.exe', '.msi', '.pkg', '.deb', '.rpm'].includes(ext)) {
    return <Package className="w-5 h-5 text-green-400" />;
  }
  if (['.mp3', '.wav', '.flac', '.aac', '.ogg'].includes(ext)) {
    return <FileText className="w-5 h-5 text-pink-400" />;
  }
  if (['.jpg', '.jpeg', '.png', '.gif', '.bmp', '.webp', '.svg'].includes(ext)) {
    return <FileText className="w-5 h-5 text-cyan-400" />;
  }
  if (['.pdf', '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx'].includes(ext)) {
    return <FileText className="w-5 h-5 text-blue-400" />;
  }
  return <FileText className="w-5 h-5 text-gray-400" />;
};

// 磁盘选择组件 - 参考 ScanPage 设计
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

  // 确保磁盘列表已加载
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

// 筛选设置弹窗 - 添加滑块和更精细的控制
const FilterModal = ({
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
  const maxSizeValue = localFilter.unit === 'MB' ? 10240 : 100; // 10GB max

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
          {/* 最小文件大小设置 */}
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
                  // 转换数值
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

            {/* 滑块 */}
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
              <span>
                {minSizeValue} {localFilter.unit}
              </span>
              <span>
                {maxSizeValue} {localFilter.unit}
              </span>
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

// 扫描进度组件 - 与磁盘扫描页面样式一致
const ScanProgressDisplay = ({
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
            {scannedSizeGB} GB
          </span>
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
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {scannedSizeGB}
          </p>
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

      {/* 扫描速度 */}
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

// 文件列表组件 - 添加删除功能
const FileList = ({
  files,
  selectedFiles,
  onToggleSelection,
  onSelectAll,
  onDeselectAll,
  onDelete,
  isDeleting,
}: {
  files: LargeFile[];
  selectedFiles: Set<string>;
  onToggleSelection: (path: string) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onDelete?: () => void;
  isDeleting?: boolean;
}) => {
  const allSelected = files.length > 0 && files.every((f) => selectedFiles.has(f.path));
  const selectedCount = selectedFiles.size;
  const selectedSize = useMemo(() => {
    let size = 0;
    for (const file of files) {
      if (selectedFiles.has(file.path)) {
        size += file.size;
      }
    }
    return size;
  }, [files, selectedFiles]);

  return (
    <div className="panel">
      {/* 工具栏 */}
      <div className="p-4 border-b border-[var(--border-color)] flex items-center justify-between">
        <div className="flex items-center gap-4">
          <button
            onClick={allSelected ? onDeselectAll : onSelectAll}
            className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
          >
            {allSelected ? (
              <CheckSquare className="w-5 h-5 text-[var(--color-primary)]" />
            ) : (
              <Square className="w-5 h-5" />
            )}
            {allSelected ? '取消全选' : '全选'}
          </button>
          <span className="text-sm text-[var(--text-secondary)]">
            共 {files.length} 个文件
            {selectedCount > 0 && (
              <span className="text-[var(--color-primary)] ml-2">
                已选择 {selectedCount} 个 ({formatBytes(selectedSize)})
              </span>
            )}
          </span>
        </div>
        {selectedCount > 0 && onDelete && (
          <motion.button
            onClick={onDelete}
            disabled={isDeleting}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-500 text-white text-sm font-medium hover:bg-red-600 transition-colors disabled:opacity-50"
          >
            <Trash2 className="w-4 h-4" />
            {isDeleting ? '删除中...' : '删除选中'}
          </motion.button>
        )}
      </div>

      {/* 文件列表 */}
      <div className="divide-y divide-[var(--border-color)] max-h-[500px] overflow-y-auto">
        {files.map((file, index) => (
          <motion.div
            key={file.path}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: index * 0.03 }}
            className="p-4 flex items-center gap-3 hover:bg-[var(--bg-tertiary)]/50 transition-colors group"
          >
            <button
              onClick={() => onToggleSelection(file.path)}
              className="flex-shrink-0"
            >
              {selectedFiles.has(file.path) ? (
                <CheckSquare className="w-5 h-5 text-[var(--color-primary)]" />
              ) : (
                <Square className="w-5 h-5 text-[var(--text-tertiary)]" />
              )}
            </button>

            <FileIcon extension={file.extension} />

            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-[var(--text-primary)] truncate">
                {file.name}
              </p>
              <p className="text-xs text-[var(--text-tertiary)] truncate">
                {file.path}
              </p>
            </div>

            <div className="text-right">
              <p className="text-sm font-medium text-[var(--text-primary)]">
                {formatBytes(file.size)}
              </p>
              <p className="text-xs text-[var(--text-tertiary)]">
                {formatDate(file.modified_time)}
              </p>
            </div>

            <button
              onClick={() => openFileLocation(file.path)}
              className="flex-shrink-0 p-2 rounded-lg opacity-0 group-hover:opacity-100 hover:bg-[var(--bg-secondary)] transition-all"
              title="打开文件位置"
            >
              <FolderOpen className="w-4 h-4 text-[var(--text-secondary)]" />
            </button>
          </motion.div>
        ))}
      </div>
    </div>
  );
};

// 删除确认弹窗
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
            <h3 className="text-lg font-semibold text-[var(--text-primary)]">
              确认删除
            </h3>
            <p className="text-sm text-[var(--text-secondary)]">
              此操作不可撤销
            </p>
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

// 主页面组件
export default function LargeFilePage() {
  const [showFilterModal, setShowFilterModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  const {
    isScanning,
    scanProgress,
    files,
    filteredFiles,
    selectedFiles,
    filter,
    searchQuery,
    selectedDisk,
    error,
    setSelectedDisk,
    setFilter,
    setSearchQuery,
    toggleFileSelection,
    selectAll,
    deselectAll,
    removeFiles,
    startScan,
    pauseScan,
    resumeScan,
    cancelScan,
    initListeners,
    cleanup,
  } = useLargeFileStore();

  // 初始化监听
  useEffect(() => {
    initListeners();
    return () => cleanup();
  }, []);

  // 页面状态
  const pageState = useMemo(() => {
    if (error) return 'error';
    if (isScanning && !scanProgress) return 'initializing';
    // Rust 返回的状态可能是大写的
    const status = scanProgress?.status?.toLowerCase();
    if (status === 'scanning') return 'scanning';
    if (status === 'paused') return 'paused';
    if (status === 'completed') return 'completed';
    if (files.length > 0) return 'completed';
    return 'idle';
  }, [isScanning, scanProgress, files.length, error]);

  // 处理删除
  const handleDelete = async () => {
    setShowDeleteModal(false);
    setIsDeleting(true);

    try {
      // 调用后端删除接口
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('move_files_to_recycle_bin', {
        paths: Array.from(selectedFiles),
      });

      // 从列表中移除已删除的文件
      removeFiles(Array.from(selectedFiles));
    } catch (error) {
      console.error('删除失败:', error);
      alert('删除失败: ' + (error instanceof Error ? error.message : '未知错误'));
    } finally {
      setIsDeleting(false);
    }
  };

  // 计算选中文件大小
  const selectedSize = useMemo(() => {
    let size = 0;
    for (const file of filteredFiles) {
      if (selectedFiles.has(file.path)) {
        size += file.size;
      }
    }
    return size;
  }, [filteredFiles, selectedFiles]);

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* 页面标题 */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-[var(--text-primary)]">
              大文件管理
            </h1>
            <p className="text-[var(--text-secondary)] text-sm mt-1">
              扫描并管理占用磁盘空间较大的文件
            </p>
          </div>

          <div className="flex items-center gap-3">
            {(pageState === 'idle' || pageState === 'completed') && (
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

        {/* 错误提示 */}
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
              </div>
              <motion.button
                onClick={startScan}
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

        {/* 主内容区域 */}
        <AnimatePresence mode="wait">
          {pageState === 'idle' && (
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
                onClick={startScan}
                disabled={!selectedDisk}
                whileHover={{ scale: !selectedDisk ? 1 : 1.02 }}
                whileTap={{ scale: !selectedDisk ? 1 : 0.98 }}
                className="flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white font-medium disabled:opacity-50 hover:shadow-lg hover:shadow-[#6366f1]/25 transition-all"
              >
                <Search className="w-5 h-5" />
                开始扫描
              </motion.button>
            </motion.div>
          )}

          {(pageState === 'scanning' || pageState === 'paused' || pageState === 'initializing') && (
            <motion.div
              key="scanning"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="panel p-6 space-y-6"
            >
              <ScanProgressDisplay
                progress={scanProgress}
                status={scanProgress?.status ?? 'scanning'}
              />
              
              <div className="flex items-center justify-center gap-3 pt-4 border-t border-[var(--border-color)]">
                {(pageState === 'scanning' || pageState === 'initializing') && scanProgress?.status?.toLowerCase() !== 'paused' && (
                  <motion.button
                    onClick={pauseScan}
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                    className="flex items-center gap-2 px-4 py-2 rounded-lg bg-amber-500 text-white text-sm font-medium transition-all duration-200 hover:bg-amber-600"
                  >
                    <Pause className="w-4 h-4" />
                    暂停扫描
                  </motion.button>
                )}
                {scanProgress?.status?.toLowerCase() === 'paused' && (
                  <motion.button
                    onClick={resumeScan}
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                    className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-[#6366f1]/25"
                  >
                    <Play className="w-4 h-4" />
                    继续扫描
                  </motion.button>
                )}
                <motion.button
                  onClick={cancelScan}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-500 text-white text-sm font-medium transition-all duration-200 hover:bg-red-600"
                >
                  <X className="w-4 h-4" />
                  取消扫描
                </motion.button>
              </div>
            </motion.div>
          )}

          {pageState === 'completed' && (
            <motion.div
              key="completed"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="space-y-4"
            >
              {/* 搜索栏和工具 */}
              <div className="flex items-center gap-4">
                <div className="flex-1 relative">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-[var(--text-tertiary)]" />
                  <input
                    type="text"
                    placeholder="搜索文件..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full pl-10 pr-4 py-2.5 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-primary)] focus:border-[var(--color-primary)]/50 focus:outline-none transition-colors"
                  />
                </div>
                <motion.button
                  onClick={startScan}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white font-medium hover:shadow-lg hover:shadow-[#6366f1]/25 transition-all"
                >
                  <RotateCcw className="w-4 h-4" />
                  重新扫描
                </motion.button>
              </div>

              {/* 文件列表 */}
              <FileList
                files={filteredFiles}
                selectedFiles={selectedFiles}
                onToggleSelection={toggleFileSelection}
                onSelectAll={selectAll}
                onDeselectAll={deselectAll}
                onDelete={() => setShowDeleteModal(true)}
                isDeleting={isDeleting}
              />
            </motion.div>
          )}
        </AnimatePresence>

        {/* 筛选弹窗 */}
        <FilterModal
          isOpen={showFilterModal}
          onClose={() => setShowFilterModal(false)}
          filter={filter}
          onApply={setFilter}
        />

        {/* 删除确认弹窗 */}
        <DeleteConfirmModal
          isOpen={showDeleteModal}
          selectedCount={selectedFiles.size}
          selectedSize={selectedSize}
          onConfirm={handleDelete}
          onCancel={() => setShowDeleteModal(false)}
        />
      </div>
    </div>
  );
}
