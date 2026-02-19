import { memo, useCallback } from 'react';
import { motion } from 'framer-motion';
import { CheckSquare, Square, HelpCircle } from 'lucide-react';
import { formatBytes } from '../../../utils/format';
import { CategoryItem } from './CategoryItem';
import type { 
  BaseFileInfo, 
  BaseCategoryInfo, 
  CategorizedFileListProps
} from './types';
import { DEFAULT_LOAD_MORE_LIMIT, DEFAULT_MAX_LIST_HEIGHT } from './types';

function CategorizedFileListInner<
  TFile extends BaseFileInfo = BaseFileInfo,
  TCategory extends BaseCategoryInfo<TFile> = BaseCategoryInfo<TFile>
>({
  categories,
  selectedFiles,
  expandedCategories,
  getCategoryKey,
  getFileKey,
  onToggleFileSelection,
  onToggleCategorySelection,
  onToggleCategoryExpand,
  onSelectAll,
  onDeselectAll,
  onExpandAll,
  onCollapseAll,
  onLoadMore,
  onOpenLocation,
  totalCount,
  totalSize: _totalSize,
  selectedCount,
  selectedSize,
  renderFileRow,
  emptyText = '暂无文件',
  showExpandCollapse = true,
  showSelectAll = true,
  maxListHeight = DEFAULT_MAX_LIST_HEIGHT,
  loadMoreLimit = DEFAULT_LOAD_MORE_LIMIT,
  title = '文件分类',
  showHelpButton = false,
  onHelpClick,
}: CategorizedFileListProps<TFile, TCategory>) {
  const allSelected = selectedCount === totalCount && totalCount > 0;

  const handleLoadMoreForCategory = useCallback(
    (categoryKey: string) => {
      if (!onLoadMore) return undefined;
      
      return async () => {
        const category = categories.find(c => getCategoryKey(c) === categoryKey);
        if (!category) return null;
        
        const currentFiles = category.files;
        return onLoadMore(categoryKey, currentFiles.length, loadMoreLimit);
      };
    },
    [onLoadMore, categories, getCategoryKey, loadMoreLimit]
  );

  if (categories.length === 0) {
    return (
      <div className="p-4 text-center text-sm text-[var(--text-tertiary)]">
        {emptyText}
      </div>
    );
  }

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: 0.5 }}
      className="mt-6"
    >
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <h4 className="text-sm font-medium text-[var(--text-primary)]">{title}</h4>
          {showHelpButton && onHelpClick && (
            <motion.button
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.9 }}
              onClick={onHelpClick}
              className="p-1 rounded-lg text-[var(--text-tertiary)] hover:text-[var(--color-primary)] hover:bg-[var(--color-primary)]/10 transition-colors"
              title="查看分类说明"
            >
              <HelpCircle className="w-4 h-4" />
            </motion.button>
          )}
        </div>
        {showExpandCollapse && (
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
        )}
      </div>

      {showSelectAll && (
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
      )}

      <div className="space-y-2">
        {categories.map((category) => {
          const categoryKey = getCategoryKey(category);
          const loadMoreHandler = handleLoadMoreForCategory(categoryKey);
          
          return (
            <CategoryItem
              key={categoryKey}
              category={category}
              isExpanded={expandedCategories.has(categoryKey)}
              selectedFiles={selectedFiles}
              onToggleExpand={() => onToggleCategoryExpand(categoryKey)}
              onToggleSelection={() => onToggleCategorySelection(categoryKey)}
              onToggleFileSelection={onToggleFileSelection}
              onLoadMore={loadMoreHandler}
              renderFileRow={renderFileRow as any}
              onOpenLocation={onOpenLocation as any}
              getFileKey={getFileKey}
              maxListHeight={maxListHeight}
              emptyText={emptyText}
            />
          );
        })}
      </div>
    </motion.div>
  );
}

const CategorizedFileList = memo(CategorizedFileListInner) as <
  TFile extends BaseFileInfo = BaseFileInfo,
  TCategory extends BaseCategoryInfo<TFile> = BaseCategoryInfo<TFile>
>(
  props: CategorizedFileListProps<TFile, TCategory>
) => JSX.Element;

export { CategorizedFileList };
