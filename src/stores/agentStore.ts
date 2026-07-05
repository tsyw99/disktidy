import { create } from 'zustand';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { AgentMessage } from '../types/agent';
import {
  ERROR_CODE_MESSAGES,
} from '../types/agent';
import { agentService } from '../services/agentService';

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
        set({ streamingContent: '', streamingToolName: null, streamingToolResult: null });
        break;

      case 'text_delta':
        if (event.content) {
          set((state) => ({
            streamingContent: state.streamingContent + event.content,
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
        set({
          streamingToolResult: event.tool_result || null,
          streamingToolName: null,
        });
        break;

      case 'done':
        if (isStreaming) {
          const finalContent = event.content || get().streamingContent;
          const assistantMsg: AgentMessage = {
            id: generateId(),
            role: 'assistant',
            content: finalContent,
            timestamp: Date.now(),
          };

          set((state) => ({
            messages: [...state.messages, assistantMsg],
            isLoading: false,
            isStreaming: false,
            streamingContent: '',
            streamingToolName: null,
            streamingToolResult: null,
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
          }));
        } else {
          set({
            error: friendlyMsg,
            errorCode,
            isLoading: false,
            isStreaming: false,
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