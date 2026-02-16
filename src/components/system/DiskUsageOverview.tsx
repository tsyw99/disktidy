import { useMemo } from 'react';
import ReactECharts from 'echarts-for-react';
import type { DiskInfo } from '../../types';
import { useUIStore } from '../../stores';
import { formatBytes } from '../../utils/format';

interface DiskUsageOverviewProps {
  diskList: DiskInfo[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

const USAGE_LEVELS = {
  low: {
    threshold: 50,
    color: '#10b981',
    gradient: ['#10b981', '#34d399'],
    label: '正常',
  },
  medium: {
    threshold: 75,
    color: '#f59e0b',
    gradient: ['#f59e0b', '#fbbf24'],
    label: '警告',
  },
  high: {
    threshold: 100,
    color: '#ef4444',
    gradient: ['#ef4444', '#f87171'],
    label: '危险',
  },
};

function getUsageLevel(usagePercent: number) {
  if (usagePercent < USAGE_LEVELS.low.threshold) return USAGE_LEVELS.low;
  if (usagePercent < USAGE_LEVELS.medium.threshold) return USAGE_LEVELS.medium;
  return USAGE_LEVELS.high;
}

export default function DiskUsageOverview({ 
  diskList, 
  onRefresh: _onRefresh,
  isLoading: _isLoading = false 
}: DiskUsageOverviewProps) {
  const theme = useUIStore((state) => state.theme);
  
  const isDark = theme === 'dark';

  const chartOption = useMemo(() => {
    const diskNames = diskList.map(disk => disk.name);
    const usageData = diskList.map(disk => {
      const level = getUsageLevel(disk.usage_percent);
      return {
        value: disk.usage_percent,
        itemStyle: {
          color: {
            type: 'linear',
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              { offset: 0, color: level.gradient[0] },
              { offset: 1, color: level.gradient[1] },
            ],
          },
          borderRadius: [6, 6, 0, 0],
        },
      };
    });

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
          const dataIndex = params[0].dataIndex;
          const disk = diskList[dataIndex];
          const level = getUsageLevel(disk.usage_percent);
          
          return `
            <div style="min-width: 200px;">
              <div style="font-weight: 600; font-size: 14px; margin-bottom: 8px; color: ${isDark ? '#f3f4f6' : '#1e293b'};">
                ${disk.name} (${disk.mount_point})
              </div>
              <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 6px;">
                <span style="display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: ${level.color};"></span>
                <span style="color: ${level.color}; font-weight: 500;">${level.label} - ${disk.usage_percent.toFixed(1)}%</span>
              </div>
              <div style="border-top: 1px solid ${isDark ? 'rgba(255,255,255,0.1)' : 'rgba(0,0,0,0.1)'}; margin: 8px 0; padding-top: 8px;">
                <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                  <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">总容量</span>
                  <span style="font-weight: 500;">${formatBytes(disk.total_size)}</span>
                </div>
                <div style="display: flex; justify-content: space-between; margin-bottom: 4px;">
                  <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">已使用</span>
                  <span style="font-weight: 500; color: ${level.color};">${formatBytes(disk.used_size)}</span>
                </div>
                <div style="display: flex; justify-content: space-between;">
                  <span style="color: ${isDark ? '#9ca3af' : '#64748b'};">可用空间</span>
                  <span style="font-weight: 500; color: #10b981;">${formatBytes(disk.free_size)}</span>
                </div>
              </div>
              <div style="color: ${isDark ? '#6b7280' : '#94a3b8'}; font-size: 11px; margin-top: 6px;">
                文件系统: ${disk.file_system}
              </div>
            </div>
          `;
        },
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '8%',
        top: '12%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: diskNames,
        axisLine: {
          lineStyle: {
            color: isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.1)',
          },
        },
        axisLabel: {
          color: isDark ? '#9ca3af' : '#64748b',
          fontSize: 12,
          fontWeight: 500,
        },
        axisTick: {
          show: false,
        },
      },
      yAxis: {
        type: 'value',
        max: 100,
        axisLine: {
          show: false,
        },
        axisLabel: {
          color: isDark ? '#9ca3af' : '#64748b',
          fontSize: 11,
          formatter: '{value}%',
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
          name: '使用率',
          type: 'bar',
          data: usageData,
          barWidth: '50%',
          animationDelay: (idx: number) => idx * 100,
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
  }, [diskList, isDark]);

  const gaugeOption = useMemo(() => {
    if (diskList.length === 0) return null;

    const totalUsed = diskList.reduce((sum, disk) => sum + disk.used_size, 0);
    const totalSize = diskList.reduce((sum, disk) => sum + disk.total_size, 0);
    const avgUsage = totalSize > 0 ? (totalUsed / totalSize) * 100 : 0;
    const level = getUsageLevel(avgUsage);

    return {
      series: [
        {
          type: 'gauge',
          startAngle: 200,
          endAngle: -20,
          min: 0,
          max: 100,
          splitNumber: 10,
          radius: '90%',
          center: ['50%', '60%'],
          itemStyle: {
            color: {
              type: 'linear',
              x: 0,
              y: 0,
              x2: 1,
              y2: 0,
              colorStops: [
                { offset: 0, color: USAGE_LEVELS.low.color },
                { offset: 0.5, color: USAGE_LEVELS.medium.color },
                { offset: 1, color: USAGE_LEVELS.high.color },
              ],
            },
          },
          progress: {
            show: true,
            width: 20,
            roundCap: true,
          },
          pointer: {
            show: false,
          },
          axisLine: {
            lineStyle: {
              width: 20,
              color: [[1, isDark ? 'rgba(255, 255, 255, 0.1)' : 'rgba(0, 0, 0, 0.08)']],
              roundCap: true,
            },
          },
          axisTick: {
            show: false,
          },
          splitLine: {
            show: false,
          },
          axisLabel: {
            show: false,
          },
          title: {
            show: false,
          },
          detail: {
            valueAnimation: true,
            width: '60%',
            lineHeight: 40,
            borderRadius: 8,
            offsetCenter: [0, '-5%'],
            fontSize: 28,
            fontWeight: 'bold',
            formatter: '{value}%',
            color: level.color,
          },
          data: [
            {
              value: avgUsage.toFixed(1),
            },
          ],
        },
      ],
      graphic: [
        {
          type: 'text',
          left: 'center',
          top: '68%',
          style: {
            text: '总磁盘使用率',
            fontSize: 13,
            fill: isDark ? '#9ca3af' : '#64748b',
          },
        },
        {
          type: 'text',
          left: 'center',
          top: '82%',
          style: {
            text: `${formatBytes(totalUsed)} / ${formatBytes(totalSize)}`,
            fontSize: 12,
            fill: isDark ? '#6b7280' : '#94a3b8',
          },
        },
      ],
      animation: true,
      animationDuration: 1500,
      animationEasing: 'cubicOut',
    };
  }, [diskList, isDark]);

  if (diskList.length === 0) {
    return (
      <div className="card p-6">
        <div className="text-center py-8 text-[var(--text-tertiary)]">
          <p>暂无磁盘数据</p>
        </div>
      </div>
    );
  }

  return (
    <div className="card p-6">
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-1 flex flex-col items-center justify-center">
          {gaugeOption && (
            <ReactECharts
              option={gaugeOption}
              style={{ width: '100%', height: 280 }}
              opts={{ renderer: 'svg' }}
            />
          )}
          <div className="flex gap-4 mt-2">
            {Object.entries(USAGE_LEVELS).map(([key, level]) => (
              <div key={key} className="flex items-center gap-1.5">
                <span 
                  className="w-2.5 h-2.5 rounded-full"
                  style={{ backgroundColor: level.color }}
                />
                <span className="text-xs text-[var(--text-tertiary)]">{level.label}</span>
              </div>
            ))}
          </div>
        </div>
        
        <div className="lg:col-span-2">
          <ReactECharts
            option={chartOption}
            style={{ width: '100%', height: 340 }}
            opts={{ renderer: 'svg' }}
            notMerge={true}
          />
        </div>
      </div>
    </div>
  );
}

export { USAGE_LEVELS, getUsageLevel };
