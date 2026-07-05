use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{AppHandle, State};
use serde::{Deserialize, Serialize};

use crate::modules::agent::{AgentManager, AgentConfig, AgentError};

/// 结构化命令错误（包含 error_code，前端可据此分类处理）
#[derive(Debug, Clone, Serialize)]
pub struct AgentCommandError {
    pub error_code: String,
    pub message: String,
}

impl From<AgentError> for AgentCommandError {
    fn from(e: AgentError) -> Self {
        Self {
            error_code: e.error_code().to_string(),
            message: e.to_string(),
        }
    }
}

impl std::fmt::Display for AgentCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Tauri 会将 Err 转为字符串传给前端，用 JSON 格式以便前端解析
        write!(f, "{}", serde_json::to_string(self).unwrap_or_else(|_| self.message.clone()))
    }
}

impl std::error::Error for AgentCommandError {}

/// Agent 状态管理（Tauri 托管状态）
pub struct AgentState {
    pub manager: RwLock<Option<Arc<AgentManager>>>,
}

impl AgentState {
    pub fn new() -> Self {
        Self {
            manager: RwLock::new(None),
        }
    }
}

/// 对话请求
#[derive(Deserialize)]
pub struct AgentChatRequest {
    pub message: String,
}

/// 非流式对话响应
#[derive(Serialize)]
pub struct AgentChatResponse {
    pub reply: String,
    pub is_executing: bool,
}

/// Agent 状态响应
#[derive(Serialize)]
pub struct AgentStatusResponse {
    pub initialized: bool,
    pub is_executing: bool,
    pub model: Option<String>,
    pub provider: Option<String>,
}

/// 初始化 Agent（加载配置、创建管理器）
#[tauri::command]
pub async fn agent_init(
    app: AppHandle,
    state: State<'_, AgentState>,
) -> Result<AgentStatusResponse, AgentCommandError> {
    let config = AgentConfig::from_settings(&app)
        .await
        .map_err(AgentCommandError::from)?;

    // 验证配置（API Key 为空时报错）
    config.validate().map_err(AgentCommandError::from)?;

    let manager = AgentManager::new(app, config);
    let is_executing = manager.is_executing();
    let model = manager.model_name().await;
    let provider = manager.provider_name().await;

    let mut guard = state.manager.write().await;
    *guard = Some(Arc::new(manager));

    Ok(AgentStatusResponse {
        initialized: true,
        is_executing,
        model: Some(model),
        provider: Some(provider),
    })
}

/// 非流式对话
#[tauri::command]
pub async fn agent_chat(
    app: AppHandle,
    state: State<'_, AgentState>,
    request: AgentChatRequest,
) -> Result<AgentChatResponse, AgentCommandError> {
    // 确保已初始化
    {
        let guard = state.manager.read().await;
        if guard.is_none() {
            drop(guard);
            agent_init(app.clone(), state.clone()).await?;
        }
    }

    let guard = state.manager.read().await;
    let manager = guard.as_ref().ok_or_else(|| AgentCommandError {
        error_code: "NOT_INITIALIZED".to_string(),
        message: "Agent 未初始化".to_string(),
    })?;

    match manager.chat(&request.message).await {
        Ok(reply) => Ok(AgentChatResponse {
            reply,
            is_executing: false,
        }),
        Err(e) => Err(AgentCommandError::from(e)),
    }
}

/// 流式对话（通过 Tauri 事件推送）
#[tauri::command]
pub async fn agent_chat_stream(
    app: AppHandle,
    state: State<'_, AgentState>,
    request: AgentChatRequest,
) -> Result<(), AgentCommandError> {
    // 确保已初始化
    {
        let guard = state.manager.read().await;
        if guard.is_none() {
            drop(guard);
            agent_init(app.clone(), state.clone()).await?;
        }
    }

    let guard = state.manager.read().await;
    let manager = guard.as_ref().ok_or_else(|| AgentCommandError {
        error_code: "NOT_INITIALIZED".to_string(),
        message: "Agent 未初始化".to_string(),
    })?;

    // 使用流式调用 — 事件由 bridge_stream_to_tauri 自动推送
    // 注意：chat_stream 内部已通过 Tauri 事件发送了带 error_code 的错误事件
    // 这里返回结构化错误，前端可从中提取 error_code
    manager.chat_stream(&request.message).await
        .map_err(AgentCommandError::from)?;

    Ok(())
}

/// 清空对话上下文
#[tauri::command]
pub async fn agent_clear_context(
    state: State<'_, AgentState>,
) -> Result<(), AgentCommandError> {
    let guard = state.manager.read().await;
    if let Some(manager) = guard.as_ref() {
        manager.clear_context().await;
    }
    Ok(())
}

/// 查询 Agent 状态
#[tauri::command]
pub async fn agent_status(
    state: State<'_, AgentState>,
) -> Result<AgentStatusResponse, AgentCommandError> {
    let guard = state.manager.read().await;
    match guard.as_ref() {
        Some(manager) => Ok(AgentStatusResponse {
            initialized: true,
            is_executing: manager.is_executing(),
            model: Some(manager.model_name().await),
            provider: Some(manager.provider_name().await),
        }),
        None => Ok(AgentStatusResponse {
            initialized: false,
            is_executing: false,
            model: None,
            provider: None,
        }),
    }
}

/// 强制重置 is_executing 标志（用于修复卡死状态）
#[tauri::command]
pub async fn agent_reset_executing(
    state: State<'_, AgentState>,
) -> Result<(), AgentCommandError> {
    let guard = state.manager.read().await;
    if let Some(manager) = guard.as_ref() {
        manager.force_reset_executing();
    }
    Ok(())
}

/// 测试 AI 连接
#[derive(Serialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn agent_test_connection(
    app: AppHandle,
) -> Result<TestConnectionResponse, AgentCommandError> {
    let config = AgentConfig::from_settings(&app)
        .await
        .map_err(AgentCommandError::from)?;

    // 验证配置
    if let Err(e) = config.validate() {
        return Ok(TestConnectionResponse {
            success: false,
            message: e.to_string(),
        });
    }

    // 尝试创建客户端并发起简单对话验证连接
    let base_url = config.get_base_url();
    let client = rig_core::providers::openai::CompletionsClient::builder()
        .api_key(&config.api_key)
        .base_url(&base_url)
        .build()
        .map_err(|e| AgentCommandError {
            error_code: "CONFIG_ERROR".to_string(),
            message: format!("创建客户端失败: {}", e),
        })?;

    // 用简单对话请求验证 API Key 和 URL 是否有效
    use rig_core::client::completion::CompletionClient;
    use rig_core::completion::Prompt;
    let agent = client
        .agent(&config.model)
        .build();

    match agent.prompt("ping").await {
        Ok(_) => Ok(TestConnectionResponse {
            success: true,
            message: format!("连接成功！提供商: {}, 模型: {}", config.provider, config.model),
        }),
        Err(e) => Ok(TestConnectionResponse {
            success: false,
            message: format!("连接失败: {}", e),
        }),
    }
}