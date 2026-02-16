import { motion } from 'framer-motion';
import { AlertTriangle, Trash2, CheckCircle2, XCircle, Loader2 } from 'lucide-react';
import Modal from '../common/Modal';
import { ProgressBar } from '../common';
import { formatBytes } from '../../utils/format';
import type { CleanResult } from '../../types';

type DeleteStep = 'confirm' | 'deleting' | 'result';

interface DeleteConfirmModalProps {
  visible: boolean;
  step: DeleteStep;
  selectedCount: number;
  selectedSize: number;
  progress?: {
    current: number;
    total: number;
    percent: number;
  };
  result?: CleanResult | null;
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
  const renderContent = () => {
    switch (step) {
      case 'confirm':
        return (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="text-center"
          >
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-red-500/10 flex items-center justify-center">
              <AlertTriangle className="w-8 h-8 text-red-500" />
            </div>
            
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
              确认删除文件
            </h3>
            
            <p className="text-sm text-[var(--text-secondary)] mb-4">
              您确定要删除选中的文件吗？此操作将把文件移动到回收站。
            </p>
            
            <div className="flex items-center justify-center gap-6 p-4 rounded-lg bg-[var(--bg-secondary)] mb-6">
              <div className="text-center">
                <p className="text-2xl font-semibold text-[var(--text-primary)]">
                  {selectedCount}
                </p>
                <p className="text-xs text-[var(--text-tertiary)]">个文件</p>
              </div>
              <div className="w-px h-10 bg-[var(--border-color)]" />
              <div className="text-center">
                <p className="text-2xl font-semibold text-[#10b981]">
                  {formatBytes(selectedSize)}
                </p>
                <p className="text-xs text-[var(--text-tertiary)]">可释放空间</p>
              </div>
            </div>
          </motion.div>
        );
        
      case 'deleting':
        return (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="text-center"
          >
            <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-[var(--color-primary)]/10 flex items-center justify-center">
              <Loader2 className="w-8 h-8 text-[var(--color-primary)] animate-spin" />
            </div>
            
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
              正在删除文件...
            </h3>
            
            <p className="text-sm text-[var(--text-secondary)] mb-4">
              请稍候，正在将选中的文件移动到回收站
            </p>
            
            <div className="mb-4">
              <ProgressBar
                percent={progress?.percent ?? 0}
                height={8}
                showShimmer
                gradient={{ from: '#3b82f6', to: '#10b981' }}
              />
            </div>
            
            <p className="text-sm text-[var(--text-tertiary)]">
              {progress?.current ?? 0} / {progress?.total ?? selectedCount} 个文件
            </p>
          </motion.div>
        );
        
      case 'result':
        const success = result && result.cleaned_files > 0;
        const hasFailures = result && result.failed_files > 0;
        
        return (
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="text-center"
          >
            <div className={`w-16 h-16 mx-auto mb-4 rounded-full flex items-center justify-center ${
              success ? 'bg-[#10b981]/10' : 'bg-red-500/10'
            }`}>
              {success ? (
                <CheckCircle2 className="w-8 h-8 text-[#10b981]" />
              ) : (
                <XCircle className="w-8 h-8 text-red-500" />
              )}
            </div>
            
            <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
              {success ? '删除完成' : '删除失败'}
            </h3>
            
            {result && (
              <>
                <p className="text-sm text-[var(--text-secondary)] mb-4">
                  {success ? (
                    <>
                      成功删除 <span className="font-medium text-[var(--text-primary)]">{result.cleaned_files}</span> 个文件，
                      释放空间 <span className="font-medium text-[#10b981]">{formatBytes(result.cleaned_size)}</span>
                      {hasFailures && (
                        <span className="text-red-400">，{result.failed_files} 个文件删除失败</span>
                      )}
                    </>
                  ) : (
                    '删除过程中出现错误，请重试或检查文件是否被占用'
                  )}
                </p>
                
                <div className="grid grid-cols-3 gap-4 p-4 rounded-lg bg-[var(--bg-secondary)] mb-4">
                  <div className="text-center">
                    <p className="text-xl font-semibold text-[var(--text-primary)]">
                      {result.cleaned_files}
                    </p>
                    <p className="text-xs text-[var(--text-tertiary)]">成功</p>
                  </div>
                  <div className="text-center">
                    <p className="text-xl font-semibold text-red-500">
                      {result.failed_files}
                    </p>
                    <p className="text-xs text-[var(--text-tertiary)]">失败</p>
                  </div>
                  <div className="text-center">
                    <p className="text-xl font-semibold text-[#10b981]">
                      {formatBytes(result.cleaned_size)}
                    </p>
                    <p className="text-xs text-[var(--text-tertiary)]">释放空间</p>
                  </div>
                </div>
                
                {hasFailures && result.errors.length > 0 && (
                  <div className="text-left p-3 rounded-lg bg-red-500/10 border border-red-500/20 mb-4 max-h-32 overflow-y-auto">
                    <p className="text-xs font-medium text-red-400 mb-2">错误详情：</p>
                    {result.errors.slice(0, 5).map((error, index) => (
                      <p key={index} className="text-xs text-[var(--text-secondary)] truncate" title={error.path}>
                        {error.path.split('\\').pop()}: {error.error_message}
                      </p>
                    ))}
                    {result.errors.length > 5 && (
                      <p className="text-xs text-[var(--text-tertiary)] mt-1">
                        还有 {result.errors.length - 5} 个错误...
                      </p>
                    )}
                  </div>
                )}
              </>
            )}
          </motion.div>
        );
    }
  };

  const getButtons = () => {
    switch (step) {
      case 'confirm':
        return [
          {
            text: '取消',
            onClick: onClose,
            variant: 'secondary' as const,
          },
          {
            text: '确认删除',
            onClick: onConfirm,
            variant: 'danger' as const,
            icon: <Trash2 className="w-4 h-4" />,
          },
        ];
        
      case 'deleting':
        return [];
        
      case 'result':
        return [
          {
            text: '完成',
            onClick: onClose,
            variant: 'primary' as const,
          },
        ];
        
      default:
        return [];
    }
  };

  return (
    <Modal
      visible={visible}
      onClose={step !== 'deleting' ? onClose : () => {}}
      title={step === 'confirm' ? '删除确认' : step === 'deleting' ? '删除中' : '删除结果'}
      buttons={getButtons()}
      size={{ width: 420 }}
      closeOnEscape={step !== 'deleting'}
      showCloseButton={step !== 'deleting'}
      closeOnOutsideClick={false}
    >
      {renderContent()}
    </Modal>
  );
}
