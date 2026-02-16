import { useEffect, useState, useRef, useCallback } from 'react';
import { motion } from 'framer-motion';
import { HardDrive, Cpu, MemoryStick, PieChart } from 'lucide-react';
import { DiskCard, SystemInfoCard, DiskUsageOverview } from '../components/system';
import { useSystemStore, useUIStore } from '../stores';
import { formatBytes } from '../utils/format';

const AUTO_REFRESH_INTERVAL = 3000;

export default function SystemPage() {
  const { systemInfo, cpuInfo, memoryInfo, diskList, isLoading, actions } = useSystemStore();
  const viewMode = useUIStore((state) => state.systemViewMode);
  const setViewMode = useUIStore((state) => state.actions.setSystemViewMode);
  const chartBtnRef = useRef<HTMLButtonElement>(null);
  const cardsBtnRef = useRef<HTMLButtonElement>(null);
  const [indicatorStyle, setIndicatorStyle] = useState({ width: 0, x: 0 });
  const pageVisibleRef = useRef(true);

  const refreshData = useCallback(() => {
    if (pageVisibleRef.current) {
      actions.refreshAll();
    }
  }, [actions]);

  useEffect(() => {
    actions.refreshAll();
  }, [actions]);

  useEffect(() => {
    const handleVisibilityChange = () => {
      pageVisibleRef.current = !document.hidden;
    };
    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, []);

  useEffect(() => {
    const intervalId = setInterval(refreshData, AUTO_REFRESH_INTERVAL);
    return () => {
      clearInterval(intervalId);
    };
  }, [refreshData]);

  useEffect(() => {
    const activeBtn = viewMode === 'chart' ? chartBtnRef.current : cardsBtnRef.current;
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
  }, [viewMode]);

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-[var(--text-primary)]">系统概览</h1>
            <p className="text-[var(--text-secondary)] text-sm mt-1">
              查看系统信息和磁盘使用情况
            </p>
          </div>
            <div className="flex bg-[var(--bg-tertiary)] rounded-lg p-1 relative">
              <motion.div
                className="absolute top-1 bottom-1 rounded-md"
                style={{
                  background: 'linear-gradient(to right, #3b82f6, #10b981)',
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
              <button
                ref={chartBtnRef}
                onClick={() => setViewMode('chart')}
                className={`relative z-10 flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors duration-200 ${
                  viewMode === 'chart'
                    ? 'text-white'
                    : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                }`}
              >
                <PieChart className="w-4 h-4" />
                图表
              </button>
              <button
                ref={cardsBtnRef}
                onClick={() => setViewMode('cards')}
                className={`relative z-10 flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium transition-colors duration-200 ${
                  viewMode === 'cards'
                    ? 'text-white'
                    : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'
                }`}
              >
                <HardDrive className="w-4 h-4" />
                卡片
              </button>
            </div>
        </div>

        {isLoading && !systemInfo ? (
          <div className="flex items-center justify-center h-64">
            <div className="flex flex-col items-center gap-4">
              <div className="w-12 h-12 border-3 border-[var(--color-primary)] border-t-transparent rounded-full animate-spin" />
              <p className="text-[var(--text-secondary)] text-sm">正在加载系统信息...</p>
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div className="lg:col-span-1">
              <SystemInfoCard 
                systemInfo={systemInfo}
                cpuInfo={cpuInfo}
                memoryInfo={memoryInfo}
                className="h-full"
              />
            </div>

            <div className="lg:col-span-2 space-y-6">
              {viewMode === 'chart' ? (
                <DiskUsageOverview 
                  diskList={diskList}
                  onRefresh={() => actions.fetchDiskList()}
                  isLoading={isLoading}
                />
              ) : (
                <div className="card p-6">
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-lg font-semibold text-[var(--text-primary)] flex items-center gap-2">
                      <HardDrive className="w-5 h-5 text-[var(--color-primary)]" />
                      磁盘概览
                    </h3>
                    <span className="text-sm text-[var(--text-tertiary)]">
                      共 {diskList.length} 个磁盘
                    </span>
                  </div>

                  {diskList.length === 0 ? (
                    <div className="text-center py-8 text-[var(--text-tertiary)]">
                      <HardDrive className="w-12 h-12 mx-auto mb-3 opacity-50" />
                      <p>未检测到磁盘信息</p>
                    </div>
                  ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      {diskList.map((disk, index) => (
                        <DiskCard key={index} disk={disk} />
                      ))}
                    </div>
                  )}
                </div>
              )}

              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <StatCard 
                  label="磁盘数量" 
                  value={diskList.length.toString()}
                  icon={<HardDrive className="w-5 h-5" />}
                />
                <StatCard 
                  label="总容量" 
                  value={formatTotalSize(diskList)}
                  icon={<HardDrive className="w-5 h-5" />}
                />
                <StatCard 
                  label="CPU核心" 
                  value={cpuInfo?.cores?.toString() || '-'}
                  icon={<Cpu className="w-5 h-5" />}
                />
                <StatCard 
                  label="内存使用" 
                  value={memoryInfo ? `${memoryInfo.usage_percent.toFixed(0)}%` : '-'}
                  icon={<MemoryStick className="w-5 h-5" />}
                />
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({ 
  label, 
  value, 
  icon 
}: { 
  label: string; 
  value: string; 
  icon: React.ReactNode;
}) {
  return (
    <div className="card p-4">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]">
          {icon}
        </div>
        <div>
          <p className="text-xs text-[var(--text-tertiary)]">{label}</p>
          <p className="text-lg font-semibold text-[var(--text-primary)]">{value}</p>
        </div>
      </div>
    </div>
  );
}

function formatTotalSize(disks: { total_size: number }[]): string {
  const total = disks.reduce((sum, disk) => sum + disk.total_size, 0);
  return formatBytes(total);
}
