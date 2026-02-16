import { useRef, useEffect, useState, ReactNode } from 'react';
import { motion } from 'framer-motion';

interface SegmentedControlOption<T extends string> {
  value: T;
  label: string;
  icon?: ReactNode;
  disabled?: boolean;
}

interface SegmentedControlProps<T extends string> {
  options: SegmentedControlOption<T>[];
  value: T;
  onChange: (value: T) => void;
  className?: string;
  color?: string;
}

export default function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
  className = '',
  color = 'linear-gradient(to right, #3b82f6, #10b981)',
}: SegmentedControlProps<T>) {
  const buttonRefs = useRef<(HTMLButtonElement | null)[]>([]);
  const [indicatorStyle, setIndicatorStyle] = useState({ width: 0, x: 0 });

  useEffect(() => {
    const activeIndex = options.findIndex(opt => opt.value === value);
    const activeBtn = buttonRefs.current[activeIndex];
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
  }, [value, options]);

  return (
    <div className={`flex bg-[var(--bg-tertiary)] rounded-lg p-1 relative ${className}`}>
      <motion.div
        className="absolute top-1 bottom-1 rounded-md"
        style={{
          background: color,
        }}
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
      {options.map((option, index) => (
        <button
          key={option.value}
          ref={el => { buttonRefs.current[index] = el; }}
          onClick={() => !option.disabled && onChange(option.value)}
          disabled={option.disabled}
          className={`relative z-10 flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors duration-200 ${
            value === option.value
              ? 'text-white'
              : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
          } ${option.disabled ? 'opacity-50 cursor-not-allowed' : ''}`}
        >
          {option.icon}
          {option.label}
        </button>
      ))}
    </div>
  );
}
