import { useEffect, useState, useMemo } from 'react';
import { motion, AnimatePresence, type Transition } from 'framer-motion';
import { RotateCcw, Zap, Search, AlertCircle, HardDrive, FolderOpen, Sparkles, Pause, Play, Trash2, X, FolderCog } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { useScanStore, useScanActions } from '../stores/scanStore';
import { useSystemStore } from '../stores/systemStore';
import { useUIStore } from '../stores';
import { ScanProgress, ScanResults, DeleteConfirmModal } from '../components/scan';
import { SegmentedControl } from '../components/common';
import type { ScanMode, CleanResult } from '../types';
import { formatBytes } from '../utils/format';

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

type DeleteStep = 'confirm' | 'deleting' | 'result';

export default function ScanPage() {
  const { scanMode, status, progress, result, cleanResult, error, selectedFiles, expandedCategories, scanId, selectedScanPath } = useScanStore();
  const actions = useScanActions();
  const { diskList, actions: systemActions } = useSystemStore();
  const setIsWorking = useUIStore((state) => state.actions.setIsWorking);
  const [showResults, setShowResults] = useState(false);
  
  const [deleteModalVisible, setDeleteModalVisible] = useState(false);
  const [deleteStep, setDeleteStep] = useState<DeleteStep>('confirm');
  const [deleteProgress, setDeleteProgress] = useState<{ current: number; total: number; percent: number } | undefined>();
  const [deleteResult, setDeleteResult] = useState<CleanResult | null>(null);

  const isScanning = status === 'scanning';
  const isPaused = status === 'paused';

  useEffect(() => {
    if (diskList.length === 0) {
      systemActions.fetchDiskList();
    }
    actions.setupListeners();
    
    return () => {
      if (status === 'scanning' || status === 'paused') {
        actions.cancelScan();
      }
      actions.cleanupListeners();
    };
  }, []);

  useEffect(() => {
    setIsWorking(isScanning || isPaused);
  }, [isScanning, isPaused, setIsWorking]);

  useEffect(() => {
    if (status === 'completed' && result) {
      const timer = requestAnimationFrame(() => {
        setShowResults(true);
      });
      return () => cancelAnimationFrame(timer);
    }
  }, [status, result]);

  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => {
        actions.clearError();
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  const canStart = status === 'idle' || status === 'completed' || status === 'cancelled' || status === 'failed';
  const isCompleted = status === 'completed';

  const handleStartScan = () => {
    setShowResults(false);
    actions.startScan();
  };

  const handlePauseScan = () => {
    actions.pauseScan();
  };

  const handleResumeScan = () => {
    actions.resumeScan();
  };

  const handleStopScan = () => {
    actions.cancelScan();
    setShowResults(false);
  };

  const handleReset = () => {
    actions.resetScan();
    setShowResults(false);
  };

  const handleDeleteClick = () => {
    setDeleteStep('confirm');
    setDeleteResult(null);
    setDeleteProgress(undefined);
    setDeleteModalVisible(true);
  };

  const handleDeleteConfirm = async () => {
    setDeleteStep('deleting');
    setDeleteProgress({ current: 0, total: selectedFiles.size, percent: 0 });
    
    const result = await actions.deleteSelectedFiles();
    
    setDeleteProgress({ 
      current: result?.cleaned_files ?? 0, 
      total: result?.total_files ?? selectedFiles.size, 
      percent: 100 
    });
    
    setDeleteResult(result);
    setDeleteStep('result');
  };

  const handleDeleteModalClose = () => {
    setDeleteModalVisible(false);
    setDeleteStep('confirm');
    setDeleteResult(null);
    setDeleteProgress(undefined);
  };

  const handleModeChange = (mode: ScanMode) => {
    if (canStart) {
      actions.setScanMode(mode);
    }
  };

  const getScanModeInfo = () => {
    if (scanMode === 'quick') {
      return {
        title: '快速扫描',
        description: '快速扫描主要文件类型目录，预计耗时少于30秒，适合日常快速清理',
      };
    }
    return {
      title: '深度扫描',
      description: '全面扫描磁盘所有文件，分析文件类型分布，适合深度清理',
    };
  };

  const scanInfo = getScanModeInfo();

  const pageState = useMemo(() => {
    if (showResults && result) return 'results';
    if (isScanning || isPaused || progress) return 'progress';
    return 'empty';
  }, [showResults, result, isScanning, isPaused, progress]);

  const selectedCount = selectedFiles.size;
  const selectedSize = useScanStore((state) => state.actions.getSelectedSize());

  const renderScanButtons = () => {
    if (isScanning) {
      return (
        <motion.button
          onClick={handlePauseScan}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-amber-500 text-white text-sm font-medium transition-all duration-200 hover:bg-amber-600"
        >
          <Pause className="w-4 h-4" />
          停止扫描
        </motion.button>
      );
    }
    
    if (isPaused) {
      return (
        <div className="flex items-center gap-2">
          <motion.button
            onClick={handleResumeScan}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-blue-500/25"
          >
            <Play className="w-4 h-4" />
            继续扫描
          </motion.button>
          <motion.button
            onClick={handleStopScan}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-500 text-white text-sm font-medium transition-all duration-200 hover:bg-red-600"
          >
            <X className="w-4 h-4" />
            取消扫描
          </motion.button>
        </div>
      );
    }
    
    if (isCompleted) {
      return (
        <motion.button
          onClick={handleStartScan}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-blue-500/25"
        >
          <RotateCcw className="w-4 h-4" />
          重新扫描
        </motion.button>
      );
    }
    
    return (
      <motion.button
        onClick={handleStartScan}
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
        className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-blue-500/25"
      >
        <Search className="w-4 h-4" />
        开始扫描
      </motion.button>
    );
  };

  const handleSelectFolder = async () => {
    const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
    if (isTauri) {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择要扫描的目录',
      });
      if (selected) {
        actions.setSelectedScanPath(selected as string);
      }
    }
  };

  const renderQuickScanEmptyState = () => (
    <motion.div
      key="empty-state"
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={pageTransition}
      className="panel p-8 min-h-[400px] flex flex-col items-center justify-center"
    >
      <div className="relative">
        <div className="w-24 h-24 rounded-2xl bg-gradient-to-br from-[#3b82f6]/20 to-[#10b981]/20 flex items-center justify-center">
          <Zap className="w-12 h-12 text-amber-500" />
        </div>
        <motion.div
          className="absolute -top-1 -right-1 w-8 h-8 rounded-full bg-gradient-to-r from-[#3b82f6] to-[#10b981] flex items-center justify-center"
          animate={{ scale: [1, 1.1, 1] }}
          transition={{ duration: 2, repeat: Infinity, ease: 'easeInOut' }}
        >
          <Sparkles className="w-4 h-4 text-white" />
        </motion.div>
      </div>
      
      <h3 className="text-xl font-semibold text-[var(--text-primary)] mt-6 mb-2">
        快速扫描
      </h3>
      <p className="text-[var(--text-secondary)] text-sm text-center max-w-md mb-6">
        快速扫描将自动检测常见的可清理目录，包括临时文件、浏览器缓存、日志文件等
      </p>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 w-full max-w-2xl mb-6">
        {[
          { name: '系统临时文件', icon: '📁' },
          { name: '浏览器缓存', icon: '🌐' },
          { name: '日志文件', icon: '📝' },
          { name: '应用缓存', icon: '💾' },
        ].map((item) => (
          <div key={item.name} className="flex items-center gap-2 p-3 rounded-lg bg-[var(--bg-tertiary)]">
            <span className="text-lg">{item.icon}</span>
            <span className="text-xs text-[var(--text-secondary)]">{item.name}</span>
          </div>
        ))}
      </div>

      <div className="flex items-center gap-2 text-xs text-[var(--text-tertiary)]">
        <FolderOpen className="w-4 h-4" />
        <span>预计扫描时间约 30 秒</span>
      </div>
    </motion.div>
  );

  const renderDeepScanEmptyState = () => (
    <motion.div
      key="deep-empty-state"
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={pageTransition}
      className="panel p-8 min-h-[400px]"
    >
      <div className="flex flex-col lg:flex-row gap-8 h-full">
        <div className="lg:w-1/3 flex flex-col">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-12 h-12 rounded-xl bg-gradient-to-br from-blue-500/20 to-purple-500/20 flex items-center justify-center">
              <HardDrive className="w-6 h-6 text-blue-500" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-[var(--text-primary)]">深度扫描</h3>
              <p className="text-xs text-[var(--text-tertiary)]">全面分析指定目录</p>
            </div>
          </div>

          <div className="space-y-4 flex-1">
            <div>
              <label className="block text-sm font-medium text-[var(--text-secondary)] mb-2">
                扫描位置
              </label>
              <div className="space-y-2">
                {diskList.slice(0, 4).map((disk) => (
                  <motion.button
                    key={disk.mount_point}
                    whileHover={{ scale: 1.01 }}
                    whileTap={{ scale: 0.99 }}
                    onClick={() => actions.setSelectedScanPath(disk.mount_point)}
                    className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-all duration-200 ${
                      selectedScanPath === disk.mount_point
                        ? 'border-[var(--color-primary)] bg-[var(--color-primary)]/5'
                        : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50'
                    }`}
                  >
                    <HardDrive className="w-5 h-5 text-[var(--color-primary)]" />
                    <div className="flex-1 text-left">
                      <p className="text-sm font-medium text-[var(--text-primary)]">{disk.mount_point}</p>
                      <p className="text-xs text-[var(--text-tertiary)]">
                        {formatBytes(disk.total_size)} · 可用 {formatBytes(disk.free_size)}
                      </p>
                    </div>
                    {selectedScanPath === disk.mount_point && (
                      <div className="w-2 h-2 rounded-full bg-[var(--color-primary)]" />
                    )}
                  </motion.button>
                ))}
              </div>
            </div>

            <div className="pt-2 border-t border-[var(--border-color)]">
              <p className="text-xs text-[var(--text-tertiary)] mb-2">或选择自定义目录</p>
              <motion.button
                whileHover={{ scale: 1.01 }}
                whileTap={{ scale: 0.99 }}
                onClick={handleSelectFolder}
                className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-all duration-200 ${
                  selectedScanPath && !diskList.some(d => d.mount_point === selectedScanPath)
                    ? 'border-[var(--color-primary)] bg-[var(--color-primary)]/5'
                    : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50 border-dashed'
                }`}
              >
                <FolderCog className="w-5 h-5 text-[var(--color-primary)]" />
                <span className={`text-sm flex-1 text-left truncate ${
                  selectedScanPath && !diskList.some(d => d.mount_point === selectedScanPath)
                    ? 'text-[var(--text-primary)]'
                    : 'text-[var(--text-secondary)]'
                }`}>
                  {selectedScanPath && !diskList.some(d => d.mount_point === selectedScanPath)
                    ? selectedScanPath
                    : '选择自定义目录...'}
                </span>
                {selectedScanPath && !diskList.some(d => d.mount_point === selectedScanPath) && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      actions.setSelectedScanPath(null);
                    }}
                    className="p-1 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
                  >
                    <X className="w-4 h-4" />
                  </button>
                )}
              </motion.button>
            </div>
          </div>
        </div>

        <div className="lg:flex-1 flex flex-col items-center justify-center bg-[var(--bg-tertiary)] rounded-xl p-8">
          <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-blue-500/20 to-purple-500/20 flex items-center justify-center mb-4">
            <Search className="w-10 h-10 text-blue-500" />
          </div>
          <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
            选择扫描目录
          </h3>
          <p className="text-[var(--text-secondary)] text-sm text-center max-w-sm mb-6">
            深度扫描将对选定目录进行全面分析，扫描所有文件并按类型分类统计
          </p>

          <div className="flex items-center gap-4 text-xs text-[var(--text-tertiary)]">
            <div className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-amber-500" />
              <span>扫描时间取决于目录大小</span>
            </div>
            <div className="flex items-center gap-1">
              <span className="w-1.5 h-1.5 rounded-full bg-blue-500" />
              <span>支持暂停和取消</span>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );

  const renderEmptyState = () => {
    if (scanMode === 'full') {
      return renderDeepScanEmptyState();
    }
    return renderQuickScanEmptyState();
  };

  const renderProgressState = () => (
    <motion.div
      key="progress-state"
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={pageTransition}
      className="panel p-6 space-y-6"
    >
      <ScanProgress progress={progress} status={status} />
      
      {isPaused && progress && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="flex items-center justify-center gap-4 pt-4 border-t border-[var(--border-color)]"
        >
          <p className="text-sm text-[var(--text-secondary)]">
            已扫描 <span className="font-medium text-[var(--text-primary)]">{progress.scanned_files.toLocaleString()}</span> 个文件，
            发现 <span className="font-medium text-[#10b981]">{formatBytes(progress.scanned_size)}</span> 可清理空间
          </p>
        </motion.div>
      )}
    </motion.div>
  );

  const renderResultsState = () => (
    <motion.div
      key="results-state"
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={pageTransition}
    >
      <ScanResults 
        result={result} 
        scanId={scanId}
        selectedFiles={selectedFiles}
        expandedCategories={expandedCategories}
        onReset={handleReset} 
        onDelete={handleDeleteClick}
        onToggleFileSelection={actions.toggleFileSelection}
        onToggleCategorySelection={actions.toggleCategorySelection}
        onSelectAll={actions.selectAllFiles}
        onDeselectAll={actions.deselectAllFiles}
        onToggleCategoryExpand={actions.toggleCategoryExpand}
        onExpandAll={actions.expandAllCategories}
        onCollapseAll={actions.collapseAllCategories}
      />
    </motion.div>
  );

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-[var(--text-primary)]">{scanInfo.title}</h1>
            <p className="text-[var(--text-secondary)] text-sm mt-1">
              {scanInfo.description}
            </p>
          </div>
          <div className="flex items-center gap-3">
            <SegmentedControl
              options={[
                { value: 'quick', label: '快速扫描', icon: <Zap className="w-4 h-4" />, disabled: !canStart },
                { value: 'full', label: '深度扫描', icon: <HardDrive className="w-4 h-4" />, disabled: !canStart },
              ]}
              value={scanMode}
              onChange={(mode) => handleModeChange(mode)}
            />
            {renderScanButtons()}
          </div>
        </div>

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

        <AnimatePresence mode="wait">
          {pageState === 'empty' && renderEmptyState()}
          {pageState === 'progress' && renderProgressState()}
          {pageState === 'results' && renderResultsState()}
        </AnimatePresence>

        <AnimatePresence>
          {cleanResult && (
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -20 }}
              className="panel p-6"
            >
              <div className="flex items-start gap-4">
                <div className="flex-shrink-0">
                  <div className="w-12 h-12 rounded-full bg-[#10b981] flex items-center justify-center">
                    <Trash2 className="w-6 h-6 text-white" />
                  </div>
                </div>
                <div className="flex-1">
                  <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
                    清理完成
                  </h3>
                  <p className="text-sm text-[var(--text-secondary)]">
                    成功删除 <span className="font-medium text-[var(--text-primary)]">{cleanResult.cleaned_files}</span> 个文件，
                    释放空间 <span className="font-medium text-[#10b981]">{formatBytes(cleanResult.cleaned_size)}</span>
                    {cleanResult.failed_files > 0 && (
                      <span className="text-red-400">，{cleanResult.failed_files} 个文件删除失败</span>
                    )}
                  </p>
                </div>
                <motion.button
                  onClick={() => actions.resetScan()}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="px-4 py-2 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] text-sm font-medium hover:border-[var(--color-primary)]/50 transition-all"
                >
                  完成
                </motion.button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      <DeleteConfirmModal
        visible={deleteModalVisible}
        step={deleteStep}
        selectedCount={selectedCount}
        selectedSize={selectedSize}
        progress={deleteProgress}
        result={deleteResult}
        onConfirm={handleDeleteConfirm}
        onClose={handleDeleteModalClose}
      />
    </div>
  );
}
