import { useMemo, useState } from 'react';
import ReactECharts from 'echarts-for-react';
import { motion, AnimatePresence } from 'framer-motion';
import { PieChart, BarChart3 } from 'lucide-react';
import type { FileTypeStats } from '../../types';
import { useUIStore } from '../../stores';
import { formatBytes } from '../../utils/format';

interface FileClassificationChartProps {
  categories: FileTypeStats[];
  totalSize: number;
  onCategoryClick?: (category: string) => void;
  selectedCategory?: string | null;
}

const CATEGORY_COLORS: Record<string, string[]> = {
  '视频': ['#8b5cf6', '#a78bfa'],
  '音频': ['#ec4899', '#f472b6'],
  '图片': ['#f59e0b', '#fbbf24'],
  '压缩包': ['#10b981', '#34d399'],
  '可执行文件': ['#3b82f6', '#60a5fa'],
  '磁盘镜像': ['#f97316', '#fb923c'],
  '文档': ['#06b6d4', '#22d3ee'],
  '数据库': ['#84cc16', '#a3e635'],
  '文本文件': ['#6b7280', '#9ca3af'],
  '配置文件': ['#6366f1', '#818cf8'],
  '系统文件': ['#ef4444', '#f87171'],
  '代码文件': ['#14b8a6', '#2dd4bf'],
  '字体文件': ['#d946ef', '#e879f9'],
  '备份文件': ['#f59e0b', '#fbbf24'],
  '临时文件': ['#94a3b8', '#cbd5e1'],
  '其他': ['#78716c', '#a8a29e'],
};

const DEFAULT_COLORS = ['#6366f1', '#8b5cf6', '#a855f7', '#d946ef', '#ec4899', '#f43f5e'];

function getCategoryColor(category: string, index: number): string[] {
  return CATEGORY_COLORS[category] || [DEFAULT_COLORS[index % DEFAULT_COLORS.length], DEFAULT_COLORS[index % DEFAULT_COLORS.length]];
}

export default function FileClassificationChart({ 
  categories, 
  totalSize,
  onCategoryClick,
}: FileClassificationChartProps) {
  const [chartType, setChartType] = useState<'pie' | 'bar'>('pie');
  const theme = useUIStore((state) => state.theme);
  const isDark = theme === 'dark';

  const chartData = useMemo(() => {
    return categories.map((cat, index) => ({
      name: cat.display_name,
      value: cat.total_size,
      count: cat.count,
      percentage: cat.percentage,
      category: cat.category,
      color: getCategoryColor(cat.category, index),
    }));
  }, [categories]);

  const pieOption = useMemo(() => {
    return {
      tooltip: {
        trigger: 'item',
        backgroundColor: isDark ? 'rgba(26, 26, 46, 0.95)' : 'rgba(255, 255, 255, 0.95)',
        borderColor: isDark ? 'rgba(139, 92, 246, 0.3)' : 'rgba(99, 102, 241, 0.2)',
        borderWidth: 1,
        borderRadius: 12,
        padding: [16, 20],
        textStyle: {
          color: isDark ? '#f3f4f6' : '#1e293b',
          fontSize: 13,
        },
        formatter: (params: any) => {
          return `
            <div style="min-width: 180px;">
              <div style="font-weight: 600; font-size: 15px; margin-bottom: 10px; color: ${isDark ? '#f3f4f6' : '#1e293b'};">
                ${params.name}
              </div>
              <div style="display: flex; justify-content: space-between; margin-bottom: 6px;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">文件数量</span>
                <span style="font-weight: 500;">${params.data.count.toLocaleString()} 个</span>
              </div>
              <div style="display: flex; justify-content: space-between; margin-bottom: 6px;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">占用空间</span>
                <span style="font-weight: 500; color: ${params.color};">${formatBytes(params.value)}</span>
              </div>
              <div style="display: flex; justify-content: space-between;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">占比</span>
                <span style="font-weight: 600; color: ${params.color};">${params.data.percentage.toFixed(2)}%</span>
              </div>
            </div>
          `;
        },
      },
      legend: {
        type: 'scroll',
        orient: 'vertical',
        right: 10,
        top: 'middle',
        itemWidth: 10,
        itemHeight: 10,
        itemGap: 10,
        textStyle: {
          color: isDark ? '#9ca3af' : '#64748b',
          fontSize: 11,
          width: 90,
          overflow: 'truncate',
        },
        formatter: (name: string) => {
          const item = chartData.find(d => d.name === name);
          if (item) {
            return `${name}  ${item.percentage.toFixed(1)}%`;
          }
          return name;
        },
        pageIconColor: isDark ? '#8b5cf6' : '#6366f1',
        pageIconInactiveColor: isDark ? '#4b5563' : '#d1d5db',
        pageTextStyle: {
          color: isDark ? '#9ca3af' : '#64748b',
        },
      },
      series: [
        {
          name: '文件分类',
          type: 'pie',
          radius: ['40%', '65%'],
          center: ['50%', '50%'],
          avoidLabelOverlap: true,
          itemStyle: {
            borderRadius: 5,
            borderColor: isDark ? 'rgba(26, 26, 46, 0.8)' : 'rgba(255, 255, 255, 0.8)',
            borderWidth: 2,
          },
          label: {
            show: false,
          },
          emphasis: {
            scale: true,
            scaleSize: 6,
            itemStyle: {
              shadowBlur: 15,
              shadowOffsetX: 0,
              shadowColor: 'rgba(0, 0, 0, 0.3)',
            },
          },
          data: chartData.map((item) => ({
            name: item.name,
            value: item.value,
            count: item.count,
            percentage: item.percentage,
            itemStyle: {
              color: {
                type: 'linear',
                x: 0,
                y: 0,
                x2: 1,
                y2: 1,
                colorStops: [
                  { offset: 0, color: item.color[0] },
                  { offset: 1, color: item.color[1] },
                ],
              },
            },
          })),
          animationType: 'scale',
          animationEasing: 'elasticOut',
          animationDelay: (idx: number) => idx * 50,
        },
      ],
      graphic: [
        {
          type: 'text',
          left: '50%',
          top: '45%',
          style: {
            text: '总计',
            fontSize: 13,
            fill: isDark ? '#9ca3af' : '#64748b',
            textAlign: 'center',
          },
        },
        {
          type: 'text',
          left: '50%',
          top: '52%',
          style: {
            text: formatBytes(totalSize),
            fontSize: 16,
            fontWeight: 'bold',
            fill: isDark ? '#f3f4f6' : '#1e293b',
            textAlign: 'center',
          },
        },
      ],
    };
  }, [chartData, totalSize, isDark]);

  const barOption = useMemo(() => {
    const sortedData = [...chartData].sort((a, b) => b.value - a.value);
    
    return {
      tooltip: {
        trigger: 'axis',
        axisPointer: {
          type: 'shadow',
          shadowStyle: {
            color: isDark ? 'rgba(139, 92, 246, 0.1)' : 'rgba(99, 102, 241, 0.1)',
          },
        },
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
          const item = params[0];
          const dataItem = sortedData[item.dataIndex];
          return `
            <div style="min-width: 180px;">
              <div style="font-weight: 600; font-size: 14px; margin-bottom: 8px; color: ${isDark ? '#f3f4f6' : '#1e293b'};">
                ${item.name}
              </div>
              <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">文件数量</span>
                <span style="font-weight: 500;">${dataItem?.count.toLocaleString()} 个</span>
              </div>
              <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">占用空间</span>
                <span style="font-weight: 500; color: ${item.color};">${formatBytes(item.value)}</span>
              </div>
              <div style="display: flex; justify-content: space-between;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">占比</span>
                <span style="font-weight: 600; color: ${item.color};">${dataItem?.percentage.toFixed(2)}%</span>
              </div>
            </div>
          `;
        },
      },
      grid: {
        left: '3%',
        right: '8%',
        bottom: '3%',
        top: '5%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: sortedData.map(d => d.name),
        axisLine: {
          lineStyle: {
            color: isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)',
          },
        },
        axisLabel: {
          color: isDark ? '#9ca3af' : '#64748b',
          fontSize: 11,
          rotate: 30,
          interval: 0,
        },
        axisTick: {
          show: false,
        },
      },
      yAxis: {
        type: 'value',
        axisLine: {
          show: false,
        },
        axisLabel: {
          color: isDark ? '#9ca3af' : '#64748b',
          fontSize: 11,
          formatter: (value: number) => {
            if (value >= 1024 * 1024 * 1024 * 1024) {
              return (value / (1024 * 1024 * 1024 * 1024)).toFixed(1) + ' TB';
            } else if (value >= 1024 * 1024 * 1024) {
              return (value / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
            } else if (value >= 1024 * 1024) {
              return (value / (1024 * 1024)).toFixed(1) + ' MB';
            }
            return formatBytes(value);
          },
        },
        splitLine: {
          lineStyle: {
            color: isDark ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.05)',
            type: 'dashed',
          },
        },
      },
      series: [
        {
          name: '占用空间',
          type: 'bar',
          data: sortedData.map((item) => ({
            value: item.value,
            itemStyle: {
              color: {
                type: 'linear',
                x: 0,
                y: 0,
                x2: 0,
                y2: 1,
                colorStops: [
                  { offset: 0, color: item.color[0] },
                  { offset: 1, color: item.color[1] },
                ],
              },
              borderRadius: [6, 6, 0, 0],
            },
          })),
          barWidth: '50%',
          animationDelay: (idx: number) => idx * 50,
          animationEasing: 'elasticOut',
          animationDuration: 800,
          emphasis: {
            itemStyle: {
              shadowBlur: 20,
              shadowColor: 'rgba(139, 92, 246, 0.5)',
            },
          },
        },
      ],
      animation: true,
      animationDuration: 1000,
      animationEasing: 'cubicOut',
    };
  }, [chartData, isDark]);

  return (
    <div className="w-full">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="panel p-4 lg:p-6 overflow-hidden"
      >
        <div className="flex items-center justify-between mb-3 lg:mb-4">
          <h3 className="text-base lg:text-lg font-semibold text-[var(--text-primary)] flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-gradient-to-r from-[#6366f1] to-[#8b5cf6]" />
            文件类型分布
          </h3>
          <div className="flex items-center gap-1 p-1 rounded-lg bg-[var(--bg-tertiary)]">
            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              onClick={() => setChartType('pie')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 ${
                chartType === 'pie'
                  ? 'bg-gradient-to-r from-[#6366f1] to-[#8b5cf6] text-white shadow-sm'
                  : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
              }`}
            >
              <PieChart className="w-3.5 h-3.5" />
              饼图
            </motion.button>
            <motion.button
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
              onClick={() => setChartType('bar')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 ${
                chartType === 'bar'
                  ? 'bg-gradient-to-r from-[#10b981] to-[#34d399] text-white shadow-sm'
                  : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
              }`}
            >
              <BarChart3 className="w-3.5 h-3.5" />
              柱状图
            </motion.button>
          </div>
        </div>
        <div className="w-full" style={{ minHeight: 300, height: 'clamp(300px, 40vw, 400px)' }}>
          <AnimatePresence mode="wait">
            {chartType === 'pie' ? (
              <motion.div
                key="pie"
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.9 }}
                transition={{ duration: 0.3 }}
                className="w-full h-full"
              >
                <ReactECharts
                  option={pieOption}
                  style={{ width: '100%', height: '100%' }}
                  opts={{ renderer: 'svg' }}
                  onEvents={{
                    click: (params: any) => {
                      if (onCategoryClick && params.name) {
                        onCategoryClick(params.name);
                      }
                    },
                  }}
                />
              </motion.div>
            ) : (
              <motion.div
                key="bar"
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                exit={{ opacity: 0, scale: 0.9 }}
                transition={{ duration: 0.3 }}
                className="w-full h-full"
              >
                <ReactECharts
                  option={barOption}
                  style={{ width: '100%', height: '100%' }}
                  opts={{ renderer: 'svg' }}
                  onEvents={{
                    click: (params: any) => {
                      if (onCategoryClick && params.name) {
                        onCategoryClick(params.name);
                      }
                    },
                  }}
                />
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </motion.div>
    </div>
  );
}
