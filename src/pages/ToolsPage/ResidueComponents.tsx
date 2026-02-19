import { motion, AnimatePresence } from 'framer-motion';
import { CheckSquare, Square, ExternalLink } from 'lucide-react';
import type { ResidueScanResult, ResidueItem } from '../../types/softwareResidue';
import { RESIDUE_TYPE_NAMES } from '../../types/softwareResidue';
import { getResidueTypeIcon, formatSize, formatDate } from './constants';
import { openFileLocation } from '../../utils/shell';

function ChevronRight({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
      <path d="M9 18l6-6-6-6" />
    </svg>
  );
}

export function ResidueCategoryPanel({
  result,
  selectedIds,
  onToggle,
  isExpanded,
  onToggleExpand,
}: {
  result: ResidueScanResult;
  selectedIds: Set<string>;
  onToggle: (id: string) => void;
  isExpanded: boolean;
  onToggleExpand: () => void;
}) {
  const allSelected = result.items.every(item => selectedIds.has(item.id));
  const someSelected = result.items.some(item => selectedIds.has(item.id));

  const handleSelectAll = () => {
    result.items.forEach(item => {
      if (allSelected) {
        if (selectedIds.has(item.id)) onToggle(item.id);
      } else {
        if (!selectedIds.has(item.id)) onToggle(item.id);
      }
    });
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className="rounded-xl border border-[var(--border-color)] overflow-hidden bg-[var(--bg-secondary)]/50"
    >
      <div
        className="flex items-center gap-3 p-4 bg-[var(--bg-secondary)] cursor-pointer hover:bg-[var(--bg-tertiary)] transition-colors"
        onClick={onToggleExpand}
      >
        <motion.div
          animate={{ rotate: isExpanded ? 90 : 0 }}
          transition={{ duration: 0.2 }}
        >
          <ChevronRight className="w-4 h-4 text-[var(--text-tertiary)]" />
        </motion.div>

        <button onClick={(e) => { e.stopPropagation(); handleSelectAll(); }}>
          {allSelected ? (
            <CheckSquare className="w-5 h-5 text-[var(--color-primary)]" />
          ) : someSelected ? (
            <div className="w-5 h-5 rounded border-2 border-[var(--color-primary)] bg-[var(--color-primary)]/30 flex items-center justify-center">
              <div className="w-2.5 h-0.5 bg-[var(--color-primary)] rounded" />
            </div>
          ) : (
            <Square className="w-5 h-5 text-[var(--text-secondary)]" />
          )}
        </button>

        <div className="flex-shrink-0 w-10 h-10 rounded-lg bg-gradient-to-br from-blue-500/20 to-purple-500/20 flex items-center justify-center">
          {getResidueTypeIcon(result.residue_type)}
        </div>

        <div className="flex-1">
          <h4 className="text-sm font-medium text-[var(--text-primary)]">
            {RESIDUE_TYPE_NAMES[result.residue_type]}
          </h4>
          <p className="text-xs text-[var(--text-secondary)]">
            {result.items.length} 项 {result.total_size > 0 && `· ${formatSize(result.total_size)}`}
          </p>
        </div>

        <span className="text-sm font-semibold text-[#10b981]">
          {formatSize(result.total_size)}
        </span>
      </div>

      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ height: 0 }}
            animate={{ height: 'auto' }}
            exit={{ height: 0 }}
            className="overflow-hidden"
          >
            <div className="px-4 pb-4 space-y-2 max-h-[300px] overflow-y-auto bg-[var(--bg-primary)]/50">
              {result.items.map((item) => (
                <ResidueItemCard
                  key={item.id}
                  item={item}
                  selected={selectedIds.has(item.id)}
                  onToggle={() => onToggle(item.id)}
                />
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </motion.div>
  );
}

function ResidueItemCard({
  item,
  selected,
  onToggle,
}: {
  item: ResidueItem;
  selected: boolean;
  onToggle: () => void;
}) {
  return (
    <motion.div
      layout
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 10 }}
      className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-all duration-200 ${
        selected ? 'bg-[var(--color-primary)]/10 border border-[var(--color-primary)]/30' : 'hover:bg-[var(--bg-tertiary)]'
      }`}
      onClick={onToggle}
    >
      <button
        className="flex-shrink-0"
        onClick={(e) => {
          e.stopPropagation();
          onToggle();
        }}
      >
        {selected ? (
          <CheckSquare className="w-4 h-4 text-[var(--color-primary)]" />
        ) : (
          <Square className="w-4 h-4 text-[var(--text-secondary)]" />
        )}
      </button>

      {getResidueTypeIcon(item.residue_type)}

      <div className="flex-1 min-w-0">
        <p className="text-sm text-[var(--text-primary)] truncate" title={item.name}>
          {item.name}
        </p>
        <p className="text-xs text-[var(--text-tertiary)] truncate" title={item.path}>
          {item.path}
        </p>
      </div>

      <div className="flex items-center gap-3 text-xs text-[var(--text-secondary)]">
        {item.size > 0 && <span>{formatSize(item.size)}</span>}
        {item.last_modified > 0 && <span>{formatDate(item.last_modified)}</span>}
        {item.residue_type !== 'registry_key' && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              openFileLocation(item.path);
            }}
            className="p-1 rounded hover:bg-[var(--bg-tertiary)] hover:text-[var(--color-primary)] transition-colors"
          >
            <ExternalLink className="w-3 h-3" />
          </button>
        )}
      </div>
    </motion.div>
  );
}
