import { ReactNode, useEffect } from 'react';
import NavigationDock from '../common/NavigationDock';
import { useUIStore } from '../../stores';

interface MainLayoutProps {
  children: ReactNode;
}

export default function MainLayout({ children }: MainLayoutProps) {
  const theme = useUIStore((state) => state.theme);

  // Apply theme class to document
  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [theme]);

  return (
    <div className="flex-1 flex flex-col bg-[var(--bg-primary)] overflow-hidden transition-colors duration-300">
      {/* Main Content */}
      <main className="flex-1 overflow-hidden relative z-10 pb-24">
        {children}
      </main>

      {/* Bottom Dock Navigation */}
      <NavigationDock />
    </div>
  );
}
