
import { useState, useEffect, useMemo } from 'react';
import { motion, AnimatePresence, type Transition } from 'framer-motion';
import { SegmentedControl } from '../components/common';
import {
  HardDrive,
  Trash2,
  Settings,
  RefreshCw,
  CheckSquare,
  Square,
  ExternalLink,
  AlertTriangle,
  FileText,
  FolderX,
  Link,
  Package,
  Download,
  Search,
  X,
  AlertCircle,
  ChevronRight,
  Pause,
  Play,
  Sparkles,
  FolderOpen,
  Layers,
  Database,
  Zap,
} from 'lucide-react';
import { useFileAnalysisStore, useFileAnalysisActions } from '../stores/fileAnalysisStore';
import { useSystemStore, useSystemActions } from '../stores/systemStore';
import { useUIStore } from '../stores';
import type { LargeFile, JunkFileType, JunkFileItem, JunkScanResult, LargeFileScanProgress } from '../types';
import { formatBytes, formatDate } from '../utils/format';
import { openFileLocation } from '../utils/shell';
import { invoke } from '@tauri-apps/api/core';

const pageVariants = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 },
};

const pageTransition: Transition = {
  type: 'spring',
  stiffness: 300,
  damping: 30,
};

const getFileIcon = (fileType: string): React.ReactNode => {
  const iconClass = 'w-5 h-5';
  switch (fileType.toLowerCase()) {
    case '.mp4':
    case '.avi':
    case '.mkv':
    case '.mov':
      return <FileText className={`${iconClass} text-purple-400`} />;
    case '.zip':
    case '.rar':
    case '.7z':
      return <Package className={`${iconClass} text-yellow-400`} />;
    case '.iso':
    case '.img':
      return <HardDrive className={`${iconClass} text-orange-400`} />;
    case '.exe':
    case '.msi':
      return <Package className={`${iconClass} text-green-400`} />;
    case '.log':
    case '.txt':
      return <FileText className={`${iconClass} text-gray-400`} />;
    default:
      return <FileText className={`${iconClass} text-blue-400`} />;
  }
};

const getJunkTypeIcon = (type: JunkFileType): React.ReactNode => {
  const iconClass = 'w-5 h-5';
  switch (type) {
    case 'empty_folders':
      return <FolderX className={`${iconClass} text-gray-400`} />;
    case 'invalid_shortcuts':
      return <Link className={`${iconClass} text-red-400`} />;
    case 'old_logs':
      return <FileText className={`${iconClass} text-yellow-400`} />;
    case 'old_installers':
      return <Package className={`${iconClass} text-orange-400`} />;
    case 'invalid_downloads':
      return <Download className={`${iconClass} text-purple-400`} />;
    case 'small_files':
      return <FileText className={`${iconClass} text-blue-400`} />;
    case 'orphaned_files':
      return <AlertTriangle className={`${iconClass} text-amber-400`} />;
    default:
      return <FileText className={`${iconClass} text-gray-400`} />;
  }
};

const getJunkTypeName = (type: JunkFileType): string => {
  const names: Record<JunkFileType, string> = {
    empty_folders: '空文件夹',
    invalid_shortcuts: '无效快捷方式',
    old_logs: '过期日志文件',
    old_installers: '旧版本安装包',
    invalid_downloads: '无效下载文件',
    small_files: '零散小文件',
    orphaned_files: '孤立文件',
  };
  return names[type];
};

function FileRow({
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
      
      <div className="flex-shrink-0">
        {getFileIcon(file.extension)}
      </div>
      
      <div className="flex-1 min-w-0">
        <p className="text-sm text-[var(--text-primary)] truncate" title={file.name}>
          {file.name}
        </p>
        <p className="text-xs text-[var(--text-tertiary)] truncate" title={file.path}>
          {file.path}
        </p>
        <div className="flex items-center gap-3 mt-1">
          <span className="text-xs font-medium text-[var(--color-primary)]">
            {formatBytes(file.size)}
          </span>
          <span className="text-xs text-[var(--text-tertiary)]">
            {formatDate(file.modified_time)}
          </span>
          <span className="text-xs text-[var(--text-tertiary)]">
            {file.file_type.toUpperCase()}
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

function LargeFileList({
  files,
  selectedFiles,
  onToggleFileSelection,
}: {
  files: LargeFile[];
  selectedFiles: Set<string>;
  onToggleFileSelection: (path: string) => void;
}) {
  const [expandedTypes, setExpandedTypes] = useState<Set<string>>(new Set());
  
  const groupedFiles = useMemo(() => {
    const groups: Map<string, LargeFile[]> = new Map();
    files.forEach(file => {
      const type = file.file_type || 'Other';
      if (!groups.has(type)) {
        groups.set(type, []);
      }
      groups.get(type)!.push(file);
    });
    
    return Array.from(groups.entries())
      .map(([type, typeFiles]) => ({
        type,
        files: typeFiles.sort((a, b) => b.size - a.size),
        totalSize: typeFiles.reduce((sum, f) => sum + f.size, 0),
      }))
      .sort((a, b) => b.totalSize - a.totalSize);
  }, [files]);

  const toggleTypeExpand = (type: string) => {
    setExpandedTypes(prev => {
      const newSet = new Set(prev);
      if (newSet.has(type)) {
        newSet.delete(type);
      } else {
        newSet.add(type);
      }
      return newSet;
    });
  };

  const selectAllInType = (type: string, select: boolean) => {
    const group = groupedFiles.find(g => g.type === type);
    if (group) {
      group.files.forEach(file => {
        const isSelected = selectedFiles.has(file.path);
        if (select && !isSelected) {
          onToggleFileSelection(file.path);
        } else if (!select && isSelected) {
          onToggleFileSelection(file.path);
        }
      });
    }
  };

  useEffect(() => {
    if (groupedFiles.length > 0 && expandedTypes.size === 0) {
      setExpandedTypes(new Set(groupedFiles.slice(0, 3).map(g => g.type)));
    }
  }, [groupedFiles.length]);

  return (
    <div className="space-y-3">
      {groupedFiles.map((group, index) => {
        const isExpanded = expandedTypes.has(group.type);
        const selectedInGroup = group.files.filter(f => selectedFiles.has(f.path)).length;
        const allSelected = selectedInGroup === group.files.length;
        const someSelected = selectedInGroup > 0 && !allSelected;

        return (
          <motion.div
            key={group.type}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ delay: index * 0.05 }}
            className="rounded-xl border border-[var(--border-color)] overflow-hidden bg-[var(--bg-secondary)]/50"
          >
            <div 
              className="flex items-center gap-3 p-4 bg-[var(--bg-secondary)] cursor-pointer hover:bg-[var(--bg-tertiary)] transition-colors"
              onClick={() => toggleTypeExpand(group.type)}
            >
              <motion.div
                animate={{ rotate: isExpanded ? 90 : 0 }}
                transition={{ duration: 0.2 }}
              >
                <ChevronRight className="w-4 h-4 text-[var(--text-tertiary)]" />
              </motion.div>
              
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  selectAllInType(group.type, !allSelected);
                }}
                className="flex-shrink-0"
              >
                {allSelected ? (
                  <CheckSquare className="w-5 h-5 text-[var(--color-primary)]" />
                ) : someSelected ? (
                  <div className="w-5 h-5 rounded border-2 border-[var(--color-primary)] bg-[var(--color-primary)]/20 flex items-center justify-center">
                    <div className="w-2 h-2 rounded-sm bg-[var(--color-primary)]" />
                  </div>
                ) : (
                  <Square className="w-5 h-5 text-[var(--text-tertiary)]" />
                )}
              </button>
              
              <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-gradient-to-br from-purple-500/20 to-blue-500/20 flex items-center justify-center">
                {getFileIcon(group.type)}
              </div>
              
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-[var(--text-primary)]">
                    {group.type.toUpperCase()}
                  </span>
                  <span className="text-xs text-[var(--text-tertiary)]">
                    ({group.files.length} 个文件)
                  </span>
                </div>
                {selectedInGroup > 0 && (
                  <p className="text-xs text-[var(--color-primary)] mt-0.5">
                    已选择 {selectedInGroup} 个文件
                  </p>
                )}
              </div>
              
              <div className="text-right">
                <span className="text-sm font-semibold text-[#10b981]">
                  {formatBytes(group.totalSize)}
                </span>
              </div>
            </div>
            
            <AnimatePresence>
              {isExpanded && (
                <motion.div
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: 'auto', opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.2 }}
                  className="overflow-hidden"
                >
                  <div className="max-h-[400px] overflow-y-auto scrollbar-thin bg-[var(--bg-primary)]/50">
                    {group.files.map((file, idx) => (
                      <FileRow
                        key={file.path}
                        file={file}
                        isSelected={selectedFiles.has(file.path)}
                        isLast={idx === group.files.length - 1}
                        onToggleSelection={() => onToggleFileSelection(file.path)}
                      />
                    ))}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        );
      })}
    </div>
  );
}

const JunkFileCard = ({ item, selected, onToggle }: { item: JunkFileItem; selected: boolean; onToggle: () => void }) => {
  return (
    <motion.div
      layout
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 10 }}
      className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-all duration-200 ${
        selected ? 'bg-[var(--color-primary)]/10 border border-[var(--color-primary)]/30' : 'hover:bg-[var(--bg-tertiary)]'
      }`}
      onClick={onToggle}
    >
      <button
        className="flex-shrink-0"
        onClick={(e) => {
          e.stopPropagation();
          onToggle();
        }}
      >
        {selected ? (
          <CheckSquare className="w-4 h-4 text-[var(--color-primary)]" />
        ) : (
          <Square className="w-4 h-4 text-[var(--text-secondary)]" />
        )}
      </button>

      {getJunkTypeIcon(item.file_type)}
      
      <div className="flex-1 min-w-0">
        <p className="text-sm text-[var(--text-primary)] truncate" title={item.path}>
          {item.path.split('\\').pop()}
        </p>
        <p className="text-xs text-[var(--text-tertiary)] truncate" title={item.path}>
          {item.path}
        </p>
        {!item.safe_to_delete && (
          <p className="text-xs text-amber-500 mt-0.5">
            <AlertTriangle className="w-3 h-3 inline mr-1" />
            {item.risk_level}
          </p>
        )}
      </div>

      <div className="flex items-center gap-3 text-xs text-[var(--text-secondary)]">
        {item.size > 0 && <span>{formatBytes(item.size)}</span>}
        {item.modified_time > 0 && <span>{formatDate(item.modified_time)}</span>}
        <button
          onClick={(e) => {
            e.stopPropagation();
            openFileLocation(item.path);
          }}
          className="p-1 rounded hover:bg-[var(--bg-tertiary)] hover:text-[var(--color-primary)] transition-colors"
        >
          <ExternalLink className="w-3 h-3" />
        </button>
      </div>
    </motion.div>
  );
};

const JunkCategoryPanel = ({ result, selectedIds, onToggle }: { 
  result: JunkScanResult; 
  selectedIds: Set<string>;
  onToggle: (id: string) => void;
}) => {
  const [expanded, setExpanded] = useState(true);
  const allSelected = result.items.every((item: JunkFileItem) => selectedIds.has(item.id));
  const someSelected = result.items.some((item: JunkFileItem) => selectedIds.has(item.id));

  const handleSelectAll = () => {
    result.items.forEach((item: JunkFileItem) => {
      if (allSelected) {
        if (selectedIds.has(item.id)) onToggle(item.id);
      } else {
        if (!selectedIds.has(item.id)) onToggle(item.id);
      }
    });
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="rounded-xl border border-[var(--border-color)] overflow-hidden bg-[var(--bg-secondary)]/50"
    >
      <div 
        className="flex items-center gap-3 p-4 bg-[var(--bg-secondary)] cursor-pointer hover:bg-[var(--bg-tertiary)] transition-colors"
        onClick={() => setExpanded(!expanded)}
      >
        <motion.div
          animate={{ rotate: expanded ? 90 : 0 }}
          transition={{ duration: 0.2 }}
        >
          <ChevronRight className="w-4 h-4 text-[var(--text-tertiary)]" />
        </motion.div>
        
        <button onClick={(e) => { e.stopPropagation(); handleSelectAll(); }}>
          {allSelected ? (
            <CheckSquare className="w-5 h-5 text-[var(--color-primary)]" />
          ) : someSelected ? (
            <div className="w-5 h-5 rounded border-2 border-[var(--color-primary)] bg-[var(--color-primary)]/30 flex items-center justify-center">
              <div className="w-2.5 h-0.5 bg-[var(--color-primary)] rounded" />
            </div>
          ) : (
            <Square className="w-5 h-5 text-[var(--text-secondary)]" />
          )}
        </button>
        
        <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-gradient-to-br from-orange-500/20 to-red-500/20 flex items-center justify-center">
          {getJunkTypeIcon(result.file_type)}
        </div>
        
        <div className="flex-1">
          <h4 className="text-sm font-medium text-[var(--text-primary)]">{getJunkTypeName(result.file_type)}</h4>
          <p className="text-xs text-[var(--text-secondary)]">
            {result.count} 项 {result.total_size > 0 && `· ${formatBytes(result.total_size)}`}
          </p>
        </div>

        <span className="text-sm font-semibold text-[#10b981]">
          {formatBytes(result.total_size)}
        </span>
      </div>

      <AnimatePresence>
        {expanded && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: 'auto' }}
            exit={{ height: 0 }}
            className="overflow-hidden"
          >
            <div className="px-4 pb-4 space-y-2 max-h-[300px] overflow-y-auto bg-[var(--bg-primary)]/50">
              {result.items.map((item) => (
                <JunkFileCard
                  key={item.id}
                  item={item}
                  selected={selectedIds.has(item.id)}
                  onToggle={() => onToggle(item.id)}
                />
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
};

const FilterSettingsModal = ({ 
  isOpen, 
  onClose, 
  filter, 
  onApply 
}: { 
  isOpen: boolean; 
  onClose: () => void; 
  filter: { minSize: number; unit: 'MB' | 'GB' };
  onApply: (filter: { minSize: number; unit: 'MB' | 'GB' }) => void;
}) => {
  const [minSize, setMinSize] = useState(filter.minSize);
  const [unit, setUnit] = useState<'MB' | 'GB'>(filter.unit);

  const presets = [
    { label: '100 MB', value: 100, unit: 'MB' as const },
    { label: '500 MB', value: 500, unit: 'MB' as const },
    { label: '1 GB', value: 1, unit: 'GB' as const },
    { label: '5 GB', value: 5, unit: 'GB' as const },
    { label: '10 GB', value: 10, unit: 'GB' as const },
  ];

  if (!isOpen) return null;

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <motion.div
        initial={{ scale: 0.95, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.95, opacity: 0 }}
        className="w-full max-w-md mx-4 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between p-4 border-b border-[var(--border-color)]">
          <h3 className="text-lg font-semibold text-[var(--text-primary)]">筛选设置</h3>
          <button onClick={onClose} className="p-1 rounded hover:bg-[var(--bg-tertiary)] transition-colors">
            <X className="w-5 h-5 text-[var(--text-secondary)]" />
          </button>
        </div>

        <div className="p-4 space-y-4">
          <div>
            <label className="block text-sm font-medium text-[var(--text-primary)] mb-2">
              文件大小阈值
            </label>
            <div className="flex gap-2">
              <input
                type="number"
                value={minSize}
                onChange={(e) => setMinSize(Number(e.target.value))}
                className="flex-1 px-3 py-2 bg-[var(--bg-tertiary)] border border-[var(--border-color)] rounded-lg text-[var(--text-primary)] focus:outline-none focus:border-[var(--color-primary)]"
              />
              <select
                value={unit}
                onChange={(e) => setUnit(e.target.value as 'MB' | 'GB')}
                className="px-3 py-2 bg-[var(--bg-tertiary)] border border-[var(--border-color)] rounded-lg text-[var(--text-primary)] focus:outline-none focus:border-[var(--color-primary)]"
              >
                <option value="MB">MB</option>
                <option value="GB">GB</option>
              </select>
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-[var(--text-primary)] mb-2">
              快速选择
            </label>
            <div className="flex flex-wrap gap-2">
              {presets.map((preset) => (
                <button
                  key={preset.label}
                  onClick={() => {
                    setMinSize(preset.value);
                    setUnit(preset.unit);
                  }}
                  className={`px-3 py-1.5 rounded-lg text-sm transition-colors ${
                    minSize === preset.value && unit === preset.unit
                      ? 'bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white'
                      : 'bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:bg-[var(--color-primary)]/10 hover:text-[var(--color-primary)]'
                  }`}
                >
                  {preset.label}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="flex justify-end gap-2 p-4 border-t border-[var(--border-color)]">
          <button
            onClick={onClose}
            className="px-4 py-2 rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
          >
            取消
          </button>
          <button
            onClick={() => {
              onApply({ minSize, unit });
              onClose();
            }}
            className="px-4 py-2 rounded-lg bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white hover:opacity-90 transition-opacity"
          >
            应用
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
};

const ConfirmDialog = ({
  isOpen,
  onClose,
  onConfirm,
  title,
  message,
  confirmText = '确认',
  danger = false,
}: {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  message: string;
  confirmText?: string;
  danger?: boolean;
}) => {
  if (!isOpen) return null;

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <motion.div
        initial={{ scale: 0.95, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.95, opacity: 0 }}
        className="w-full max-w-sm mx-4 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="p-6 text-center">
          <div className={`w-12 h-12 mx-auto mb-4 rounded-full flex items-center justify-center ${
            danger ? 'bg-red-500/10' : 'bg-[var(--color-primary)]/10'
          }`}>
            <AlertTriangle className={`w-6 h-6 ${danger ? 'text-red-500' : 'text-[var(--color-primary)]'}`} />
          </div>
          <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">{title}</h3>
          <p className="text-sm text-[var(--text-secondary)]">{message}</p>
        </div>

        <div className="flex gap-2 p-4 border-t border-[var(--border-color)]">
          <button
            onClick={onClose}
            className="flex-1 px-4 py-2 rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
          >
            取消
          </button>
          <button
            onClick={() => {
              onConfirm();
              onClose();
            }}
            className={`flex-1 px-4 py-2 rounded-lg text-white transition-opacity hover:opacity-90 ${
              danger ? 'bg-red-500' : 'bg-gradient-to-r from-[#6366f1] to-[#8b5cf6]'
            }`}
          >
            {confirmText}
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
};

function DiskSelector({ disabled = false }: { disabled?: boolean }) {
  const { diskList } = useSystemStore();
  const { selectedDisk } = useFileAnalysisStore();
  const actions = useFileAnalysisActions();

  const formatDiskSize = (bytes: number): string => {
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1000) {
      return `${(gb / 1024).toFixed(1)} TB`;
    }
    return `${gb.toFixed(0)} GB`;
  };

  return (
    <div className="space-y-3">
      <label className="block text-sm font-medium text-[var(--text-secondary)]">
        选择扫描磁盘
      </label>
      <div className="space-y-2">
        {diskList.slice(0, 4).map((disk) => {
          const isSelected = selectedDisk === disk.mount_point;
          const usedPercent = ((disk.total_size - disk.free_size) / disk.total_size) * 100;
          
          return (
            <motion.button
              key={disk.mount_point}
              whileHover={{ scale: disabled ? 1 : 1.01 }}
              whileTap={{ scale: disabled ? 1 : 0.99 }}
              onClick={() => !disabled && actions.setSelectedDisk(disk.mount_point)}
              disabled={disabled}
              className={`w-full flex items-center gap-3 p-3 rounded-xl border transition-all duration-200 ${
                isSelected
                  ? 'border-[var(--color-primary)] bg-[var(--color-primary)]/5 shadow-sm'
                  : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50 hover:bg-[var(--bg-tertiary)]'
              } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
            >
              <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                isSelected 
                  ? 'bg-gradient-to-br from-[#6366f1] to-[#8b5cf6]' 
                  : 'bg-[var(--bg-tertiary)]'
              }`}>
                <HardDrive className={`w-5 h-5 ${isSelected ? 'text-white' : 'text-[var(--color-primary)]'}`} />
              </div>
              <div className="flex-1 text-left">
                <p className="text-sm font-medium text-[var(--text-primary)]">{disk.mount_point}</p>
                <div className="flex items-center gap-2 mt-1">
                  <div className="flex-1 h-1.5 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
                    <div 
                      className={`h-full rounded-full ${
                        usedPercent > 80 ? 'bg-red-500' : usedPercent > 60 ? 'bg-amber-500' : 'bg-[#10b981]'
                      }`}
                      style={{ width: `${usedPercent}%` }}
                    />
                  </div>
                  <span className="text-xs text-[var(--text-tertiary)]">
                    {formatDiskSize(disk.free_size)} 可用
                  </span>
                </div>
              </div>
              {isSelected && (
                <motion.div
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  className="w-2.5 h-2.5 rounded-full bg-[var(--color-primary)]"
                />
              )}
            </motion.button>
          );
        })}
      </div>
    </div>
  );
}

// 空状态组件 - 大文件管理
const LargeFileEmptyState = () => (
  <motion.div
    variants={pageVariants}
    initial="initial"
    animate="animate"
    exit="exit"
    transition={pageTransition}
    className="panel p-8 min-h-[400px] flex flex-col items-center justify-center"
  >
    <div className="relative">
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
    
    <h3 className="text-xl font-semibold text-[var(--text-primary)] mt-6 mb-2">
      大文件管理
    </h3>
    <p className="text-[var(--text-secondary)] text-sm text-center max-w-md mb-6">
      扫描磁盘找出占用空间较大的文件，帮助您释放磁盘空间
    </p>

    <div className="grid grid-cols-2 md:grid-cols-4 gap-3 w-full max-w-2xl mb-6">
      {[
        { name: '视频文件', icon: '🎬' },
        { name: '压缩包', icon: '📦' },
        { name: '镜像文件', icon: '💿' },
        { name: '安装包', icon: '⚙️' },
      ].map((item) => (
        <div key={item.name} className="flex items-center gap-2 p-3 rounded-lg bg-[var(--bg-tertiary)]">
          <span className="text-lg">{item.icon}</span>
          <span className="text-xs text-[var(--text-secondary)]">{item.name}</span>
        </div>
      ))}
    </div>

    <div className="flex items-center gap-2 text-xs text-[var(--text-tertiary)]">
      <FolderOpen className="w-4 h-4" />
      <span>选择磁盘后点击"开始扫描"</span>
    </div>
  </motion.div>
);

// 空状态组件 - 零碎文件
const JunkFileEmptyState = () => (
  <motion.div
    variants={pageVariants}
    initial="initial"
    animate="animate"
    exit="exit"
    transition={pageTransition}
    className="panel p-8 min-h-[400px] flex flex-col items-center justify-center"
  >
    <div className="relative">
      <div className="w-24 h-24 rounded-2xl bg-gradient-to-br from-orange-500/20 to-red-500/20 flex items-center justify-center">
        <Trash2 className="w-12 h-12 text-orange-500" />
      </div>
      <motion.div
        className="absolute -top-1 -right-1 w-8 h-8 rounded-full bg-gradient-to-r from-orange-500 to-red-500 flex items-center justify-center"
        animate={{ scale: [1, 1.1, 1] }}
        transition={{ duration: 2, repeat: Infinity, ease: 'easeInOut' }}
      >
        <Sparkles className="w-4 h-4 text-white" />
      </motion.div>
    </div>
    
    <h3 className="text-xl font-semibold text-[var(--text-primary)] mt-6 mb-2">
      零碎文件清理
    </h3>
    <p className="text-[var(--text-secondary)] text-sm text-center max-w-md mb-6">
      检测系统中的无效文件和零碎文件，一键清理释放空间
    </p>

    <div className="grid grid-cols-2 md:grid-cols-4 gap-3 w-full max-w-2xl mb-6">
      {[
        { name: '空文件夹', icon: '📁' },
        { name: '无效快捷方式', icon: '🔗' },
        { name: '过期日志', icon: '📝' },
        { name: '旧安装包', icon: '💾' },
      ].map((item) => (
        <div key={item.name} className="flex items-center gap-2 p-3 rounded-lg bg-[var(--bg-tertiary)]">
          <span className="text-lg">{item.icon}</span>
          <span className="text-xs text-[var(--text-secondary)]">{item.name}</span>
        </div>
      ))}
    </div>

    <div className="flex items-center gap-2 text-xs text-[var(--text-tertiary)]">
      <Zap className="w-4 h-4" />
      <span>点击"开始扫描"检测零碎文件</span>
    </div>
  </motion.div>
);

// 扫描进度组件
function ScanProgressBar({ 
  progress, 
  isPaused, 
  onPause, 
  onResume, 
  onCancel 
}: { 
  progress: LargeFileScanProgress;
  isPaused: boolean;
  onPause: () => void;
  onResume: () => void;
  onCancel: () => void;
}) {
  return (
    <motion.div
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={pageTransition}
      className="panel p-6 space-y-6"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <span className="text-sm font-medium text-[var(--text-primary)]">扫描进度</span>
          {isPaused && (
            <span className="flex items-center gap-1 text-xs text-amber-500 bg-amber-500/10 px-2 py-0.5 rounded-full">
              <Pause className="w-3 h-3" />
              已暂停
            </span>
          )}
        </div>
        <span className="text-sm font-semibold text-[var(--color-primary)]">
          {isPaused ? '已暂停' : `${Math.round(progress.percent)}%`}
        </span>
      </div>

      <div className="relative">
        <div className="h-3 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
          <motion.div
            className="h-full rounded-full"
            style={{
              background: isPaused 
                ? 'linear-gradient(to right, #f59e0b, #d97706)'
                : 'linear-gradient(to right, #6366f1, #8b5cf6)',
            }}
            initial={{ width: 0 }}
            animate={{ width: `${progress.percent}%` }}
            transition={{ duration: 0.3 }}
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
            {progress.scanned_files.toLocaleString()}
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
            <Database className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[var(--text-primary)]">
            {progress.found_files.toLocaleString()}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">发现大文件</p>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="text-center"
        >
          <div className="flex items-center justify-center gap-2 mb-2">
            <Layers className="w-5 h-5 text-[var(--text-secondary)]" />
          </div>
          <p className="text-2xl font-semibold text-[#10b981]">
            {formatBytes(progress.scanned_size)}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">已扫描大小</p>
        </motion.div>
      </div>

      {progress.current_path && (
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

      <div className="flex items-center justify-center gap-3 pt-4 border-t border-[var(--border-color)]">
        {isPaused ? (
          <motion.button
            onClick={onResume}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-purple-500/25"
          >
            <Play className="w-4 h-4" />
            继续扫描
          </motion.button>
        ) : (
          <motion.button
            onClick={onPause}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-amber-500 text-white text-sm font-medium transition-all duration-200 hover:bg-amber-600"
          >
            <Pause className="w-4 h-4" />
            暂停扫描
          </motion.button>
        )}
        <motion.button
          onClick={onCancel}
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

// 统计卡片组件
function StatCard({ 
  icon, 
  value, 
  label, 
  color = 'primary',
  delay = 0 
}: { 
  icon: React.ReactNode; 
  value: string | number; 
  label: string; 
  color?: 'primary' | 'success' | 'warning';
  delay?: number;
}) {
  const colorClasses = {
    primary: 'text-[var(--color-primary)]',
    success: 'text-[#10b981]',
    warning: 'text-amber-500',
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay }}
      className="text-center p-5 rounded-xl bg-[var(--bg-secondary)] border border-[var(--border-color)]/50 hover:border-[var(--color-primary)]/30 transition-colors"
    >
      <div className="flex items-center justify-center gap-2 mb-3">
        {icon}
      </div>
      <p className={`text-2xl font-semibold ${colorClasses[color]}`}>
        {value}
      </p>
      <p className="text-sm text-[var(--text-tertiary)] mt-1">{label}</p>
    </motion.div>
  );
}

export default function AnalyzePage() {
  const {
    activeTab,
    selectedDisk,
    largeFiles,
    filteredLargeFiles,
    junkResults,
    selectedLargeFiles,
    selectedJunkFiles,
    isScanning,
    scanProgress,
    filter,
    searchQuery,
    error,
    scanResult,
  } = useFileAnalysisStore();
  
  const actions = useFileAnalysisActions();
  const { diskList } = useSystemStore();
  const systemActions = useSystemActions();
  const setIsWorking = useUIStore((state) => state.actions.setIsWorking);
  
  const [showFilterModal, setShowFilterModal] = useState(false);
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [confirmAction, setConfirmAction] = useState<{ action: () => void; title: string; message: string } | null>(null);

  useEffect(() => {
    if (diskList.length === 0) {
      systemActions.fetchDiskList();
    }
  }, [diskList.length, systemActions]);

  useEffect(() => {
    setIsWorking(isScanning);
  }, [isScanning, setIsWorking]);

  const handleScan = async () => {
    if (activeTab === 'large') {
      await actions.scanLargeFiles(selectedDisk, filter.minSize, filter.unit);
    } else {
      await actions.scanJunkFiles();
    }
  };

  const handleDeleteSelected = async () => {
    if (activeTab === 'large') {
      const count = selectedLargeFiles.size;
      const totalSize = filteredLargeFiles
        .filter(f => selectedLargeFiles.has(f.path))
        .reduce((sum, f) => sum + f.size, 0);
      
      setConfirmAction({
        action: async () => {
          const paths = Array.from(selectedLargeFiles);
          try {
            await invoke('clean_files', { 
              files: paths, 
              options: { move_to_recycle_bin: true } 
            });
            actions.deselectAllLargeFiles();
            await actions.scanLargeFiles(selectedDisk, filter.minSize, filter.unit);
          } catch (err) {
            console.error('删除文件失败:', err);
          }
        },
        title: '确认删除文件',
        message: `即将删除 ${count} 个文件，共 ${formatBytes(totalSize)}。此操作不可撤销。`,
      });
    } else {
      const allItems = junkResults.flatMap(r => r.items);
      const selectedItems = allItems.filter(i => selectedJunkFiles.has(i.id));
      const safeItems = selectedItems.filter(i => i.safe_to_delete);
      const unsafeItems = selectedItems.filter(i => !i.safe_to_delete);
      
      const count = safeItems.length;
      const totalSize = safeItems.reduce((sum, i) => sum + i.size, 0);
      
      if (count === 0) {
        setConfirmAction({
          action: () => {},
          title: '无法删除',
          message: `所选 ${selectedItems.length} 项均不安全删除。请检查风险等级。`,
        });
        return;
      }
      
      setConfirmAction({
        action: async () => {
          const paths = safeItems.map(i => i.path);
          try {
            await invoke('clean_files', { 
              files: paths, 
              options: { move_to_recycle_bin: true } 
            });
            actions.deselectAllJunkFiles();
            await actions.scanJunkFiles();
          } catch (err) {
            console.error('清理文件失败:', err);
          }
        },
        title: '确认清理文件',
        message: unsafeItems.length > 0 
          ? `即将清理 ${count} 个安全文件（${formatBytes(totalSize)}），跳过 ${unsafeItems.length} 个高风险文件。`
          : `即将清理 ${count} 项，共 ${formatBytes(totalSize)}。`,
      });
    }
    setShowConfirmDialog(true);
  };

  const selectedLargeFileSize = filteredLargeFiles
    .filter(f => selectedLargeFiles.has(f.path))
    .reduce((sum, f) => sum + f.size, 0);

  const totalJunkSize = junkResults.reduce((sum, r) => sum + r.total_size, 0);
  const selectedJunkSize = junkResults
    .flatMap(r => r.items)
    .filter(i => selectedJunkFiles.has(i.id))
    .reduce((sum, i) => sum + i.size, 0);

  const isPaused = scanProgress?.status === 'paused';

  // 页面状态
  const pageState = useMemo(() => {
    if (isScanning || scanProgress) return 'progress';
    if (activeTab === 'large' && largeFiles.length > 0) return 'results';
    if (activeTab === 'junk' && junkResults.length > 0) return 'results';
    return 'empty';
  }, [isScanning, scanProgress, activeTab, largeFiles, junkResults]);

  const renderScanButtons = () => {
    if (isScanning) {
      return (
        <motion.button
          onClick={handleScan}
          disabled
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white text-sm font-medium opacity-50 cursor-not-allowed"
        >
          <RefreshCw className="w-4 h-4 animate-spin" />
          扫描中...
        </motion.button>
      );
    }
    
    return (
      <motion.button
        onClick={handleScan}
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
        className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-purple-500/25"
      >
        <Search className="w-4 h-4" />
        开始扫描
      </motion.button>
    );
  };

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* 页面标题 */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-[var(--text-primary)]">文件分析</h1>
            <p className="text-[var(--text-secondary)] text-sm mt-1">
              分析磁盘文件，管理大文件和清理零碎文件
            </p>
          </div>

          <div className="flex items-center gap-3">
            <SegmentedControl
              options={[
                { value: 'large', label: '大文件管理', icon: <Database className="w-4 h-4" /> },
                { value: 'junk', label: '零碎文件', icon: <Trash2 className="w-4 h-4" /> },
              ]}
              value={activeTab}
              onChange={(value) => actions.setActiveTab(value as 'large' | 'junk')}
              color="linear-gradient(to right, #6366f1, #8b5cf6)"
            />

            {activeTab === 'large' && (
              <motion.button
                onClick={() => setShowFilterModal(true)}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-secondary)] text-sm hover:border-[var(--color-primary)]/50 hover:text-[var(--text-primary)] transition-colors"
              >
                <Settings className="w-4 h-4" />
                筛选
              </motion.button>
            )}

            {renderScanButtons()}
          </div>
        </div>

        {/* 错误提示 */}
        <AnimatePresence mode="wait">
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="flex items-center gap-3 p-4 rounded-lg bg-red-500/10 border border-red-500/20"
            >
              <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0" />
              <p className="text-sm text-red-400">{error}</p>
            </motion.div>
          )}
        </AnimatePresence>

        {/* 主内容区域 */}
        <AnimatePresence mode="wait">
          {pageState === 'progress' && scanProgress && (
            <ScanProgressBar
              progress={scanProgress}
              isPaused={isPaused}
              onPause={() => actions.pauseScan()}
              onResume={() => actions.resumeScan()}
              onCancel={() => actions.cancelScan()}
            />
          )}

          {pageState === 'empty' && activeTab === 'large' && <LargeFileEmptyState />}
          {pageState === 'empty' && activeTab === 'junk' && <JunkFileEmptyState />}

          {pageState === 'results' && (
            <motion.div
              key="results"
              variants={pageVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              transition={pageTransition}
              className="space-y-6"
            >
              {activeTab === 'large' ? (
                <>
                  {/* 磁盘选择和搜索 */}
                  <div className="panel p-6">
                    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <DiskSelector disabled={isScanning} />
                      
                      <div>
                        <label className="block text-sm font-medium text-[var(--text-secondary)] mb-2">
                          搜索过滤
                        </label>
                        <div className="relative">
                          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--text-tertiary)]" />
                          <input
                            type="text"
                            value={searchQuery}
                            onChange={(e) => actions.setSearchQuery(e.target.value)}
                            placeholder="搜索文件名、路径或类型..."
                            className="w-full pl-10 pr-10 py-3 rounded-xl bg-[var(--bg-tertiary)] border border-[var(--border-color)] text-[var(--text-primary)] placeholder-[var(--text-tertiary)] focus:outline-none focus:border-[var(--color-primary)] transition-colors"
                          />
                          {searchQuery && (
                            <button
                              onClick={() => actions.setSearchQuery('')}
                              className="absolute right-3 top-1/2 -translate-y-1/2 p-1 rounded hover:bg-[var(--bg-secondary)]"
                            >
                              <X className="w-4 h-4 text-[var(--text-tertiary)]" />
                            </button>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* 统计卡片 */}
                  {scanResult && largeFiles.length > 0 && (
                    <div className="grid grid-cols-3 gap-4">
                      <StatCard
                        icon={<FileText className="w-5 h-5 text-[var(--text-secondary)]" />}
                        value={scanResult.total_files.toLocaleString()}
                        label="大文件总数"
                        delay={0.1}
                      />
                      <StatCard
                        icon={<Database className="w-5 h-5 text-[var(--text-secondary)]" />}
                        value={formatBytes(scanResult.total_size)}
                        label="占用空间"
                        color="success"
                        delay={0.2}
                      />
                      <StatCard
                        icon={<Layers className="w-5 h-5 text-[var(--text-secondary)]" />}
                        value={filter.unit === 'GB' ? `${filter.minSize} GB` : `${filter.minSize} MB`}
                        label="筛选阈值"
                        delay={0.3}
                      />
                    </div>
                  )}

                  {/* 选择工具栏 */}
                  {largeFiles.length > 0 && (
                    <div className="flex items-center justify-between p-4 rounded-xl bg-[var(--bg-secondary)] border border-[var(--border-color)]/50">
                      <div className="flex items-center gap-3">
                        <button
                          onClick={() => {
                            if (selectedLargeFiles.size === filteredLargeFiles.length) {
                              actions.deselectAllLargeFiles();
                            } else {
                              actions.selectAllLargeFiles();
                            }
                          }}
                          className="flex items-center gap-2"
                        >
                          {selectedLargeFiles.size === filteredLargeFiles.length && filteredLargeFiles.length > 0 ? (
                            <CheckSquare className="w-5 h-5 text-[var(--color-primary)]" />
                          ) : selectedLargeFiles.size > 0 ? (
                            <div className="w-5 h-5 rounded border-2 border-[var(--color-primary)] bg-[var(--color-primary)]/20 flex items-center justify-center">
                              <div className="w-2 h-2 rounded-sm bg-[var(--color-primary)]" />
                            </div>
                          ) : (
                            <Square className="w-5 h-5 text-[var(--text-tertiary)]" />
                          )}
                          <span className="text-sm text-[var(--text-primary)]">
                            {selectedLargeFiles.size === filteredLargeFiles.length && filteredLargeFiles.length > 0 ? '取消全选' : '全选'}
                          </span>
                        </button>
                      </div>
                      
                      <div className="flex items-center gap-4">
                        <span className="text-sm text-[var(--text-secondary)]">
                          共 {filteredLargeFiles.length} 个文件
                          {searchQuery && ` (从 ${largeFiles.length} 个筛选)`}
                        </span>
                        {selectedLargeFiles.size > 0 && (
                          <>
                            <span className="text-sm text-[var(--text-secondary)]">
                              已选择 <span className="font-medium text-[var(--text-primary)]">{selectedLargeFiles.size}</span> 个
                            </span>
                            <span className="text-sm font-semibold text-[#10b981]">
                              {formatBytes(selectedLargeFileSize)}
                            </span>
                          </>
                        )}
                      </div>
                    </div>
                  )}

                  {/* 文件列表 */}
                  <LargeFileList
                    files={filteredLargeFiles}
                    selectedFiles={selectedLargeFiles}
                    onToggleFileSelection={actions.toggleLargeFileSelection}
                  />
                </>
              ) : (
                <>
                  {/* 零碎文件统计 */}
                  {junkResults.length > 0 && (
                    <div className="grid grid-cols-3 gap-4">
                      <StatCard
                        icon={<Layers className="w-5 h-5 text-[var(--text-secondary)]" />}
                        value={junkResults.reduce((sum, r) => sum + r.count, 0)}
                        label="发现项目"
                        delay={0.1}
                      />
                      <StatCard
                        icon={<Database className="w-5 h-5 text-[var(--text-secondary)]" />}
                        value={formatBytes(totalJunkSize)}
                        label="可清理空间"
                        color="success"
                        delay={0.2}
                      />
                      <StatCard
                        icon={<CheckSquare className="w-5 h-5 text-[var(--text-secondary)]" />}
                        value={selectedJunkFiles.size}
                        label="已选择项目"
                        color={selectedJunkFiles.size > 0 ? 'success' : 'primary'}
                        delay={0.3}
                      />
                    </div>
                  )}

                  {/* 选择工具栏 */}
                  {junkResults.length > 0 && (
                    <div className="flex items-center justify-between p-4 rounded-xl bg-[var(--bg-secondary)] border border-[var(--border-color)]/50">
                      <div className="flex items-center gap-4">
                        <span className="text-sm text-[var(--text-secondary)]">
                          共 {junkResults.reduce((sum, r) => sum + r.count, 0)} 项，总计 {formatBytes(totalJunkSize)}
                        </span>
                        {selectedJunkFiles.size > 0 && (
                          <span className="text-sm text-[var(--color-primary)]">
                            已选择 {selectedJunkFiles.size} 项，{formatBytes(selectedJunkSize)}
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-2">
                        <motion.button
                          onClick={() => actions.selectAllJunkFiles()}
                          whileHover={{ scale: 1.02 }}
                          whileTap={{ scale: 0.98 }}
                          className="text-xs text-[var(--text-secondary)] hover:text-[var(--color-primary)] transition-colors px-2 py-1 rounded hover:bg-[var(--bg-tertiary)]"
                        >
                          全选
                        </motion.button>
                        <span className="text-[var(--text-tertiary)]">|</span>
                        <motion.button
                          onClick={() => actions.deselectAllJunkFiles()}
                          whileHover={{ scale: 1.02 }}
                          whileTap={{ scale: 0.98 }}
                          className="text-xs text-[var(--text-secondary)] hover:text-[var(--color-primary)] transition-colors px-2 py-1 rounded hover:bg-[var(--bg-tertiary)]"
                        >
                          取消全选
                        </motion.button>
                      </div>
                    </div>
                  )}

                  {/* 零碎文件列表 */}
                  <div className="space-y-3">
                    {junkResults.map((result, index) => (
                      <motion.div
                        key={result.file_type}
                        initial={{ opacity: 0, y: 10 }}
                        animate={{ opacity: 1, y: 0 }}
                        transition={{ delay: index * 0.05 }}
                      >
                        <JunkCategoryPanel
                          result={result}
                          selectedIds={selectedJunkFiles}
                          onToggle={actions.toggleJunkFileSelection}
                        />
                      </motion.div>
                    ))}
                  </div>
                </>
              )}
            </motion.div>
          )}
        </AnimatePresence>

        {/* 底部操作栏 */}
        <AnimatePresence>
          {activeTab === 'large' && selectedLargeFiles.size > 0 && (
            <motion.div
              initial={{ y: 20, opacity: 0 }}
              animate={{ y: 0, opacity: 1 }}
              exit={{ y: 20, opacity: 0 }}
              className="fixed bottom-24 left-1/2 -translate-x-1/2 flex items-center gap-3 px-6 py-3 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-lg backdrop-blur-sm"
            >
              <span className="text-sm text-[var(--text-secondary)]">
                已选择 <span className="font-medium text-[var(--text-primary)]">{selectedLargeFiles.size}</span> 个文件
              </span>
              <span className="text-sm font-semibold text-[#10b981]">
                {formatBytes(selectedLargeFileSize)}
              </span>
              <div className="w-px h-5 bg-[var(--border-color)]" />
              <motion.button
                onClick={() => console.log('移动到外接硬盘')}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-primary)] text-sm hover:bg-[var(--color-primary)]/10 transition-colors"
              >
                <ExternalLink className="w-4 h-4" />
                移动至外接硬盘
              </motion.button>
              <motion.button
                onClick={handleDeleteSelected}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-red-500 text-white text-sm hover:bg-red-600 transition-colors"
              >
                <Trash2 className="w-4 h-4" />
                删除选中
              </motion.button>
            </motion.div>
          )}

          {activeTab === 'junk' && selectedJunkFiles.size > 0 && (
            <motion.div
              initial={{ y: 20, opacity: 0 }}
              animate={{ y: 0, opacity: 1 }}
              exit={{ y: 20, opacity: 0 }}
              className="fixed bottom-24 left-1/2 -translate-x-1/2 flex items-center gap-3 px-6 py-3 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-lg backdrop-blur-sm"
            >
              <span className="text-sm text-[var(--text-secondary)]">
                已选择 <span className="font-medium text-[var(--text-primary)]">{selectedJunkFiles.size}</span> 项
              </span>
              <span className="text-sm font-semibold text-[#10b981]">
                {formatBytes(selectedJunkSize)}
              </span>
              <div className="w-px h-5 bg-[var(--border-color)]" />
              <motion.button
                onClick={handleDeleteSelected}
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                className="flex items-center gap-1.5 px-4 py-2 rounded-lg bg-gradient-to-r from-red-500 to-red-600 text-white text-sm hover:shadow-lg hover:shadow-red-500/25 transition-all"
              >
                <Trash2 className="w-4 h-4" />
                一键清理
              </motion.button>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      <FilterSettingsModal
        isOpen={showFilterModal}
        onClose={() => setShowFilterModal(false)}
        filter={filter}
        onApply={(newFilter) => actions.setFilter(newFilter)}
      />

      <ConfirmDialog
        isOpen={showConfirmDialog}
        onClose={() => setShowConfirmDialog(false)}
        onConfirm={() => {
          if (confirmAction) {
            confirmAction.action();
          }
        }}
        title={confirmAction?.title || ''}
        message={confirmAction?.message || ''}
        confirmText="确认删除"
        danger
      />
    </div>
  );
}
