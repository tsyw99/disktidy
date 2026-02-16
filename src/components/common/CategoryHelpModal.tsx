import { motion } from 'framer-motion';
import { HelpCircle, AlertTriangle, CheckCircle, Info } from 'lucide-react';
import Modal from './Modal';

interface CategoryInfo {
  name: string;
  displayName: string;
  description: string;
}

interface CategoryHelpModalProps {
  visible: boolean;
  onClose: () => void;
  categories: CategoryInfo[];
}

const safetyIcons = {
  safe: { icon: CheckCircle, color: 'text-green-500', bg: 'bg-green-500/10' },
  low: { icon: CheckCircle, color: 'text-blue-500', bg: 'bg-blue-500/10' },
  medium: { icon: AlertTriangle, color: 'text-amber-500', bg: 'bg-amber-500/10' },
};

const defaultCategories: CategoryInfo[] = [
  { name: 'system_temp', displayName: '系统临时文件', description: '该类文件通常为应用程序运行过程中产生的临时数据，一般可安全清理。包括系统临时目录和用户临时目录中的文件。' },
  { name: 'browser_cache', displayName: '浏览器缓存', description: '浏览器在访问网页时下载的缓存文件，包括图片、脚本、样式表等。清理后可释放磁盘空间，但可能导致网页首次加载稍慢。' },
  { name: 'app_cache', displayName: '应用程序缓存', description: '各类应用程序产生的缓存数据，如IDE编译缓存、工具软件缓存等。清理前建议确认应用程序已关闭。' },
  { name: 'log_files', displayName: '日志文件', description: '应用程序和系统运行产生的日志记录文件。清理后不影响程序运行，但可能丢失历史运行记录。' },
  { name: 'recycle_bin', displayName: '回收站', description: '已删除但尚未永久清除的文件。清理回收站将永久删除这些文件，请确认其中没有重要数据。' },
  { name: 'npm_cache', displayName: 'NPM缓存', description: 'Node.js包管理器下载的包缓存。清理后下次安装相同包时需要重新下载。' },
  { name: 'pip_cache', displayName: 'Pip缓存', description: 'Python包管理器下载的包缓存。清理后下次安装相同包时需要重新下载。' },
  { name: 'thumbnail_cache', displayName: '缩略图缓存', description: '系统为图片和视频文件生成的缩略图缓存。清理后系统会重新生成缩略图。' },
  { name: 'crash_dumps', displayName: '崩溃转储文件', description: '程序崩溃时生成的调试信息文件。清理后不影响系统运行，但可能影响问题排查。' },
  { name: 'windows_update', displayName: 'Windows更新缓存', description: 'Windows更新下载的临时文件。清理可能影响系统更新，建议在系统稳定后清理。' },
];

function CategoryItem({ category, index }: { category: CategoryInfo; index: number }) {
  const safetyLevel = category.name === 'windows_update' ? 'medium' : 
                      category.name === 'app_cache' ? 'low' : 'safe';
  const safetyInfo = safetyIcons[safetyLevel];
  const SafetyIcon = safetyInfo.icon;

  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: index * 0.05 }}
      className="flex items-start gap-3 p-4 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)]"
    >
      <div className={`flex-shrink-0 w-8 h-8 rounded-lg ${safetyInfo.bg} flex items-center justify-center`}>
        <SafetyIcon className={`w-4 h-4 ${safetyInfo.color}`} />
      </div>
      <div className="flex-1 min-w-0">
        <h4 className="text-sm font-medium text-[var(--text-primary)] mb-1">
          {category.displayName}
        </h4>
        <p className="text-xs text-[var(--text-secondary)] leading-relaxed">
          {category.description}
        </p>
      </div>
    </motion.div>
  );
}

export default function CategoryHelpModal({ visible, onClose, categories }: CategoryHelpModalProps) {
  const displayCategories = categories.length > 0 ? categories : defaultCategories;

  return (
    <Modal
      visible={visible}
      onClose={onClose}
      title={
        <div className="flex items-center gap-2">
          <HelpCircle className="w-5 h-5 text-[var(--color-primary)]" />
          <span>文件分类说明</span>
        </div>
      }
      size={{ width: 600, maxHeight: '80vh' }}
      closeOnOutsideClick={true}
      buttons={[
        { text: '知道了', onClick: onClose, variant: 'primary' }
      ]}
    >
      <div className="space-y-4">
        <div className="flex items-center gap-2 p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
          <Info className="w-4 h-4 text-blue-500 flex-shrink-0" />
          <p className="text-xs text-blue-400">
            以下分类说明帮助您了解各类文件的性质和清理建议，请根据实际情况谨慎选择要清理的文件。
          </p>
        </div>

        <div className="flex items-center gap-4 text-xs text-[var(--text-tertiary)] mb-2">
          <div className="flex items-center gap-1.5">
            <CheckCircle className="w-3.5 h-3.5 text-green-500" />
            <span>安全清理</span>
          </div>
          <div className="flex items-center gap-1.5">
            <CheckCircle className="w-3.5 h-3.5 text-blue-500" />
            <span>低风险</span>
          </div>
          <div className="flex items-center gap-1.5">
            <AlertTriangle className="w-3.5 h-3.5 text-amber-500" />
            <span>中等风险</span>
          </div>
        </div>

        <div className="space-y-2 max-h-[400px] overflow-y-auto pr-2 scrollbar-thin">
          {displayCategories.map((category, index) => (
            <CategoryItem key={category.name} category={category} index={index} />
          ))}
        </div>

        <div className="flex items-center gap-2 p-3 rounded-lg bg-amber-500/10 border border-amber-500/20">
          <AlertTriangle className="w-4 h-4 text-amber-500 flex-shrink-0" />
          <p className="text-xs text-amber-400">
            重要提示：清理前请确保已关闭相关应用程序，避免清理正在使用的文件。
          </p>
        </div>
      </div>
    </Modal>
  );
}

export { defaultCategories };
export type { CategoryInfo };
