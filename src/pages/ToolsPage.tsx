import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { open } from '@tauri-apps/plugin-dialog';
import { FileClassificationChart, CategoryDetailPanel } from '../components/tools';
import { fileClassifierService } from '../services';
import { useSystemStore, useSystemActions } from '../stores/systemStore';
import { useUIStore } from '../stores';
import type { 
  FileClassificationResult, 
  FileTypeStats,
  FileClassificationOptions,
  ResidueType,
  ResidueItem,
  ResidueScanResult,
  ResidueScanProgress,
  ResidueScanOptions,
  DeleteResidueResult,
} from '../types';
import { RESIDUE_TYPE_NAMES } from '../types/softwareResidue';
import {
  Package,
  Trash2,
  FolderTree,
  Cpu,
  Search,
  RefreshCw,
  CheckCircle,
  AlertCircle,
  HardDrive,
  FileText,
  CheckSquare,
  Square,
  Filter,
  Info,
  Loader2,
  FolderOpen,
  X,
  Database,
  Layers,
  Zap,
  FileKey,
  Settings2,
  AlertTriangle,
  ExternalLink,
  Shield,
  ShieldCheck,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { openFileLocation } from '../utils/shell';
import type { DriverInfo } from '../types/system';

type ToolModule = 'residual' | 'fileclassify' | 'driver';

const residualTypes: { id: ResidueType; name: string; icon: React.ReactNode; description: string }[] = [
  { id: 'leftover_folder', name: '遗留目录', icon: <FolderTree className="w-4 h-4" />, description: '已卸载软件的残留目录' },
  { id: 'registry_key', name: '注册表项', icon: <FileKey className="w-4 h-4" />, description: '已卸载软件的注册表残留项' },
  { id: 'cache_file', name: '缓存文件', icon: <Database className="w-4 h-4" />, description: '已卸载软件的缓存文件' },
  { id: 'config_file', name: '配置文件', icon: <Settings2 className="w-4 h-4" />, description: '已卸载软件的配置文件' },
];

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

function formatDate(timestamp: number): string {
  if (!timestamp) return '-';
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
}

function getResidueTypeIcon(type: ResidueType): React.ReactNode {
  const iconClass = 'w-5 h-5';
  switch (type) {
    case 'leftover_folder':
      return <FolderTree className={`${iconClass} text-blue-400`} />;
    case 'registry_key':
      return <FileKey className={`${iconClass} text-purple-400`} />;
    case 'cache_file':
      return <Database className={`${iconClass} text-orange-400`} />;
    case 'config_file':
      return <Settings2 className={`${iconClass} text-green-400`} />;
    default:
      return <FileText className={`${iconClass} text-gray-400`} />;
  }
}

const pageVariants = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 },
};

const defaultClassificationOptions: FileClassificationOptions = {
  max_depth: 5,
  include_hidden: false,
  include_system: false,
  exclude_paths: [
    'Windows',
    'Program Files',
    'Program Files (x86)',
    'ProgramData',
    '$Recycle.Bin',
    'System Volume Information',
  ],
  max_files: 100000,
  top_n_categories: 15,
};

export default function ToolsPage() {
  const [activeModule, setActiveModule] = useState<ToolModule>('fileclassify');
  const [isScanning, setIsScanning] = useState(false);
  const [scanComplete, setScanComplete] = useState(false);
  const [residueResults, setResidueResults] = useState<ResidueScanResult[]>([]);
  const [scanProgress, setScanProgress] = useState<ResidueScanProgress | null>(null);
  const [selectedDisk, setSelectedDisk] = useState<string>('all');
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set());
  const [drivers, setDrivers] = useState<DriverInfo[]>([]);
  const [isLoadingDrivers, setIsLoadingDrivers] = useState(false);
  const [driverToDelete, setDriverToDelete] = useState<DriverInfo | null>(null);
  const [showDriverDeleteConfirm, setShowDriverDeleteConfirm] = useState(false);
  const [isDeletingDriver, setIsDeletingDriver] = useState(false);
  const [driverSearchQuery, setDriverSearchQuery] = useState('');
  const [driverTypeFilter, setDriverTypeFilter] = useState<string>('all');
  const [showTypeDropdown, setShowTypeDropdown] = useState(false);
  const progressIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const [scanOptions, setScanOptions] = useState<ResidueScanOptions>({
    include_leftover_folders: true,
    include_registry_keys: true,
    include_cache_files: true,
    include_config_files: true,
    scan_all_drives: true,
    custom_scan_paths: [],
  });
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const [classificationResult, setClassificationResult] = useState<FileClassificationResult | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<FileTypeStats | null>(null);
  const [isClassifying, setIsClassifying] = useState(false);
  const [classifyError, setClassifyError] = useState<string | null>(null);
  
  const { diskList } = useSystemStore();
  const systemActions = useSystemActions();
  const setIsWorking = useUIStore((state) => state.actions.setIsWorking);
  const isWorking = isScanning || isClassifying || isLoadingDrivers || isDeletingDriver;

  useEffect(() => {
    setIsWorking(isWorking);
  }, [isWorking, setIsWorking]);

  useEffect(() => {
    if (diskList.length === 0) {
      systemActions.fetchDiskList();
    }
  }, [diskList.length, systemActions]);

  const handleScanResiduals = async () => {
    setIsScanning(true);
    setScanComplete(false);
    setResidueResults([]);
    setSelectedItems(new Set());
    setScanProgress(null);

    try {
      const results = await invoke<ResidueScanResult[]>('residue_scan_start', {
        options: scanOptions,
      });
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
          const progress = await invoke<ResidueScanProgress | null>('residue_scan_progress');
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
      const result = await invoke<DeleteResidueResult>('residue_delete_items', {
        itemIds: idsToDelete,
        moveToRecycleBin: true,
      });

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

  const handleClassifyFiles = useCallback(async () => {
    setIsClassifying(true);
    setClassifyError(null);
    setClassificationResult(null);
    setSelectedCategory(null);

    try {
      const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
      
      if (!isTauri) {
        await new Promise(resolve => setTimeout(resolve, 1500));
        const mockResult: FileClassificationResult = {
          scan_id: 'mock-scan-id',
          path: selectedDisk === 'all' ? 'C:\\' : selectedDisk,
          total_files: 125847,
          total_size: 256789456789,
          total_folders: 8456,
          categories: [
            { category: '视频', display_name: '视频', count: 1256, total_size: 85678945678, percentage: 33.36, extensions: {} },
            { category: '图片', display_name: '图片', count: 45678, total_size: 45678945678, percentage: 17.79, extensions: {} },
            { category: '音频', display_name: '音频', count: 8934, total_size: 34567894567, percentage: 13.46, extensions: {} },
            { category: '文档', display_name: '文档', count: 23456, total_size: 23456789012, percentage: 9.13, extensions: {} },
            { category: '压缩包', display_name: '压缩包', count: 3456, total_size: 19876543210, percentage: 7.74, extensions: {} },
            { category: '可执行文件', display_name: '可执行文件', count: 5678, total_size: 15678945678, percentage: 6.11, extensions: {} },
            { category: '磁盘镜像', display_name: '磁盘镜像', count: 234, total_size: 12345678901, percentage: 4.81, extensions: {} },
            { category: '数据库', display_name: '数据库', count: 567, total_size: 9876543210, percentage: 3.85, extensions: {} },
            { category: '配置文件', display_name: '配置文件', count: 12345, total_size: 5678901234, percentage: 2.21, extensions: {} },
            { category: '文本文件', display_name: '文本文件', count: 8765, total_size: 3456789012, percentage: 1.35, extensions: {} },
            { category: '系统文件', display_name: '系统文件', count: 456, total_size: 2345678901, percentage: 0.91, extensions: {} },
            { category: '其他', display_name: '其他', count: 34522, total_size: 4567890123, percentage: 1.78, extensions: {} },
          ],
          duration_ms: 1234,
          largest_files: [],
        };
        setClassificationResult(mockResult);
        return;
      }

      let path = selectedDisk;
      if (selectedDisk === 'all') {
        path = 'C:\\';
      }

      const result = await fileClassifierService.startClassifyFiles(path, defaultClassificationOptions);
      if (result.cancelled) {
        setClassifyError('扫描已取消');
      } else {
        setClassificationResult(result);
      }
    } catch (err) {
      console.error('文件分类失败:', err);
      setClassifyError(err instanceof Error ? err.message : '文件分类失败，请重试');
    } finally {
      setIsClassifying(false);
    }
  }, [selectedDisk]);

  const handleCancelClassify = useCallback(async () => {
    try {
      await fileClassifierService.cancelClassifyFiles();
    } catch (err) {
      console.error('取消扫描失败:', err);
    }
  }, []);

  const handleCategoryClick = useCallback((categoryName: string) => {
    if (classificationResult) {
      const cat = classificationResult.categories.find(c => c.display_name === categoryName);
      setSelectedCategory(cat || null);
    }
  }, [classificationResult]);

  const loadDrivers = async () => {
    setIsLoadingDrivers(true);
    try {
      const result = await invoke<DriverInfo[]>('driver_get_list');
      setDrivers(result);
    } catch (error) {
      console.error('加载驱动列表失败:', error);
    } finally {
      setIsLoadingDrivers(false);
    }
  };

  const handleDeleteDriver = async () => {
    if (!driverToDelete) return;
    
    setIsDeletingDriver(true);
    try {
      await invoke('driver_delete', { infName: driverToDelete.inf_name });
      setDrivers(prev => prev.filter(d => d.id !== driverToDelete.id));
      setShowDriverDeleteConfirm(false);
      setDriverToDelete(null);
    } catch (error) {
      console.error('删除驱动失败:', error);
      alert(`删除驱动失败: ${error}`);
    } finally {
      setIsDeletingDriver(false);
    }
  };

  const confirmDeleteDriver = (driver: DriverInfo) => {
    setDriverToDelete(driver);
    setShowDriverDeleteConfirm(true);
  };

  useEffect(() => {
    if (activeModule === 'driver' && drivers.length === 0 && !isLoadingDrivers) {
      loadDrivers();
    }
  }, [activeModule]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (showTypeDropdown && !target.closest('.driver-type-dropdown')) {
        setShowTypeDropdown(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [showTypeDropdown]);

  const filteredDrivers = useMemo(() => {
    return drivers.filter(driver => {
      const matchesSearch = driverSearchQuery === '' || 
        driver.name.toLowerCase().includes(driverSearchQuery.toLowerCase()) ||
        driver.provider.toLowerCase().includes(driverSearchQuery.toLowerCase()) ||
        driver.device_name.toLowerCase().includes(driverSearchQuery.toLowerCase());
      
      const matchesType = driverTypeFilter === 'all' || driver.driver_type === driverTypeFilter;
      
      return matchesSearch && matchesType;
    });
  }, [drivers, driverSearchQuery, driverTypeFilter]);

  const driverTypes = useMemo(() => {
    const types = new Set(drivers.map(d => d.driver_type));
    return Array.from(types).sort();
  }, [drivers]);

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'Running':
        return { bg: 'bg-emerald-500/10', text: 'text-emerald-500', label: '运行中' };
      case 'Stopped':
        return { bg: 'bg-amber-500/10', text: 'text-amber-500', label: '已停止' };
      case 'Error':
        return { bg: 'bg-red-500/10', text: 'text-red-500', label: '异常' };
      default:
        return { bg: 'bg-gray-500/10', text: 'text-gray-500', label: '未知' };
    }
  };

  const totalResidualCount = residueResults.reduce((sum, r) => sum + r.items.length, 0);
  const totalResidualSize = residueResults.reduce((sum, r) => sum + r.total_size, 0);
  const selectedResiduals = residueResults.flatMap(r => r.items.filter(i => selectedItems.has(i.id)));
  const selectedResidualSize = selectedResiduals.reduce((sum, r) => sum + r.size, 0);

  const modules = [
    { id: 'residual' as ToolModule, name: '软件残留清理', icon: <Package className="w-5 h-5" />, description: '扫描已卸载软件的残留文件和配置' },
    { id: 'fileclassify' as ToolModule, name: '文件分类大盘', icon: <FolderTree className="w-5 h-5" />, description: '按类型分类统计磁盘文件' },
    { id: 'driver' as ToolModule, name: '驱动管理', icon: <Cpu className="w-5 h-5" />, description: '查看和管理系统驱动程序' },
  ];

  const globalIsWorking = useUIStore((state) => state.isWorking);

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
    <div className="h-full overflow-auto p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-[var(--text-primary)]">其他功能</h1>
            <p className="text-[var(--text-secondary)] text-sm mt-1">
              高级系统维护工具，助您深度优化电脑
            </p>
          </div>
        </div>

        <div className="flex gap-4 border-b border-[var(--border-color)] pb-4 items-center">
          {modules.map(module => (
            <motion.button
              key={module.id}
              whileHover={globalIsWorking ? {} : { scale: 1.02 }}
              whileTap={globalIsWorking ? {} : { scale: 0.98 }}
              onClick={() => !globalIsWorking && setActiveModule(module.id)}
              disabled={globalIsWorking}
              className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all duration-200 ${
                activeModule === module.id
                  ? 'bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white'
                  : globalIsWorking
                  ? 'bg-[var(--bg-secondary)] text-[var(--text-tertiary)] border border-[var(--border-color)] cursor-not-allowed opacity-60'
                  : 'bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] border border-[var(--border-color)]'
              }`}
            >
              {module.icon}
              <span className="text-sm font-medium">{module.name}</span>
            </motion.button>
          ))}
          {globalIsWorking && (
            <motion.div
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              className="flex items-center gap-2 text-sm text-[var(--color-primary)] ml-auto"
            >
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>功能进行中，请稍候...</span>
            </motion.div>
          )}
        </div>

        <AnimatePresence mode="wait">
          {activeModule === 'residual' && (
            <motion.div
              key="residual"
              variants={pageVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="space-y-6"
            >
              <div className="flex gap-6 items-stretch">
                <div className="w-full lg:w-1/3 flex-shrink-0">
                  <div className="panel p-6 space-y-6 h-full flex flex-col">
                    <div>
                      <h3 className="text-sm font-semibold text-[var(--text-primary)] mb-3 flex items-center gap-2">
                        <Filter className="w-4 h-4 text-[var(--color-primary)]" />
                        扫描范围
                      </h3>
                      <div className="space-y-2">
                        {residualTypes.map(type => (
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
              </div>
            </motion.div>
          )}

          {activeModule === 'fileclassify' && (
            <motion.div
              key="fileclassify"
              variants={pageVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="space-y-6"
            >
              <div className="flex gap-6 items-stretch">
                <div className="w-full lg:w-1/4 flex-shrink-0">
                  <div className="panel p-6 space-y-4 h-full flex flex-col">
                    <h3 className="text-sm font-semibold text-[var(--text-primary)] flex items-center gap-2">
                      <HardDrive className="w-4 h-4 text-[var(--color-primary)]" />
                      扫描位置
                    </h3>
                    <div className="space-y-2">
                      {diskList.slice(0, 4).map(disk => (
                        <motion.button
                          key={disk.mount_point}
                          whileHover={{ scale: 1.02 }}
                          whileTap={{ scale: 0.98 }}
                          onClick={() => setSelectedDisk(disk.mount_point)}
                          className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-all duration-200 ${
                            selectedDisk === disk.mount_point
                              ? 'border-[var(--color-primary)] bg-[var(--color-primary)]/5'
                              : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50'
                          }`}
                        >
                          <HardDrive className="w-5 h-5 text-[var(--color-primary)]" />
                          <div className="flex-1 text-left">
                            <span className={`text-sm font-medium ${selectedDisk === disk.mount_point ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}`}>
                              {disk.mount_point}
                            </span>
                            <p className="text-xs text-[var(--text-tertiary)]">
                              {formatSize(disk.free_size)} 可用
                            </p>
                          </div>
                        </motion.button>
                      ))}
                      <motion.button
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                        onClick={() => setSelectedDisk('all')}
                        className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-all duration-200 ${
                          selectedDisk === 'all'
                            ? 'border-[var(--color-primary)] bg-[var(--color-primary)]/5'
                            : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50'
                        }`}
                      >
                        <Database className="w-5 h-5 text-[var(--color-primary)]" />
                        <span className={`text-sm font-medium ${selectedDisk === 'all' ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}`}>
                          全磁盘
                        </span>
                      </motion.button>
                    </div>

                    <div className="pt-2 border-t border-[var(--border-color)]">
                      <p className="text-xs text-[var(--text-tertiary)] mb-2">或选择指定目录</p>
                      <motion.button
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                        onClick={async () => {
                          const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;
                          if (isTauri) {
                            const selected = await open({
                              directory: true,
                              multiple: false,
                              title: '选择要扫描的目录',
                            });
                            if (selected) {
                              setSelectedDisk(selected as string);
                            }
                          } else {
                            const path = prompt('请输入目录路径:', 'D:\\Projects');
                            if (path) {
                              setSelectedDisk(path);
                            }
                          }
                        }}
                        className={`w-full flex items-center gap-3 p-3 rounded-lg border transition-all duration-200 ${
                          !['all', ...diskList.map(d => d.mount_point)].includes(selectedDisk)
                            ? 'border-[var(--color-primary)] bg-[var(--color-primary)]/5'
                            : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50 border-dashed'
                        }`}
                      >
                        <FolderOpen className="w-5 h-5 text-[var(--color-primary)]" />
                        <span className={`text-sm font-medium truncate ${!['all', ...diskList.map(d => d.mount_point)].includes(selectedDisk) ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}`}>
                          {!['all', ...diskList.map(d => d.mount_point)].includes(selectedDisk) ? selectedDisk : '选择目录...'}
                        </span>
                        {!['all', ...diskList.map(d => d.mount_point)].includes(selectedDisk) && (
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              setSelectedDisk('all');
                            }}
                            className="ml-auto p-1 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-tertiary)] hover:text-[var(--text-primary)]"
                          >
                            <X className="w-4 h-4" />
                          </button>
                        )}
                      </motion.button>
                    </div>
                  </div>
                </div>

                <div className="flex-1 min-w-0 space-y-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <h3 className="text-lg font-semibold text-[var(--text-primary)]">文件分类统计</h3>
                      <span className="text-xs px-2 py-1 rounded-full bg-[var(--color-primary)]/10 text-[var(--color-primary)]">
                        {selectedDisk === 'all' ? '全磁盘' : selectedDisk}
                      </span>
                    </div>
                    <motion.button
                      whileHover={{ scale: 1.02 }}
                      whileTap={{ scale: 0.98 }}
                      onClick={handleClassifyFiles}
                      disabled={isClassifying}
                      className="flex items-center justify-center gap-2 px-5 py-2.5 rounded-lg bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white text-sm font-medium transition-all duration-200 hover:shadow-lg hover:shadow-blue-500/25 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {isClassifying ? (
                        <>
                          <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                          分析中...
                        </>
                      ) : (
                        <>
                          <Zap className="w-4 h-4" />
                          开始分析
                        </>
                      )}
                    </motion.button>
                  </div>
                  {classifyError && (
                    <motion.div
                      initial={{ opacity: 0, y: -10 }}
                      animate={{ opacity: 1, y: 0 }}
                      className="flex items-center gap-3 p-4 rounded-lg bg-red-500/10 border border-red-500/20"
                    >
                      <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0" />
                      <p className="text-sm text-red-400">{classifyError}</p>
                    </motion.div>
                  )}

                  {isClassifying && (
                    <motion.div
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      className="panel p-8 min-h-[400px] flex flex-col items-center justify-center"
                    >
                      <div className="relative">
                        <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-[#3b82f6]/20 to-[#10b981]/20 flex items-center justify-center">
                          <FolderTree className="w-10 h-10 text-[var(--color-primary)]" />
                        </div>
                        <motion.div
                          className="absolute -top-1 -right-1 w-8 h-8 rounded-full bg-gradient-to-r from-[#3b82f6] to-[#10b981] flex items-center justify-center"
                          animate={{ rotate: 360 }}
                          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
                        >
                          <Loader2 className="w-4 h-4 text-white" />
                        </motion.div>
                      </div>
                      <h3 className="text-xl font-semibold text-[var(--text-primary)] mt-6 mb-2">
                        正在分析文件分类
                      </h3>
                      <p className="text-[var(--text-secondary)] text-sm text-center max-w-md mb-6">
                        正在扫描目录并统计文件类型分布，请稍候...
                      </p>
                      <div className="w-64 h-1.5 bg-[var(--bg-tertiary)] rounded-full overflow-hidden mb-6">
                        <motion.div
                          className="h-full bg-gradient-to-r from-[#3b82f6] to-[#10b981] rounded-full"
                          initial={{ x: '-100%' }}
                          animate={{ x: '100%' }}
                          transition={{ duration: 1.5, repeat: Infinity, ease: 'easeInOut' }}
                        />
                      </div>
                      <motion.button
                        whileHover={{ scale: 1.02 }}
                        whileTap={{ scale: 0.98 }}
                        onClick={handleCancelClassify}
                        className="flex items-center gap-2 px-5 py-2.5 rounded-lg bg-[var(--bg-tertiary)] border border-red-500/30 text-red-400 text-sm font-medium transition-all duration-200 hover:bg-red-500/10 hover:border-red-500/50"
                      >
                        <X className="w-4 h-4" />
                        取消扫描
                      </motion.button>
                    </motion.div>
                  )}

                  {!isClassifying && !classificationResult && !classifyError && (
                    <div className="panel p-8 min-h-[400px] flex flex-col items-center justify-center">
                      <div className="w-20 h-20 rounded-full bg-[var(--color-primary)]/10 flex items-center justify-center mb-4">
                        <FolderTree className="w-10 h-10 text-[var(--color-primary)] opacity-50" />
                      </div>
                      <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">文件分类大盘</h3>
                      <p className="text-[var(--text-secondary)] text-sm max-w-md text-center">
                        选择磁盘或目录后点击"开始分析"，系统将统计文件类型分布并以图表展示
                      </p>
                    </div>
                  )}

                  {!isClassifying && classificationResult && (
                    <>
                      <div className="grid grid-cols-4 gap-4">
                        <motion.div
                          initial={{ opacity: 0, y: 10 }}
                          animate={{ opacity: 1, y: 0 }}
                          transition={{ delay: 0.1 }}
                          className="panel p-4"
                        >
                          <div className="flex items-center gap-2 mb-2">
                            <FileText className="w-4 h-4 text-[var(--color-primary)]" />
                            <span className="text-xs text-[var(--text-tertiary)]">文件总数</span>
                          </div>
                          <p className="text-2xl font-bold text-[var(--text-primary)]">
                            {classificationResult.total_files.toLocaleString()}
                          </p>
                        </motion.div>

                        <motion.div
                          initial={{ opacity: 0, y: 10 }}
                          animate={{ opacity: 1, y: 0 }}
                          transition={{ delay: 0.2 }}
                          className="panel p-4"
                        >
                          <div className="flex items-center gap-2 mb-2">
                            <HardDrive className="w-4 h-4 text-[#10b981]" />
                            <span className="text-xs text-[var(--text-tertiary)]">总大小</span>
                          </div>
                          <p className="text-2xl font-bold text-[#10b981]">
                            {formatSize(classificationResult.total_size)}
                          </p>
                        </motion.div>

                        <motion.div
                          initial={{ opacity: 0, y: 10 }}
                          animate={{ opacity: 1, y: 0 }}
                          transition={{ delay: 0.3 }}
                          className="panel p-4"
                        >
                          <div className="flex items-center gap-2 mb-2">
                            <FolderTree className="w-4 h-4 text-[#f59e0b]" />
                            <span className="text-xs text-[var(--text-tertiary)]">文件夹数</span>
                          </div>
                          <p className="text-2xl font-bold text-[#f59e0b]">
                            {classificationResult.total_folders.toLocaleString()}
                          </p>
                        </motion.div>

                        <motion.div
                          initial={{ opacity: 0, y: 10 }}
                          animate={{ opacity: 1, y: 0 }}
                          transition={{ delay: 0.4 }}
                          className="panel p-4"
                        >
                          <div className="flex items-center gap-2 mb-2">
                            <Layers className="w-4 h-4 text-[#8b5cf6]" />
                            <span className="text-xs text-[var(--text-tertiary)]">文件类型</span>
                          </div>
                          <p className="text-2xl font-bold text-[#8b5cf6]">
                            {classificationResult.categories.length}
                          </p>
                        </motion.div>
                      </div>

                      <FileClassificationChart
                        categories={classificationResult.categories}
                        totalSize={classificationResult.total_size}
                        onCategoryClick={handleCategoryClick}
                      />

                      <CategoryDetailPanel
                        category={selectedCategory}
                        largestFiles={classificationResult.largest_files}
                        onClose={() => setSelectedCategory(null)}
                      />
                    </>
                  )}
                </div>
              </div>
            </motion.div>
          )}

          {activeModule === 'driver' && (
            <motion.div
              key="driver"
              variants={pageVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="space-y-6"
            >
              <div className="panel p-6">
                <div className="flex items-center justify-between mb-6">
                  <div className="flex items-center gap-4">
                    <h3 className="text-lg font-semibold text-[var(--text-primary)]">驱动程序列表</h3>
                    <span className="text-sm text-[var(--text-secondary)]">
                      共 {drivers.length} 个驱动程序
                    </span>
                  </div>
                  <motion.button
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                    onClick={loadDrivers}
                    disabled={isLoadingDrivers}
                    className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-secondary)] text-sm font-medium transition-all duration-200 hover:border-[var(--color-primary)]/50 hover:text-[var(--text-primary)] disabled:opacity-50"
                  >
                    <RefreshCw className={`w-4 h-4 ${isLoadingDrivers ? 'animate-spin' : ''}`} />
                    刷新列表
                  </motion.button>
                </div>

                <div className="flex items-center gap-4 mb-6">
                  <div className="flex-1 relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--text-tertiary)]" />
                    <input
                      type="text"
                      placeholder="搜索驱动名称、提供商或设备..."
                      value={driverSearchQuery}
                      onChange={(e) => setDriverSearchQuery(e.target.value)}
                      className="w-full pl-10 pr-4 py-2 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-color)] text-[var(--text-primary)] placeholder-[var(--text-tertiary)] focus:outline-none focus:border-[var(--color-primary)] transition-colors text-sm"
                    />
                  </div>
                  <div className="relative driver-type-dropdown">
                    <motion.button
                      whileHover={{ scale: 1.02 }}
                      whileTap={{ scale: 0.98 }}
                      onClick={() => setShowTypeDropdown(!showTypeDropdown)}
                      className="flex items-center gap-2 px-4 py-2 rounded-xl bg-[var(--bg-tertiary)] border border-[var(--border-color)] text-[var(--text-primary)] text-sm font-medium transition-all duration-200 hover:border-[var(--color-primary)]/50 min-w-[120px]"
                    >
                      <Filter className="w-4 h-4 text-[var(--color-primary)]" />
                      <span>{driverTypeFilter === 'all' ? '全部类型' : driverTypeFilter}</span>
                      <motion.svg 
                        className="w-4 h-4 ml-auto text-[var(--text-tertiary)]" 
                        fill="none" 
                        stroke="currentColor" 
                        viewBox="0 0 24 24"
                        animate={{ rotate: showTypeDropdown ? 180 : 0 }}
                        transition={{ duration: 0.2 }}
                      >
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                      </motion.svg>
                    </motion.button>
                    
                    <AnimatePresence>
                      {showTypeDropdown && (
                        <motion.div
                          initial={{ opacity: 0, y: -10, scale: 0.95 }}
                          animate={{ opacity: 1, y: 0, scale: 1 }}
                          exit={{ opacity: 0, y: -10, scale: 0.95 }}
                          transition={{ duration: 0.15 }}
                          className="absolute top-full left-0 mt-2 w-48 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-xl overflow-hidden z-50"
                        >
                          <div className="py-1 max-h-60 overflow-y-auto">
                            <button
                              onClick={() => {
                                setDriverTypeFilter('all');
                                setShowTypeDropdown(false);
                              }}
                              className={`w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-colors ${
                                driverTypeFilter === 'all' 
                                  ? 'bg-[var(--color-primary)]/10 text-[var(--color-primary)]' 
                                  : 'text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
                              }`}
                            >
                              <div className={`w-2 h-2 rounded-full ${driverTypeFilter === 'all' ? 'bg-[var(--color-primary)]' : 'bg-transparent'}`} />
                              全部类型
                              {driverTypeFilter === 'all' && (
                                <CheckCircle className="w-4 h-4 ml-auto text-[var(--color-primary)]" />
                              )}
                            </button>
                            {driverTypes.map(type => (
                              <button
                                key={type}
                                onClick={() => {
                                  setDriverTypeFilter(type);
                                  setShowTypeDropdown(false);
                                }}
                                className={`w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-colors ${
                                  driverTypeFilter === type 
                                    ? 'bg-[var(--color-primary)]/10 text-[var(--color-primary)]' 
                                    : 'text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
                                }`}
                              >
                                <div className={`w-2 h-2 rounded-full ${driverTypeFilter === type ? 'bg-[var(--color-primary)]' : 'bg-transparent'}`} />
                                {type}
                                {driverTypeFilter === type && (
                                  <CheckCircle className="w-4 h-4 ml-auto text-[var(--color-primary)]" />
                                )}
                              </button>
                            ))}
                          </div>
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </div>
                </div>

                {isLoadingDrivers && drivers.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-16">
                    <Loader2 className="w-10 h-10 text-[var(--color-primary)] animate-spin mb-4" />
                    <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">正在加载驱动列表</h3>
                    <p className="text-[var(--text-secondary)] text-sm">请稍候...</p>
                  </div>
                ) : filteredDrivers.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-16">
                    <div className="w-16 h-16 rounded-full bg-[var(--color-primary)]/10 flex items-center justify-center mb-4">
                      <Cpu className="w-8 h-8 text-[var(--color-primary)] opacity-50" />
                    </div>
                    <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
                      {driverSearchQuery || driverTypeFilter !== 'all' ? '未找到匹配的驱动' : '暂无驱动数据'}
                    </h3>
                    <p className="text-[var(--text-secondary)] text-sm">
                      {driverSearchQuery || driverTypeFilter !== 'all' ? '请尝试其他搜索条件' : '点击刷新列表获取驱动程序信息'}
                    </p>
                  </div>
                ) : (
                  <div className="overflow-x-auto">
                    <table className="w-full">
                      <thead>
                        <tr className="border-b border-[var(--border-color)]">
                          <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">设备名称</th>
                          <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">类型</th>
                          <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">版本</th>
                          <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">提供商</th>
                          <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">日期</th>
                          <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">签名</th>
                          <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">状态</th>
                          <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">操作</th>
                        </tr>
                      </thead>
                      <tbody>
                        {filteredDrivers.map(driver => {
                          const statusBadge = getStatusBadge(driver.status);
                          return (
                            <tr key={driver.id} className="border-b border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors">
                              <td className="py-4 px-4">
                                <div className="flex items-center gap-3">
                                  <div className="w-8 h-8 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]">
                                    <Cpu className="w-4 h-4" />
                                  </div>
                                  <div>
                                    <span className="text-sm text-[var(--text-primary)] block">{driver.name}</span>
                                    <span className="text-xs text-[var(--text-tertiary)] truncate block max-w-[200px]" title={driver.inf_name}>
                                      {driver.inf_name}
                                    </span>
                                  </div>
                                </div>
                              </td>
                              <td className="py-4 px-4">
                                <span className="text-sm text-[var(--text-secondary)]">{driver.driver_type}</span>
                              </td>
                              <td className="py-4 px-4">
                                <span className="text-sm text-[var(--text-primary)] font-mono">{driver.version}</span>
                              </td>
                              <td className="py-4 px-4">
                                <span className="text-sm text-[var(--text-secondary)]">{driver.provider}</span>
                              </td>
                              <td className="py-4 px-4">
                                <span className="text-sm text-[var(--text-secondary)]">{driver.date}</span>
                              </td>
                              <td className="py-4 px-4">
                                {driver.signed ? (
                                  <span className="inline-flex items-center gap-1 text-emerald-500 whitespace-nowrap">
                                    <ShieldCheck className="w-4 h-4 flex-shrink-0" />
                                    <span className="text-xs">已签名</span>
                                  </span>
                                ) : (
                                  <span className="inline-flex items-center gap-1 text-amber-500 whitespace-nowrap">
                                    <Shield className="w-4 h-4 flex-shrink-0" />
                                    <span className="text-xs">未签名</span>
                                  </span>
                                )}
                              </td>
                              <td className="py-4 px-4">
                                <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium whitespace-nowrap ${statusBadge.bg} ${statusBadge.text}`}>
                                  {driver.status === 'Running' ? (
                                    <CheckCircle className="w-3 h-3 flex-shrink-0" />
                                  ) : (
                                    <AlertCircle className="w-3 h-3 flex-shrink-0" />
                                  )}
                                  {statusBadge.label}
                                </span>
                              </td>
                              <td className="py-4 px-4">
                                <div className="flex items-center gap-2">
                                  <button 
                                    className="p-1.5 rounded hover:bg-red-500/10 text-[var(--text-tertiary)] hover:text-red-500 transition-colors" 
                                    title="删除驱动"
                                    onClick={() => confirmDeleteDriver(driver)}
                                  >
                                    <Trash2 className="w-4 h-4" />
                                  </button>
                                </div>
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                )}

                <div className="flex items-center gap-2 p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 mt-6">
                  <AlertTriangle className="w-4 h-4 text-amber-500 flex-shrink-0" />
                  <p className="text-xs text-amber-600 dark:text-amber-400">
                    <strong>注意：</strong>删除驱动可能导致相关硬件无法正常工作。请谨慎操作，建议仅删除不再使用的旧版本驱动。
                  </p>
                </div>

                <div className="flex items-center gap-2 p-3 rounded-lg bg-blue-500/10 border border-blue-500/20 mt-3">
                  <Info className="w-4 h-4 text-blue-500 flex-shrink-0" />
                  <p className="text-xs text-blue-600 dark:text-blue-400">
                    <strong>提示：</strong>目前仅支持删除驱动功能，其他功能（如更新、备份、还原）正在开发中。
                  </p>
                </div>
              </div>

              <AnimatePresence>
                {showDriverDeleteConfirm && driverToDelete && (
                  <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
                    onClick={() => !isDeletingDriver && setShowDriverDeleteConfirm(false)}
                  >
                    <motion.div
                      initial={{ scale: 0.95, opacity: 0 }}
                      animate={{ scale: 1, opacity: 1 }}
                      exit={{ scale: 0.95, opacity: 0 }}
                      className="w-full max-w-md mx-4 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-xl"
                      onClick={(e) => e.stopPropagation()}
                    >
                      <div className="p-6">
                        <div className="flex items-center gap-4 mb-4">
                          <div className="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center">
                            <AlertTriangle className="w-6 h-6 text-red-500" />
                          </div>
                          <div>
                            <h3 className="text-lg font-semibold text-[var(--text-primary)]">确认删除驱动</h3>
                            <p className="text-sm text-[var(--text-secondary)]">此操作不可撤销</p>
                          </div>
                        </div>

                        <div className="p-4 rounded-lg bg-[var(--bg-tertiary)] mb-4">
                          <div className="space-y-2">
                            <div className="flex justify-between">
                              <span className="text-sm text-[var(--text-tertiary)]">驱动名称</span>
                              <span className="text-sm text-[var(--text-primary)] font-medium">{driverToDelete.name}</span>
                            </div>
                            <div className="flex justify-between">
                              <span className="text-sm text-[var(--text-tertiary)]">版本</span>
                              <span className="text-sm text-[var(--text-primary)] font-mono">{driverToDelete.version}</span>
                            </div>
                            <div className="flex justify-between">
                              <span className="text-sm text-[var(--text-tertiary)]">提供商</span>
                              <span className="text-sm text-[var(--text-primary)]">{driverToDelete.provider}</span>
                            </div>
                            <div className="flex justify-between">
                              <span className="text-sm text-[var(--text-tertiary)]">INF文件</span>
                              <span className="text-xs text-[var(--text-primary)] font-mono">{driverToDelete.inf_name}</span>
                            </div>
                          </div>
                        </div>

                        <p className="text-sm text-[var(--text-secondary)] mb-6">
                          删除此驱动后，相关硬件可能无法正常工作。确定要继续吗？
                        </p>

                        <div className="flex gap-3">
                          <button
                            onClick={() => setShowDriverDeleteConfirm(false)}
                            disabled={isDeletingDriver}
                            className="flex-1 px-4 py-2 rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors disabled:opacity-50"
                          >
                            取消
                          </button>
                          <button
                            onClick={handleDeleteDriver}
                            disabled={isDeletingDriver}
                            className="flex-1 px-4 py-2 rounded-lg bg-red-500 text-white hover:bg-red-600 transition-colors flex items-center justify-center gap-2 disabled:opacity-50"
                          >
                            {isDeletingDriver ? (
                              <>
                                <Loader2 className="w-4 h-4 animate-spin" />
                                删除中...
                              </>
                            ) : (
                              <>
                                <Trash2 className="w-4 h-4" />
                                确认删除
                              </>
                            )}
                          </button>
                        </div>
                      </div>
                    </motion.div>
                  </motion.div>
                )}
              </AnimatePresence>
            </motion.div>
          )}
        </AnimatePresence>
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

function ResidueCategoryPanel({
  result,
  selectedIds,
  onToggle,
  isExpanded,
  onToggleExpand,
}: {
  result: ResidueScanResult;
  selectedIds: Set<string>;
  onToggle: (id: string) => void;
  isExpanded: boolean;
  onToggleExpand: () => void;
}) {
  const allSelected = result.items.every(item => selectedIds.has(item.id));
  const someSelected = result.items.some(item => selectedIds.has(item.id));

  const handleSelectAll = () => {
    result.items.forEach(item => {
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
        onClick={onToggleExpand}
      >
        <motion.div
          animate={{ rotate: isExpanded ? 90 : 0 }}
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

        <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-gradient-to-br from-blue-500/20 to-purple-500/20 flex items-center justify-center">
          {getResidueTypeIcon(result.residue_type)}
        </div>

        <div className="flex-1">
          <h4 className="text-sm font-medium text-[var(--text-primary)]">
            {RESIDUE_TYPE_NAMES[result.residue_type]}
          </h4>
          <p className="text-xs text-[var(--text-secondary)]">
            {result.items.length} 项 {result.total_size > 0 && `· ${formatSize(result.total_size)}`}
          </p>
        </div>

        <span className="text-sm font-semibold text-[#10b981]">
          {formatSize(result.total_size)}
        </span>
      </div>

      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: 'auto' }}
            exit={{ height: 0 }}
            className="overflow-hidden"
          >
            <div className="px-4 pb-4 space-y-2 max-h-[300px] overflow-y-auto bg-[var(--bg-primary)]/50">
              {result.items.map((item) => (
                <ResidueItemCard
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
}

function ResidueItemCard({
  item,
  selected,
  onToggle,
}: {
  item: ResidueItem;
  selected: boolean;
  onToggle: () => void;
}) {
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

      {getResidueTypeIcon(item.residue_type)}

      <div className="flex-1 min-w-0">
        <p className="text-sm text-[var(--text-primary)] truncate" title={item.name}>
          {item.name}
        </p>
        <p className="text-xs text-[var(--text-tertiary)] truncate" title={item.path}>
          {item.path}
        </p>
      </div>

      <div className="flex items-center gap-3 text-xs text-[var(--text-secondary)]">
        {item.size > 0 && <span>{formatSize(item.size)}</span>}
        {item.last_modified > 0 && <span>{formatDate(item.last_modified)}</span>}
        {item.residue_type !== 'registry_key' && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              openFileLocation(item.path);
            }}
            className="p-1 rounded hover:bg-[var(--bg-tertiary)] hover:text-[var(--color-primary)] transition-colors"
          >
            <ExternalLink className="w-3 h-3" />
          </button>
        )}
      </div>
    </motion.div>
  );
}

function ChevronRight({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M9 18l6-6-6-6" />
    </svg>
  );
}

function ConfirmDialog({
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
}) {
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
}
