use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tauri::{AppHandle, Emitter};
use log::{info, debug, warn, error};

use rig_core::providers::openai;
use rig_core::completion::Prompt;
use rig_core::completion::message::Message;
use rig_core::streaming::StreamingPrompt;
use rig_core::client::completion::CompletionClient;

use crate::modules::agent::config::AgentConfig;
use crate::modules::agent::context::ConversationContext;
use crate::modules::agent::error::AgentError;
use crate::modules::agent::prompts::SYSTEM_PROMPT;
use crate::modules::agent::stream_bridge::bridge_stream_to_tauri;
use crate::modules::agent::tools::{
    DiskScanTool, FileClassifierTool, LargeFileTool, CleanerTool,
    AppCacheTool, SoftwareResidueTool, GarbageAnalyzerTool,
    FileSearchTool, FileDeleteTool,
};

/// 最大重试次数
const MAX_RETRIES: u32 = 2;
/// 初始重试延迟（毫秒）
const RETRY_DELAY_MS: u64 = 1000;
/// 最大连续失败次数
const MAX_CONSECUTIVE_FAILURES: u32 = 3;

/// is_executing 标志的 RAII 守卫
/// 确保即使发生 panic 或 future 被 cancel，标志也会被重置
struct ExecutingGuard {
    flag: Arc<AtomicBool>,
}

impl ExecutingGuard {
    fn new(flag: Arc<AtomicBool>) -> Option<Self> {
        // 使用 compare_exchange 确保原子性地检查并设置
        match flag.compare_exchange(
            false, // 期望当前值为 false
            true,  // 设置为 true
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => {
                debug!("is_executing 标志已设置 (guard 创建)");
                Some(Self { flag })
            }
            Err(_) => {
                warn!("is_executing 标志已被占用，无法创建 guard");
                None
            }
        }
    }
}

impl Drop for ExecutingGuard {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::SeqCst);
        debug!("is_executing 标志已重置 (guard 释放)");
    }
}

pub struct AgentManager {
    app: AppHandle,
    config: Arc<RwLock<AgentConfig>>,
    client: Arc<RwLock<Option<openai::CompletionsClient>>>,
    context: Arc<RwLock<ConversationContext>>,
    is_executing: Arc<AtomicBool>,
    consecutive_failures: Arc<AtomicU32>,
}

impl AgentManager {
    pub fn new(app: AppHandle, config: AgentConfig) -> Self {
        info!(
            "AgentManager 初始化: provider={}, model={}",
            config.provider, config.model
        );
        Self {
            app,
            config: Arc::new(RwLock::new(config)),
            client: Arc::new(RwLock::new(None)),
            context: Arc::new(RwLock::new(ConversationContext::new(20))),
            is_executing: Arc::new(AtomicBool::new(false)),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn is_executing(&self) -> bool {
        self.is_executing.load(Ordering::SeqCst)
    }

    /// 强制重置 is_executing 标志（用于前端修复卡死状态）
    pub fn force_reset_executing(&self) {
        if self.is_executing.load(Ordering::SeqCst) {
            warn!("强制重置 is_executing 标志（可能之前任务异常终止）");
            self.is_executing.store(false, Ordering::SeqCst);
        }
    }

    pub async fn model_name(&self) -> String {
        self.config.read().await.model.clone()
    }

    pub async fn provider_name(&self) -> String {
        self.config.read().await.provider.clone()
    }

    /// 更新配置（用于 settings 变更后重新初始化）
    pub async fn update_config(&self, config: AgentConfig) {
        info!(
            "更新 Agent 配置: provider={}, model={}",
            config.provider, config.model
        );
        *self.config.write().await = config;
        // 使缓存失效，下次请求时重建客户端
        *self.client.write().await = None;
    }

    pub async fn clear_context(&self) {
        info!("清空对话上下文");
        let mut ctx = self.context.write().await;
        ctx.clear();
    }

    /// 非流式对话
    pub async fn chat(&self, user_input: &str) -> Result<String, AgentError> {
        // 使用 RAII guard 确保 is_executing 在任何退出路径下都会被重置
        let _guard = match ExecutingGuard::new(self.is_executing.clone()) {
            Some(g) => g,
            None => {
                warn!("尝试在任务执行中发起新对话");
                return Err(AgentError::TaskInProgress);
            }
        };

        info!("开始非流式对话: {}", truncate_for_log(user_input));

        let result = self.execute_with_retry(|| self.execute_chat(user_input)).await;

        self.handle_result(result, user_input).await
    }

    /// 流式对话（通过 Tauri 事件推送）
    pub async fn chat_stream(&self, user_input: &str) -> Result<String, AgentError> {
        // 使用 RAII guard 确保 is_executing 在任何退出路径下都会被重置
        let _guard = match ExecutingGuard::new(self.is_executing.clone()) {
            Some(g) => g,
            None => {
                warn!("尝试在任务执行中发起流式对话");
                return Err(AgentError::TaskInProgress);
            }
        };

        info!("开始流式对话: {}", truncate_for_log(user_input));

        let message_id = uuid::Uuid::new_v4().to_string();
        debug!("消息 ID: {}", message_id);

        // 发送 thinking 开始事件
        let start_payload = serde_json::json!({
            "type": "thinking_start",
            "message_id": message_id,
        });
        let _ = self.app.emit("agent-stream-event", &start_payload);

        let result = self.execute_with_retry(|| {
            self.execute_chat_stream(user_input, &message_id)
        }).await;

        // _guard 在此作用域结束时 Drop，自动重置 is_executing = false

        match result {
            Ok(response) => {
                self.consecutive_failures.store(0, Ordering::SeqCst);
                let mut ctx = self.context.write().await;
                ctx.add_user_message(user_input.to_string());
                ctx.add_assistant_message(response.clone());
                info!("流式对话完成，响应长度: {} 字符", response.len());
                Ok(response)
            }
            Err(e) => {
                error!("流式对话失败: {}", e);
                let error_payload = serde_json::json!({
                    "type": "error",
                    "message_id": message_id,
                    "error": e.to_string(),
                    "error_code": e.error_code(),
                });
                let _ = self.app.emit("agent-stream-event", &error_payload);

                let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
                if failures >= MAX_CONSECUTIVE_FAILURES {
                    error!("连续失败 {} 次，停止执行", failures);
                    return Err(AgentError::ConsecutiveFailuresCount(failures as usize));
                }
                Err(e)
            }
        }
    }

    /// 处理非流式结果
    async fn handle_result(
        &self,
        result: Result<String, AgentError>,
        user_input: &str,
    ) -> Result<String, AgentError> {
        match result {
            Ok(response) => {
                self.consecutive_failures.store(0, Ordering::SeqCst);
                let mut ctx = self.context.write().await;
                ctx.add_user_message(user_input.to_string());
                ctx.add_assistant_message(response.clone());
                info!("非流式对话完成，响应长度: {} 字符", response.len());
                Ok(response)
            }
            Err(e) => {
                error!("非流式对话失败: {}", e);
                let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
                if failures >= MAX_CONSECUTIVE_FAILURES {
                    error!("连续失败 {} 次，停止执行", failures);
                    return Err(AgentError::ConsecutiveFailuresCount(failures as usize));
                }
                Err(e)
            }
        }
    }

    /// 带重试的执行器
    async fn execute_with_retry<F, Fut>(&self, f: F) -> Result<String, AgentError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<String, AgentError>>,
    {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let delay = Duration::from_millis(RETRY_DELAY_MS * (2u64.pow(attempt - 1)));
                debug!("重试第 {} 次，等待 {:?}", attempt, delay);
                tokio::time::sleep(delay).await;
            }

            match f().await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    if !e.is_retryable() {
                        return Err(e);
                    }
                    warn!("第 {} 次尝试失败 (可重试): {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AgentError::Unknown("重试耗尽".to_string())))
    }

    /// 非流式执行
    async fn execute_chat(&self, user_input: &str) -> Result<String, AgentError> {
        let client = self.create_client().await?;
        let model = self.config.read().await.model.clone();
        debug!("创建 LLM 客户端成功");

        let agent = client
            .agent(&model)
            .preamble(SYSTEM_PROMPT)
            .tool(DiskScanTool::new(self.app.clone()))
            .tool(FileClassifierTool::new())
            .tool(LargeFileTool::new(self.app.clone()))
            .tool(CleanerTool::new(self.app.clone()))
            .tool(AppCacheTool::new(self.app.clone()))
            .tool(SoftwareResidueTool::new(self.app.clone()))
            .tool(GarbageAnalyzerTool::new())
            .tool(FileSearchTool::new())
            .tool(FileDeleteTool::new())
            .build();

        debug!("构建 Agent 完成，已注册 9 个工具");

        let context = self.context.read().await;
        let history = context.get_messages();
        let history_len = history.len();
        drop(context);
        debug!("加载对话历史: {} 条消息", history_len);

        let response: String = agent
            .prompt(user_input)
            .with_history(history)
            .await
            .map_err(|e| classify_llm_error(e))?;

        Ok(response)
    }

    /// 流式执行
    async fn execute_chat_stream(
        &self,
        user_input: &str,
        message_id: &str,
    ) -> Result<String, AgentError> {
        let client = self.create_client().await?;
        let model = self.config.read().await.model.clone();
        debug!("创建流式 LLM 客户端成功");

        let agent = client
            .agent(&model)
            .preamble(SYSTEM_PROMPT)
            .tool(DiskScanTool::new(self.app.clone()))
            .tool(FileClassifierTool::new())
            .tool(LargeFileTool::new(self.app.clone()))
            .tool(CleanerTool::new(self.app.clone()))
            .tool(AppCacheTool::new(self.app.clone()))
            .tool(SoftwareResidueTool::new(self.app.clone()))
            .tool(GarbageAnalyzerTool::new())
            .tool(FileSearchTool::new())
            .tool(FileDeleteTool::new())
            .build();

        debug!("构建 Agent 完成，已注册 9 个工具");

        let context = self.context.read().await;
        let history = context.get_messages();
        let history_len = history.len();
        drop(context);
        debug!("加载对话历史: {} 条消息", history_len);

        let prompt_msg = Message::user(user_input);

        let stream = agent
            .stream_prompt(prompt_msg)
            .multi_turn(5)
            .with_history(history)
            .await;

        let full_response = bridge_stream_to_tauri(stream, &self.app, message_id)
            .await
            .map_err(|e| AgentError::StreamError(e))?;

        Ok(full_response)
    }

    async fn create_client(&self) -> Result<openai::CompletionsClient, AgentError> {
        // 尝试从缓存获取
        {
            let guard = self.client.read().await;
            if let Some(client) = guard.as_ref() {
                debug!("从缓存获取 LLM 客户端");
                return Ok(client.clone());
            }
        }

        // 读取配置
        let config = self.config.read().await;
        config.validate()?;
        let api_key = config.api_key.clone();
        let base_url = config.get_base_url();
        drop(config);

        // 创建新客户端（使用 CompletionsClient 以支持 DeepSeek 等 Chat Completions API）
        let client = openai::CompletionsClient::builder()
            .api_key(&api_key)
            .base_url(&base_url)
            .build()
            .map_err(|e| {
                let err_msg = e.to_string();
                if err_msg.contains("timeout") || err_msg.contains("timed out") {
                    AgentError::Timeout(format!("创建客户端超时: {}", err_msg))
                } else if err_msg.contains("connection") || err_msg.contains("resolve") {
                    AgentError::NetworkError(format!("创建客户端网络错误: {}", err_msg))
                } else {
                    AgentError::ConfigError(format!("创建客户端失败: {}", err_msg))
                }
            })?;

        // 缓存客户端
        {
            let mut guard = self.client.write().await;
            *guard = Some(client.clone());
        }

        debug!("创建并缓存 LLM 客户端成功");
        Ok(client)
    }
}

/// 分类 LLM 错误，转换为更具体的 AgentError
fn classify_llm_error(e: rig_core::completion::PromptError) -> AgentError {
    let msg = e.to_string();
    let msg_lower = msg.to_lowercase();

    if msg_lower.contains("rate limit") || msg_lower.contains("too many requests") {
        AgentError::RateLimit(msg)
    } else if msg_lower.contains("timeout") || msg_lower.contains("timed out") {
        AgentError::Timeout(msg)
    } else if msg_lower.contains("connection") || msg_lower.contains("network") || msg_lower.contains("dns") {
        AgentError::NetworkError(msg)
    } else if msg_lower.contains("unauthorized") || msg_lower.contains("invalid api key") {
        AgentError::ConfigError(format!("API Key 无效: {}", msg))
    } else if msg_lower.contains("not found") && msg_lower.contains("model") {
        AgentError::ConfigError(format!("模型不存在: {}", msg))
    } else {
        AgentError::LlmError(msg)
    }
}

/// 截断日志消息（避免日志过长）
fn truncate_for_log(input: &str) -> String {
    if input.len() > 100 {
        format!("{}...", &input[..100])
    } else {
        input.to_string()
    }
}