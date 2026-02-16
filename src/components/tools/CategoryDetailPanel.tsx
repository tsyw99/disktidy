import { useMemo } from 'react';
import ReactECharts from 'echarts-for-react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, FileText, Folder, HardDrive, Clock } from 'lucide-react';
import type { FileTypeStats, FileBriefInfo } from '../../types';
import { useUIStore } from '../../stores';
import { formatBytes, formatDate } from '../../utils/format';
import { openFileLocation } from '../../utils/shell';

interface CategoryDetailPanelProps {
  category: FileTypeStats | null;
  largestFiles: FileBriefInfo[];
  onClose: () => void;
}

export default function CategoryDetailPanel({ 
  category, 
  largestFiles,
  onClose 
}: CategoryDetailPanelProps) {
  const theme = useUIStore((state) => state.theme);
  const isDark = theme === 'dark';

  const extensionData = useMemo(() => {
    if (!category) return [];
    return Object.values(category.extensions)
      .sort((a, b) => b.total_size - a.total_size)
      .slice(0, 10);
  }, [category]);

  const extensionChartOption = useMemo(() => {
    if (extensionData.length === 0) return null;

    return {
      tooltip: {
        trigger: 'item',
        backgroundColor: isDark ? 'rgba(26, 26, 46, 0.95)' : 'rgba(255, 255, 255, 0.95)',
        borderColor: isDark ? 'rgba(139, 92, 246, 0.3)' : 'rgba(99, 102, 241, 0.2)',
        borderWidth: 1,
        borderRadius: 8,
        padding: [12, 16],
        textStyle: {
          color: isDark ? '#f3f4f6' : '#1e293b',
          fontSize: 13,
        },
        formatter: (params: any) => {
          return `
            <div>
              <div style="font-weight: 600; margin-bottom: 6px;">.${params.name || '无扩展名'}</div>
              <div style="display: flex; justify-content: space-between; gap: 20px;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">文件数</span>
                <span>${params.data.count.toLocaleString()}</span>
              </div>
              <div style="display: flex; justify-content: space-between; gap: 20px;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">大小</span>
                <span style="font-weight: 500;">${formatBytes(params.value)}</span>
              </div>
            </div>
          `;
        },
      },
      series: [
        {
          type: 'pie',
          radius: ['50%', '70%'],
          center: ['50%', '50%'],
          avoidLabelOverlap: true,
          itemStyle: {
            borderRadius: 4,
            borderColor: isDark ? 'rgba(26, 26, 46, 0.8)' : 'rgba(255, 255, 255, 0.8)',
            borderWidth: 2,
          },
          label: {
            show: true,
            position: 'outside',
            color: isDark ? '#9ca3af' : '#64748b',
            fontSize: 11,
            formatter: (params: any) => {
              const ext = params.name || '无';
              return `.${ext}`;
            },
          },
          labelLine: {
            show: true,
            length: 10,
            length2: 10,
            lineStyle: {
              color: isDark ? '#4b5563' : '#d1d5db',
            },
          },
          data: extensionData.map((ext, index) => ({
            name: ext.extension,
            value: ext.total_size,
            count: ext.count,
            itemStyle: {
              color: [
                '#6366f1', '#8b5cf6', '#a855f7', '#d946ef', '#ec4899',
                '#f43f5e', '#ef4444', '#f97316', '#f59e0b', '#eab308'
              ][index % 10],
            },
          })),
          animationType: 'scale',
          animationEasing: 'elasticOut',
        },
      ],
    };
  }, [extensionData, isDark]);

  const relatedFiles = useMemo(() => {
    if (!category) return [];
    return largestFiles.filter(f => f.category === category.category).slice(0, 5);
  }, [category, largestFiles]);

  return (
    <AnimatePresence>
      {category && (
        <motion.div
          initial={{ opacity: 0, x: 20 }}
          animate={{ opacity: 1, x: 0 }}
          exit={{ opacity: 0, x: 20 }}
          className="panel p-6"
        >
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-[var(--text-primary)] flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-gradient-to-r from-[#6366f1] to-[#8b5cf6]" />
              {category.display_name} 详情
            </h3>
            <motion.button
              whileHover={{ scale: 1.1 }}
              whileTap={{ scale: 0.9 }}
              onClick={onClose}
              className="p-2 rounded-lg hover:bg-[var(--bg-tertiary)] text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors"
            >
              <X className="w-5 h-5" />
            </motion.button>
          </div>

          <div className="grid grid-cols-3 gap-4 mb-6">
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1 }}
              className="p-4 rounded-xl bg-[var(--bg-secondary)] border border-[var(--border-color)]/50"
            >
              <div className="flex items-center gap-2 mb-2">
                <FileText className="w-4 h-4 text-[var(--color-primary)]" />
                <span className="text-xs text-[var(--text-tertiary)]">文件数量</span>
              </div>
              <p className="text-2xl font-bold text-[var(--text-primary)]">
                {category.count.toLocaleString()}
              </p>
            </motion.div>

            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2 }}
              className="p-4 rounded-xl bg-[var(--bg-secondary)] border border-[var(--border-color)]/50"
            >
              <div className="flex items-center gap-2 mb-2">
                <HardDrive className="w-4 h-4 text-[#10b981]" />
                <span className="text-xs text-[var(--text-tertiary)]">占用空间</span>
              </div>
              <p className="text-2xl font-bold text-[#10b981]">
                {formatBytes(category.total_size)}
              </p>
            </motion.div>

            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 }}
              className="p-4 rounded-xl bg-[var(--bg-secondary)] border border-[var(--border-color)]/50"
            >
              <div className="flex items-center gap-2 mb-2">
                <Folder className="w-4 h-4 text-[#f59e0b]" />
                <span className="text-xs text-[var(--text-tertiary)]">占比</span>
              </div>
              <p className="text-2xl font-bold text-[#f59e0b]">
                {category.percentage.toFixed(1)}%
              </p>
            </motion.div>
          </div>

          {extensionChartOption && extensionData.length > 1 && (
            <div className="mb-6">
              <h4 className="text-sm font-medium text-[var(--text-secondary)] mb-3">
                扩展名分布
              </h4>
              <ReactECharts
                option={extensionChartOption}
                style={{ width: '100%', height: 200 }}
                opts={{ renderer: 'svg' }}
              />
            </div>
          )}

          {relatedFiles.length > 0 && (
            <div>
              <h4 className="text-sm font-medium text-[var(--text-secondary)] mb-3 flex items-center gap-2">
                <Clock className="w-4 h-4" />
                大文件列表
              </h4>
              <div className="space-y-2">
                {relatedFiles.map((file, index) => (
                  <motion.div
                    key={file.path}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    transition={{ delay: index * 0.05 }}
                    className="flex items-center gap-3 p-3 rounded-lg bg-[var(--bg-tertiary)] hover:bg-[var(--bg-secondary)] transition-colors group cursor-pointer"
                    onClick={() => openFileLocation(file.path)}
                  >
                    <div className="w-8 h-8 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]">
                      <FileText className="w-4 h-4" />
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm text-[var(--text-primary)] truncate" title={file.name}>
                        {file.name}
                      </p>
                      <p className="text-xs text-[var(--text-tertiary)] truncate" title={file.path}>
                        {file.path}
                      </p>
                    </div>
                    <div className="text-right flex-shrink-0">
                      <p className="text-sm font-medium text-[var(--text-primary)]">
                        {formatBytes(file.size)}
                      </p>
                      <p className="text-xs text-[var(--text-tertiary)]">
                        {formatDate(file.modified_time)}
                      </p>
                    </div>
                  </motion.div>
                ))}
              </div>
            </div>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
