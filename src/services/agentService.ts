import { invoke } from '@tauri-apps/api/core';
import type {
  AgentChatResponse,
  AgentStatusResponse,
  TestConnectionResponse,
} from '../types/agent';

export const agentService = {
  /** 初始化 Agent */
  init: (): Promise<AgentStatusResponse> =>
    invoke<AgentStatusResponse>('agent_init'),

  /** 非流式对话 */
  chat: (message: string): Promise<AgentChatResponse> =>
    invoke<AgentChatResponse>('agent_chat', {
      request: { message },
    }),

  /** 流式对话（通过事件推送） */
  chatStream: (message: string): Promise<void> =>
    invoke<void>('agent_chat_stream', {
      request: { message },
    }),

  /** 清空对话上下文 */
  clearContext: (): Promise<void> =>
    invoke<void>('agent_clear_context'),

  /** 查询 Agent 状态 */
  status: (): Promise<AgentStatusResponse> =>
    invoke<AgentStatusResponse>('agent_status'),

  /** 测试 AI 连接 */
  testConnection: (): Promise<TestConnectionResponse> =>
    invoke<TestConnectionResponse>('agent_test_connection'),
};