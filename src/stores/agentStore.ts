import { create } from 'zustand';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { AgentMessage, ToolCallRecord } from '../types/agent';
import {
  ERROR_CODE_MESSAGES,
} from '../types/agent';
import { agentService } from '../services/agentService';

/**
 * 前端侧去重：将 delta 安全追加到 accumulated，避免重复内容
 * 作为后端去重的补充安全网
 */
function appendDedup(accumulated: string, delta: string): string {
  if (!delta) return accumulated;
  if (!accumulated) {
    // 即使 accumulated 为空，也检测 delta 内部是否重复
    return dedupInternal(delta);
  }

  const accLen = accumulated.length;
  const deltaLen = delta.length;

  // delta 完全被 accumulated 末尾覆盖
  if (deltaLen <= accLen && accumulated.slice(accLen - deltaLen) === delta) {
    return accumulated;
  }

  // 检测重叠：accumulated="好的", delta="的好的" → 保留"好的"
  for (let overlap = Math.min(deltaLen, accLen); overlap > 0; overlap--) {
    if (accumulated.slice(accLen - overlap) === delta.slice(0, overlap)) {
      const remaining = delta.slice(overlap);
      return accumulated + dedupInternal(remaining);
    }
  }

  return accumulated + dedupInternal(delta);
}

function dedupInternal(s: string): string {
  const len = s.length;
  if (len < 2) return s;

  // 尝试所有可能的重复周期
  for (let period = 1; period <= Math.floor(len / 2); period++) {
    if (len % period !== 0) continue;
    const pattern = s.slice(0, period);
    let allMatch = true;
    for (let i = period; i < len; i += period) {
      if (s.slice(i, i + period) !== pattern) { allMatch = false; break; }
    }
    if (allMatch && len > period) return pattern;
  }

  // 检查前半重复
  for (let half = 1; half <= Math.floor(len / 2); half++) {
    if (s.slice(0, half) === s.slice(half, half * 2)) {
      return dedupInternal(s.slice(half));
    }
  }

  return s;
}

/**
 * 从 tool result 中提取纯 HTML 内容
 * 后端可能返回 Rust Debug 格式包装的字符串
 */
function extractHtmlFromResult(raw: string): string {
  // 直接以 HTML 开头
  const trimmed = raw.trim();
  if (trimmed.startsWith('<!DOCTYPE') || trimmed.startsWith('<html')) {
    return trimmed;
  }

  // 从 Rust Debug 格式中提取: UserContent { ... "<html>...</html>" }
  const htmlMatch = trimmed.match(/<!DOCTYPE html>[\s\S]*?<\/html>/i);
  if (htmlMatch) return htmlMatch[0];

  const altMatch = trimmed.match(/<html[^>]*>[\s\S]*?<\/html>/i);
  if (altMatch) return altMatch[0];

  return trimmed;
}

/** 从 generate_html 工具结果中提取 saved_path */
function extractFilePathFromResult(raw: string): string | null {
  const match = raw.match(/saved_path:\s*"([^"]+)"/);
  return match ? match[1] : null;
}

/**
 * 从 Tauri invoke 错误中提取结构化错误码和消息
 * 后端 AgentCommandError 以 JSON 字符串格式返回: {"error_code":"XXX","message":"..."}
 */
function parseCommandError(e: unknown): { errorCode: string; message: string } {
  const errorStr = String(e);
  try {
    const parsed = JSON.parse(errorStr);
    if (parsed && typeof parsed.error_code === 'string') {
      return {
        errorCode: parsed.error_code,
        message: parsed.message || errorStr,
      };
    }
  } catch {
    // 不是 JSON，回退到纯文本
  }
  return { errorCode: 'UNKNOWN', message: errorStr };
}

interface StreamEvent {
  type: string;
  message_id: string;
  content?: string;
  tool_name?: string;
  tool_args?: string;
  tool_result?: string;
  error?: string;
  error_code?: string;
  content_type?: string;
}

interface AgentStore {
  // 状态
  initialized: boolean;
  isStreaming: boolean;
  isLoading: boolean;
  messages: AgentMessage[];
  error: string | null;
  errorCode: string | null;

  // 流式状态
  streamingContent: string;
  streamingToolName: string | null;
  streamingToolResult: string | null;
  streamingToolResultType: string | null;
  streamingToolCalls: ToolCallRecord[];

  // 重试状态
  lastFailedMessage: string | null;

  // 操作
  init: () => Promise<void>;
  sendMessage: (content: string) => Promise<void>;
  sendStreamMessage: (content: string) => Promise<void>;
  retryLastMessage: () => Promise<void>;
  clearContext: () => Promise<void>;

  // 内部
  setError: (error: string | null, code?: string | null) => void;
  addMessage: (message: AgentMessage) => void;
  handleStreamEvent: (event: StreamEvent) => void;
  _unlisten: UnlistenFn | null;
  _setupListener: () => Promise<void>;
}

const generateId = () => `msg_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;

export const useAgentStore = create<AgentStore>((set, get) => ({
  initialized: false,
  isStreaming: false,
  isLoading: false,
  messages: [],
  error: null,
  errorCode: null,
  streamingContent: '',
  streamingToolName: null,
  streamingToolResult: null,
  streamingToolResultType: null,
  streamingToolCalls: [],
  lastFailedMessage: null,
  _unlisten: null,

  _setupListener: async () => {
    const { _unlisten } = get();
    if (_unlisten) return;

    const unlisten = await listen<StreamEvent>('agent-stream-event', (event) => {
      get().handleStreamEvent(event.payload);
    });
    set({ _unlisten: unlisten });
  },

  init: async () => {
    try {
      set({ isLoading: true, error: null, errorCode: null });
      const status = await agentService.init();
      set({
        initialized: status.initialized,
        isLoading: false,
      });
      await get()._setupListener();
    } catch (e) {
      const { errorCode, message } = parseCommandError(e);
      set({
        error: message,
        errorCode,
        isLoading: false,
      });
    }
  },

  sendMessage: async (content: string) => {
    const { isLoading } = get();
    if (isLoading) return;

    const userMsg: AgentMessage = {
      id: generateId(),
      role: 'user',
      content,
      timestamp: Date.now(),
    };

    set((state) => ({
      messages: [...state.messages, userMsg],
      isLoading: true,
      error: null,
      errorCode: null,
    }));

    try {
      const response = await agentService.chat(content);
      const assistantMsg: AgentMessage = {
        id: generateId(),
        role: 'assistant',
        content: response.reply,
        timestamp: Date.now(),
      };

      set((state) => ({
        messages: [...state.messages, assistantMsg],
        isLoading: false,
      }));
    } catch (e) {
      const { errorCode, message } = parseCommandError(e);
      set({
        error: message,
        errorCode,
        isLoading: false,
      });
    }
  },

  sendStreamMessage: async (content: string) => {
    const { isLoading, isStreaming } = get();
    if (isLoading || isStreaming) return;

    const userMsg: AgentMessage = {
      id: generateId(),
      role: 'user',
      content,
      timestamp: Date.now(),
    };

    set((state) => ({
      messages: [...state.messages, userMsg],
      isLoading: true,
      isStreaming: true,
      error: null,
      errorCode: null,
      streamingContent: '',
      streamingToolName: null,
      streamingToolResult: null,
      streamingToolResultType: null,
      streamingToolCalls: [],
      lastFailedMessage: null,
    }));

    try {
      await agentService.chatStream(content);
    } catch (e) {
      // 仅在流式事件处理器尚未处理错误时才设置错误状态
      // 流式事件的 error 类型已在 handleStreamEvent 中处理
      // 如果 handleStreamEvent 已处理（此时 isStreaming 已被设为 false），则不覆盖
      const currentState = get();
      if (currentState.isStreaming) {
        // 流式事件未处理错误，使用 invoke 错误
        const { errorCode, message } = parseCommandError(e);
        set({
          error: message,
          errorCode,
          isLoading: false,
          isStreaming: false,
          lastFailedMessage: content,
        });

        // 如果是 TASK_IN_PROGRESS 错误，自动重置后端卡死状态
        if (errorCode === 'TASK_IN_PROGRESS') {
          try {
            await agentService.resetExecuting();
            // 重置后更新错误提示
            set({
              error: '之前的任务异常终止，已自动重置。请重新发送消息。',
              errorCode: 'TASK_IN_PROGRESS',
            });
          } catch {
            // 重置失败，忽略
          }
        }
      } else {
        // 流式事件已处理，仅保存 lastFailedMessage 以支持重试
        const { errorCode } = parseCommandError(e);
        set({
          lastFailedMessage: errorCode !== 'TASK_IN_PROGRESS' ? content : null,
        });
      }
    }
  },

  retryLastMessage: async () => {
    const { lastFailedMessage, isLoading, isStreaming } = get();
    if (isLoading || isStreaming || !lastFailedMessage) return;

    // 移除最后一条 assistant 消息（如果有的话）
    set((state) => {
      const msgs = [...state.messages];
      // 移除错误导致的空消息
      if (msgs.length > 0 && msgs[msgs.length - 1].role === 'assistant' && msgs[msgs.length - 1].error) {
        msgs.pop();
      }
      return { messages: msgs, error: null, errorCode: null };
    });

    await get().sendStreamMessage(lastFailedMessage);
  },

  handleStreamEvent: (event: StreamEvent) => {
    const { isStreaming } = get();

    switch (event.type) {
      case 'thinking_start':
        set({ streamingContent: '', streamingToolName: null, streamingToolResult: null, streamingToolResultType: null, streamingToolCalls: [] });
        break;

      case 'text_delta':
        if (event.content) {
          set((state) => ({
            streamingContent: appendDedup(state.streamingContent, event.content!),
          }));
        }
        break;

      case 'tool_call_start':
        set({
          streamingToolName: event.tool_name || null,
          streamingToolResult: null,
        });
        break;

      case 'tool_result':
        set((state) => {
          const newCall: ToolCallRecord = {
            toolName: state.streamingToolName || event.tool_name || 'unknown',
            toolResult: event.tool_result || '',
            resultType: event.content_type || null,
          };
          return {
            streamingToolResult: event.tool_result || null,
            streamingToolResultType: event.content_type || null,
            streamingToolName: null,
            streamingToolCalls: [...state.streamingToolCalls, newCall],
          };
        });
        break;

      case 'done':
        if (isStreaming) {
          const finalContent = event.content || get().streamingContent;
          const toolCalls = [...get().streamingToolCalls];
          // 从工具调用中提取 HTML 报告
          const htmlCall = toolCalls.find(tc => tc.resultType === 'html');
          const htmlReport = htmlCall ? extractHtmlFromResult(htmlCall.toolResult) : undefined;
          // 从 generate_html 结果中提取文件路径
          const generateHtmlCall = toolCalls.find(tc => tc.toolName === 'generate_html');
          const reportFilePath = generateHtmlCall ? extractFilePathFromResult(generateHtmlCall.toolResult) ?? undefined : undefined;

          const assistantMsg: AgentMessage = {
            id: generateId(),
            role: 'assistant',
            content: finalContent,
            timestamp: Date.now(),
            toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
            htmlReport,
            reportFilePath,
          };

          set((state) => ({
            messages: [...state.messages, assistantMsg],
            isLoading: false,
            isStreaming: false,
            streamingContent: '',
            streamingToolName: null,
            streamingToolResult: null,
            streamingToolResultType: null,
            streamingToolCalls: [],
            lastFailedMessage: null,
          }));
        }
        break;

      case 'error': {
        const errorCode = event.error_code || 'UNKNOWN';
        const errorMsg = event.error || '未知错误';
        const friendlyMsg = ERROR_CODE_MESSAGES[errorCode]
          ? `${ERROR_CODE_MESSAGES[errorCode]}\n\n${errorMsg}`
          : errorMsg;

        // 如果流式中有部分内容，保存为错误消息
        const streamingContent = get().streamingContent;
        if (streamingContent) {
          const errorAssistantMsg: AgentMessage = {
            id: generateId(),
            role: 'assistant',
            content: streamingContent + '\n\n---\n⚠️ ' + friendlyMsg,
            timestamp: Date.now(),
            error: true,
          };
          set((state) => ({
            messages: [...state.messages, errorAssistantMsg],
            isLoading: false,
            isStreaming: false,
            streamingContent: '',
            streamingToolName: null,
            streamingToolResult: null,
            streamingToolResultType: null,
            streamingToolCalls: [],
          }));
        } else {
          set({
            error: friendlyMsg,
            errorCode,
            isLoading: false,
            isStreaming: false,
            streamingToolCalls: [],
          });
        }
        break;
      }
    }
  },

  clearContext: async () => {
    try {
      await agentService.clearContext();
      set({
        messages: [],
        error: null,
        errorCode: null,
        lastFailedMessage: null,
      });
    } catch (e) {
      const { errorCode, message } = parseCommandError(e);
      set({ error: message, errorCode });
    }
  },

  setError: (error: string | null, code?: string | null) =>
    set({ error, errorCode: code || null }),
  addMessage: (message: AgentMessage) =>
    set((state) => ({ messages: [...state.messages, message] })),
}));