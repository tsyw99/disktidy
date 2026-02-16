import { ReactNode, useEffect, useCallback, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X } from 'lucide-react';

type AnimationType = 'fade' | 'scale' | 'slideUp' | 'slideDown' | 'slideLeft' | 'slideRight' | 'zoom';

interface ModalButton {
  text: string;
  onClick: () => void;
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  disabled?: boolean;
  loading?: boolean;
  icon?: ReactNode;
}

interface ModalSize {
  width?: string | number;
  height?: string | number;
  maxWidth?: string | number;
  maxHeight?: string | number;
  minWidth?: string | number;
  minHeight?: string | number;
}

interface ModalProps {
  visible: boolean;
  onClose: () => void;
  title?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  buttons?: ModalButton[];
  size?: ModalSize;
  overlay?: {
    opacity?: number;
    closeOnClick?: boolean;
    blur?: boolean;
    color?: string;
  };
  animation?: {
    type?: AnimationType;
    duration?: number;
    delay?: number;
  };
  closeOnEscape?: boolean;
  showCloseButton?: boolean;
  closeOnOutsideClick?: boolean;
  className?: string;
  overlayClassName?: string;
  contentClassName?: string;
  headerClassName?: string;
  bodyClassName?: string;
  footerClassName?: string;
  preventScroll?: boolean;
  zIndex?: number;
  mounted?: boolean;
  onMounted?: () => void;
  onUnmounted?: () => void;
}

const animationVariants = {
  fade: {
    initial: { opacity: 0 },
    animate: { opacity: 1 },
    exit: { opacity: 0 },
  },
  scale: {
    initial: { opacity: 0, scale: 0.9 },
    animate: { opacity: 1, scale: 1 },
    exit: { opacity: 0, scale: 0.9 },
  },
  slideUp: {
    initial: { opacity: 0, y: 20 },
    animate: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: 20 },
  },
  slideDown: {
    initial: { opacity: 0, y: -20 },
    animate: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: -20 },
  },
  slideLeft: {
    initial: { opacity: 0, x: 20 },
    animate: { opacity: 1, x: 0 },
    exit: { opacity: 0, x: 20 },
  },
  slideRight: {
    initial: { opacity: 0, x: -20 },
    animate: { opacity: 1, x: 0 },
    exit: { opacity: 0, x: -20 },
  },
  zoom: {
    initial: { opacity: 0, scale: 0.5 },
    animate: { opacity: 1, scale: 1 },
    exit: { opacity: 0, scale: 0.5 },
  },
};

const buttonVariants = {
  primary: 'bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white hover:shadow-lg hover:shadow-blue-500/25',
  secondary: 'bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-primary)] hover:border-[var(--color-primary)]/50',
  danger: 'bg-gradient-to-r from-[#ef4444] to-[#f97316] text-white hover:shadow-lg hover:shadow-red-500/25',
  ghost: 'bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] hover:text-[var(--text-primary)]',
};

function parseSize(value: string | number | undefined): string | undefined {
  if (value === undefined) return undefined;
  if (typeof value === 'number') return `${value}px`;
  return value;
}

export default function Modal({
  visible,
  onClose,
  title,
  children,
  footer,
  buttons,
  size = {},
  overlay = {},
  animation = {},
  closeOnEscape = true,
  showCloseButton = true,
  closeOnOutsideClick,
  className = '',
  overlayClassName = '',
  contentClassName = '',
  headerClassName = '',
  bodyClassName = '',
  footerClassName = '',
  preventScroll = true,
  zIndex = 1000,
  mounted,
  onMounted,
  onUnmounted,
}: ModalProps) {
  const {
    opacity: overlayOpacity = 0.5,
    closeOnClick: overlayCloseOnClick = false,
    blur: overlayBlur = false,
    color: overlayColor = 'var(--bg-primary)',
  } = overlay;

  const {
    type: animationType = 'scale',
    duration: animationDuration = 0.2,
    delay: animationDelay = 0,
  } = animation;

  const {
    width = 'auto',
    height = 'auto',
    maxWidth = '90vw',
    maxHeight = '90vh',
    minWidth,
    minHeight,
  } = size;

  const modalRef = useRef<HTMLDivElement>(null);
  const previousActiveElement = useRef<HTMLElement | null>(null);

  const handleClose = useCallback(() => {
    onClose();
  }, [onClose]);

  const handleOverlayClick = useCallback(
    (e: React.MouseEvent) => {
      if (e.target === e.currentTarget) {
        const shouldClose = closeOnOutsideClick !== undefined ? closeOnOutsideClick : overlayCloseOnClick;
        if (shouldClose) {
          handleClose();
        }
      }
    },
    [closeOnOutsideClick, overlayCloseOnClick, handleClose]
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === 'Escape' && closeOnEscape) {
        handleClose();
      }
    },
    [closeOnEscape, handleClose]
  );

  useEffect(() => {
    if (visible) {
      previousActiveElement.current = document.activeElement as HTMLElement;
      document.addEventListener('keydown', handleKeyDown);
      
      if (preventScroll) {
        document.body.style.overflow = 'hidden';
      }

      if (onMounted) {
        onMounted();
      }

      return () => {
        document.removeEventListener('keydown', handleKeyDown);
        
        if (preventScroll) {
          document.body.style.overflow = '';
        }

        if (previousActiveElement.current) {
          previousActiveElement.current.focus();
        }

        if (onUnmounted) {
          onUnmounted();
        }
      };
    }
  }, [visible, handleKeyDown, preventScroll, onMounted, onUnmounted]);

  useEffect(() => {
    if (visible && modalRef.current) {
      const focusableElements = modalRef.current.querySelectorAll(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      if (focusableElements.length > 0) {
        (focusableElements[0] as HTMLElement).focus();
      }
    }
  }, [visible]);

  const modalStyle: React.CSSProperties = {
    width: parseSize(width),
    height: parseSize(height),
    maxWidth: parseSize(maxWidth),
    maxHeight: parseSize(maxHeight),
    minWidth: parseSize(minWidth),
    minHeight: parseSize(minHeight),
  };

  const overlayStyle: React.CSSProperties = {
    backgroundColor: overlayColor,
    opacity: overlayOpacity,
    backdropFilter: overlayBlur ? 'blur(4px)' : undefined,
    WebkitBackdropFilter: overlayBlur ? 'blur(4px)' : undefined,
  };

  const selectedAnimation = animationVariants[animationType];

  const renderFooter = () => {
    if (footer) return footer;
    
    if (buttons && buttons.length > 0) {
      return (
        <div className="flex items-center justify-end gap-3">
          {buttons.map((button, index) => (
            <motion.button
              key={index}
              whileHover={{ scale: button.disabled ? 1 : 1.02 }}
              whileTap={{ scale: button.disabled ? 1 : 0.98 }}
              onClick={button.onClick}
              disabled={button.disabled || button.loading}
              className={`flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed ${
                buttonVariants[button.variant || 'secondary']
              }`}
            >
              {button.loading ? (
                <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
              ) : button.icon ? (
                button.icon
              ) : null}
              {button.text}
            </motion.button>
          ))}
        </div>
      );
    }
    
    return null;
  };

  if (mounted !== undefined && !mounted) {
    return null;
  }

  return (
    <AnimatePresence>
      {visible && (
        <div
          className={`fixed inset-0 flex items-center justify-center ${className}`}
          style={{ zIndex }}
        >
          <motion.div
            className={`absolute inset-0 ${overlayClassName}`}
            style={overlayStyle}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: animationDuration, delay: animationDelay }}
            onClick={handleOverlayClick}
          />

          <motion.div
            ref={modalRef}
            className={`relative bg-[var(--bg-primary)] rounded-xl shadow-2xl border border-[var(--border-color)] flex flex-col overflow-hidden ${contentClassName}`}
            style={modalStyle}
            initial={selectedAnimation.initial}
            animate={selectedAnimation.animate}
            exit={selectedAnimation.exit}
            transition={{
              duration: animationDuration,
              delay: animationDelay,
              type: 'spring',
              stiffness: 300,
              damping: 25,
            }}
            role="dialog"
            aria-modal="true"
            aria-labelledby={title ? 'modal-title' : undefined}
          >
            {(title || showCloseButton) && (
              <div
                className={`flex items-center justify-between px-6 py-4 border-b border-[var(--border-color)] ${headerClassName}`}
              >
                <div id="modal-title" className="text-lg font-semibold text-[var(--text-primary)]">
                  {title}
                </div>
                {showCloseButton && (
                  <motion.button
                    whileHover={{ scale: 1.1 }}
                    whileTap={{ scale: 0.9 }}
                    onClick={handleClose}
                    className="p-1.5 rounded-lg text-[var(--text-tertiary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
                    aria-label="关闭"
                  >
                    <X className="w-5 h-5" />
                  </motion.button>
                )}
              </div>
            )}

            <div className={`flex-1 overflow-auto p-6 ${bodyClassName}`}>{children}</div>

            {(footer || (buttons && buttons.length > 0)) && (
              <div
                className={`px-6 py-4 border-t border-[var(--border-color)] bg-[var(--bg-secondary)] ${footerClassName}`}
              >
                {renderFooter()}
              </div>
            )}
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}

export type { ModalProps, ModalButton, ModalSize, AnimationType };
