import { motion } from 'framer-motion';
import { AlertTriangle } from 'lucide-react';

interface ConfirmDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  message: string;
  confirmText?: string;
  danger?: boolean;
}

export function ConfirmDialog({
  isOpen,
  onClose,
  onConfirm,
  title,
  message,
  confirmText = '确认',
  danger = false,
}: ConfirmDialogProps) {
  if (!isOpen) return null;

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={onClose}
    >
      <motion.div
        initial={{ scale: 0.95, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        exit={{ scale: 0.95, opacity: 0 }}
        className="w-full max-w-sm mx-4 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="p-6 text-center">
          <div className={`w-12 h-12 mx-auto mb-4 rounded-full flex items-center justify-center ${
            danger ? 'bg-red-500/10' : 'bg-[var(--color-primary)]/10'
          }`}>
            <AlertTriangle className={`w-6 h-6 ${danger ? 'text-red-500' : 'text-[var(--color-primary)]'}`} />
          </div>
          <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">{title}</h3>
          <p className="text-sm text-[var(--text-secondary)]">{message}</p>
        </div>

        <div className="flex gap-2 p-4 border-t border-[var(--border-color)]">
          <button
            onClick={onClose}
            className="flex-1 px-4 py-2 rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
          >
            取消
          </button>
          <button
            onClick={() => {
              onConfirm();
              onClose();
            }}
            className={`flex-1 px-4 py-2 rounded-lg text-white transition-opacity hover:opacity-90 ${
              danger ? 'bg-red-500' : 'bg-gradient-to-r from-[#6366f1] to-[#8b5cf6]'
            }`}
          >
            {confirmText}
          </button>
        </div>
      </motion.div>
    </motion.div>
  );
}
