import { HardDrive, ChevronDown } from 'lucide-react';
import { useSystemStore } from '../../stores/systemStore';
import { useScanStore } from '../../stores/scanStore';
import { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

interface DiskSelectorProps {
  disabled?: boolean;
}

export default function DiskSelector({ disabled = false }: DiskSelectorProps) {
  const { diskList } = useSystemStore();
  const { selectedDisk, actions } = useScanStore();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // 点击外部关闭下拉框
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const formatDiskSize = (bytes: number): string => {
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1000) {
      return `${(gb / 1024).toFixed(1)} TB`;
    }
    return `${gb.toFixed(0)} GB`;
  };

  const getDiskLabel = (disk: { mount_point: string; total_size: number }) => {
    return `${disk.mount_point} (${formatDiskSize(disk.total_size)})`;
  };

  const selectedDiskInfo = diskList.find(d => d.mount_point === selectedDisk);

  return (
    <div className="relative" ref={dropdownRef}>
      <label className="block text-sm font-medium text-[var(--text-secondary)] mb-2">
        选择扫描磁盘
      </label>
      <button
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled}
        className={`w-full flex items-center justify-between px-4 py-3 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-color)] transition-all duration-200 ${
          disabled 
            ? 'opacity-50 cursor-not-allowed' 
            : 'hover:border-[var(--color-primary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]/20'
        }`}
      >
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center">
            <HardDrive className="w-4 h-4 text-[var(--color-primary)]" />
          </div>
          <div className="text-left">
            <p className="text-sm font-medium text-[var(--text-primary)]">
              {selectedDiskInfo ? getDiskLabel(selectedDiskInfo) : selectedDisk}
            </p>
            {selectedDiskInfo && (
              <p className="text-xs text-[var(--text-tertiary)]">
                可用 {formatDiskSize(selectedDiskInfo.free_size)}
              </p>
            )}
          </div>
        </div>
        <ChevronDown 
          className={`w-5 h-5 text-[var(--text-tertiary)] transition-transform duration-200 ${
            isOpen ? 'rotate-180' : ''
          }`} 
        />
      </button>

      <AnimatePresence>
        {isOpen && !disabled && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.15 }}
            className="absolute z-50 w-full mt-2 py-1 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] shadow-lg shadow-black/20"
          >
            {diskList.length === 0 ? (
              <div className="px-4 py-3 text-center text-sm text-[var(--text-tertiary)]">
                未检测到磁盘
              </div>
            ) : (
              diskList.map((disk) => (
                <button
                  key={disk.mount_point}
                  onClick={() => {
                    actions.setSelectedDisk(disk.mount_point);
                    setIsOpen(false);
                  }}
                  className={`w-full flex items-center gap-3 px-4 py-2.5 transition-colors duration-150 ${
                    disk.mount_point === selectedDisk
                      ? 'bg-[var(--color-primary)]/10 text-[var(--color-primary)]'
                      : 'text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
                  }`}
                >
                  <div className="w-8 h-8 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center">
                    <HardDrive className="w-4 h-4 text-[var(--color-primary)]" />
                  </div>
                  <div className="text-left flex-1">
                    <p className="text-sm font-medium">{getDiskLabel(disk)}</p>
                    <p className="text-xs text-[var(--text-tertiary)]">
                      可用 {formatDiskSize(disk.free_size)}
                    </p>
                  </div>
                  {disk.mount_point === selectedDisk && (
                    <div className="w-2 h-2 rounded-full bg-[var(--color-primary)]" />
                  )}
                </button>
              ))
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
