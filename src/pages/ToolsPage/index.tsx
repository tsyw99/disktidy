import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Package, FolderTree, Cpu, Loader2 } from 'lucide-react';
import { useUIStore } from '../../stores';
import { SoftwareResidueTab } from './SoftwareResidueTab';
import { FileClassificationTab } from './FileClassificationTab';
import { DriverManagerTab } from './DriverManagerTab';

type ToolModule = 'residual' | 'fileclassify' | 'driver';

const pageVariants = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 },
};

const modules = [
  { id: 'residual' as ToolModule, name: '软件残留清理', icon: <Package className="w-5 h-5" />, description: '扫描已卸载软件的残留文件和配置' },
  { id: 'fileclassify' as ToolModule, name: '文件分类大盘', icon: <FolderTree className="w-5 h-5" />, description: '按类型分类统计磁盘文件' },
  { id: 'driver' as ToolModule, name: '驱动管理', icon: <Cpu className="w-5 h-5" />, description: '查看和管理系统驱动程序' },
];

export default function ToolsPage() {
  const [activeModule, setActiveModule] = useState<ToolModule>('fileclassify');

  const globalIsWorking = useUIStore((state) => state.isWorking);

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-[var(--text-primary)]">其他功能</h1>
            <p className="text-[var(--text-secondary)] text-sm mt-1">
              高级系统维护工具，助您深度优化电脑
            </p>
          </div>
        </div>

        <div className="flex gap-4 border-b border-[var(--border-color)] pb-4 items-center">
          {modules.map(module => (
            <motion.button
              key={module.id}
              whileHover={globalIsWorking ? {} : { scale: 1.02 }}
              whileTap={globalIsWorking ? {} : { scale: 0.98 }}
              onClick={() => !globalIsWorking && setActiveModule(module.id)}
              disabled={globalIsWorking}
              className={`flex items-center gap-2 px-4 py-2 rounded-lg transition-all duration-200 ${
                activeModule === module.id
                  ? 'bg-gradient-to-r from-[#3b82f6] to-[#10b981] text-white'
                  : globalIsWorking
                  ? 'bg-[var(--bg-secondary)] text-[var(--text-tertiary)] border border-[var(--border-color)] cursor-not-allowed opacity-60'
                  : 'bg-[var(--bg-secondary)] text-[var(--text-secondary)] hover:text-[var(--text-primary)] border border-[var(--border-color)]'
              }`}
            >
              {module.icon}
              <span className="text-sm font-medium">{module.name}</span>
            </motion.button>
          ))}
          {globalIsWorking && (
            <motion.div
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              className="flex items-center gap-2 text-sm text-[var(--color-primary)] ml-auto"
            >
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>功能进行中，请稍候...</span>
            </motion.div>
          )}
        </div>

        <AnimatePresence mode="wait">
          {activeModule === 'residual' && (
            <motion.div
              key="residual"
              variants={pageVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="space-y-6"
            >
              <SoftwareResidueTab />
            </motion.div>
          )}

          {activeModule === 'fileclassify' && (
            <motion.div
              key="fileclassify"
              variants={pageVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="space-y-6"
            >
              <FileClassificationTab />
            </motion.div>
          )}

          {activeModule === 'driver' && (
            <motion.div
              key="driver"
              variants={pageVariants}
              initial="initial"
              animate="animate"
              exit="exit"
              className="space-y-6"
            >
              <DriverManagerTab />
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}
