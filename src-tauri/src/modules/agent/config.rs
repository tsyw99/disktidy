use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use super::error::AgentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// LLM 提供商: "deepseek", "openai_compatible", "glm", "kimi"
    pub provider: String,
    /// API Key
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// Base URL（用于 OpenAI 兼容 API）
    pub base_url: Option<String>,
    /// 最大 Token 数
    pub max_tokens: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: "deepseek".to_string(),
            api_key: String::new(),
            model: "deepseek-chat".to_string(),
            base_url: None,
            max_tokens: Some(4096),
            temperature: Some(0.7),
        }
    }
}

impl AgentConfig {
    /// 从 AppHandle 获取 SettingsManager 并读取 AI 配置
    pub async fn from_settings(app: &AppHandle) -> Result<Self, AgentError> {
        let settings_state = app.try_state::<crate::commands::settings::SettingsState>();

        if let Some(state) = settings_state {
            let config = state.get_ai_config().await;
            Ok(config)
        } else {
            // SettingsState 未注册，返回默认配置
            Ok(Self::default())
        }
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<(), AgentError> {
        if self.api_key.is_empty() {
            return Err(AgentError::ConfigError("API Key 不能为空，请在设置中配置".to_string()));
        }

        if self.model.is_empty() {
            return Err(AgentError::ConfigError("模型名称不能为空".to_string()));
        }

        // 对于 OpenAI 兼容 API，需要 base_url
        if self.provider == "openai_compatible" && self.base_url.is_none() {
            return Err(AgentError::ConfigError(
                "OpenAI 兼容 API 需要提供 base_url".to_string(),
            ));
        }

        Ok(())
    }

    /// 获取完整的 base_url
    pub fn get_base_url(&self) -> String {
        match self.provider.as_str() {
            "deepseek" => "https://api.deepseek.com/v1".to_string(),
            "glm" => "https://open.bigmodel.cn/api/paas/v4".to_string(),
            "kimi" => "https://api.moonshot.cn/v1".to_string(),
            "openai_compatible" => self.base_url.clone().unwrap_or_default(),
            _ => "https://api.deepseek.com/v1".to_string(),
        }
    }
}