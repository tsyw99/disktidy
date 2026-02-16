import { useRef, useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { Zap, Search } from 'lucide-react';
import type { ScanMode } from '../../types';
import { useScanStore } from '../../stores/scanStore';

interface ScanModeSelectorProps {
  disabled?: boolean;
}

export default function ScanModeSelector({ disabled = false }: ScanModeSelectorProps) {
  const { scanMode, actions } = useScanStore();
  const quickBtnRef = useRef<HTMLButtonElement>(null);
  const deepBtnRef = useRef<HTMLButtonElement>(null);
  const [indicatorStyle, setIndicatorStyle] = useState({ width: 0, x: 0 });

  useEffect(() => {
    const activeBtn = scanMode === 'quick' ? quickBtnRef.current : deepBtnRef.current;
    if (activeBtn) {
      const container = activeBtn.parentElement;
      if (container) {
        const containerRect = container.getBoundingClientRect();
        const btnRect = activeBtn.getBoundingClientRect();
        setIndicatorStyle({
          width: btnRect.width,
          x: btnRect.left - containerRect.left - 4,
        });
      }
    }
  }, [scanMode]);

  const handleModeChange = (mode: ScanMode) => {
    if (!disabled) {
      actions.setScanMode(mode);
    }
  };

  return (
    <div className="flex bg-[var(--bg-tertiary)] rounded-lg p-1 relative">
      <motion.div
        className="absolute top-1 bottom-1 bg-[var(--color-primary)] rounded-md"
        initial={false}
        animate={{
          width: indicatorStyle.width,
          x: indicatorStyle.x,
        }}
        transition={{
          type: 'spring',
          stiffness: 400,
          damping: 30,
        }}
      />
      <button
        ref={quickBtnRef}
        onClick={() => handleModeChange('quick')}
        disabled={disabled}
        className={`relative z-10 flex items-center gap-1.5 px-4 py-2 rounded-md text-sm font-medium transition-colors duration-200 ${
          scanMode === 'quick'
            ? 'text-white'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
        } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
      >
        <Zap className="w-4 h-4" />
        快速扫描
      </button>
      <button
        ref={deepBtnRef}
        onClick={() => handleModeChange('deep')}
        disabled={disabled}
        className={`relative z-10 flex items-center gap-1.5 px-4 py-2 rounded-md text-sm font-medium transition-colors duration-200 ${
          scanMode === 'deep'
            ? 'text-white'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
        } ${disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
      >
        <Search className="w-4 h-4" />
        深度扫描
      </button>
    </div>
  );
}
