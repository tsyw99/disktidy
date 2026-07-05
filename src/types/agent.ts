// Agent 消息类型
export interface AgentMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  error?: boolean;
  retryCount?: number;
}

// Agent 流式事件
export interface AgentStreamEvent {
  type: 'text_delta' | 'tool_call_start' | 'tool_result' | 'done' | 'error' | 'thinking_start';
  message_id: string;
  content?: string;
  tool_name?: string;
  tool_args?: string;
  tool_result?: string;
  error?: string;
  error_code?: string;
}

// Agent 状态
export interface AgentState {
  initialized: boolean;
  isStreaming: boolean;
  isLoading: boolean;
  messages: AgentMessage[];
  error: string | null;
  errorCode: string | null;
}

// Agent 对话响应
export interface AgentChatResponse {
  reply: string;
  is_executing: boolean;
}

// Agent 状态响应
export interface AgentStatusResponse {
  initialized: boolean;
  is_executing: boolean;
  model: string | null;
  provider: string | null;
}

// 测试连接响应
export interface TestConnectionResponse {
  success: boolean;
  message: string;
}

// Tauri 事件名
export const EVENT_AGENT_STREAM = 'agent-stream-event';

// 错误码映射
export const ERROR_CODE_MESSAGES: Record<string, string> = {
  CONFIG_ERROR: '配置错误，请检查 API Key 和模型设置',
  LLM_ERROR: 'AI 服务调用失败',
  TOOL_ERROR: '工具执行失败',
  TOOL_NOT_FOUND: '请求的工具不存在',
  CONTEXT_ERROR: '对话上下文错误',
  STREAM_ERROR: '流式传输中断',
  TASK_IN_PROGRESS: '任务正在执行中，请等待',
  CONSECUTIVE_FAILURES: '连续失败次数过多，已自动停止',
  NOT_INITIALIZED: 'AI 助手未初始化',
  TIMEOUT: '请求超时，请检查网络连接',
  PERMISSION_DENIED: '操作被拒绝',
  VALIDATION_ERROR: '输入验证失败',
  RATE_LIMIT: '请求过于频繁，请稍后重试',
  NETWORK_ERROR: '网络连接失败，请检查网络',
  UNKNOWN: '未知错误',
};

// 可重试的错误码
export const RETRYABLE_ERROR_CODES = [
  'LLM_ERROR',
  'NETWORK_ERROR',
  'RATE_LIMIT',
  'TIMEOUT',
  'STREAM_ERROR',
];