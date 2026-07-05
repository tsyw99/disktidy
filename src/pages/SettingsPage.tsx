import { useState } from 'react';
import { Settings, Sun, Moon, Monitor, HardDrive, Trash2, Shield, Bell, FolderX, FileCheck, AlertTriangle, Construction, Info, Github, User, X, FolderOpen, Bot, Eye, EyeOff, Wifi, Loader2, CheckCircle2, XCircle } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { useUIStore, useSettingsStore, useSettingsActions } from '../stores';
import { Modal } from '../components/common';
import { APP_VERSION } from '../utils/constants';
import { agentService } from '../services/agentService';
import { settingsService } from '../services/settingsService';

function SettingsPage() {
  const theme = useUIStore((state) => state.theme);
  const { toggleTheme } = useUIStore((state) => state.actions);
  
  const scanSettings = useSettingsStore((state) => state.scanSettings);
  const aiSettings = useSettingsStore((state) => state.aiSettings);
  const { setScanSettings, addExcludePath, removeExcludePath, setAiSettings, resetAiSettings } = useSettingsActions();

  const [activeModal, setActiveModal] = useState<string | null>(null);
  const [showApiKey, setShowApiKey] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  const closeModal = () => {
    setActiveModal(null);
  };

  const showDevelopingToast = () => {
    setActiveModal('developing');
  };

  const handleSelectExcludePath = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择要排除的目录',
      });
      if (selected && typeof selected === 'string') {
        addExcludePath(selected);
      }
    } catch (error) {
      console.error('选择目录失败:', error);
    }
  };

  const handleRemoveExcludePath = (path: string) => {
    removeExcludePath(path);
  };

  const handleSaveScanSettings = () => {
    // 设置已通过响应式更新保存到 store，这里只需关闭弹窗
    closeModal();
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      // 先同步设置到后端
      await settingsService.updatePartial({
        ai_provider: aiSettings.provider,
        ai_api_key: aiSettings.apiKey,
        ai_model: aiSettings.model,
        ai_base_url: aiSettings.baseUrl,
        ai_max_tokens: aiSettings.maxTokens,
        ai_temperature: aiSettings.temperature,
      });
      const result = await agentService.testConnection();
      setTestResult({ success: result.success, message: result.message });
    } catch (e) {
      setTestResult({ success: false, message: String(e) });
    } finally {
      setTesting(false);
    }
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
              icon={<Bot className="w-5 h-5" />}
              title="AI 助手"
              description={`${aiSettings.provider} / ${aiSettings.model}`}
              onClick={() => setActiveModal('aiSettings')}
            />
            <SettingItem
              icon={<HardDrive className="w-5 h-5" />}
              title="磁盘扫描设置"
              description={`已配置 ${scanSettings.excludePaths.length} 个排除目录`}
              onClick={() => setActiveModal('diskScan')}
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
              <p className="text-lg font-semibold text-[var(--text-primary)]">DiskTidy v{APP_VERSION}</p>
              <p>Windows 磁盘清理工具</p>
              <p className="text-[var(--text-tertiary)]">使用 React + Rust 构建</p>
              <p className="text-[var(--text-tertiary)] text-xs">感谢 Magic UI 和 React Bits</p>
            </div>

            <div className="border-t border-[var(--border-color)] pt-4 space-y-3">
              <div className="flex items-center gap-3 text-sm text-[var(--text-secondary)]">
                <User className="w-4 h-4 text-[var(--color-primary)]" />
                <span>作者：踏上云雾</span>
              </div>
              <div className="flex items-center gap-3 text-sm text-[var(--text-secondary)]">
                <Github className="w-4 h-4 text-[var(--color-primary)]" />
                <span>GitHub：@tsyw99</span>
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
          { text: '保存设置', onClick: handleSaveScanSettings, variant: 'primary' },
        ]}
      >
        <div className="space-y-4">
          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3 flex items-center gap-2">
              <FolderX className="w-4 h-4 text-[var(--color-primary)]" />
              排除目录
            </h4>
            <div className="space-y-2 max-h-40 overflow-y-auto">
              {scanSettings.excludePaths.length === 0 ? (
                <p className="text-xs text-[var(--text-tertiary)] py-2">暂无排除目录</p>
              ) : (
                scanSettings.excludePaths.map((path) => (
                  <div key={path} className="flex items-center justify-between p-2 rounded bg-[var(--bg-tertiary)]">
                    <span className="text-xs text-[var(--text-secondary)] font-mono truncate mr-2">{path}</span>
                    <button 
                      onClick={() => handleRemoveExcludePath(path)}
                      className="text-xs text-[var(--text-tertiary)] hover:text-red-500 transition-colors flex-shrink-0"
                    >
                      <X className="w-3.5 h-3.5" />
                    </button>
                  </div>
                ))
              )}
            </div>
            <button 
              onClick={handleSelectExcludePath}
              className="mt-3 w-full px-3 py-2 text-xs bg-[var(--color-primary)] text-white rounded hover:opacity-90 flex items-center justify-center gap-2"
            >
              <FolderOpen className="w-3.5 h-3.5" />
              选择要排除的目录
            </button>
          </div>

          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">扫描选项</h4>
            <div className="space-y-3">
              <label className="flex items-center gap-3 cursor-pointer">
                <input 
                  type="checkbox" 
                  checked={scanSettings.includeHidden}
                  onChange={(e) => setScanSettings({ includeHidden: e.target.checked })}
                  className="w-4 h-4 rounded border-[var(--border-color)]" 
                />
                <span className="text-sm text-[var(--text-secondary)]">扫描隐藏文件</span>
              </label>
              <label className="flex items-center gap-3 cursor-pointer">
                <input 
                  type="checkbox" 
                  checked={scanSettings.includeSystem}
                  onChange={(e) => setScanSettings({ includeSystem: e.target.checked })}
                  className="w-4 h-4 rounded border-[var(--border-color)]" 
                />
                <span className="text-sm text-[var(--text-secondary)]">扫描系统文件</span>
              </label>
            </div>
          </div>
          
          <div className="p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
            <p className="text-xs text-blue-600 dark:text-blue-400">
              提示：排除目录设置仅对磁盘扫描功能生效，专项清理（微信/QQ等）使用独立算法不受此设置影响。
            </p>
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
        visible={activeModal === 'aiSettings'}
        onClose={closeModal}
        title="AI 助手设置"
        size={{ width: 520, maxWidth: '90vw' }}
        animation={{ type: 'scale', duration: 0.25 }}
        overlay={{ opacity: 0.6, blur: true }}
        buttons={[
          { text: '重置', onClick: () => { resetAiSettings(); closeModal(); }, variant: 'ghost' },
          { text: '关闭', onClick: closeModal, variant: 'secondary' },
        ]}
      >
        <div className="space-y-4">
          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">LLM 提供商</h4>
            <select
              value={aiSettings.provider}
              onChange={(e) => { setAiSettings({ provider: e.target.value }); setTestResult(null); }}
              className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-primary)] border border-[var(--border-color)] select-custom"
            >
              <option value="deepseek">DeepSeek</option>
              <option value="openai_compatible">OpenAI 兼容</option>
              <option value="glm">ChatGLM（智谱）</option>
              <option value="kimi">Kimi（月之暗面）</option>
            </select>
          </div>

          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">API Key</h4>
            <div className="relative">
              <input
                type={showApiKey ? 'text' : 'password'}
                value={aiSettings.apiKey}
                onChange={(e) => { setAiSettings({ apiKey: e.target.value }); setTestResult(null); }}
                placeholder="输入 API Key..."
                className="w-full px-3 py-2 pr-10 text-sm rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-primary)] border border-[var(--border-color)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
              />
              <button
                type="button"
                onClick={() => setShowApiKey(!showApiKey)}
                className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                title={showApiKey ? '隐藏 API Key' : '显示 API Key'}
              >
                {showApiKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
            </div>
            <p className="text-xs text-[var(--text-tertiary)] mt-1">
              API Key 将安全存储在你的本地配置文件中
            </p>
          </div>

          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">模型名称</h4>
            <input
              type="text"
              value={aiSettings.model}
              onChange={(e) => { setAiSettings({ model: e.target.value }); setTestResult(null); }}
              placeholder="如 deepseek-chat..."
              className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-primary)] border border-[var(--border-color)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
            />
          </div>

          {aiSettings.provider === 'openai_compatible' && (
            <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
              <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">Base URL</h4>
              <input
                type="text"
                value={aiSettings.baseUrl}
                onChange={(e) => { setAiSettings({ baseUrl: e.target.value }); setTestResult(null); }}
                placeholder="如 https://api.openai.com/v1..."
                className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-primary)] border border-[var(--border-color)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
              />
            </div>
          )}

          <div className="grid grid-cols-2 gap-3">
            <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
              <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">最大 Token</h4>
              <input
                type="number"
                value={aiSettings.maxTokens}
                onChange={(e) => { setAiSettings({ maxTokens: Number(e.target.value) }); setTestResult(null); }}
                className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-primary)] border border-[var(--border-color)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
              />
            </div>
            <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
              <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">温度</h4>
              <input
                type="number"
                step="0.1"
                min="0"
                max="2"
                value={aiSettings.temperature}
                onChange={(e) => { setAiSettings({ temperature: Number(e.target.value) }); setTestResult(null); }}
                className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-primary)] border border-[var(--border-color)] focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)]"
              />
            </div>
          </div>

          {/* 测试结果 */}
          {testResult && (
            <div
              className={`flex items-center gap-2 p-3 rounded-lg text-sm ${
                testResult.success
                  ? 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300 border border-green-200 dark:border-green-800'
                  : 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300 border border-red-200 dark:border-red-800'
              }`}
            >
              {testResult.success ? (
                <CheckCircle2 className="w-4 h-4 flex-shrink-0" />
              ) : (
                <XCircle className="w-4 h-4 flex-shrink-0" />
              )}
              <span className="break-words">{testResult.message}</span>
            </div>
          )}

          {/* 测试连接按钮 */}
          <button
            onClick={handleTestConnection}
            disabled={testing || !aiSettings.apiKey.trim()}
            className="w-full flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-medium rounded-lg border border-[var(--border-color)] text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {testing ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Wifi className="w-4 h-4" />
            )}
            测试连接
          </button>

          <div className="p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
            <p className="text-xs text-blue-600 dark:text-blue-400">
              提示：修改 AI 设置后，需要重新连接 AI 助手才能生效。支持 DeepSeek、GLM、Kimi 等主流大模型。
            </p>
          </div>
        </div>
      </Modal>

      {/* 免责声明 */}
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

          <div className="space-y-3 text-sm text-[var(--text-secondary)]">
            <h4 className="font-medium text-[var(--text-primary)]">应用数据存储位置</h4>
            <div className="space-y-1.5 text-xs font-mono bg-[var(--bg-secondary)] p-3 rounded-lg">
              <p className="text-[var(--text-tertiary)]">• C:\Users\&lt;用户名&gt;\AppData\Roaming\DiskTidy\settings.json</p>
              <p className="text-[var(--text-tertiary)]">• C:\Users\&lt;用户名&gt;\AppData\Local\DiskTidy\app_paths.json</p>
              <p className="text-[var(--text-tertiary)]">• C:\Users\&lt;用户名&gt;\AppData\Local\DiskTidy\app_cache_scan.json</p>
            </div>
            <p className="text-xs text-amber-600 dark:text-amber-400 flex items-center gap-1.5">
              <AlertTriangle className="w-3.5 h-3.5" />
              非必要请勿手动修改以上文件，否则可能导致应用异常
            </p>
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

export default SettingsPage;

