import { LayoutDashboard, Search, Trash2, Settings, FileSearch, Wrench, Loader2 } from 'lucide-react';
import { Dock, DockIcon } from '../ui/Dock';
import { useUIStore } from '../../stores';
import type { PageType } from '../../types';

interface NavItem {
  id: PageType;
  icon: React.ReactNode;
  label: string;
}

const navItems: NavItem[] = [
  {
    id: 'system',
    icon: <LayoutDashboard className="w-5 h-5" />,
    label: '系统概览',
  },
  {
    id: 'scan',
    icon: <Search className="w-5 h-5" />,
    label: '磁盘扫描',
  },
  {
    id: 'clean',
    icon: <Trash2 className="w-5 h-5" />,
    label: '专项清理',
  },
  {
    id: 'analyze',
    icon: <FileSearch className="w-5 h-5" />,
    label: '文件分析',
  },
  {
    id: 'tools',
    icon: <Wrench className="w-5 h-5" />,
    label: '其他功能',
  },
  {
    id: 'settings',
    icon: <Settings className="w-5 h-5" />,
    label: '设置',
  },
];

export default function NavigationDock() {
  const currentPage = useUIStore((state) => state.currentPage);
  const isWorking = useUIStore((state) => state.isWorking);
  const { setCurrentPage } = useUIStore((state) => state.actions);

  return (
    <div className="dock-container">
      <Dock className="dock-nav-custom" iconMagnification={56} iconDistance={100}>
        {navItems.map((item) => (
          <DockIcon
            key={item.id}
            className={`dock-icon-custom ${currentPage === item.id ? 'active' : ''} ${isWorking && currentPage !== item.id ? 'disabled' : ''}`}
            onClick={() => !isWorking && setCurrentPage(item.id)}
            tooltip={isWorking && currentPage !== item.id ? '功能进行中，请稍候...' : item.label}
          >
            {isWorking && currentPage !== item.id ? (
              <Loader2 className="w-5 h-5 animate-spin opacity-50" />
            ) : (
              item.icon
            )}
          </DockIcon>
        ))}
      </Dock>
    </div>
  );
}
