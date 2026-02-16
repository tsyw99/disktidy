import { useState } from 'react';
import { Settings, Sun, Moon, Monitor, HardDrive, Trash2, Shield, Bell, FolderX, FileCheck, AlertTriangle, Construction, Info, Github, User } from 'lucide-react';
import { useUIStore } from '../stores';
import { Modal } from '../components/common';

export default function SettingsPage() {
  const theme = useUIStore((state) => state.theme);
  const { toggleTheme } = useUIStore((state) => state.actions);

  const [activeModal, setActiveModal] = useState<string | null>(null);

  const closeModal = () => setActiveModal(null);

  const showDevelopingToast = () => {
    setActiveModal('developing');
  };

  return (
    <div className="h-full overflow-auto p-6">
      <div className="max-w-4xl mx-auto space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-[var(--text-primary)]">设置</h1>
          <p className="text-[var(--text-secondary)] text-sm mt-1">
            应用设置和偏好配置
          </p>
        </div>

        <div className="card p-6">
          <h2 className="text-lg font-semibold text-[var(--text-primary)] mb-4 flex items-center gap-2">
            <Monitor className="w-5 h-5 text-[var(--color-primary)]" />
            外观设置
          </h2>
          
          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 rounded-lg bg-[var(--bg-secondary)]">
              <div className="flex items-center gap-3">
                {theme === 'light' ? (
                  <Sun className="w-5 h-5 text-amber-500" />
                ) : (
                  <Moon className="w-5 h-5 text-purple-400" />
                )}
                <div>
                  <p className="font-medium text-[var(--text-primary)]">主题模式</p>
                  <p className="text-sm text-[var(--text-tertiary)]">
                    当前: {theme === 'light' ? '浅色模式' : '深色模式'}
                  </p>
                </div>
              </div>
              
              <button
                onClick={toggleTheme}
                className="theme-toggle"
                aria-label="切换主题"
              >
                <div className="theme-toggle-thumb">
                  {theme === 'light' ? (
                    <Sun className="w-3.5 h-3.5" />
                  ) : (
                    <Moon className="w-3.5 h-3.5" />
                  )}
                </div>
              </button>
            </div>
          </div>
        </div>

        <div className="card p-6">
          <h2 className="text-lg font-semibold text-[var(--text-primary)] mb-4 flex items-center gap-2">
            <Settings className="w-5 h-5 text-[var(--color-primary)]" />
            常规设置
          </h2>
          
          <div className="space-y-3">
            <SettingItem
              icon={<HardDrive className="w-5 h-5" />}
              title="磁盘扫描设置"
              description="配置扫描范围和排除项"
              onClick={showDevelopingToast}
            />
            <SettingItem
              icon={<Trash2 className="w-5 h-5" />}
              title="清理规则"
              description="自定义垃圾文件清理规则"
              onClick={showDevelopingToast}
            />
            <SettingItem
              icon={<Shield className="w-5 h-5" />}
              title="安全设置"
              description="配置安全扫描选项"
              onClick={showDevelopingToast}
            />
            <SettingItem
              icon={<Bell className="w-5 h-5" />}
              title="通知设置"
              description="管理应用通知偏好"
              onClick={showDevelopingToast}
            />
          </div>
        </div>

        <div className="card p-6">
          <h2 className="text-lg font-semibold text-[var(--text-primary)] mb-4 flex items-center gap-2">
            <Info className="w-5 h-5 text-[var(--color-primary)]" />
            关于
          </h2>
          <div className="space-y-4">
            <div className="space-y-2 text-sm text-[var(--text-secondary)]">
              <p className="text-lg font-semibold text-[var(--text-primary)]">DiskTidy v1.6.0 测试版</p>
              <p>Windows 磁盘清理工具</p>
              <p className="text-[var(--text-tertiary)]">使用 Tauri + React 构建</p>
              <p className="text-[var(--text-tertiary)] text-xs">感谢 Magic UI 和 React Bits</p>
            </div>

            <div className="border-t border-[var(--border-color)] pt-4 space-y-3">
              <div className="flex items-center gap-3 text-sm text-[var(--text-secondary)]">
                <User className="w-4 h-4 text-[var(--color-primary)]" />
                <span>作者：踏上云雾</span>
              </div>
              <div className="flex items-center gap-3 text-sm text-[var(--text-secondary)]">
                <Github className="w-4 h-4 text-[var(--color-primary)]" />
                <span>GitHub：@tsyw120</span>
              </div>
            </div>

            <button
              onClick={() => setActiveModal('disclaimer')}
              className="w-full mt-2 flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-[var(--bg-secondary)] hover:bg-[var(--bg-tertiary)] text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors"
            >
              <AlertTriangle className="w-4 h-4" />
              使用须知
            </button>
          </div>
        </div>
      </div>

      <Modal
        visible={activeModal === 'developing'}
        onClose={closeModal}
        title="功能开发中"
        size={{ width: 400, maxWidth: '90vw' }}
        animation={{ type: 'scale', duration: 0.25 }}
        overlay={{ opacity: 0.6, blur: true }}
        buttons={[
          { text: '我知道了', onClick: closeModal, variant: 'primary' },
        ]}
      >
        <div className="flex flex-col items-center justify-center py-6 text-center">
          <div className="w-16 h-16 rounded-full bg-amber-500/10 flex items-center justify-center mb-4">
            <Construction className="w-8 h-8 text-amber-500" />
          </div>
          <h3 className="text-lg font-semibold text-[var(--text-primary)] mb-2">
            正在开发中
          </h3>
          <p className="text-sm text-[var(--text-secondary)]">
            该功能正在紧张开发中，敬请期待后续版本更新
          </p>
        </div>
      </Modal>

      <Modal
        visible={activeModal === 'diskScan'}
        onClose={closeModal}
        title="磁盘扫描设置"
        size={{ width: 520, maxWidth: '90vw' }}
        animation={{ type: 'scale', duration: 0.25 }}
        overlay={{ opacity: 0.6, blur: true }}
        buttons={[
          { text: '取消', onClick: closeModal, variant: 'secondary' },
          { text: '保存设置', onClick: closeModal, variant: 'primary' },
        ]}
      >
        <div className="space-y-4">
          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3 flex items-center gap-2">
              <FolderX className="w-4 h-4 text-[var(--color-primary)]" />
              排除目录
            </h4>
            <div className="space-y-2">
              {['C:\\Windows\\System32', 'C:\\Program Files', 'C:\\$Recycle.Bin'].map((path) => (
                <div key={path} className="flex items-center justify-between p-2 rounded bg-[var(--bg-tertiary)]">
                  <span className="text-xs text-[var(--text-secondary)] font-mono">{path}</span>
                  <button className="text-xs text-[var(--text-tertiary)] hover:text-red-500 transition-colors">
                    移除
                  </button>
                </div>
              ))}
            </div>
            <button className="mt-3 text-xs text-[var(--color-primary)] hover:underline">
              + 添加排除目录
            </button>
          </div>

          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">扫描选项</h4>
            <div className="space-y-3">
              <label className="flex items-center gap-3 cursor-pointer">
                <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-[var(--border-color)]" />
                <span className="text-sm text-[var(--text-secondary)]">扫描隐藏文件</span>
              </label>
              <label className="flex items-center gap-3 cursor-pointer">
                <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-[var(--border-color)]" />
                <span className="text-sm text-[var(--text-secondary)]">扫描系统文件</span>
              </label>
              <label className="flex items-center gap-3 cursor-pointer">
                <input type="checkbox" className="w-4 h-4 rounded border-[var(--border-color)]" />
                <span className="text-sm text-[var(--text-secondary)]">深度扫描（耗时较长）</span>
              </label>
            </div>
          </div>
        </div>
      </Modal>

      <Modal
        visible={activeModal === 'cleanRules'}
        onClose={closeModal}
        title="清理规则配置"
        size={{ width: 600, maxWidth: '90vw' }}
        animation={{ type: 'slideUp', duration: 0.3 }}
        overlay={{ opacity: 0.5, blur: false }}
        buttons={[
          { text: '重置默认', onClick: closeModal, variant: 'ghost' },
          { text: '取消', onClick: closeModal, variant: 'secondary' },
          { text: '应用规则', onClick: closeModal, variant: 'primary' },
        ]}
      >
        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-3">
            {[
              { name: '临时文件', pattern: '*.tmp', enabled: true },
              { name: '日志文件', pattern: '*.log', enabled: true },
              { name: '缓存文件', pattern: 'cache/*', enabled: true },
              { name: '备份文件', pattern: '*.bak', enabled: false },
              { name: '缩略图缓存', pattern: 'Thumbs.db', enabled: true },
              { name: '系统临时', pattern: '%TEMP%/*', enabled: true },
            ].map((rule, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 rounded-lg bg-[var(--bg-secondary)] border border-[var(--border-color)]"
              >
                <div className="flex items-center gap-3">
                  <FileCheck className="w-4 h-4 text-[var(--color-primary)]" />
                  <div>
                    <p className="text-sm font-medium text-[var(--text-primary)]">{rule.name}</p>
                    <p className="text-xs text-[var(--text-tertiary)] font-mono">{rule.pattern}</p>
                  </div>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    defaultChecked={rule.enabled}
                    className="sr-only peer"
                  />
                  <div className="w-9 h-5 bg-[var(--bg-tertiary)] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[var(--color-primary)]"></div>
                </label>
              </div>
            ))}
          </div>

          <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/20">
            <div className="flex items-start gap-2">
              <AlertTriangle className="w-4 h-4 text-amber-500 flex-shrink-0 mt-0.5" />
              <p className="text-xs text-amber-600 dark:text-amber-400">
                请谨慎配置清理规则，错误的规则可能导致重要文件被误删
              </p>
            </div>
          </div>
        </div>
      </Modal>

      <Modal
        visible={activeModal === 'security'}
        onClose={closeModal}
        title="安全设置"
        size={{ width: 480, maxWidth: '90vw' }}
        animation={{ type: 'zoom', duration: 0.2 }}
        overlay={{ opacity: 0.7, blur: true }}
        buttons={[
          { text: '关闭', onClick: closeModal, variant: 'secondary' },
          { text: '保存', onClick: closeModal, variant: 'primary' },
        ]}
      >
        <div className="space-y-4">
          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3 flex items-center gap-2">
              <Shield className="w-4 h-4 text-[var(--color-primary)]" />
              安全扫描选项
            </h4>
            <div className="space-y-3">
              <label className="flex items-center justify-between cursor-pointer">
                <span className="text-sm text-[var(--text-secondary)]">删除前确认</span>
                <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-[var(--border-color)]" />
              </label>
              <label className="flex items-center justify-between cursor-pointer">
                <span className="text-sm text-[var(--text-secondary)]">创建还原点</span>
                <input type="checkbox" defaultChecked className="w-4 h-4 rounded border-[var(--border-color)]" />
              </label>
              <label className="flex items-center justify-between cursor-pointer">
                <span className="text-sm text-[var(--text-secondary)]">安全删除模式</span>
                <input type="checkbox" className="w-4 h-4 rounded border-[var(--border-color)]" />
              </label>
            </div>
          </div>

          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">信任列表</h4>
            <p className="text-xs text-[var(--text-tertiary)]">
              添加到信任列表的文件和文件夹将不会被扫描和清理
            </p>
            <button className="mt-3 text-xs text-[var(--color-primary)] hover:underline">
              管理信任列表 →
            </button>
          </div>
        </div>
      </Modal>

      <Modal
        visible={activeModal === 'notification'}
        onClose={closeModal}
        title="通知设置"
        size={{ width: 420, maxWidth: '90vw' }}
        animation={{ type: 'fade', duration: 0.15 }}
        overlay={{ opacity: 0.4 }}
        buttons={[
          { text: '关闭', onClick: closeModal, variant: 'secondary' },
        ]}
      >
        <div className="space-y-4">
          <div className="space-y-3">
            {[
              { label: '扫描完成通知', enabled: true },
              { label: '清理完成通知', enabled: true },
              { label: '系统警告通知', enabled: true },
              { label: '更新提醒', enabled: false },
              { label: '每周报告', enabled: false },
            ].map((item, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 rounded-lg bg-[var(--bg-secondary)]"
              >
                <span className="text-sm text-[var(--text-secondary)]">{item.label}</span>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    defaultChecked={item.enabled}
                    className="sr-only peer"
                  />
                  <div className="w-9 h-5 bg-[var(--bg-tertiary)] peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[var(--color-primary)]"></div>
                </label>
              </div>
            ))}
          </div>
        </div>
      </Modal>

      <Modal
        visible={activeModal === 'disclaimer'}
        onClose={closeModal}
        title="使用须知"
        size={{ width: 520, maxWidth: '90vw' }}
        animation={{ type: 'scale', duration: 0.25 }}
        overlay={{ opacity: 0.6, blur: true }}
        buttons={[
          { text: '我已了解', onClick: closeModal, variant: 'primary' },
        ]}
      >
        <div className="space-y-4 py-2">
          <div className="flex items-start gap-3 p-4 rounded-lg bg-red-500/10 border border-red-500/20">
            <AlertTriangle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
            <div>
              <h4 className="text-sm font-semibold text-red-600 dark:text-red-400 mb-1">免责声明</h4>
              <p className="text-xs text-red-600/80 dark:text-red-400/80 leading-relaxed">
                本应用仅供学习和个人使用，作者不对用户使用本应用删除的文件导致的任何系统故障、数据丢失或其他问题承担任何责任。
              </p>
            </div>
          </div>

          <div className="space-y-3 text-sm text-[var(--text-secondary)]">
            <h4 className="font-medium text-[var(--text-primary)]">使用说明</h4>
            <ul className="space-y-2 text-xs leading-relaxed">
              <li className="flex items-start gap-2">
                <span className="text-[var(--color-primary)]">•</span>
                <span>在使用本应用进行文件清理前，请务必备份重要数据</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-[var(--color-primary)]">•</span>
                <span>请仔细确认要删除的文件，避免误删系统关键文件</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-[var(--color-primary)]">•</span>
                <span>建议优先使用"移至回收站"功能，以便误删时恢复</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-[var(--color-primary)]">•</span>
                <span>本应用已内置系统目录保护，但仍请谨慎操作</span>
              </li>
            </ul>
          </div>

          <div className="pt-3 border-t border-[var(--border-color)]">
            <p className="text-xs text-[var(--text-tertiary)] text-center">
              继续使用本应用即表示您已阅读并同意上述条款
            </p>
          </div>
        </div>
      </Modal>
    </div>
  );
}

function SettingItem({
  icon,
  title,
  description,
  onClick,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="w-full flex items-center gap-4 p-4 rounded-lg bg-[var(--bg-secondary)] hover:bg-[var(--bg-tertiary)] transition-colors text-left"
    >
      <div className="w-10 h-10 rounded-lg bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]">
        {icon}
      </div>
      <div className="flex-1">
        <p className="font-medium text-[var(--text-primary)]">{title}</p>
        <p className="text-sm text-[var(--text-tertiary)]">{description}</p>
      </div>
    </button>
  );
}
