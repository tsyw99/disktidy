import { memo } from 'react';
import { motion } from 'framer-motion';
import { CheckSquare, Square, ExternalLink } from 'lucide-react';
import { formatBytes } from '../../../utils/format';
import type { BaseFileInfo, FileRowProps } from './types';

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
}

function DefaultFileRow<TFile extends BaseFileInfo>({
  file,
  isSelected,
  isLast,
  onToggleSelection,
  onOpenLocation,
}: FileRowProps<TFile>) {
  const handleOpenLocation = async (e: React.MouseEvent) => {
    e.stopPropagation();
    onOpenLocation?.(file);
  };

  const fileName = file.name ?? file.path.split(/[/\\]/).pop() ?? file.path;

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
        <p className="text-sm text-[var(--text-primary)] truncate" title={fileName}>
          {fileName}
        </p>
        <p className="text-xs text-[var(--text-tertiary)] truncate" title={file.path}>
          {file.path}
        </p>
        <div className="flex items-center gap-3 mt-1">
          <span className="text-xs text-[var(--text-tertiary)]">
            {formatBytes(file.size)}
          </span>
          {file.modified_time && (
            <span className="text-xs text-[var(--text-tertiary)]">
              {formatDate(file.modified_time)}
            </span>
          )}
        </div>
      </div>

      {onOpenLocation && (
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={handleOpenLocation}
          className="flex-shrink-0 p-1.5 rounded-lg text-[var(--text-tertiary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-secondary)] transition-colors"
          title="打开文件所在位置"
        >
          <ExternalLink className="w-4 h-4" />
        </motion.button>
      )}
    </div>
  );
}

const MemoizedFileRow = memo(DefaultFileRow) as <TFile extends BaseFileInfo>(
  props: FileRowProps<TFile>
) => JSX.Element;

export { MemoizedFileRow as FileRow };
export { formatDate };
