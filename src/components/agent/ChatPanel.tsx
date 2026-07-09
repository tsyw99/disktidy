import { useState, useRef, useEffect } from 'react';
import { Send, Trash2, Loader2, Wrench, CheckCircle2, RotateCcw, AlertTriangle, Settings, ChevronDown, ChevronRight, StopCircle } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAgentStore } from '../../stores/agentStore';
import { agentService } from '../../services/agentService';
import MarkdownRenderer from './MarkdownRenderer';
import HtmlReportCard from './HtmlReportCard';
import type { AgentMessage, ToolCallRecord } from '../../types/agent';
import {
  ERROR_CODE_MESSAGES,
  RETRYABLE_ERROR_CODES,
} from '../../types/agent';

interface ChatPanelProps {
  onOpenSettings?: () => void;
}

/** 单条工具调用展示 */
function ToolCallItem({ call, isLast }: { call: ToolCallRecord; isLast: boolean }) {
  const [expanded, setExpanded] = useState(false);
  const isHtml = call.resultType === 'html';
  const preview = isHtml ? 'HTML 报告' : (call.toolResult.length > 80 ? call.toolResult.slice(0, 80) + '...' : call.toolResult);

  return (
    <div className="text-xs">
      <button
        onClick={() => setExpanded(!expanded)}
        className="flex items-center gap-1.5 w-full text-left py-0.5 hover:bg-white/50 dark:hover:bg-gray-600/30 rounded px-1 -mx-1 transition-colors"
      >
        {expanded ? <ChevronDown className="w-3 h-3 flex-shrink-0" /> : <ChevronRight className="w-3 h-3 flex-shrink-0" />}
        <CheckCircle2 className="w-3 h-3 text-green-500 flex-shrink-0" />
        <span className="font-medium text-gray-600 dark:text-gray-300 truncate">{call.toolName}</span>
        <span className="text-gray-400 dark:text-gray-500 truncate">— {preview}</span>
      </button>
      {expanded && (
        <div className={`ml-5 mt-0.5 p-1.5 rounded bg-white/60 dark:bg-gray-600/40 max-h-32 overflow-y-auto whitespace-pre-wrap break-all text-gray-500 dark:text-gray-400 ${isLast ? '' : 'border-l-2 border-green-200 dark:border-green-800'}`}>
          {call.toolResult}
        </div>
      )}
    </div>
  );
}

/** 助手消息气泡中的工具调用摘要 */
function ToolCallsSummary({ toolCalls }: { toolCalls: ToolCallRecord[] }) {
  const [showTools, setShowTools] = useState(false);

  return (
    <div className="mt-2 pt-2 border-t border-gray-200 dark:border-gray-600">
      <button
        onClick={() => setShowTools(!showTools)}
        className="flex items-center gap-1 text-xs text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
      >
        {showTools ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
        <Wrench className="w-3 h-3" />
        <span>已调用 {toolCalls.length} 个工具</span>
      </button>
      <AnimatePresence>
        {showTools && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden"
          >
            <div className="mt-1.5 space-y-0.5">
              {toolCalls.map((call, idx) => (
                <ToolCallItem key={idx} call={call} isLast={idx === toolCalls.length - 1} />
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

/** 助手消息组件 */
function AssistantBubble({ message }: { message: AgentMessage }) {
  return (
    <div className="bg-gray-100 dark:bg-gray-700 rounded-2xl rounded-bl-md px-4 py-2.5 max-w-[85%] text-sm">
      <MarkdownRenderer content={message.content} />
      {message.toolCalls && message.toolCalls.length > 0 && (
        <ToolCallsSummary toolCalls={message.toolCalls} />
      )}
      {(message.htmlReport || message.reportFilePath) && (
        <div className="mt-2">
          <HtmlReportCard
            htmlContent={message.htmlReport}
            filePath={message.reportFilePath}
          />
        </div>
      )}
    </div>
  );
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
    streamingToolCalls,
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

  /** 强制停止：重置前端状态 + 后端 is_executing 标志 */
  const handleForceStop = async () => {
    // 通过 store 内部方法清除看门狗和重置状态
    useAgentStore.getState()._clearWatchdog();
    useAgentStore.setState({
      isLoading: false,
      isStreaming: false,
      streamingContent: '',
      streamingToolName: null,
      streamingToolResult: null,
      streamingToolResultType: null,
      streamingToolCalls: [],
      error: '已手动停止 AI 响应。',
      errorCode: null,
    });
    // 异步重置后端
    try {
      await agentService.resetExecuting();
    } catch { /* 忽略 */ }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  // auto-resize textarea
  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    const el = e.target;
    el.style.height = 'auto';
    el.style.height = Math.min(el.scrollHeight, 160) + 'px';
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
            <p className="text-sm">我可以帮你扫描磁盘、分析文件、整理目录</p>
            <p className="text-sm">试试问我：</p>
            <p className="text-xs mt-1 text-gray-400 dark:text-gray-600">
              "帮我查看 E:\项目 目录下有什么文件"<br />
              "帮我整理一下桌面"
            </p>
          </div>
        )}

        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}
          >
            {msg.role === 'user' ? (
              <div className="bg-blue-500 text-white rounded-2xl rounded-br-md px-4 py-2.5 max-w-[80%] text-sm leading-relaxed">
                <div className="whitespace-pre-wrap break-words">{msg.content}</div>
              </div>
            ) : (
              <AssistantBubble message={msg} />
            )}
          </div>
        ))}

        {/* 流式：工具调用状态（执行中） */}
        {isStreaming && streamingToolName && (
          <div className="flex justify-start">
            <div className="bg-amber-50 dark:bg-amber-900/20 rounded-2xl rounded-bl-md px-4 py-2.5 max-w-[80%]">
              <div className="flex items-center gap-2 text-sm text-amber-700 dark:text-amber-300">
                <Wrench className="w-4 h-4 animate-pulse" />
                <span className="font-medium">正在执行: {streamingToolName}</span>
              </div>
              <div className="text-xs text-amber-600 dark:text-amber-400 mt-1">
                正在处理中，请稍候...
              </div>
            </div>
          </div>
        )}

        {/* 流式：工具调用结果（执行完成，非 HTML） */}
        {isStreaming && streamingToolResult && !streamingToolCalls.some(c => c.toolResult === streamingToolResult && c.resultType === 'html') && (
          <div className="flex justify-start max-w-[80%]">
            <div className="bg-green-50 dark:bg-green-900/20 rounded-2xl rounded-bl-md px-4 py-2.5 w-full">
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

        {/* 流式：实时文本 */}
        {isStreaming && streamingContent && (
          <div className="flex justify-start">
            <div className="bg-gray-100 dark:bg-gray-700 rounded-2xl rounded-bl-md px-4 py-2.5 max-w-[85%]">
              <MarkdownRenderer content={streamingContent} />
              <span className="dot-typing"><span /><span /><span /></span>
            </div>
          </div>
        )}

        {/* 加载指示器 */}
        {isLoading && !isStreaming && !streamingContent && (
          <div className="flex justify-start">
            <div className="bg-gray-100 dark:bg-gray-700 rounded-2xl rounded-bl-md px-4 py-3">
              <Loader2 className="w-4 h-4 animate-spin text-gray-400" />
            </div>
          </div>
        )}

        {isStreaming && !streamingContent && !streamingToolName && !streamingToolResult && (
          <div className="flex justify-start">
            <div className="bg-gray-100 dark:bg-gray-700 rounded-2xl rounded-bl-md px-4 py-3">
              <div className="flex items-center gap-2">
                <Loader2 className="w-4 h-4 animate-spin text-gray-400" />
                <span className="text-sm text-gray-500 dark:text-gray-400">AI 正在思考...</span>
              </div>
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
            onChange={handleInput}
            onKeyDown={handleKeyDown}
            placeholder="输入消息，如：帮我查看E:\\项目目录，帮我整理桌面..."
            disabled={isLoading}
            className="flex-1 resize-none rounded-xl border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-4 py-2.5 text-sm text-gray-800 dark:text-gray-200 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-400 focus:border-transparent disabled:opacity-50"
            style={{ maxHeight: 160 }}
          />
          {isLoading && (
            <button
              onClick={handleForceStop}
              className="p-2.5 rounded-xl bg-red-500 text-white hover:bg-red-600 transition-colors flex-shrink-0"
              title="强制停止 AI 响应"
            >
              <StopCircle className="w-4 h-4" />
            </button>
          )}
          <button
            onClick={handleSend}
            disabled={!input.trim() || isLoading}
            className="p-2.5 rounded-xl bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex-shrink-0"
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
