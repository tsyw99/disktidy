import { Loader2 } from 'lucide-react';
import { useAgentStore } from '../../stores/agentStore';

export default function LlmConfigPanel() {
  const { initialized, isLoading, init, error } = useAgentStore();

  const handleInit = async () => {
    await init();
  };

  return (
    <div className="border-b border-gray-200 dark:border-gray-700">
      {/* 状态栏 */}
      <div className="flex items-center justify-between px-4 py-2">
        <div className="flex items-center gap-2">
          <div
            className={`w-2 h-2 rounded-full ${
              initialized ? 'bg-green-500' : 'bg-gray-400'
            }`}
          />
          <span className="text-xs text-gray-500 dark:text-gray-400">
            {initialized ? '已连接' : '未连接'}
          </span>
          {error && (
            <span className="text-xs text-red-500 ml-2 truncate max-w-[200px]">{error}</span>
          )}
        </div>

        <div className="flex items-center gap-2">
          {!initialized && (
            <button
              onClick={handleInit}
              disabled={isLoading}
              className="px-3 py-1 text-xs bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 transition-colors flex items-center gap-1"
            >
              {isLoading && <Loader2 className="w-3 h-3 animate-spin" />}
              连接
            </button>
          )}
        </div>
      </div>
    </div>
  );
}