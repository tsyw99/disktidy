import { useMemo, useState, useCallback } from 'react';
import { motion } from 'framer-motion';
import { 
  CheckCircle2, FileText, FolderOpen, Trash2, RotateCcw, Inbox,
  ExternalLink, CheckSquare, Square
} from 'lucide-react';
import type { ScanResult, FileInfo } from '../../types';
import { formatBytes } from '../../utils/format';
import { openFileLocation } from '../../utils/shell';
import { scanService } from '../../services/scanService';
import { useScanStore } from '../../stores/scanStore';
import { 
  CategorizedFileList, 
  CategoryHelpModal,
  type BaseCategoryInfo,
  type FileRowProps,
} from '../common';

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

interface ScanCategoryInfo extends BaseCategoryInfo<FileInfo> {
  name: string;
  description: string;
}

function ScanFileRow({ file, isSelected, isLast, onToggleSelection, onOpenLocation }: FileRowProps<FileInfo>) {
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
            {new Date(file.modified_time).toLocaleString('zh-CN', {
              year: 'numeric',
              month: '2-digit',
              day: '2-digit',
              hour: '2-digit',
              minute: '2-digit',
            })}
          </span>
        </div>
      </div>
      
      {onOpenLocation && (
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={(e) => {
            e.stopPropagation();
            onOpenLocation(file);
          }}
          className="flex-shrink-0 p-1.5 rounded-lg text-[var(--text-tertiary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-secondary)] transition-colors"
          title="打开文件所在位置"
        >
          <ExternalLink className="w-4 h-4" />
        </motion.button>
      )}
    </div>
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
  const selectedSize = useScanStore((state) => state.actions.getSelectedSize());

  const categoryInfoList = useMemo(() => {
    return result.categories.map(cat => ({
      name: cat.name,
      displayName: cat.display_name,
      description: cat.description || '',
    }));
  }, [result.categories]);

  const categories: ScanCategoryInfo[] = useMemo(() => {
    return result.categories.map(cat => ({
      key: cat.name,
      name: cat.name,
      displayName: cat.display_name,
      description: cat.description,
      files: cat.files,
      fileCount: cat.file_count,
      totalSize: cat.total_size,
      hasMore: cat.has_more,
    }));
  }, [result.categories]);

  const handleLoadMore = useCallback(async (
    categoryKey: string, 
    offset: number, 
    limit: number
  ) => {
    if (!scanId) return null;
    
    try {
      const response = await scanService.getCategoryFiles(scanId, categoryKey, offset, limit);
      if (response) {
        return {
          files: response.files,
          hasMore: response.has_more,
        };
      }
    } catch {
      // 静默处理错误
    }
    return null;
  }, [scanId]);

  const handleOpenLocation = useCallback(async (file: FileInfo) => {
    await openFileLocation(file.path);
  }, []);

  const renderFileRow = useCallback((props: FileRowProps<FileInfo>) => {
    return (
      <ScanFileRow
        key={props.file.path}
        file={props.file}
        isSelected={props.isSelected}
        isLast={props.isLast}
        onToggleSelection={props.onToggleSelection}
        onOpenLocation={props.onOpenLocation}
      />
    );
  }, []);

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
        <CategorizedFileList<FileInfo, ScanCategoryInfo>
          categories={categories}
          selectedFiles={selectedFiles}
          expandedCategories={expandedCategories}
          getCategoryKey={(c) => c.key}
          getFileKey={(f) => f.path}
          onToggleFileSelection={onToggleFileSelection}
          onToggleCategorySelection={onToggleCategorySelection}
          onToggleCategoryExpand={onToggleCategoryExpand}
          onSelectAll={onSelectAll}
          onDeselectAll={onDeselectAll}
          onExpandAll={onExpandAll}
          onCollapseAll={onCollapseAll}
          onLoadMore={handleLoadMore}
          onOpenLocation={handleOpenLocation}
          totalCount={result.total_files}
          totalSize={result.total_size}
          selectedCount={selectedCount}
          selectedSize={selectedSize}
          renderFileRow={renderFileRow}
          title="文件分类"
          showHelpButton
          onHelpClick={() => setHelpModalVisible(true)}
        />
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
