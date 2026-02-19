import { FolderTree, FileKey, Database, Settings2, FileText } from 'lucide-react';
import type { ResidueType } from '../../types/softwareResidue';

export const RESIDUE_TYPES: { id: ResidueType; name: string; icon: React.ReactNode; description: string }[] = [
  { id: 'leftover_folder', name: '遗留目录', icon: <FolderTree className="w-4 h-4" />, description: '已卸载软件的残留目录' },
  { id: 'registry_key', name: '注册表项', icon: <FileKey className="w-4 h-4" />, description: '已卸载软件的注册表残留项' },
  { id: 'cache_file', name: '缓存文件', icon: <Database className="w-4 h-4" />, description: '已卸载软件的缓存文件' },
  { id: 'config_file', name: '配置文件', icon: <Settings2 className="w-4 h-4" />, description: '已卸载软件的配置文件' },
];

export function getResidueTypeIcon(type: ResidueType): React.ReactNode {
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

export function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

export function formatDate(timestamp: number): string {
  if (!timestamp) return '-';
  const date = new Date(timestamp * 1000);
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
}

export const DRIVER_STATUS_MAP: Record<string, { bg: string; text: string; label: string }> = {
  Running: { bg: 'bg-emerald-500/10', text: 'text-emerald-500', label: '运行中' },
  Stopped: { bg: 'bg-amber-500/10', text: 'text-amber-500', label: '已停止' },
  Error: { bg: 'bg-red-500/10', text: 'text-red-500', label: '异常' },
};
