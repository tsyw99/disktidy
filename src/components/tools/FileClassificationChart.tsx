import { useMemo } from 'react';
import ReactECharts from 'echarts-for-react';
import { motion } from 'framer-motion';
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
  const theme = useUIStore((state) => state.theme);
  const isDark = theme === 'dark';

  const chartData = useMemo(() => {
    return categories.map((cat, index) => ({
      name: cat.display_name,
      value: cat.total_size,
      count: cat.count,
      percentage: cat.percentage,
      itemStyle: {
        color: {
          type: 'linear',
          x: 0,
          y: 0,
          x2: 1,
          y2: 1,
          colorStops: [
            { offset: 0, color: getCategoryColor(cat.category, index)[0] },
            { offset: 1, color: getCategoryColor(cat.category, index)[1] },
          ],
        },
      },
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
        right: 20,
        top: 'center',
        itemWidth: 12,
        itemHeight: 12,
        itemGap: 12,
        textStyle: {
          color: isDark ? '#9ca3af' : '#64748b',
          fontSize: 12,
          rich: {
            name: {
              width: 80,
              fontSize: 12,
            },
            value: {
              width: 60,
              align: 'right',
              fontSize: 12,
              fontWeight: 500,
            },
          },
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
          radius: ['45%', '75%'],
          center: ['35%', '50%'],
          avoidLabelOverlap: true,
          itemStyle: {
            borderRadius: 6,
            borderColor: isDark ? 'rgba(26, 26, 46, 0.8)' : 'rgba(255, 255, 255, 0.8)',
            borderWidth: 2,
          },
          label: {
            show: false,
          },
          emphasis: {
            scale: true,
            scaleSize: 8,
            itemStyle: {
              shadowBlur: 20,
              shadowOffsetX: 0,
              shadowColor: 'rgba(0, 0, 0, 0.3)',
            },
          },
          data: chartData,
          animationType: 'scale',
          animationEasing: 'elasticOut',
          animationDelay: (idx: number) => idx * 50,
        },
      ],
      graphic: [
        {
          type: 'text',
          left: '28%',
          top: '42%',
          style: {
            text: '总计',
            fontSize: 14,
            fill: isDark ? '#9ca3af' : '#64748b',
            textAlign: 'center',
          },
        },
        {
          type: 'text',
          left: '28%',
          top: '52%',
          style: {
            text: formatBytes(totalSize),
            fontSize: 18,
            fontWeight: 'bold',
            fill: isDark ? '#f3f4f6' : '#1e293b',
            textAlign: 'center',
          },
        },
      ],
    };
  }, [chartData, totalSize, isDark, categories]);

  const barOption = useMemo(() => {
    const topCategories = categories.slice(0, 10);
    
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
          const cat = categories.find(c => c.display_name === item.name);
          return `
            <div style="min-width: 160px;">
              <div style="font-weight: 600; margin-bottom: 8px;">${item.name}</div>
              <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">文件数量</span>
                <span>${cat?.count.toLocaleString()} 个</span>
              </div>
              <div style="display: flex; justify-content: space-between;">
                <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">占用空间</span>
                <span style="font-weight: 500; color: ${item.color};">${formatBytes(item.value)}</span>
              </div>
            </div>
          `;
        },
      },
      grid: {
        left: '3%',
        right: '12%',
        bottom: '3%',
        top: '3%',
        containLabel: true,
      },
      xAxis: {
        type: 'value',
        axisLine: { show: false },
        axisTick: { show: false },
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
      yAxis: {
        type: 'category',
        data: topCategories.map(c => c.display_name).reverse(),
        axisLine: { show: false },
        axisTick: { show: false },
        axisLabel: {
          color: isDark ? '#e5e7eb' : '#374151',
          fontSize: 12,
          fontWeight: 500,
        },
      },
      series: [
        {
          name: '占用空间',
          type: 'bar',
          data: topCategories.map((cat, index) => ({
            value: cat.total_size,
            itemStyle: {
              color: {
                type: 'linear',
                x: 0,
                y: 0,
                x2: 1,
                y2: 0,
                colorStops: [
                  { offset: 0, color: getCategoryColor(cat.category, index)[0] },
                  { offset: 1, color: getCategoryColor(cat.category, index)[1] },
                ],
              },
              borderRadius: [0, 4, 4, 0],
            },
          })).reverse(),
          barWidth: '60%',
          animationDelay: (idx: number) => idx * 100,
          animationEasing: 'elasticOut',
        },
      ],
    };
  }, [categories, isDark]);

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <motion.div
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.5 }}
        className="panel p-6"
      >
        <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-4 flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-gradient-to-r from-[#6366f1] to-[#8b5cf6]" />
          文件类型分布
        </h3>
        <ReactECharts
          option={pieOption}
          style={{ width: '100%', height: 400 }}
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

      <motion.div
        initial={{ opacity: 0, x: 20 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.5, delay: 0.1 }}
        className="panel p-6"
      >
        <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-4 flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-gradient-to-r from-[#10b981] to-[#34d399]" />
          空间占用排行
        </h3>
        <ReactECharts
          option={barOption}
          style={{ width: '100%', height: 400 }}
          opts={{ renderer: 'svg' }}
        />
      </motion.div>
    </div>
  );
}
