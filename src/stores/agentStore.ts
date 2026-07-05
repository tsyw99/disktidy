import { create } from 'zustand';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { AgentMessage } from '../types/agent';
import {
  ERROR_CODE_MESSAGES,
} from '../types/agent';
import { agentService } from '../services/agentService';

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
      set({
        error: String(e),
        errorCode: 'CONFIG_ERROR',
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
      set({
        error: String(e),
        errorCode: 'UNKNOWN',
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
      const errorStr = String(e);
      set({
        error: errorStr,
        errorCode: 'UNKNOWN',
        isLoading: false,
        isStreaming: false,
        lastFailedMessage: content,
      });
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
      set({ error: String(e), errorCode: 'CONTEXT_ERROR' });
    }
  },

  setError: (error: string | null, code?: string | null) =>
    set({ error, errorCode: code || null }),
  addMessage: (message: AgentMessage) =>
    set((state) => ({ messages: [...state.messages, message] })),
}));