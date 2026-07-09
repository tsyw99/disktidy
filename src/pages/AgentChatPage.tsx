import { useEffect, useState, useCallback } from 'react';
import ChatPanel from '../components/agent/ChatPanel';
import LlmConfigPanel from '../components/agent/LlmConfigPanel';
import { useAgentStore } from '../stores/agentStore';
import { useSettingsStore, useSettingsActions } from '../stores';
import { Modal } from '../components/common';
import { agentService } from '../services/agentService';
import { settingsService } from '../services/settingsService';
import { Eye, EyeOff, Wifi, Loader2, CheckCircle2, XCircle } from 'lucide-react';

export default function AgentChatPage() {
  const { initialized, init } = useAgentStore();
  const aiSettings = useSettingsStore((state) => state.aiSettings);
  const { setAiSettings } = useSettingsActions();
  const [showSettingsModal, setShowSettingsModal] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

  // 进入页面时自动尝试初始化
  useEffect(() => {
    if (!initialized) {
      init().catch(() => {});
    }
  }, []);

  const handleOpenSettings = useCallback(() => {
    setShowSettingsModal(true);
  }, []);

  const handleCloseSettings = useCallback(() => {
    setShowSettingsModal(false);
    setTestResult(null);
  }, []);

  const syncSettingsToBackend = async () => {
    await settingsService.updatePartial({
      ai_provider: aiSettings.provider,
      ai_api_key: aiSettings.apiKey,
      ai_model: aiSettings.model,
      ai_base_url: aiSettings.baseUrl,
      ai_max_tokens: aiSettings.maxTokens,
      ai_temperature: aiSettings.temperature,
    });
  };

  const handleTestConnection = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      await syncSettingsToBackend();
      const result = await agentService.testConnection();
      setTestResult({ success: result.success, message: result.message });
    } catch (e) {
      setTestResult({ success: false, message: String(e) });
    } finally {
      setTesting(false);
    }
  };

  const handleSaveAndConnect = async () => {
    setTestResult(null);
    await syncSettingsToBackend();
    await init();
    setShowSettingsModal(false);
  };

  return (
    <div className="flex flex-col h-full bg-white dark:bg-gray-900">
      <LlmConfigPanel />
      <div className="flex-1 overflow-hidden">
        <ChatPanel onOpenSettings={handleOpenSettings} />
      </div>

      {/* AI 设置模态弹窗 */}
      <Modal
        visible={showSettingsModal}
        onClose={handleCloseSettings}
        title="AI 助手设置"
        size={{ width: 520, maxWidth: '90vw' }}
        animation={{ type: 'scale', duration: 0.25 }}
        overlay={{ opacity: 0.6, blur: true }}
        buttons={[]}
      >
        <div className="space-y-4">
          {/* LLM 提供商 */}
          <div className="p-4 rounded-lg bg-[var(--bg-secondary)]">
            <h4 className="text-sm font-medium text-[var(--text-primary)] mb-3">LLM 提供商</h4>
            <select
              value={aiSettings.provider}
              onChange={(e) => { 
                const newProvider = e.target.value;
                let newModel = aiSettings.model; // 默认保持当前模型
                
                // 根据提供商自动设置推荐的模型
                if (newProvider === 'glm' && !aiSettings.model.startsWith('glm')) {
                  newModel = 'glm-4';
                } else if (newProvider === 'deepseek' && !aiSettings.model.startsWith('deepseek')) {
                  newModel = 'deepseek-chat';
                } else if (newProvider === 'kimi' && !aiSettings.model.startsWith('kimi')) {
                  newModel = 'moonshot-v1-auto';
                } else if (newProvider === 'openai_compatible') {
                  newModel = 'gpt-3.5-turbo'; // 或者保持当前模型
                }
                
                setAiSettings({ provider: newProvider, model: newModel }); 
                setTestResult(null); 
              }}
              className="w-full px-3 py-2 text-sm rounded-lg bg-[var(--bg-tertiary)] text-[var(--text-primary)] border border-[var(--border-color)] select-custom"
            >
              <option value="deepseek">DeepSeek</option>
              <option value="openai_compatible">OpenAI 兼容</option>
              <option value="glm">ChatGLM（智谱）</option>
              <option value="kimi">Kimi（月之暗面）</option>
            </select>
          </div>

          {/* API Key */}
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
          </div>

          {/* 模型名称 */}
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

          {/* Base URL（仅 OpenAI 兼容时显示） */}
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

          {/* 高级选项 */}
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

          {/* 操作按钮 */}
          <div className="flex gap-2 pt-1">
            <button
              onClick={handleTestConnection}
              disabled={testing || !aiSettings.apiKey.trim()}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-medium rounded-lg border border-[var(--border-color)] text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {testing ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Wifi className="w-4 h-4" />
              )}
              测试连接
            </button>
            <button
              onClick={handleSaveAndConnect}
              disabled={!aiSettings.apiKey.trim()}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2.5 text-sm font-medium rounded-lg bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              保存并连接
            </button>
          </div>

          <div className="p-3 rounded-lg bg-blue-500/10 border border-blue-500/20">
            <p className="text-xs text-blue-600 dark:text-blue-400">
              提示：修改 AI 设置后，需要重新连接 AI 助手才能生效。支持 DeepSeek、GLM、Kimi 等主流大模型。
            </p>
          </div>
        </div>
      </Modal>
    </div>
  );
}