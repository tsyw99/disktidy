import { useMemo, useState, useCallback, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  CheckCircle2, FileText, FolderOpen, Trash2, RotateCcw, Inbox, 
  ChevronRight, ExternalLink, CheckSquare, Square, HelpCircle
} from 'lucide-react';
import type { ScanResult, FileCategory, FileInfo } from '../../types';
import { formatBytes } from '../../utils/format';
import { openFileLocation } from '../../utils/shell';
import { scanService } from '../../services/scanService';
import { CategoryHelpModal } from '../common';

interface ScanResultsProps {
  result: ScanResult | null;
  scanId: string | null;
  selectedFiles: Set<string>;
  expandedCategories: Set<string>;
  onReset?: () => void;
  onDelete?: () => void;
  onToggleFileSelection: (filePath: string) => void;
  onToggleCategorySelection: (categoryName: string) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onToggleCategoryExpand: (categoryName: string) => void;
  onExpandAll: () => void;
  onCollapseAll: () => void;
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function FileRow({
  file,
  isSelected,
  isLast,
  onToggleSelection,
}: {
  file: FileInfo;
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

interface CategoryFilesState {
  files: FileInfo[];
  total: number;
  hasMore: boolean;
  isLoading: boolean;
}

function CategoryItem({
  category,
  scanId,
  isExpanded,
  isSelected,
  selectedFiles,
  onToggleExpand,
  onToggleSelection,
  onToggleFileSelection,
}: {
  category: FileCategory;
  scanId: string | null;
  isExpanded: boolean;
  isSelected: boolean;
  selectedFiles: Set<string>;
  onToggleExpand: () => void;
  onToggleSelection: () => void;
  onToggleFileSelection: (filePath: string) => void;
}) {
  const [filesState, setFilesState] = useState<CategoryFilesState>({
    files: category.files,
    total: category.file_count,
    hasMore: category.has_more,
    isLoading: false,
  });
  
  const loadMoreRef = useRef<HTMLDivElement>(null);
  const categoryRef = useRef<HTMLDivElement>(null);
  const wasExpandedRef = useRef(false);
  
  const selectedInCategory = useMemo(() => {
    return filesState.files.filter(f => selectedFiles.has(f.path)).length;
  }, [filesState.files, selectedFiles]);

  useEffect(() => {
    if (isExpanded && !wasExpandedRef.current && categoryRef.current) {
      categoryRef.current.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
    wasExpandedRef.current = isExpanded;
  }, [isExpanded]);

  const loadMore = useCallback(async () => {
    if (!scanId || filesState.isLoading || !filesState.hasMore) return;
    
    setFilesState(prev => ({ ...prev, isLoading: true }));
    
    try {
      const response = await scanService.getCategoryFiles(
        scanId,
        category.name,
        filesState.files.length,
        50
      );
      
      if (response) {
        setFilesState(prev => ({
          ...prev,
          files: [...prev.files, ...response.files],
          hasMore: response.has_more,
          isLoading: false,
        }));
      }
    } catch (e) {
      setFilesState(prev => ({ ...prev, isLoading: false }));
    }
  }, [scanId, category.name, filesState.files.length, filesState.hasMore, filesState.isLoading]);

  useEffect(() => {
    if (!isExpanded || !filesState.hasMore || filesState.isLoading) return;
    
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          loadMore();
        }
      },
      { threshold: 0.1, rootMargin: '100px' }
    );
    
    const currentRef = loadMoreRef.current;
    if (currentRef) {
      observer.observe(currentRef);
    }
    
    return () => {
      if (currentRef) {
        observer.unobserve(currentRef);
      }
    };
  }, [isExpanded, filesState.hasMore, filesState.isLoading, loadMore]);

  useEffect(() => {
    if (isExpanded && filesState.files.length === 0 && filesState.hasMore) {
      loadMore();
    }
  }, [isExpanded]);

  return (
    <motion.div
      ref={categoryRef}
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      className="rounded-lg border border-[var(--border-color)] overflow-hidden"
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
        
        <button
          onClick={(e) => {
            e.stopPropagation();
            onToggleSelection();
          }}
          className="flex-shrink-0"
        >
          {isSelected ? (
            <CheckSquare className="w-5 h-5 text-[var(--color-primary)]" />
          ) : selectedInCategory > 0 ? (
            <div className="w-5 h-5 rounded border-2 border-[var(--color-primary)] bg-[var(--color-primary)]/20 flex items-center justify-center">
              <div className="w-2 h-2 rounded-sm bg-[var(--color-primary)]" />
            </div>
          ) : (
            <Square className="w-5 h-5 text-[var(--text-tertiary)]" />
          )}
        </button>
        
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-[var(--text-primary)]">
              {category.display_name}
            </span>
            <span className="text-xs text-[var(--text-tertiary)]">
              ({filesState.total} 个文件)
            </span>
          </div>
          {selectedInCategory > 0 && (
            <p className="text-xs text-[var(--color-primary)] mt-0.5">
              已选择 {selectedInCategory} 个文件
            </p>
          )}
        </div>
        
        <span className="text-sm font-medium text-[#10b981]">
          {formatBytes(category.total_size)}
        </span>
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
            {filesState.files.length > 0 ? (
              <div className="max-h-[300px] overflow-y-auto scrollbar-thin">
                {filesState.files.map((file, index) => {
                  const isLast = index === filesState.files.length - 1 && !filesState.hasMore;
                  return (
                    <FileRow
                      key={file.path}
                      file={file}
                      isSelected={selectedFiles.has(file.path)}
                      isLast={isLast}
                      onToggleSelection={() => onToggleFileSelection(file.path)}
                    />
                  );
                })}
                {filesState.hasMore && (
                  <div 
                    ref={loadMoreRef}
                    className="h-1"
                  />
                )}
              </div>
            ) : (
              <div className="p-4 text-center text-sm text-[var(--text-tertiary)]">
                暂无文件
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}

export default function ScanResults({
  result,
  scanId,
  selectedFiles,
  expandedCategories,
  onReset,
  onDelete,
  onToggleFileSelection,
  onToggleCategorySelection,
  onSelectAll,
  onDeselectAll,
  onToggleCategoryExpand,
  onExpandAll,
  onCollapseAll,
}: ScanResultsProps) {
  const [helpModalVisible, setHelpModalVisible] = useState(false);

  if (!result) {
    return null;
  }

  const hasFiles = result.total_files > 0;
  const selectedCount = selectedFiles.size;
  const totalFiles = result.total_files;
  const allSelected = selectedCount === totalFiles && totalFiles > 0;

  const selectedSize = useMemo(() => {
    let size = 0;
    for (const category of result.categories) {
      for (const file of category.files) {
        if (selectedFiles.has(file.path)) {
          size += file.size;
        }
      }
    }
    return size;
  }, [result.categories, selectedFiles]);

  const categoryInfoList = useMemo(() => {
    return result.categories.map(cat => ({
      name: cat.name,
      displayName: cat.display_name,
      description: cat.description || '',
    }));
  }, [result.categories]);

  if (!hasFiles) {
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
          className="rounded-xl border-2 border-[#3b82f6] bg-[#3b82f6]/5 p-6"
        >
          <div className="flex items-start gap-4">
            <div className="flex-shrink-0">
              <div className="w-12 h-12 rounded-full bg-[#3b82f6] flex items-center justify-center">
                <Inbox className="w-7 h-7 text-white" />
              </div>
            </div>

            <div className="flex-1 min-w-0">
              <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
                扫描完成
              </h3>
              <p className="text-sm text-[var(--text-secondary)] leading-relaxed">
                未发现可清理的垃圾文件，您的磁盘非常干净！
              </p>
            </div>
          </div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="flex justify-center mt-6"
        >
          {onReset && (
            <motion.button
              onClick={onReset}
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              className="flex items-center gap-2 px-6 py-3 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-primary)] font-medium hover:border-[var(--color-primary)]/50 hover:bg-[var(--color-primary)]/5 transition-all duration-300"
            >
              <RotateCcw className="w-5 h-5" />
              重新扫描
            </motion.button>
          )}
        </motion.div>
      </motion.div>
    );
  }

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
              共扫描 <span className="font-medium text-[var(--text-primary)]">{result.total_files.toLocaleString()}</span> 个文件，
              发现可清理空间 <span className="font-medium text-[#10b981]">{formatBytes(result.total_size)}</span>
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
            {result.total_files.toLocaleString()}
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
            {(result.total_folders ?? 0).toLocaleString()}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">文件夹总数</p>
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
            {formatBytes(result.total_size)}
          </p>
          <p className="text-sm text-[var(--text-tertiary)]">可清理空间</p>
        </motion.div>
      </div>

      {result.categories && result.categories.length > 0 && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.5 }}
          className="mt-6"
        >
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <h4 className="text-sm font-medium text-[var(--text-primary)]">文件分类</h4>
              <motion.button
                whileHover={{ scale: 1.1 }}
                whileTap={{ scale: 0.9 }}
                onClick={() => setHelpModalVisible(true)}
                className="p-1 rounded-lg text-[var(--text-tertiary)] hover:text-[var(--color-primary)] hover:bg-[var(--color-primary)]/10 transition-colors"
                title="查看分类说明"
              >
                <HelpCircle className="w-4 h-4" />
              </motion.button>
            </div>
            <div className="flex items-center gap-2">
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={onExpandAll}
                className="text-xs text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors"
              >
                全部展开
              </motion.button>
              <span className="text-[var(--text-tertiary)]">|</span>
              <motion.button
                whileHover={{ scale: 1.02 }}
                whileTap={{ scale: 0.98 }}
                onClick={onCollapseAll}
                className="text-xs text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors"
              >
                全部折叠
              </motion.button>
            </div>
          </div>

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
          
          <div className="space-y-2">
            {result.categories.map((category) => (
              <CategoryItem
                key={category.name}
                category={category}
                scanId={scanId}
                isExpanded={expandedCategories.has(category.name)}
                isSelected={category.files.every(f => selectedFiles.has(f.path))}
                selectedFiles={selectedFiles}
                onToggleExpand={() => onToggleCategoryExpand(category.name)}
                onToggleSelection={() => onToggleCategorySelection(category.name)}
                onToggleFileSelection={onToggleFileSelection}
              />
            ))}
          </div>
        </motion.div>
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

        {onReset && (
          <motion.button
            onClick={onReset}
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            className="flex items-center gap-2 px-6 py-3 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-primary)] font-medium hover:border-[var(--color-primary)]/50 hover:bg-[var(--color-primary)]/5 transition-all duration-300"
          >
            <RotateCcw className="w-5 h-5" />
            重新扫描
          </motion.button>
        )}
      </motion.div>

      <CategoryHelpModal
        visible={helpModalVisible}
        onClose={() => setHelpModalVisible(false)}
        categories={categoryInfoList}
      />
    </motion.div>
  );
}
