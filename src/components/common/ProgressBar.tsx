import { motion } from 'framer-motion';
import { CSSProperties } from 'react';

export interface ProgressBarProps {
  percent: number;
  height?: number | string;
  width?: number | string;
  color?: string;
  gradient?: {
    from: string;
    to: string;
    direction?: 'left' | 'right' | 'top' | 'bottom';
  };
  showShimmer?: boolean;
  indeterminate?: boolean;
  borderRadius?: number | string;
  className?: string;
  style?: CSSProperties;
  animateDuration?: number;
}

export default function ProgressBar({
  percent,
  height = 12,
  width = '100%',
  color,
  gradient,
  showShimmer = false,
  indeterminate = false,
  borderRadius = 9999,
  className = '',
  style,
  animateDuration = 0.3,
}: ProgressBarProps) {
  const clampedPercent = Math.min(100, Math.max(0, percent));

  const getBackgroundStyle = (): CSSProperties => {
    if (gradient) {
      const direction = gradient.direction || 'right';
      const gradientDirection = {
        left: 'to left',
        right: 'to right',
        top: 'to top',
        bottom: 'to bottom',
      }[direction];

      return {
        background: `linear-gradient(${gradientDirection}, ${gradient.from}, ${gradient.to})`,
      };
    }

    if (color) {
      return {
        backgroundColor: color,
      };
    }

    return {
      background: 'linear-gradient(to right, #3b82f6, #10b981)',
    };
  };

  const heightValue = typeof height === 'number' ? `${height}px` : height;
  const widthValue = typeof width === 'number' ? `${width}px` : width;
  const borderRadiusValue = typeof borderRadius === 'number' ? `${borderRadius}px` : borderRadius;

  if (indeterminate) {
    return (
      <div
        className={`relative overflow-hidden bg-[var(--bg-secondary)] ${className}`}
        style={{
          height: heightValue,
          width: widthValue,
          borderRadius: borderRadiusValue,
          ...style,
        }}
      >
        <motion.div
          className="absolute inset-y-0 w-1/3"
          style={{
            ...getBackgroundStyle(),
            borderRadius: borderRadiusValue,
          }}
          animate={{
            x: ['-100%', '300%'],
          }}
          transition={{
            duration: 1.5,
            repeat: Infinity,
            ease: 'easeInOut',
          }}
        >
          {showShimmer && (
            <motion.div
              className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent"
              animate={{ x: ['-100%', '100%'] }}
              transition={{ duration: 1.5, repeat: Infinity, ease: 'linear' }}
            />
          )}
        </motion.div>
      </div>
    );
  }

  return (
    <div
      className={`relative overflow-hidden bg-[var(--bg-secondary)] ${className}`}
      style={{
        height: heightValue,
        width: widthValue,
        borderRadius: borderRadiusValue,
        ...style,
      }}
    >
      <motion.div
        className="absolute inset-y-0 left-0"
        style={{
          ...getBackgroundStyle(),
          borderRadius: borderRadiusValue,
        }}
        initial={{ width: 0 }}
        animate={{ width: `${clampedPercent}%` }}
        transition={{ duration: animateDuration, ease: 'easeOut' }}
      >
        {showShimmer && (
          <motion.div
            className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent"
            animate={{ x: ['-100%', '100%'] }}
            transition={{ duration: 1.5, repeat: Infinity, ease: 'linear' }}
          />
        )}
      </motion.div>
    </div>
  );
}
