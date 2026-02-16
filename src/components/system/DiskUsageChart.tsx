import ReactECharts from 'echarts-for-react';

interface DiskUsageChartProps {
  used: number;
  total: number;
  color?: string;
  size?: number;
}

export default function DiskUsageChart({ 
  used, 
  total, 
  color = '#8b5cf6',
  size = 80 
}: DiskUsageChartProps) {
  const percentage = total > 0 ? (used / total) * 100 : 0;

  const option = {
    series: [
      {
        type: 'pie',
        radius: ['65%', '85%'],
        center: ['50%', '50%'],
        startAngle: 90,
        silent: true,
        data: [
          {
            value: percentage,
            name: '已使用',
            itemStyle: {
              color: {
                type: 'linear',
                x: 0,
                y: 0,
                x2: 1,
                y2: 1,
                colorStops: [
                  { offset: 0, color: color },
                  { offset: 1, color: `${color}80` }
                ]
              },
              borderRadius: 4,
            },
          },
          {
            value: 100 - percentage,
            name: '可用',
            itemStyle: {
              color: 'rgba(75, 85, 99, 0.3)',
              borderRadius: 4,
            },
          },
        ],
        label: {
          show: false,
        },
        emphasis: {
          disabled: true,
        },
      },
    ],
  };

  return (
    <div className="relative" style={{ width: size, height: size }}>
      <ReactECharts
        option={option}
        style={{ width: size, height: size }}
        opts={{ renderer: 'svg' }}
      />
      <div className="absolute inset-0 flex items-center justify-center">
        <span className="text-sm font-semibold text-white">
          {percentage.toFixed(0)}%
        </span>
      </div>
    </div>
  );
}
