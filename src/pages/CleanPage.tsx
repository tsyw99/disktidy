import { useEffect, useMemo, useState } from 'react';
import { motion, AnimatePresence, type Transition } from 'framer-motion';
import { VirtualFileList } from '../components/common/VirtualFileList';
import { AppCacheScanProgress, DeleteConfirmModal, PathConfigModal } from '../components/clean';
import { formatBytes, formatDate } from '../utils/format';
import { ENABLED_APPS, APP_CONFIGS, CATEGORY_CONFIGS } from '../utils/constants';
import { useAppCacheStore, useAppCacheActions } from '../stores/appCacheStore';
import { useUIStore } from '../stores';
import { cleanService } from '../services/cleanService';
import { openFileLocation } from '../utils/shell';
import {
  Trash2,
  Search,
  CheckSquare,
  Square,
  ChevronDown,
  ChevronUp,
  Image,
  Video,
  FileText,
  Package,
  Clock,
  Smile,
  MessageCircle,
  Users,
  HardDrive,
  Calendar,
  Filter,
  AlertCircle,
  X,
  Pause,
  Play,
  RotateCcw,
  Settings,
  FolderOpen,
  ExternalLink,
} from 'lucide-react';
import type { AppType, CleanCategory, AppCacheFile, CleanResult, CleanProgress } from '../types';

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

interface AppConfig {
  id: AppType;
  name: string;
  icon: React.ReactNode;
  color: string;
  bgColor: string;
}

interface CategoryConfig {
  id: CleanCategory;
  name: string;
  icon: React.ReactNode;
  description: string;
}

const apps: AppConfig[] = APP_CONFIGS.map(app => ({
  id: app.id as AppType,
  name: app.name,
  icon: app.id === 'wechat' || app.id === 'qq' 
    ? <MessageCircle className="w-5 h-5" /> 
    : <Users className="w-5 h-5" />,
  color: app.color,
  bgColor: app.bgColor,
}));

const categories: CategoryConfig[] = CATEGORY_CONFIGS.map(cat => ({
  id: cat.id as CleanCategory,
  name: cat.name,
  icon: cat.id === 'chat_images' || cat.id === 'thumb_cache'
    ? <Image className="w-4 h-4" />
    : cat.id === 'video_files'
    ? <Video className="w-4 h-4" />
    : cat.id === 'emoji_cache'
    ? <Smile className="w-4 h-4" />
    : cat.id === 'cache_data'
    ? <Clock className="w-4 h-4" />
    : cat.id === 'install_packages'
    ? <Package className="w-4 h-4" />
    : <FileText className="w-4 h-4" />,
  description: cat.description,
}));

const VIRTUAL_LIST_THRESHOLD = 50;

export default function CleanPage() {
  const {
    status,
    progress,
    result,
    error,
    selectedApps,
    selectedCategories,
    selectedFiles,
    expandedCategories,
    cleanResult,
    configuredApps,
    configLoaded,
  } = useAppCacheStore();
  const actions = useAppCacheActions();
  const setIsWorking = useUIStore((state) => state.actions.setIsWorking);

  console.log('[CleanPage] State:', { status, progress: progress?.percent, resultFiles: result?.files?.length, error, configuredApps: Array.from(configuredApps) });

  const [deleteModalVisible, setDeleteModalVisible] = useState(false);
  const [deleteStep, setDeleteStep] = useState<DeleteStep>('confirm');
  const [deleteProgress, setDeleteProgress] = useState<{ current: number; total: number; percent: number } | undefined>();
  const [deleteResult, setDeleteResult] = useState<CleanResult | null>(null);
  const [pathConfigModalVisible, setPathConfigModalVisible] = useState(false);

  const isScanning = status === 'scanning';
  const isPaused = status === 'paused';

  useEffect(() => {
    setIsWorking(isScanning || isPaused);
  }, [isScanning, isPaused, setIsWorking]);

  useEffect(() => {
    console.log('[CleanPage] Setting up listeners and loading config');
    actions.setupListeners();
    actions.loadConfig();
    
    const setupCleanListeners = async () => {
      const unlistenProgress = await cleanService.onProgress((progress: CleanProgress) => {
        setDeleteProgress({
          current: progress.cleaned_files,
          total: progress.total_files,
          percent: progress.percent,
        });
      });

      const unlistenComplete = await cleanService.onComplete((result: CleanResult) => {
        setDeleteResult(result);
        setDeleteStep('result');
      });

      return () => {
        unlistenProgress();
        unlistenComplete();
      };
    };

    const cleanup = setupCleanListeners();
    return () => {
      cleanup.then(fn => fn());
    };
  }, []);

  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => {
        actions.clearError();
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  const isCompleted = status === 'completed';
  const canStart = status === 'idle' || status === 'completed' || status === 'cancelled' || status === 'error';

  const hasConfiguredApps = configuredApps.size > 0;
  const selectedAppsConfigured = selectedApps.every(app => configuredApps.has(app));

  const handleStartScan = () => {
    if (!selectedAppsConfigured) {
      setPathConfigModalVisible(true);
      return;
    }
    actions.startScan();
  };

  const handlePauseScan = () => {
    actions.pauseScan();
  };

  const handleResumeScan = () => {
    actions.resumeScan();
  };

  const handleCancelScan = () => {
    actions.cancelScan();
  };

  const handleReset = () => {
    actions.resetScan();
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

    await actions.deleteSelectedFiles();
  };

  const handleDeleteModalClose = () => {
    setDeleteModalVisible(false);
    setDeleteStep('confirm');
    setDeleteResult(null);
    setDeleteProgress(undefined);
  };

  const handleSelectAll = () => {
    const allSelected = filteredFiles.every(f => selectedFiles.has(f.path));
    if (allSelected) {
      actions.deselectAllFiles();
    } else {
      actions.selectAllFiles();
    }
  };

  const handleSelectAllInCategory = (category: CleanCategory, _selected: boolean) => {
    actions.toggleCategorySelection(category);
  };

  const toggleCategoryExpand = (categoryId: CleanCategory) => {
    actions.toggleCategoryExpand(categoryId);
  };

  const handleConfigChange = () => {
    actions.loadConfig();
  };

  const filteredFiles = useMemo(() => {
    if (!result) return [];
    return result.files.filter(f =>
      selectedApps.includes(f.app) &&
      (selectedCategories.length === 0 || selectedCategories.includes(f.category))
    );
  }, [result, selectedApps, selectedCategories]);

  const groupedFiles = useMemo(() => {
    const groups: Record<CleanCategory, AppCacheFile[]> = {} as Record<CleanCategory, AppCacheFile[]>;
    filteredFiles.forEach(file => {
      if (!groups[file.category]) {
        groups[file.category] = [];
      }
      groups[file.category].push(file);
    });
    return groups;
  }, [filteredFiles]);

  const selectedCount = selectedFiles.size;
  const selectedSize = useMemo(() => {
    if (!result) return 0;
    let size = 0;
    for (const file of result.files) {
      if (selectedFiles.has(file.path)) {
        size += file.size;
      }
    }
    return size;
  }, [result, selectedFiles]);

  const totalSize = filteredFiles.reduce((sum, f) => sum + f.size, 0);

  const virtualListFiles = useMemo(() => {
    return filteredFiles.map(file => {
      const app = apps.find(a => a.id === file.app);
      return {
        id: file.id,
        path: file.path,
        name: file.name,
        size: file.size,
        app: file.app,
        appName: app?.name || '',
        appColor: app?.color || '',
        appBgColor: app?.bgColor || '',
        chatObject: file.chatObject,
        modifiedAt: file.modifiedAt,
        selected: selectedFiles.has(file.path),
      };
    });
  }, [filteredFiles, selectedFiles]);

  const useVirtualList = filteredFiles.length > VIRTUAL_LIST_THRESHOLD;

  const pageState = useMemo(() => {
    if (isCompleted && result) return 'results';
    if (isScanning || isPaused || progress) return 'progress';
    return 'empty';
  }, [isCompleted, result, isScanning, isPaused, progress]);

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
          暂停扫描
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
            onClick={handleCancelScan}
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
          onClick={handleReset}
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-blue-500/25"
        >
          <RotateCcw className="w-4 h-4" />
          重新选择
        </motion.button>
      );
    }

    return (
      <motion.button
        onClick={handleStartScan}
        disabled={selectedApps.length === 0}
        whileHover={{ scale: 1.02 }}
        whileTap={{ scale: 0.98 }}
        className="flex items-center justify-center gap-2 px-5 py-2.5 rounded-lg bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-blue-500/25 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <Search className="w-4 h-4" />
        开始扫描
      </motion.button>
    );
  };

  const renderEmptyState = () => (
    <motion.div
      key="placeholder"
      variants={pageVariants}
      initial="initial"
      animate="animate"
      exit="exit"
      transition={pageTransition}
      className="panel p-8 h-full min-h-[350px] flex flex-col items-center justify-center"
    >
      <div className="text-center">
        {!hasConfiguredApps ? (
          <>
            <div className="w-20 h-20 rounded-full bg-amber-500/10 flex items-center justify-center mx-auto mb-4">
              <FolderOpen className="w-10 h-10 text-amber-500" />
            </div>
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">请先配置应用路径</h3>
            <p className="text-[var(--text-secondary)] text-sm max-w-md">
              为了扫描应用缓存文件，请点击右上角的"路径设置"按钮配置应用的文件存储路径
            </p>
          </>
        ) : (
          <>
            <div className="w-20 h-20 rounded-full bg-[var(--color-primary)]/10 flex items-center justify-center mx-auto mb-4">
              <Trash2 className="w-10 h-10 text-[var(--color-primary)] opacity-50" />
            </div>
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">选择应用和清理内容</h3>
            <p className="text-[var(--text-secondary)] text-sm max-w-md">
              在左侧选择要清理的应用程序和文件类型，然后点击"开始扫描"按钮进行扫描
            </p>
          </>
        )}
      </div>
    </motion.div>
  );

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
      <AppCacheScanProgress progress={progress} status={status} />

      {isPaused && progress && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          className="flex items-center justify-center gap-4 pt-4 border-t border-[var(--border-color)]"
        >
          <p className="text-sm text-[var(--text-secondary)]">
            已扫描 <span className="font-medium text-[var(--text-primary)]">{progress.scannedFiles.toLocaleString()}</span> 个文件，
            发现 <span className="font-medium text-[#10b981]">{formatBytes(progress.scannedSize)}</span> 可清理空间
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
      className="panel p-6 h-full flex flex-col"
    >
      <div className="flex items-center justify-between flex-shrink-0">
        <div className="flex items-center gap-4">
          <h3 className="text-lg font-semibold text-[var(--text-primary)]">扫描结果</h3>
          <span className="text-sm text-[var(--text-secondary)]">
            共 {filteredFiles.length} 个文件，总计 {formatBytes(totalSize)}
          </span>
        </div>
      </div>

      {filteredFiles.length === 0 ? (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <div className="w-16 h-16 rounded-full bg-green-500/10 flex items-center justify-center mx-auto mb-4">
              <Trash2 className="w-8 h-8 text-green-500" />
            </div>
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">未发现可清理文件</h3>
            <p className="text-sm text-[var(--text-secondary)]">所选应用未发现可清理的缓存文件</p>
          </div>
        </div>
      ) : (
        <>
          <div className="flex items-center justify-between p-3 rounded-lg bg-[var(--bg-tertiary)] flex-shrink-0 mt-2.5 mb-5">
            <div className="flex items-center gap-3">
              <button
                onClick={handleSelectAll}
                className="flex items-center gap-2 text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
              >
                {filteredFiles.every(f => selectedFiles.has(f.path)) ? (
                  <CheckSquare className="w-4 h-4 text-[var(--color-primary)]" />
                ) : (
                  <Square className="w-4 h-4" />
                )}
                全选
              </button>
              <span className="text-sm text-[var(--text-tertiary)]">
                已选择 {selectedCount} 个文件
              </span>
            </div>
            <div className="flex items-center gap-3">
              <span className="text-sm text-[var(--text-secondary)]">
                已选: <span className="font-medium text-[var(--color-primary)]">{formatBytes(selectedSize)}</span>
              </span>
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={handleDeleteClick}
                disabled={selectedCount === 0}
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gradient-to-r from-[#ef4444] to-[#f97316] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-red-500/25 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Trash2 className="w-4 h-4" />
                删除选中 ({selectedCount})
              </motion.button>
            </div>
          </div>

          <div className="space-y-3 flex-1 overflow-y-auto pr-2 min-h-0">
            {useVirtualList ? (
              <div className="border border-[var(--border-color)] rounded-lg overflow-hidden">
                <div className="p-3 bg-[var(--bg-secondary)] border-b border-[var(--border-color)]">
                  <span className="text-sm text-[var(--text-secondary)]">
                    共 {filteredFiles.length} 个文件
                  </span>
                </div>
                <VirtualFileList
                  files={virtualListFiles}
                  onFileToggle={(fileId) => {
                    const file = result?.files.find(f => f.id === fileId);
                    if (file) {
                      actions.toggleFileSelection(file.path);
                    }
                  }}
                  height={400}
                  itemHeight={56}
                />
              </div>
            ) : (
              categories.map(category => {
                const categoryFiles = groupedFiles[category.id] || [];
                if (categoryFiles.length === 0) return null;
                const isExpanded = expandedCategories.has(category.id);
                const categorySize = categoryFiles.reduce((sum, f) => sum + f.size, 0);
                const selectedInCategory = categoryFiles.filter(f => selectedFiles.has(f.path)).length;

                return (
                  <div key={category.id} className="border border-[var(--border-color)] rounded-lg overflow-hidden">
                    <div className="w-full flex items-center justify-between p-3 bg-[var(--bg-secondary)]">
                      <button
                        onClick={() => toggleCategoryExpand(category.id)}
                        className="flex items-center gap-3 flex-1"
                      >
                        <div className="w-8 h-8 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]">
                          {category.icon}
                        </div>
                        <div className="text-left">
                          <span className="text-sm font-medium text-[var(--text-primary)]">{category.name}</span>
                          <span className="text-xs text-[var(--text-tertiary)] ml-2">
                            {categoryFiles.length} 个文件 · {formatBytes(categorySize)}
                          </span>
                        </div>
                      </button>
                      <div className="flex items-center gap-2">
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleSelectAllInCategory(category.id, !categoryFiles.every(f => selectedFiles.has(f.path)));
                          }}
                          className="flex items-center gap-1.5 px-2 py-1 rounded text-xs transition-colors hover:bg-[var(--bg-tertiary)]"
                          title={categoryFiles.every(f => selectedFiles.has(f.path)) ? '取消全选' : '全选此类'}
                        >
                          {categoryFiles.every(f => selectedFiles.has(f.path)) ? (
                            <CheckSquare className="w-4 h-4 text-[var(--color-primary)]" />
                          ) : (
                            <Square className="w-4 h-4 text-[var(--text-tertiary)]" />
                          )}
                          <span className={categoryFiles.every(f => selectedFiles.has(f.path)) ? 'text-[var(--color-primary)]' : 'text-[var(--text-tertiary)]'}>
                            {categoryFiles.every(f => selectedFiles.has(f.path)) ? '取消' : '全选'}
                          </span>
                        </button>
                        {selectedInCategory > 0 && !categoryFiles.every(f => selectedFiles.has(f.path)) && (
                          <span className="text-xs px-2 py-0.5 rounded-full bg-[var(--color-primary)]/10 text-[var(--color-primary)]">
                            已选 {selectedInCategory}
                          </span>
                        )}
                        <button
                          onClick={() => toggleCategoryExpand(category.id)}
                          className="p-1"
                        >
                          {isExpanded ? <ChevronUp className="w-4 h-4 text-[var(--text-tertiary)]" /> : <ChevronDown className="w-4 h-4 text-[var(--text-tertiary)]" />}
                        </button>
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
                          <div className="p-2 space-y-1">
                            {categoryFiles.map(file => {
                              const app = apps.find(a => a.id === file.app);
                              return (
                                <motion.div
                                  key={file.id}
                                  initial={{ opacity: 0, x: -10 }}
                                  animate={{ opacity: 1, x: 0 }}
                                  className={`flex items-center gap-3 p-2 rounded-lg transition-colors ${
                                    selectedFiles.has(file.path)
                                      ? 'bg-[var(--color-primary)]/5 border border-[var(--color-primary)]/20'
                                      : 'hover:bg-[var(--bg-tertiary)]'
                                  }`}
                                >
                                  <button
                                    onClick={() => actions.toggleFileSelection(file.path)}
                                    className="flex-shrink-0"
                                  >
                                    {selectedFiles.has(file.path) ? (
                                      <CheckSquare className="w-4 h-4 text-[var(--color-primary)]" />
                                    ) : (
                                      <Square className="w-4 h-4 text-[var(--text-tertiary)]" />
                                    )}
                                  </button>
                                  <div className="flex-1 min-w-0">
                                    <div className="flex items-center gap-2">
                                      <span className="text-sm text-[var(--text-primary)] truncate">{file.name}</span>
                                      <span className="text-xs px-1.5 py-0.5 rounded" style={{ backgroundColor: app?.bgColor, color: app?.color }}>
                                        {app?.name}
                                      </span>
                                    </div>
                                    <div className="flex items-center gap-3 mt-1 text-xs text-[var(--text-tertiary)]">
                                      <span className="flex items-center gap-1">
                                        <Users className="w-3 h-3" />
                                        {file.chatObject}
                                      </span>
                                      <span className="flex items-center gap-1">
                                        <Calendar className="w-3 h-3" />
                                        {formatDate(file.modifiedAt)}
                                      </span>
                                      <span className="flex items-center gap-1">
                                        <HardDrive className="w-3 h-3" />
                                        {formatBytes(file.size)}
                                      </span>
                                    </div>
                                  </div>
                                  <button
                                    onClick={() => openFileLocation(file.path)}
                                    className="flex-shrink-0 p-1.5 rounded-lg hover:bg-[var(--bg-tertiary)] transition-colors group"
                                    title="打开所在文件夹"
                                  >
                                    <ExternalLink className="w-4 h-4 text-[var(--text-tertiary)] group-hover:text-[var(--color-primary)]" />
                                  </button>
                                </motion.div>
                              );
                            })}
                          </div>
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </div>
                );
              })
            )}
          </div>

          <div className="flex items-center gap-2 p-3 rounded-lg bg-yellow-500/10 border border-yellow-500/20 flex-shrink-0">
            <AlertCircle className="w-4 h-4 text-yellow-500 flex-shrink-0" />
            <p className="text-xs text-yellow-600 dark:text-yellow-400">
              清理过程中将保留所有聊天记录文本内容，确保不影响应用的正常使用及日常聊天功能
            </p>
          </div>
        </>
      )}
    </motion.div>
  );

  const renderAppConfigStatus = (app: AppConfig) => {
    const isConfigured = configuredApps.has(app.id);
    return (
      <div className="absolute top-1 right-1">
        {isConfigured ? (
          <div className="w-2 h-2 rounded-full bg-green-500" title="已配置路径" />
        ) : (
          <div className="w-2 h-2 rounded-full bg-gray-300" title="未配置路径" />
        )}
      </div>
    );
  };

  return (
    <div className="h-full overflow-auto p-6 pb-20">
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-[var(--text-primary)]">专项清理</h1>
            <p className="text-[var(--text-secondary)] text-sm mt-1">
              针对特定应用的深度清理，安全释放磁盘空间
            </p>
          </div>
          <div className="flex items-center gap-3">
            <motion.button
              onClick={() => setPathConfigModalVisible(true)}
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              className="flex items-center gap-2 px-4 py-2 rounded-lg border border-[var(--border-color)] text-sm font-medium text-[var(--text-secondary)] hover:border-[var(--color-primary)]/50 transition-all"
            >
              <Settings className="w-4 h-4" />
              路径设置
            </motion.button>
            {renderScanButtons()}
          </div>
        </div>

        <AnimatePresence mode="wait">
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="flex items-center justify-between gap-2 p-4 rounded-lg bg-red-500/10 border border-red-500/20"
            >
              <div className="flex items-center gap-2">
                <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0" />
                <p className="text-sm text-red-500">{error}</p>
              </div>
              <button onClick={() => actions.clearError()} className="text-red-500 hover:text-red-600">
                <X className="w-4 h-4" />
              </button>
            </motion.div>
          )}
        </AnimatePresence>

        {!hasConfiguredApps && configLoaded && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-center gap-3 p-4 rounded-lg bg-amber-500/10 border border-amber-500/20"
          >
            <AlertCircle className="w-5 h-5 text-amber-500 flex-shrink-0" />
            <div className="flex-1">
              <p className="text-sm text-amber-600 dark:text-amber-400">
                请先配置应用的文件存储路径，然后才能进行扫描
              </p>
            </div>
            <button
              onClick={() => setPathConfigModalVisible(true)}
              className="px-3 py-1.5 rounded-lg bg-amber-500 text-white text-sm font-medium hover:bg-amber-600 transition-colors"
            >
              立即配置
            </button>
          </motion.div>
        )}

        <div className="flex gap-6">
          <div className={`w-full lg:w-1/3 flex-shrink-0 ${pageState === 'progress' ? 'hidden lg:block' : ''}`}>
            <div className="panel p-6 space-y-6 h-full flex flex-col">
              <div>
                <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-3 flex items-center gap-2">
                  <MessageCircle className="w-4 h-4 text-[var(--color-primary)]" />
                  选择应用
                </h3>
                <div className="grid grid-cols-2 gap-2">
                  {apps.map(app => {
                    const isEnabled = ENABLED_APPS.includes(app.id);
                    return (
                      <motion.button
                        key={app.id}
                        whileHover={isEnabled ? { scale: 1.02 } : {}}
                        whileTap={isEnabled ? { scale: 0.98 } : {}}
                        onClick={() => isEnabled && actions.toggleApp(app.id)}
                        disabled={!canStart || !isEnabled}
                        className={`relative flex items-center gap-2 p-3 rounded-lg border transition-all duration-200 ${
                          selectedApps.includes(app.id)
                            ? 'border-[var(--color-primary)] bg-[var(--color-primary)]/5'
                            : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50'
                        } ${(!canStart || !isEnabled) ? 'opacity-50 cursor-not-allowed' : ''}`}
                      >
                        <div className={`w-8 h-8 rounded-lg ${app.bgColor} flex items-center justify-center`} style={{ color: app.color }}>
                          {app.icon}
                        </div>
                        <div className="flex-1 text-left">
                          <span className={`text-sm font-medium ${selectedApps.includes(app.id) ? 'text-[var(--text-primary)]' : isEnabled ? 'text-[var(--text-secondary)]' : 'text-[var(--text-tertiary)]'}`}>
                            {app.name}
                          </span>
                          {!isEnabled && (
                            <p className="text-xs text-amber-500 mt-0.5">开发中</p>
                          )}
                        </div>
                        {isEnabled && renderAppConfigStatus(app)}
                      </motion.button>
                    );
                  })}
                </div>
              </div>

              <div>
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-sm font-semibold text-[var(--text-primary)] flex items-center gap-2">
                    <Filter className="w-4 h-4 text-[var(--color-primary)]" />
                    清理内容
                  </h3>
                  <button
                    onClick={() => {
                      const allSelected = categories.every(c => selectedCategories.includes(c.id));
                      if (allSelected) {
                        actions.setSelectedCategories([]);
                      } else {
                        actions.setSelectedCategories(categories.map(c => c.id));
                      }
                    }}
                    disabled={!canStart}
                    className="text-xs text-[var(--color-primary)] hover:text-[var(--color-primary)]/80 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {categories.every(c => selectedCategories.includes(c.id)) ? '取消全选' : '全选'}
                  </button>
                </div>
                <div className="space-y-2">
                  {categories.map(category => (
                    <motion.button
                      key={category.id}
                      whileHover={{ scale: 1.01 }}
                      whileTap={{ scale: 0.99 }}
                      onClick={() => actions.toggleCategory(category.id)}
                      disabled={!canStart}
                      className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-all duration-200 ${
                        selectedCategories.includes(category.id)
                          ? 'border-[var(--color-primary)] bg-[var(--color-primary)]/5'
                          : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50'
                      } ${!canStart ? 'opacity-50 cursor-not-allowed' : ''}`}
                    >
                      <div className={`w-8 h-8 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]`}>
                        {category.icon}
                      </div>
                      <div className="flex-1 text-left">
                        <span className={`text-sm font-medium ${selectedCategories.includes(category.id) ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}`}>
                          {category.name}
                        </span>
                        <p className="text-xs text-[var(--text-tertiary)] mt-0.5">{category.description}</p>
                      </div>
                    </motion.button>
                  ))}
                </div>
              </div>
            </div>
          </div>

          <div className="flex-1 min-w-0">
            <AnimatePresence mode="wait">
              {pageState === 'empty' && renderEmptyState()}
              {pageState === 'progress' && renderProgressState()}
              {pageState === 'results' && renderResultsState()}
            </AnimatePresence>
          </div>
        </div>

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

      <PathConfigModal
        visible={pathConfigModalVisible}
        onClose={() => setPathConfigModalVisible(false)}
        onConfigChange={handleConfigChange}
      />
    </div>
  );
}
