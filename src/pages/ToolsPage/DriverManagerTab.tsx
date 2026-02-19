import { useState, useEffect, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Cpu, Search, RefreshCw, CheckCircle, AlertCircle, Trash2, AlertTriangle, Info, Loader2, Shield, ShieldCheck, Filter } from 'lucide-react';
import { driverService } from '../../services';
import type { DriverInfo } from '../../types/system';
import { DRIVER_STATUS_MAP } from './constants';

export function DriverManagerTab() {
  const [drivers, setDrivers] = useState<DriverInfo[]>([]);
  const [isLoadingDrivers, setIsLoadingDrivers] = useState(false);
  const [driverToDelete, setDriverToDelete] = useState<DriverInfo | null>(null);
  const [showDriverDeleteConfirm, setShowDriverDeleteConfirm] = useState(false);
  const [isDeletingDriver, setIsDeletingDriver] = useState(false);
  const [driverSearchQuery, setDriverSearchQuery] = useState('');
  const [driverTypeFilter, setDriverTypeFilter] = useState<string>('all');
  const [showTypeDropdown, setShowTypeDropdown] = useState(false);

  const loadDrivers = async () => {
    setIsLoadingDrivers(true);
    try {
      const result = await driverService.getList();
      setDrivers(result);
    } catch (error) {
      console.error('加载驱动列表失败:', error);
    } finally {
      setIsLoadingDrivers(false);
    }
  };

  const handleDeleteDriver = async () => {
    if (!driverToDelete) return;

    setIsDeletingDriver(true);
    try {
      await driverService.delete(driverToDelete.inf_name);
      setDrivers(prev => prev.filter(d => d.id !== driverToDelete.id));
      setShowDriverDeleteConfirm(false);
      setDriverToDelete(null);
    } catch (error) {
      console.error('删除驱动失败:', error);
      alert(`删除驱动失败: ${error}`);
    } finally {
      setIsDeletingDriver(false);
    }
  };

  const confirmDeleteDriver = (driver: DriverInfo) => {
    setDriverToDelete(driver);
    setShowDriverDeleteConfirm(true);
  };

  useEffect(() => {
    if (drivers.length === 0 && !isLoadingDrivers) {
      loadDrivers();
    }
  }, []);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (showTypeDropdown && !target.closest('.driver-type-dropdown')) {
        setShowTypeDropdown(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [showTypeDropdown]);

  const filteredDrivers = useMemo(() => {
    return drivers.filter(driver => {
      const matchesSearch = driverSearchQuery === '' ||
        driver.name.toLowerCase().includes(driverSearchQuery.toLowerCase()) ||
        driver.provider.toLowerCase().includes(driverSearchQuery.toLowerCase()) ||
        driver.device_name.toLowerCase().includes(driverSearchQuery.toLowerCase());

      const matchesType = driverTypeFilter === 'all' || driver.driver_type === driverTypeFilter;

      return matchesSearch && matchesType;
    });
  }, [drivers, driverSearchQuery, driverTypeFilter]);

  const driverTypes = useMemo(() => {
    const types = new Set(drivers.map(d => d.driver_type));
    return Array.from(types).sort();
  }, [drivers]);

  const getStatusBadge = (status: string) => {
    return DRIVER_STATUS_MAP[status] || { bg: 'bg-gray-500/10', text: 'text-gray-500', label: '未知' };
  };

  return (
    <div className="panel p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <h3 className="text-lg font-semibold text-[var(--text-primary)]">驱动程序列表</h3>
          <span className="text-sm text-[var(--text-secondary)]">
            共 {drivers.length} 个驱动程序
          </span>
        </div>
        <motion.button
          whileHover={{ scale: 1.02 }}
          whileTap={{ scale: 0.98 }}
          onClick={loadDrivers}
          disabled={isLoadingDrivers}
          className="flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)] text-[var(--text-secondary)] text-sm font-medium transition-all duration-200 hover:border-[var(--color-primary)]/50 hover:text-[var(--text-primary)] disabled:opacity-50"
        >
          <RefreshCw className={`w-4 h-4 ${isLoadingDrivers ? 'animate-spin' : ''}`} />
          刷新列表
        </motion.button>
      </div>

      <div className="flex items-center gap-4 mb-6">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-[var(--text-tertiary)]" />
          <input
            type="text"
            placeholder="搜索驱动名称、提供商或设备..."
            value={driverSearchQuery}
            onChange={(e) => setDriverSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 rounded-lg bg-[var(--bg-tertiary)] border border-[var(--border-color)] text-[var(--text-primary)] placeholder-[var(--text-tertiary)] focus:outline-none focus:border-[var(--color-primary)] transition-colors text-sm"
          />
        </div>
        <div className="relative driver-type-dropdown">
          <motion.button
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={() => setShowTypeDropdown(!showTypeDropdown)}
            className="flex items-center gap-2 px-4 py-2 rounded-xl bg-[var(--bg-tertiary)] border border-[var(--border-color)] text-[var(--text-primary)] text-sm font-medium transition-all duration-200 hover:border-[var(--color-primary)]/50 min-w-[120px]"
          >
            <Filter className="w-4 h-4 text-[var(--color-primary)]" />
            <span>{driverTypeFilter === 'all' ? '全部类型' : driverTypeFilter}</span>
            <motion.svg
              className="w-4 h-4 ml-auto text-[var(--text-tertiary)]"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              animate={{ rotate: showTypeDropdown ? 180 : 0 }}
              transition={{ duration: 0.2 }}
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </motion.svg>
          </motion.button>

          <AnimatePresence>
            {showTypeDropdown && (
              <motion.div
                initial={{ opacity: 0, y: -10, scale: 0.95 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, y: -10, scale: 0.95 }}
                transition={{ duration: 0.15 }}
                className="absolute top-full left-0 mt-2 w-48 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-xl overflow-hidden z-50"
              >
                <div className="py-1 max-h-60 overflow-y-auto">
                  <button
                    onClick={() => {
                      setDriverTypeFilter('all');
                      setShowTypeDropdown(false);
                    }}
                    className={`w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-colors ${
                      driverTypeFilter === 'all'
                        ? 'bg-[var(--color-primary)]/10 text-[var(--color-primary)]'
                        : 'text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
                    }`}
                  >
                    <div className={`w-2 h-2 rounded-full ${driverTypeFilter === 'all' ? 'bg-[var(--color-primary)]' : 'bg-transparent'}`} />
                    全部类型
                    {driverTypeFilter === 'all' && (
                      <CheckCircle className="w-4 h-4 ml-auto text-[var(--color-primary)]" />
                    )}
                  </button>
                  {driverTypes.map(type => (
                    <button
                      key={type}
                      onClick={() => {
                        setDriverTypeFilter(type);
                        setShowTypeDropdown(false);
                      }}
                      className={`w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-colors ${
                        driverTypeFilter === type
                          ? 'bg-[var(--color-primary)]/10 text-[var(--color-primary)]'
                          : 'text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)]'
                      }`}
                    >
                      <div className={`w-2 h-2 rounded-full ${driverTypeFilter === type ? 'bg-[var(--color-primary)]' : 'bg-transparent'}`} />
                      {type}
                      {driverTypeFilter === type && (
                        <CheckCircle className="w-4 h-4 ml-auto text-[var(--color-primary)]" />
                      )}
                    </button>
                  ))}
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>

      {isLoadingDrivers && drivers.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16">
          <Loader2 className="w-10 h-10 text-[var(--color-primary)] animate-spin mb-4" />
          <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">正在加载驱动列表</h3>
          <p className="text-[var(--text-secondary)] text-sm">请稍候...</p>
        </div>
      ) : filteredDrivers.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16">
          <div className="w-16 h-16 rounded-full bg-[var(--color-primary)]/10 flex items-center justify-center mb-4">
            <Cpu className="w-8 h-8 text-[var(--color-primary)] opacity-50" />
          </div>
          <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
            {driverSearchQuery || driverTypeFilter !== 'all' ? '未找到匹配的驱动' : '暂无驱动数据'}
          </h3>
          <p className="text-[var(--text-secondary)] text-sm">
            {driverSearchQuery || driverTypeFilter !== 'all' ? '请尝试其他搜索条件' : '点击刷新列表获取驱动程序信息'}
          </p>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-[var(--border-color)]">
                <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">设备名称</th>
                <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">类型</th>
                <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">版本</th>
                <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">提供商</th>
                <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">日期</th>
                <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">签名</th>
                <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">状态</th>
                <th className="text-left py-3 px-4 text-xs font-semibold text-[var(--text-tertiary)] uppercase tracking-wider">操作</th>
              </tr>
            </thead>
            <tbody>
              {filteredDrivers.map(driver => {
                const statusBadge = getStatusBadge(driver.status);
                return (
                  <tr key={driver.id} className="border-b border-[var(--border-color)] hover:bg-[var(--bg-tertiary)] transition-colors">
                    <td className="py-4 px-4">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]">
                          <Cpu className="w-4 h-4" />
                        </div>
                        <div>
                          <span className="text-sm text-[var(--text-primary)] block">{driver.name}</span>
                          <span className="text-xs text-[var(--text-tertiary)] truncate block max-w-[200px]" title={driver.inf_name}>
                            {driver.inf_name}
                          </span>
                        </div>
                      </div>
                    </td>
                    <td className="py-4 px-4">
                      <span className="text-sm text-[var(--text-secondary)]">{driver.driver_type}</span>
                    </td>
                    <td className="py-4 px-4">
                      <span className="text-sm text-[var(--text-primary)] font-mono">{driver.version}</span>
                    </td>
                    <td className="py-4 px-4">
                      <span className="text-sm text-[var(--text-secondary)]">{driver.provider}</span>
                    </td>
                    <td className="py-4 px-4">
                      <span className="text-sm text-[var(--text-secondary)]">{driver.date}</span>
                    </td>
                    <td className="py-4 px-4">
                      {driver.signed ? (
                        <span className="inline-flex items-center gap-1 text-emerald-500 whitespace-nowrap">
                          <ShieldCheck className="w-4 h-4 flex-shrink-0" />
                          <span className="text-xs">已签名</span>
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 text-amber-500 whitespace-nowrap">
                          <Shield className="w-4 h-4 flex-shrink-0" />
                          <span className="text-xs">未签名</span>
                        </span>
                      )}
                    </td>
                    <td className="py-4 px-4">
                      <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium whitespace-nowrap ${statusBadge.bg} ${statusBadge.text}`}>
                        {driver.status === 'Running' ? (
                          <CheckCircle className="w-3 h-3 flex-shrink-0" />
                        ) : (
                          <AlertCircle className="w-3 h-3 flex-shrink-0" />
                        )}
                        {statusBadge.label}
                      </span>
                    </td>
                    <td className="py-4 px-4">
                      <div className="flex items-center gap-2">
                        <button
                          className="p-1.5 rounded hover:bg-red-500/10 text-[var(--text-tertiary)] hover:text-red-500 transition-colors"
                          title="删除驱动"
                          onClick={() => confirmDeleteDriver(driver)}
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      <div className="flex items-center gap-2 p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 mt-6">
        <AlertTriangle className="w-4 h-4 text-amber-500 flex-shrink-0" />
        <p className="text-xs text-amber-600 dark:text-amber-400">
          <strong>注意：</strong>删除驱动可能导致相关硬件无法正常工作。请谨慎操作，建议仅删除不再使用的旧版本驱动。
        </p>
      </div>

      <div className="flex items-center gap-2 p-3 rounded-lg bg-blue-500/10 border border-blue-500/20 mt-3">
        <Info className="w-4 h-4 text-blue-500 flex-shrink-0" />
        <p className="text-xs text-blue-600 dark:text-blue-400">
          <strong>提示：</strong>目前仅支持删除驱动功能，其他功能（如更新、备份、还原）正在开发中。
        </p>
      </div>

      <AnimatePresence>
        {showDriverDeleteConfirm && driverToDelete && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
            onClick={() => !isDeletingDriver && setShowDriverDeleteConfirm(false)}
          >
            <motion.div
              initial={{ scale: 0.95, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.95, opacity: 0 }}
              className="w-full max-w-md mx-4 bg-[var(--bg-secondary)] border border-[var(--border-color)] rounded-xl shadow-xl"
              onClick={(e) => e.stopPropagation()}
            >
              <div className="p-6">
                <div className="flex items-center gap-4 mb-4">
                  <div className="w-12 h-12 rounded-full bg-red-500/10 flex items-center justify-center">
                    <AlertTriangle className="w-6 h-6 text-red-500" />
                  </div>
                  <div>
                    <h3 className="text-lg font-semibold text-[var(--text-primary)]">确认删除驱动</h3>
                    <p className="text-sm text-[var(--text-secondary)]">此操作不可撤销</p>
                  </div>
                </div>

                <div className="p-4 rounded-lg bg-[var(--bg-tertiary)] mb-4">
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span className="text-sm text-[var(--text-tertiary)]">驱动名称</span>
                      <span className="text-sm text-[var(--text-primary)] font-medium">{driverToDelete.name}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-[var(--text-tertiary)]">版本</span>
                      <span className="text-sm text-[var(--text-primary)] font-mono">{driverToDelete.version}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-[var(--text-tertiary)]">提供商</span>
                      <span className="text-sm text-[var(--text-primary)]">{driverToDelete.provider}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-sm text-[var(--text-tertiary)]">INF文件</span>
                      <span className="text-xs text-[var(--text-primary)] font-mono">{driverToDelete.inf_name}</span>
                    </div>
                  </div>
                </div>

                <p className="text-sm text-[var(--text-secondary)] mb-6">
                  删除此驱动后，相关硬件可能无法正常工作。确定要继续吗？
                </p>

                <div className="flex gap-3">
                  <button
                    onClick={() => setShowDriverDeleteConfirm(false)}
                    disabled={isDeletingDriver}
                    className="flex-1 px-4 py-2 rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors disabled:opacity-50"
                  >
                    取消
                  </button>
                  <button
                    onClick={handleDeleteDriver}
                    disabled={isDeletingDriver}
                    className="flex-1 px-4 py-2 rounded-lg bg-red-500 text-white hover:bg-red-600 transition-colors flex items-center justify-center gap-2 disabled:opacity-50"
                  >
                    {isDeletingDriver ? (
                      <>
                        <Loader2 className="w-4 h-4 animate-spin" />
                        删除中...
                      </>
                    ) : (
                      <>
                        <Trash2 className="w-4 h-4" />
                        确认删除
                      </>
                    )}
                  </button>
                </div>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
