import { memo, useState, useMemo, useEffect, useRef, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { ChevronRight, CheckSquare, Square } from 'lucide-react';
import { formatBytes } from '../../../utils/format';
import { FileRow } from './FileRow';
import type { BaseFileInfo, BaseCategoryInfo, CategoryFilesState, FileRowProps } from './types';
import { DEFAULT_MAX_LIST_HEIGHT } from './types';

interface CategoryItemProps<TFile extends BaseFileInfo = BaseFileInfo> {
  category: BaseCategoryInfo<TFile>;
  isExpanded: boolean;
  selectedFiles: Set<string>;
  onToggleExpand: () => void;
  onToggleSelection: () => void;
  onToggleFileSelection: (filePath: string) => void;
  onLoadMore?: () => Promise<{ files: TFile[]; hasMore: boolean } | null>;
  renderFileRow?: (props: FileRowProps<TFile>) => React.ReactNode;
  onOpenLocation?: (file: TFile) => void;
  getFileKey: (file: TFile) => string;
  maxListHeight?: number;
  emptyText?: string;
}

function CategoryItemInner<TFile extends BaseFileInfo>({
  category,
  isExpanded,
  selectedFiles,
  onToggleExpand,
  onToggleSelection,
  onToggleFileSelection,
  onLoadMore,
  renderFileRow,
  onOpenLocation,
  getFileKey,
  maxListHeight = DEFAULT_MAX_LIST_HEIGHT,
  emptyText = '暂无文件',
}: CategoryItemProps<TFile>) {
  const [filesState, setFilesState] = useState<CategoryFilesState<TFile>>({
    files: category.files,
    total: category.fileCount,
    hasMore: category.hasMore ?? false,
    isLoading: false,
  });

  const loadMoreRef = useRef<HTMLDivElement>(null);
  const categoryRef = useRef<HTMLDivElement>(null);
  const wasExpandedRef = useRef(false);

  useEffect(() => {
    setFilesState({
      files: category.files,
      total: category.fileCount,
      hasMore: category.hasMore ?? false,
      isLoading: false,
    });
  }, [category.files, category.fileCount, category.hasMore]);

  const selectedInCategory = useMemo(() => {
    return filesState.files.filter(f => selectedFiles.has(getFileKey(f))).length;
  }, [filesState.files, selectedFiles, getFileKey]);

  const allSelected = filesState.files.length > 0 && 
    filesState.files.every(f => selectedFiles.has(getFileKey(f)));

  useEffect(() => {
    if (isExpanded && !wasExpandedRef.current && categoryRef.current) {
      categoryRef.current.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
    wasExpandedRef.current = isExpanded;
  }, [isExpanded]);

  const handleLoadMore = useCallback(async () => {
    if (!onLoadMore || filesState.isLoading || !filesState.hasMore) return;

    setFilesState(prev => ({ ...prev, isLoading: true }));

    try {
      const result = await onLoadMore();
      if (result) {
        setFilesState(prev => ({
          ...prev,
          files: [...prev.files, ...result.files],
          hasMore: result.hasMore,
          isLoading: false,
        }));
      }
    } catch {
      setFilesState(prev => ({ ...prev, isLoading: false }));
    }
  }, [onLoadMore, filesState.isLoading, filesState.hasMore]);

  useEffect(() => {
    if (!isExpanded || !filesState.hasMore || filesState.isLoading || !onLoadMore) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting) {
          handleLoadMore();
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
  }, [isExpanded, filesState.hasMore, filesState.isLoading, handleLoadMore, onLoadMore]);

  useEffect(() => {
    if (isExpanded && filesState.files.length === 0 && filesState.hasMore && onLoadMore) {
      handleLoadMore();
    }
  }, [isExpanded]);

  const renderFileRowInternal = (file: TFile, index: number) => {
    const fileKey = getFileKey(file);
    const isLast = index === filesState.files.length - 1 && !filesState.hasMore;
    const isSelected = selectedFiles.has(fileKey);

    const props: FileRowProps<TFile> = {
      file,
      isSelected,
      isLast,
      onToggleSelection: () => onToggleFileSelection(fileKey),
      onOpenLocation,
    };

    if (renderFileRow) {
      return renderFileRow(props);
    }

    return <FileRow key={fileKey} {...props} />;
  };

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
          {allSelected ? (
            <CheckSquare className="w-5 h-5 text-[var(--color-primary)]" />
          ) : selectedInCategory > 0 ? (
            <div className="w-5 h-5 rounded border-2 border-[var(--color-primary)] bg-[var(--color-primary)]/20 flex items-center justify-center">
              <div className="w-2 h-2 rounded-sm bg-[var(--color-primary)]" />
            </div>
          ) : (
            <Square className="w-5 h-5 text-[var(--text-tertiary)]" />
          )}
        </button>

        {category.icon && (
          <div 
            className={`w-10 h-10 rounded-lg bg-[var(--bg-tertiary)] flex items-center justify-center ${category.iconColor ?? ''}`}
          >
            {category.icon}
          </div>
        )}

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-[var(--text-primary)]">
              {category.displayName}
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
          {formatBytes(category.totalSize)}
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
              <div 
                className="overflow-y-auto scrollbar-thin"
                style={{ maxHeight: maxListHeight }}
              >
                {filesState.files.map((file, index) => renderFileRowInternal(file, index))}
                {filesState.hasMore && (
                  <div ref={loadMoreRef} className="h-1" />
                )}
                {filesState.isLoading && (
                  <div className="flex items-center justify-center py-3 text-sm text-[var(--text-tertiary)]">
                    加载中...
                  </div>
                )}
              </div>
            ) : (
              <div className="p-4 text-center text-sm text-[var(--text-tertiary)]">
                {emptyText}
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}

const CategoryItem = memo(CategoryItemInner) as <TFile extends BaseFileInfo>(
  props: CategoryItemProps<TFile>
) => JSX.Element;

export { CategoryItem };
export type { CategoryItemProps };
