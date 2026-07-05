import { useState, useRef, useEffect, useCallback } from 'react';
import { ChevronDown, AlertCircle, Check } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

interface SelectProps {
  id?: string;
  label?: string;
  value: string;
  options: SelectOption[];
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  error?: string;
  helperText?: string;
  icon?: React.ReactNode;
  className?: string;
}

export default function Select({
  id,
  label,
  value,
  options,
  onChange,
  placeholder,
  disabled = false,
  error,
  helperText,
  icon,
  className = '',
}: SelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const optionRefs = useRef<(HTMLDivElement | null)[]>([]);
  const selectId = id || `select-${Math.random().toString(36).slice(2, 9)}`;

  const selectedOption = options.find((opt) => opt.value === value);
  const enabledOptions = options.filter((opt) => !opt.disabled);

  const handleToggle = () => {
    if (!disabled) {
      setIsOpen((prev) => !prev);
    }
  };

  const handleSelect = (optionValue: string, optionDisabled?: boolean) => {
    if (optionDisabled) return;
    onChange(optionValue);
    setIsOpen(false);
    triggerRef.current?.focus();
  };

  const handleClickOutside = useCallback((event: MouseEvent) => {
    if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
      setIsOpen(false);
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      const selectedIndex = enabledOptions.findIndex((opt) => opt.value === value);
      setHighlightedIndex(selectedIndex >= 0 ? selectedIndex : 0);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
      setHighlightedIndex(-1);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen, handleClickOutside, value, enabledOptions]);

  useEffect(() => {
    if (isOpen && highlightedIndex >= 0) {
      const highlightedOption = enabledOptions[highlightedIndex];
      const renderIndex = options.findIndex((opt) => opt.value === highlightedOption?.value);
      const optionElement = optionRefs.current[renderIndex];
      optionElement?.scrollIntoView({ block: 'nearest' });
    }
  }, [highlightedIndex, isOpen, enabledOptions, options]);

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (disabled) return;

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        if (!isOpen) {
          setIsOpen(true);
        } else {
          setHighlightedIndex((prev) =>
            prev < enabledOptions.length - 1 ? prev + 1 : prev
          );
        }
        break;
      case 'ArrowUp':
        event.preventDefault();
        if (!isOpen) {
          setIsOpen(true);
        } else {
          setHighlightedIndex((prev) => (prev > 0 ? prev - 1 : prev));
        }
        break;
      case 'Enter':
      case ' ': // Space
        event.preventDefault();
        if (isOpen && highlightedIndex >= 0) {
          handleSelect(enabledOptions[highlightedIndex].value);
        } else {
          setIsOpen(true);
        }
        break;
      case 'Escape':
        event.preventDefault();
        setIsOpen(false);
        break;
      case 'Tab':
        setIsOpen(false);
        break;
      default:
        break;
    }
  };

  return (
    <div className={`space-y-1.5 ${className}`} ref={containerRef}>
      {label && (
        <label
          htmlFor={selectId}
          className="text-sm font-medium text-[var(--text-primary)] flex items-center gap-2"
        >
          {icon && <span className="text-[var(--color-primary)]">{icon}</span>}
          {label}
        </label>
      )}
      <div className="relative">
        <button
          id={selectId}
          ref={triggerRef}
          type="button"
          onClick={handleToggle}
          onKeyDown={handleKeyDown}
          disabled={disabled}
          aria-haspopup="listbox"
          aria-expanded={isOpen}
          className={`w-full flex items-center justify-between px-4 py-2.5 rounded-lg bg-[var(--bg-primary)] border text-left transition-all disabled:opacity-60 disabled:cursor-not-allowed ${
            error
              ? 'border-[var(--color-error)] focus:ring-[var(--color-error)]/30 focus:border-[var(--color-error)]'
              : isOpen
              ? 'border-[var(--color-primary)] ring-2 ring-[var(--color-primary)]/30'
              : 'border-[var(--border-color)] hover:border-[var(--color-primary)]/50 focus:ring-2 focus:ring-[var(--color-primary)]/30 focus:border-[var(--color-primary)]'
          }`}
        >
          <span
            className={`block truncate ${
              selectedOption ? 'text-[var(--text-primary)]' : 'text-[var(--text-tertiary)]'
            }`}
          >
            {selectedOption ? selectedOption.label : placeholder || '请选择'}
          </span>
          <motion.div
            animate={{ rotate: isOpen ? 180 : 0 }}
            transition={{ duration: 0.2 }}
            className="ml-2 flex-shrink-0 text-[var(--text-tertiary)]"
          >
            <ChevronDown className="w-4 h-4" />
          </motion.div>
        </button>

        <AnimatePresence>
          {isOpen && (
            <motion.div
              initial={{ opacity: 0, y: -8, scale: 0.96 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              exit={{ opacity: 0, y: -8, scale: 0.96 }}
              transition={{ duration: 0.15 }}
              className="absolute z-50 left-0 right-0 mt-1.5 py-1.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] shadow-xl shadow-black/10 max-h-60 overflow-auto"
              role="listbox"
            >
              {options.map((option, index) => {
                const isSelected = option.value === value;
                const isHighlighted =
                  enabledOptions.findIndex((opt) => opt.value === option.value) ===
                  highlightedIndex;
                const isDisabled = option.disabled;

                return (
                  <div
                    key={option.value}
                    ref={(el) => {
                      optionRefs.current[index] = el;
                    }}
                    onClick={() => handleSelect(option.value, option.disabled)}
                    onMouseEnter={() => {
                      if (!isDisabled) {
                        const enabledIndex = enabledOptions.findIndex(
                          (opt) => opt.value === option.value
                        );
                        setHighlightedIndex(enabledIndex);
                      }
                    }}
                    role="option"
                    aria-selected={isSelected}
                    className={`flex items-center justify-between px-3 py-2 mx-1.5 rounded-md text-sm cursor-pointer transition-colors ${
                      isDisabled
                        ? 'opacity-40 cursor-not-allowed text-[var(--text-tertiary)]'
                        : isSelected
                        ? 'bg-[var(--color-primary)]/10 text-[var(--color-primary)]'
                        : isHighlighted
                        ? 'bg-[var(--bg-tertiary)] text-[var(--text-primary)]'
                        : 'text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
                    }`}
                  >
                    <span className="truncate">{option.label}</span>
                    {isSelected && !isDisabled && (
                      <Check className="w-4 h-4 flex-shrink-0 ml-2" />
                    )}
                  </div>
                );
              })}
            </motion.div>
          )}
        </AnimatePresence>
      </div>
      {error && (
        <motion.div
          initial={{ opacity: 0, y: -5 }}
          animate={{ opacity: 1, y: 0 }}
          className="flex items-center gap-1.5 text-xs text-[var(--color-error)]"
        >
          <AlertCircle className="w-3.5 h-3.5" />
          {error}
        </motion.div>
      )}
      {helperText && !error && (
        <p className="text-xs text-[var(--text-secondary)]">{helperText}</p>
      )}
    </div>
  );
}
