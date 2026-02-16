import { memo } from 'react';
import { List } from 'react-window';
import { CheckSquare, Square, Users, Calendar, HardDrive, ExternalLink } from 'lucide-react';
import { openFileLocation } from '../../utils/shell';
import { formatBytes } from '../../utils/format';

interface FileItem {
  id: string;
  path: string;
  name: string;
  size: number;
  app: string;
  appName: string;
  appColor: string;
  appBgColor: string;
  chatObject: string;
  modifiedAt: number;
  selected: boolean;
}

interface VirtualFileListProps {
  files: FileItem[];
  onFileToggle: (fileId: string) => void;
  height?: number;
  itemHeight?: number;
}

function formatDate(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
}

interface RowProps {
  files: FileItem[];
  onFileToggle: (fileId: string) => void;
}

function FileRow({ 
  index, 
  style, 
  files, 
  onFileToggle 
}: { 
  index: number; 
  style: React.CSSProperties; 
  files: FileItem[]; 
  onFileToggle: (fileId: string) => void;
}): JSX.Element | null {
  const file = files[index];

  return (
    <div
      style={style}
      className={`flex items-center gap-3 px-2 py-1 rounded-lg transition-colors ${
        file.selected
          ? 'bg-[var(--color-primary)]/5 border border-[var(--color-primary)]/20'
          : 'hover:bg-[var(--bg-tertiary)]'
      }`}
    >
      <button
        onClick={() => onFileToggle(file.id)}
        className="flex-shrink-0"
      >
        {file.selected ? (
          <CheckSquare className="w-4 h-4 text-[var(--color-primary)]" />
        ) : (
          <Square className="w-4 h-4 text-[var(--text-tertiary)]" />
        )}
      </button>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm text-[var(--text-primary)] truncate">{file.name}</span>
          <span
            className="text-xs px-1.5 py-0.5 rounded"
            style={{ backgroundColor: file.appBgColor, color: file.appColor }}
          >
            {file.appName}
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
    </div>
  );
}

const MemoizedFileRow = memo(FileRow);

export function VirtualFileList({
  files,
  onFileToggle,
  height = 300,
  itemHeight = 56,
}: VirtualFileListProps) {
  return (
    <List<RowProps>
      style={{ height, width: '100%' }}
      rowCount={files.length}
      rowHeight={itemHeight}
      rowComponent={MemoizedFileRow as (props: { index: number; style: React.CSSProperties } & RowProps) => JSX.Element | null}
      rowProps={{ files, onFileToggle }}
    />
  );
}

export default VirtualFileList;
