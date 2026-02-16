import { motion, AnimatePresence } from 'framer-motion';
import { AlertCircle, CheckCircle, XCircle, Loader2 } from 'lucide-react';
import { Modal } from '../common';
import type { CleanResult } from '../../types';
import { formatBytes } from '../../utils/format';

type DeleteStep = 'confirm' | 'deleting' | 'result';

interface DeleteConfirmModalProps {
  visible: boolean;
  step: DeleteStep;
  selectedCount: number;
  selectedSize: number;
  progress?: { current: number; total: number; percent: number };
  result: CleanResult | null;
  onConfirm: () => void;
  onClose: () => void;
}

export default function DeleteConfirmModal({
  visible,
  step,
  selectedCount,
  selectedSize,
  progress,
  result,
  onConfirm,
  onClose,
}: DeleteConfirmModalProps) {
  return (
    <Modal
      visible={visible}
      onClose={onClose}
      title={step === 'confirm' ? '确认删除' : step === 'deleting' ? '正在删除' : '删除结果'}
      size={{ maxWidth: '400px' }}
      closeOnOutsideClick={step !== 'deleting'}
    >
      <AnimatePresence mode="wait">
        {step === 'confirm' && (
          <motion.div
            key="confirm"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
          >
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-full bg-red-500/10 flex items-center justify-center">
                <AlertCircle className="w-5 h-5 text-red-500" />
              </div>
              <div>
                <p className="text-sm text-[var(--text-secondary)]">此操作将移动文件到回收站</p>
              </div>
            </div>

            <div className="p-4 rounded-lg bg-[var(--bg-tertiary)] mb-4">
              <div className="flex justify-between text-sm mb-2">
                <span className="text-[var(--text-secondary)]">选中文件</span>
                <span className="text-[var(--text-primary)] font-medium">{selectedCount} 个</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-[var(--text-secondary)]">释放空间</span>
                <span className="text-[var(--color-primary)] font-medium">{formatBytes(selectedSize)}</span>
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={onClose}
                className="flex-1 px-4 py-2 rounded-lg border border-[var(--border-color)] text-[var(--text-secondary)] text-sm font-medium hover:bg-[var(--bg-secondary)] transition-colors"
              >
                取消
              </button>
              <button
                onClick={onConfirm}
                className="flex-1 px-4 py-2 rounded-lg bg-red-500 text-white text-sm font-medium hover:bg-red-600 transition-colors"
              >
                确认删除
              </button>
            </div>
          </motion.div>
        )}

        {step === 'deleting' && (
          <motion.div
            key="deleting"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="text-center py-4"
          >
            <div className="w-12 h-12 rounded-full bg-[var(--color-primary)]/10 flex items-center justify-center mx-auto mb-4">
              <Loader2 className="w-6 h-6 text-[var(--color-primary)] animate-spin" />
            </div>
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-4">正在删除文件</h3>
            {progress && (
              <>
                <div className="w-full h-2 bg-[var(--bg-tertiary)] rounded-full overflow-hidden mb-2">
                  <motion.div
                    className="h-full bg-[var(--color-primary)]"
                    initial={{ width: 0 }}
                    animate={{ width: `${progress.percent}%` }}
                  />
                </div>
                <p className="text-sm text-[var(--text-secondary)]">
                  {progress.current} / {progress.total} 个文件
                </p>
              </>
            )}
          </motion.div>
        )}

        {step === 'result' && result && (
          <motion.div
            key="result"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
          >
            <div className="text-center mb-4">
              <div className={`w-12 h-12 rounded-full mx-auto mb-4 flex items-center justify-center ${
                result.failed_files > 0 ? 'bg-yellow-500/10' : 'bg-green-500/10'
              }`}>
                {result.failed_files > 0 ? (
                  <AlertCircle className="w-6 h-6 text-yellow-500" />
                ) : (
                  <CheckCircle className="w-6 h-6 text-green-500" />
                )}
              </div>
              <h3 className="text-lg font-semibold text-[var(--text-primary)]">
                {result.failed_files > 0 ? '删除完成（部分失败）' : '删除完成'}
              </h3>
            </div>

            <div className="space-y-2 mb-4">
              <div className="flex justify-between text-sm p-3 rounded-lg bg-[var(--bg-tertiary)]">
                <span className="text-[var(--text-secondary)]">处理文件</span>
                <span className="text-[var(--text-primary)] font-medium">{result.total_files} 个</span>
              </div>
              <div className="flex justify-between text-sm p-3 rounded-lg bg-green-500/5">
                <span className="text-[var(--text-secondary)]">成功删除</span>
                <span className="text-green-500 font-medium">{result.cleaned_files} 个</span>
              </div>
              {result.failed_files > 0 && (
                <div className="flex justify-between text-sm p-3 rounded-lg bg-red-500/5">
                  <span className="text-[var(--text-secondary)]">删除失败</span>
                  <span className="text-red-500 font-medium">{result.failed_files} 个</span>
                </div>
              )}
              <div className="flex justify-between text-sm p-3 rounded-lg bg-[var(--color-primary)]/5">
                <span className="text-[var(--text-secondary)]">释放空间</span>
                <span className="text-[var(--color-primary)] font-medium">{formatBytes(result.cleaned_size)}</span>
              </div>
              <div className="flex justify-between text-sm p-3 rounded-lg bg-[var(--bg-tertiary)]">
                <span className="text-[var(--text-secondary)]">耗时</span>
                <span className="text-[var(--text-primary)] font-medium">{(result.duration_ms / 1000).toFixed(2)} 秒</span>
              </div>
            </div>

            {result.errors.length > 0 && (
              <div className="mb-4 max-h-32 overflow-y-auto">
                <p className="text-xs text-[var(--text-tertiary)] mb-2">失败详情：</p>
                {result.errors.slice(0, 5).map((err, idx) => (
                  <div key={idx} className="flex items-start gap-2 text-xs text-red-500 mb-1">
                    <XCircle className="w-3 h-3 flex-shrink-0 mt-0.5" />
                    <span className="truncate">{err.error_message}</span>
                  </div>
                ))}
                {result.errors.length > 5 && (
                  <p className="text-xs text-[var(--text-tertiary)]">...还有 {result.errors.length - 5} 个错误</p>
                )}
              </div>
            )}

            <button
              onClick={onClose}
              className="w-full px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:opacity-90 transition-opacity"
            >
              完成
            </button>
          </motion.div>
        )}
      </AnimatePresence>
    </Modal>
  );
}
