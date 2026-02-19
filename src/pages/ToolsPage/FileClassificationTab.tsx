import { useState, useCallback, useEffect } from 'react';
import { motion } from 'framer-motion';
import { open } from '@tauri-apps/plugin-dialog';
import { HardDrive, FileText, FolderTree, Layers, Zap, AlertCircle, Loader2, Database, FolderOpen, X } from 'lucide-react';
import { FileClassificationChart, CategoryDetailPanel } from '../../components/tools';
import { optimizedFileClassifier, type LegendValidationResult } from '../../services';
import { useSystemStore, useSystemActions } from '../../stores/systemStore';
import type { FileClassificationResult, FileTypeStats } from '../../types';

export function FileClassificationTab() {
  const [selectedDisk, setSelectedDisk] = useState<string>('all');
  const [classificationResult, setClassificationResult] = useState<FileClassificationResult | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<FileTypeStats | null>(null);
  const [isClassifying, setIsClassifying] = useState(false);
  const [classifyError, setClassifyError] = useState<string | null>(null);
  const [, setClassificationValidation] = useState<LegendValidationResult | null>(null);

  const { diskList } = useSystemStore();
  const systemActions = useSystemActions();

  useEffect(() => {
    if (diskList.length === 0) {
      systemActions.fetchDiskList();
    }
  }, [diskList.length, systemActions]);

  const handleClassifyFiles = useCallback(async () => {
    setIsClassifying(true);
    setClassifyError(null);
    setClassificationResult(null);
    setSelectedCategory(null);
    setClassificationValidation(null);

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
        const validation = optimizedFileClassifier.validateResult(mockResult);
        setClassificationValidation(validation);
        return;
      }

      let scanPath = selectedDisk;
      if (selectedDisk === 'all') {
        scanPath = 'C:\\';
      }

      console.log('[FileClassification] Scanning path:', scanPath, '(selectedDisk:', selectedDisk, ')');
      const result = await optimizedFileClassifier.startClassifyFiles(scanPath);
      console.log('[FileClassification] Result - path:', result.path, 'files:', result.total_files, 'size:', result.total_size);

      if (result.cancelled) {
        setClassifyError('扫描已取消');
      } else {
        setClassificationResult(result);
        const validation = optimizedFileClassifier.validateResult(result);
        setClassificationValidation(validation);

        if (!validation.isValid) {
          console.warn('文件分类结果验证警告:', validation.suggestions);
        }
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
      await optimizedFileClassifier.cancelClassifyFiles();
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

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
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
                <div
                  onClick={(e) => {
                    e.stopPropagation();
                    setSelectedDisk('all');
                  }}
                  className="ml-auto p-1 rounded hover:bg-[var(--bg-tertiary)] text-[var(--text-tertiary)] hover:text-[var(--text-primary)] cursor-pointer"
                >
                  <X className="w-4 h-4" />
                </div>
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
  );
}
