use thiserror::Error;

/// Agent 错误类型
///
/// 覆盖 Agent 运行过程中可能遇到的所有错误场景，
/// 包括配置、LLM 调用、工具执行、流式处理、超时等。
#[derive(Debug, Error)]
pub enum AgentError {
    /// 配置错误：API Key 缺失、模型名称无效、base_url 不正确等
    #[error("配置错误: {0}")]
    ConfigError(String),

    /// LLM 调用失败：网络错误、API 返回错误、速率限制等
    #[error("LLM 调用失败: {0}")]
    LlmError(String),

    /// 工具执行失败：工具内部逻辑错误
    #[error("工具执行失败: {0}")]
    ToolError(String),

    /// 工具未找到：前端请求了不存在的工具
    #[error("工具未找到: 工具 '{0}' 未注册")]
    ToolNotFound(String),

    /// 上下文错误：对话历史操作失败
    #[error("上下文错误: {0}")]
    ContextError(String),

    /// 流式处理错误：SSE 解析失败、事件发送失败
    #[error("流式处理错误: {0}")]
    StreamError(String),

    /// 任务正在执行中，无法并发调用
    #[error("任务正在执行中，请等待当前任务完成")]
    TaskInProgress,

    /// 连续失败次数过多，已自动停止
    #[error("连续失败 {0} 次，已自动停止执行。请检查配置和网络连接")]
    ConsecutiveFailuresCount(usize),

    /// 未初始化：Agent 尚未初始化
    #[error("Agent 未初始化，请先调用 agent_init")]
    NotInitialized,

    /// 超时错误：LLM 调用或工具执行超时
    #[error("操作超时: {0}")]
    Timeout(String),

    /// 权限拒绝：用户拒绝了敏感操作
    #[error("权限拒绝: {0}")]
    PermissionDenied(String),

    /// 输入验证错误：用户输入无效
    #[error("输入验证失败: {0}")]
    ValidationError(String),

    /// 速率限制：API 请求过于频繁
    #[error("API 速率限制: {0}。请稍后重试")]
    RateLimit(String),

    /// 网络错误：网络连接失败
    #[error("网络错误: {0}")]
    NetworkError(String),

    /// 未知错误：未分类的错误
    #[error("未知错误: {0}")]
    Unknown(String),
}

impl AgentError {
    /// 判断错误是否可重试
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AgentError::LlmError(_)
                | AgentError::NetworkError(_)
                | AgentError::RateLimit(_)
                | AgentError::Timeout(_)
                | AgentError::StreamError(_)
        )
    }

    /// 获取错误码（用于前端分类）
    pub fn error_code(&self) -> &'static str {
        match self {
            AgentError::ConfigError(_) => "CONFIG_ERROR",
            AgentError::LlmError(_) => "LLM_ERROR",
            AgentError::ToolError(_) => "TOOL_ERROR",
            AgentError::ToolNotFound(_) => "TOOL_NOT_FOUND",
            AgentError::ContextError(_) => "CONTEXT_ERROR",
            AgentError::StreamError(_) => "STREAM_ERROR",
            AgentError::TaskInProgress => "TASK_IN_PROGRESS",
            AgentError::ConsecutiveFailuresCount(_) => "CONSECUTIVE_FAILURES",
            AgentError::NotInitialized => "NOT_INITIALIZED",
            AgentError::Timeout(_) => "TIMEOUT",
            AgentError::PermissionDenied(_) => "PERMISSION_DENIED",
            AgentError::ValidationError(_) => "VALIDATION_ERROR",
            AgentError::RateLimit(_) => "RATE_LIMIT",
            AgentError::NetworkError(_) => "NETWORK_ERROR",
            AgentError::Unknown(_) => "UNKNOWN",
        }
    }
}