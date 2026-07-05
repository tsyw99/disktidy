import { useState, useRef, useEffect } from 'react';
import { Send, Trash2, Loader2, Wrench, CheckCircle2, RotateCcw, AlertTriangle, Settings } from 'lucide-react';
import { useAgentStore } from '../../stores/agentStore';
import MarkdownRenderer from './MarkdownRenderer';
import {
  ERROR_CODE_MESSAGES,
  RETRYABLE_ERROR_CODES,
} from '../../types/agent';

interface ChatPanelProps {
  onOpenSettings?: () => void;
}

export default function ChatPanel({ onOpenSettings }: ChatPanelProps) {
  const [input, setInput] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const {
    messages,
    isLoading,
    isStreaming,
    streamingContent,
    streamingToolName,
    streamingToolResult,
    error,
    errorCode,
    sendStreamMessage,
    retryLastMessage,
    clearContext,
  } = useAgentStore();

  // 自动滚动到底部
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages, streamingContent, streamingToolName, streamingToolResult]);

  // 自动聚焦输入框
  useEffect(() => {
    if (!isLoading) {
      inputRef.current?.focus();
    }
  }, [isLoading]);

  const handleSend = async () => {
    const trimmed = input.trim();
    if (!trimmed || isLoading) return;
    setInput('');
    await sendStreamMessage(trimmed);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* 顶部工具栏 */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <h2 className="text-lg font-semibold text-gray-800 dark:text-gray-200">AI 助手</h2>
        <div className="flex items-center gap-1">
          <button
            onClick={onOpenSettings}
            className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 dark:text-gray-400 transition-colors"
            title="AI 设置"
          >
            <Settings className="w-4 h-4" />
          </button>
          <button
            onClick={clearContext}
            disabled={isLoading}
            className="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 dark:text-gray-400 transition-colors disabled:opacity-50"
            title="清空对话"
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* 消息列表 */}
      <div className="flex-1 overflow-y-auto px-4 py-4 space-y-4">
        {messages.length === 0 && !isStreaming && (
          <div className="flex flex-col items-center justify-center h-full text-gray-400 dark:text-gray-500">
            <p className="text-lg mb-2">👋 你好！我是 DiskTidy AI 助手</p>
            <p className="text-sm">我可以帮你扫描磁盘、分析文件、清理垃圾</p>
            <p className="text-sm">试试问我："帮我扫描 C 盘"</p>
          </div>
        )}

        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            <div
              className={`max-w-[80%] rounded-2xl px-4 py-2.5 text-sm leading-relaxed ${
                msg.role === 'user'
                  ? 'bg-blue-500 text-white rounded-br-md'
                  : 'bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded-bl-md'
              }`}
            >
              {msg.role === 'user' ? (
                <div className="whitespace-pre-wrap break-words">{msg.content}</div>
              ) : (
                <MarkdownRenderer content={msg.content} />
              )}
            </div>
          </div>
        ))}

        {/* 流式内容：工具调用状态 */}
        {isStreaming && streamingToolName && (
          <div className="flex justify-start">
            <div className="bg-amber-50 dark:bg-amber-900/20 rounded-2xl rounded-bl-md px-4 py-2.5 max-w-[80%]">
              <div className="flex items-center gap-2 text-sm text-amber-700 dark:text-amber-300">
                <Wrench className="w-4 h-4 animate-pulse" />
                <span className="font-medium">正在执行: {streamingToolName}</span>
              </div>
            </div>
          </div>
        )}

        {/* 流式内容：工具结果 */}
        {isStreaming && streamingToolResult && (
          <div className="flex justify-start">
            <div className="bg-green-50 dark:bg-green-900/20 rounded-2xl rounded-bl-md px-4 py-2.5 max-w-[80%]">
              <div className="flex items-center gap-2 text-sm text-green-700 dark:text-green-300">
                <CheckCircle2 className="w-4 h-4" />
                <span className="font-medium">工具执行完成</span>
              </div>
              <div className="text-xs text-green-600 dark:text-green-400 mt-1 max-h-20 overflow-y-auto whitespace-pre-wrap">
                {streamingToolResult}
              </div>
            </div>
          </div>
        )}

        {/* 流式内容：实时文本 */}
        {isStreaming && streamingContent && (
          <div className="flex justify-start">
            <div className="bg-gray-100 dark:bg-gray-700 rounded-2xl rounded-bl-md px-4 py-2.5 max-w-[80%]">
              <MarkdownRenderer content={streamingContent} />
              <span className="inline-block w-1.5 h-4 bg-blue-500 ml-0.5 animate-pulse align-text-bottom" />
            </div>
          </div>
        )}

        {/* 加载指示器（等待首个 token） */}
        {isLoading && !isStreaming && !streamingContent && (
          <div className="flex justify-start">
            <div className="bg-gray-100 dark:bg-gray-700 rounded-2xl rounded-bl-md px-4 py-3">
              <Loader2 className="w-4 h-4 animate-spin text-gray-400" />
            </div>
          </div>
        )}

        {/* 流式加载中但没有内容时 */}
        {isStreaming && !streamingContent && !streamingToolName && (
          <div className="flex justify-start">
            <div className="bg-gray-100 dark:bg-gray-700 rounded-2xl rounded-bl-md px-4 py-3">
              <Loader2 className="w-4 h-4 animate-spin text-gray-400" />
            </div>
          </div>
        )}

        {/* 错误提示 */}
        {error && (
          <div className="flex justify-center">
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl px-4 py-3 max-w-[90%]">
              <div className="flex items-center gap-2 text-red-600 dark:text-red-400 mb-1">
                <AlertTriangle className="w-4 h-4 flex-shrink-0" />
                <span className="font-medium text-sm">
                  {errorCode && ERROR_CODE_MESSAGES[errorCode]
                    ? ERROR_CODE_MESSAGES[errorCode]
                    : '发生错误'}
                </span>
                {errorCode && (
                  <span className="text-xs bg-red-100 dark:bg-red-800/30 px-1.5 py-0.5 rounded">
                    {errorCode}
                  </span>
                )}
              </div>
              <div className="text-xs text-red-500 dark:text-red-400 whitespace-pre-wrap break-words">
                {error}
              </div>
              {errorCode && RETRYABLE_ERROR_CODES.includes(errorCode) && (
                <button
                  onClick={retryLastMessage}
                  className="mt-2 flex items-center gap-1 text-xs text-red-600 dark:text-red-400 hover:text-red-700 dark:hover:text-red-300 transition-colors"
                >
                  <RotateCcw className="w-3 h-3" />
                  重试
                </button>
              )}
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* 输入区域 */}
      <div className="border-t border-gray-200 dark:border-gray-700 p-4">
        <div className="flex items-end gap-2">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="输入消息..."
            rows={1}
            disabled={isLoading}
            className="flex-1 resize-none rounded-xl border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-4 py-2.5 text-sm text-gray-800 dark:text-gray-200 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent disabled:opacity-50"
          />
          <button
            onClick={handleSend}
            disabled={!input.trim() || isLoading}
            className="p-2.5 rounded-xl bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isLoading ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Send className="w-4 h-4" />
            )}
          </button>
        </div>
      </div>
    </div>
  );
}