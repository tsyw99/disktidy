import { useState, useEffect, useRef } from 'react';
import { motion } from 'framer-motion';
import { Package, Search, RefreshCw, CheckCircle, AlertCircle, Trash2, CheckSquare, Square, Filter, Loader2 } from 'lucide-react';
import { softwareResidueService } from '../../services';
import type { ResidueScanResult, ResidueScanProgress, ResidueScanOptions } from '../../types/softwareResidue';
import { RESIDUE_TYPES } from './constants';
import { ResidueCategoryPanel } from './ResidueComponents';
import { ConfirmDialog } from './ConfirmDialog';

export function SoftwareResidueTab() {
  const [isScanning, setIsScanning] = useState(false);
  const [scanComplete, setScanComplete] = useState(false);
  const [residueResults, setResidueResults] = useState<ResidueScanResult[]>([]);
  const [scanProgress, setScanProgress] = useState<ResidueScanProgress | null>(null);
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const [scanOptions, setScanOptions] = useState<ResidueScanOptions>({
    include_leftover_folders: true,
    include_registry_keys: true,
    include_cache_files: true,
    include_config_files: true,
    scan_all_drives: true,
    custom_scan_paths: [],
  });
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const progressIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const handleScanResiduals = async () => {
    setIsScanning(true);
    setScanComplete(false);
    setResidueResults([]);
    setSelectedItems(new Set());
    setScanProgress(null);

    try {
      const results = await softwareResidueService.startScan(scanOptions);
      setResidueResults(results);
      setScanComplete(true);
    } catch (error) {
      console.error('扫描失败:', error);
    } finally {
      setIsScanning(false);
      setScanProgress(null);
    }
  };

  useEffect(() => {
    if (isScanning) {
      progressIntervalRef.current = setInterval(async () => {
        try {
          const progress = await softwareResidueService.getProgress();
          if (progress) {
            setScanProgress(progress);
          }
        } catch (error) {
          console.error('获取进度失败:', error);
        }
      }, 200);
    } else {
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
        progressIntervalRef.current = null;
      }
    }

    return () => {
      if (progressIntervalRef.current) {
        clearInterval(progressIntervalRef.current);
      }
    };
  }, [isScanning]);

  const handleSelectAllResiduals = () => {
    const allIds = residueResults.flatMap(r => r.items.map(i => i.id));
    if (selectedItems.size === allIds.length) {
      setSelectedItems(new Set());
    } else {
      setSelectedItems(new Set(allIds));
    }
  };

  const handleResidualToggle = (id: string) => {
    setSelectedItems(prev => {
      const newSet = new Set(prev);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      return newSet;
    });
  };

  const handleDeleteResiduals = async () => {
    const idsToDelete = Array.from(selectedItems);
    if (idsToDelete.length === 0) return;

    try {
      const result = await softwareResidueService.deleteItems(idsToDelete, true);

      if (result.deleted_count > 0) {
        setResidueResults(prev =>
          prev.map(r => ({
            ...r,
            items: r.items.filter(i => !idsToDelete.includes(i.id)),
          })).filter(r => r.items.length > 0)
        );
        setSelectedItems(new Set());
      }

      if (result.failed_count > 0) {
        console.warn('部分项目删除失败:', result.failed_items);
      }
    } catch (error) {
      console.error('删除失败:', error);
    }

    setShowDeleteConfirm(false);
  };

  const toggleCategoryExpand = (categoryId: string) => {
    setExpandedCategories(prev => {
      const newSet = new Set(prev);
      if (newSet.has(categoryId)) {
        newSet.delete(categoryId);
      } else {
        newSet.add(categoryId);
      }
      return newSet;
    });
  };

  const totalResidualCount = residueResults.reduce((sum, r) => sum + r.items.length, 0);
  const totalResidualSize = residueResults.reduce((sum, r) => sum + r.total_size, 0);
  const selectedResiduals = residueResults.flatMap(r => r.items.filter(i => selectedItems.has(i.id)));
  const selectedResidualSize = selectedResiduals.reduce((sum, r) => sum + r.size, 0);

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const renderResidualContent = () => {
    if (!scanComplete && !isScanning) {
      return (
        <div className="panel p-8 h-full min-h-[400px] flex flex-col items-center justify-center">
          <div className="text-center">
            <div className="w-20 h-20 rounded-full bg-[var(--color-primary)]/10 flex items-center justify-center mx-auto mb-4">
              <Package className="w-10 h-10 text-[var(--color-primary)] opacity-50" />
            </div>
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">扫描软件残留</h3>
            <p className="text-[var(--text-secondary)] text-sm max-w-md">
              点击"开始扫描"按钮，系统将自动检测已卸载软件的残留文件和配置信息
            </p>
          </div>
        </div>
      );
    }

    if (isScanning && scanProgress) {
      return (
        <div className="panel p-6 h-full min-h-[400px] flex flex-col">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-[var(--text-primary)]">正在扫描</h3>
            <span className="text-sm text-[var(--text-secondary)]">
              已用时 {scanProgress.elapsed_time} 秒
            </span>
          </div>

          <div className="flex-1 flex flex-col justify-center">
            <div className="mb-8">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm text-[var(--text-secondary)]">{scanProgress.current_phase}</span>
                <span className="text-sm font-medium text-[var(--color-primary)]">
                  {Math.round(scanProgress.percent)}%
                </span>
              </div>
              <div className="h-3 bg-[var(--bg-tertiary)] rounded-full overflow-hidden">
                <motion.div
                  className="h-full bg-gradient-to-r from-[#3b82f6] to-[#10b981] rounded-full relative"
                  initial={{ width: 0 }}
                  animate={{ width: `${scanProgress.percent}%` }}
                  transition={{ duration: 0.3, ease: 'easeOut' }}
                >
                  <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-[shimmer_1.5s_infinite]" />
                </motion.div>
              </div>
            </div>

            <div className="space-y-4">
              <div className="flex items-center gap-3 p-4 rounded-lg bg-[var(--bg-tertiary)]">
                <div className="w-10 h-10 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]">
                  <Loader2 className="w-5 h-5 animate-spin" />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium text-[var(--text-primary)] mb-1">当前扫描路径</p>
                  <p className="text-xs text-[var(--text-tertiary)] truncate font-mono">
                    {scanProgress.current_path || '正在初始化...'}
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 rounded-lg bg-[var(--bg-tertiary)]">
                  <p className="text-xs text-[var(--text-tertiary)] mb-1">已扫描项目</p>
                  <p className="text-2xl font-bold text-[var(--text-primary)]">
                    {scanProgress.scanned_count}
                  </p>
                </div>
                <div className="p-4 rounded-lg bg-[var(--bg-tertiary)]">
                  <p className="text-xs text-[var(--text-tertiary)] mb-1">发现残留</p>
                  <p className="text-2xl font-bold text-[var(--color-primary)]">
                    {scanProgress.found_count}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      );
    }

    return (
      <div className="panel p-6 h-full flex flex-col">
        <div className="flex items-center justify-between flex-shrink-0 mb-4">
          <div className="flex items-center gap-4">
            <h3 className="text-lg font-semibold text-[var(--text-primary)]">扫描结果</h3>
            <span className="text-sm text-[var(--text-secondary)]">
              共 {totalResidualCount} 项，总计 {formatSize(totalResidualSize)}
            </span>
          </div>
        </div>

        {residueResults.length === 0 || totalResidualCount === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center">
            <div className="w-16 h-16 rounded-full bg-emerald-500/10 flex items-center justify-center mb-4">
              <CheckCircle className="w-8 h-8 text-emerald-500" />
            </div>
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">扫描完成</h3>
            <p className="text-[var(--text-secondary)] text-sm">未发现软件残留文件</p>
          </div>
        ) : (
          <>
            <div className="flex items-center justify-between p-3 rounded-lg bg-[var(--bg-tertiary)] flex-shrink-0 mb-4">
              <div className="flex items-center gap-3">
                <button
                  onClick={handleSelectAllResiduals}
                  className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
                >
                  {selectedItems.size === totalResidualCount ? (
                    <CheckSquare className="w-4 h-4 text-[var(--color-primary)]" />
                  ) : (
                    <Square className="w-4 h-4" />
                  )}
                  全选
                </button>
                <span className="text-sm text-[var(--text-tertiary)]">
                  已选择 {selectedItems.size} 项
                </span>
              </div>
              <div className="flex items-center gap-3">
                <span className="text-sm text-[var(--text-secondary)]">
                  已选: <span className="font-medium text-[var(--color-primary)]">{formatSize(selectedResidualSize)}</span>
                </span>
                <motion.button
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  onClick={() => setShowDeleteConfirm(true)}
                  disabled={selectedItems.size === 0}
                  className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#ef4444] to-[#f97316] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-red-500/25 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Trash2 className="w-4 h-4" />
                  删除选中
                </motion.button>
              </div>
            </div>

            <div className="space-y-3 flex-1 overflow-y-auto pr-2 min-h-0">
              {residueResults.map((result) => (
                <ResidueCategoryPanel
                  key={result.residue_type}
                  result={result}
                  selectedIds={selectedItems}
                  onToggle={handleResidualToggle}
                  isExpanded={expandedCategories.has(result.residue_type)}
                  onToggleExpand={() => toggleCategoryExpand(result.residue_type)}
                />
              ))}
            </div>

            <div className="flex items-center gap-2 p-3 rounded-lg bg-yellow-500/10 border border-yellow-500/20 flex-shrink-0 mt-4">
              <AlertCircle className="w-4 h-4 text-yellow-500 flex-shrink-0" />
              <p className="text-xs text-yellow-600 dark:text-yellow-400">
                删除前请确认这些文件确实不再需要，部分注册表项可能影响系统稳定性
              </p>
            </div>
          </>
        )}
      </div>
    );
  };

  return (
    <div className="flex gap-6 items-stretch">
      <div className="w-full lg:w-1/3 flex-shrink-0">
        <div className="panel p-6 space-y-6 h-full flex flex-col">
          <div>
            <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-3 flex items-center gap-2">
              <Filter className="w-4 h-4 text-[var(--color-primary)]" />
              扫描范围
            </h3>
            <div className="space-y-2">
              {RESIDUE_TYPES.map(type => (
                <label
                  key={type.id}
                  className="flex items-center gap-3 p-3 rounded-lg bg-[var(--bg-tertiary)] cursor-pointer hover:bg-[var(--bg-tertiary)]/80 transition-colors"
                >
                  <input
                    type="checkbox"
                    checked={
                      type.id === 'leftover_folder' ? scanOptions.include_leftover_folders :
                      type.id === 'registry_key' ? scanOptions.include_registry_keys :
                      type.id === 'cache_file' ? scanOptions.include_cache_files :
                      scanOptions.include_config_files
                    }
                    onChange={(e) => {
                      setScanOptions(prev => ({
                        ...prev,
                        [type.id === 'leftover_folder' ? 'include_leftover_folders' :
                         type.id === 'registry_key' ? 'include_registry_keys' :
                         type.id === 'cache_file' ? 'include_cache_files' :
                         'include_config_files']: e.target.checked,
                      }));
                    }}
                    className="w-4 h-4 rounded border-[var(--border-color)] text-[var(--color-primary)] focus:ring-[var(--color-primary)]"
                  />
                  <div className="w-8 h-8 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]">
                    {type.icon}
                  </div>
                  <div className="flex-1">
                    <span className="text-sm text-[var(--text-primary)]">{type.name}</span>
                    <p className="text-xs text-[var(--text-tertiary)]">{type.description}</p>
                  </div>
                </label>
              ))}
            </div>
          </div>

          <div className="pt-4 mt-auto">
            {!scanComplete ? (
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={handleScanResiduals}
                disabled={isScanning}
                className="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-lg bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-blue-500/25 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isScanning ? (
                  <>
                    <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    扫描中...
                  </>
                ) : (
                  <>
                    <Search className="w-4 h-4" />
                    开始扫描
                  </>
                )}
              </motion.button>
            ) : (
              <motion.button
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                onClick={() => {
                  setScanComplete(false);
                  setResidueResults([]);
                  setSelectedItems(new Set());
                }}
                className="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-secondary)] text-sm font-medium transition-all duration-200 hover:border-[var(--color-primary)]/50 hover:text-[var(--text-primary)]"
              >
                <RefreshCw className="w-4 h-4" />
                重新扫描
              </motion.button>
            )}
          </div>
        </div>
      </div>

      <div className="flex-1 min-w-0">
        {renderResidualContent()}
      </div>

      <ConfirmDialog
        isOpen={showDeleteConfirm}
        onClose={() => setShowDeleteConfirm(false)}
        onConfirm={handleDeleteResiduals}
        title="确认删除残留文件"
        message={`即将删除 ${selectedItems.size} 项，共 ${formatSize(selectedResidualSize)}。此操作将移动到回收站，可恢复。`}
        confirmText="确认删除"
        danger
      />
    </div>
  );
}
