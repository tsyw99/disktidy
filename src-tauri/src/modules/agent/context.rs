use std::collections::VecDeque;

use rig_core::completion::message::Message;

/// 模型上下文窗口的保守上限（token 数）。
/// DeepSeek V3 = 64K, GLM-4 = 128K, Kimi = 128K。这里取 50K 留足余量给系统提示词和工具定义。
const MAX_CONTEXT_TOKENS: usize = 50_000;

/// 按字符数估算 token 数。混排中英文场景：中文约 1 token / 1.5 字符，英文约 1 token / 4 字符。
/// 保守取 1 token / 2 字符，实际比真实 token 数偏高，确保不超限。
fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    (text.chars().count() + 1) / 2
}

/// 从 Message 中提取文本内容用于 token 估算。
/// rig 的 Message 内部使用 ContentBlock 存储内容，这里通过反序列化提取纯文本。
fn extract_message_text(msg: &rig_core::completion::message::Message) -> String {
    // 使用 serde 序列化后提取文本：将 Message JSON 序列化，然后提取 content 字段的纯文本
    // 这是兼容不同 rig 版本的最稳健方式
    if let Ok(json) = serde_json::to_value(msg) {
        if let Some(content) = json.get("content") {
            if let Some(arr) = content.as_array() {
                return arr
                    .iter()
                    .filter_map(|block| {
                        block.get("text").and_then(|t| t.as_str()).map(String::from)
                    })
                    .collect::<Vec<_>>()
                    .join("");
            }
        }
    }
    String::new()
}

pub struct ConversationContext {
    /// 历史消息队列
    messages: VecDeque<Message>,
    /// 最大保留消息轮数（防止超出 Token 限制）
    max_turns: usize,
}

impl ConversationContext {
    pub fn new(max_turns: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            max_turns,
        }
    }

    /// 添加用户消息
    pub fn add_user_message(&mut self, content: String) {
        self.messages.push_back(Message::user(content));
        self.truncate_if_needed();
    }

    /// 添加助手回复
    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push_back(Message::assistant(content));
        self.truncate_if_needed();
    }

    /// 获取当前历史消息（用于传递给 Rig）
    pub fn get_messages(&self) -> Vec<Message> {
        self.messages.iter().cloned().collect()
    }

    /// 预估当前上下文总 token 数
    pub fn estimated_tokens(&self) -> usize {
        self.messages
            .iter()
            .map(|msg| estimate_tokens(&extract_message_text(msg)))
            .sum()
    }

    /// 获取当前消息数量
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// 清空上下文
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// 智能截断：超过 token 预算时逐步删除旧消息，保留至少 2 轮
    fn truncate_if_needed(&mut self) {
        // 第一阶段：按 max_turns 数量限制
        let max_messages = self.max_turns * 2;
        while self.messages.len() > max_messages {
            self.messages.pop_front();
        }

        // 第二阶段：按 token 预算限制（最低保留最近 2 轮 = 4 条消息）
        while self.messages.len() > 4 && self.estimated_tokens() > MAX_CONTEXT_TOKENS {
            // 每次删除最早的一对（user + assistant）
            if self.messages.len() >= 2 {
                self.messages.pop_front();
                if !self.messages.is_empty() {
                    self.messages.pop_front();
                }
            } else {
                self.messages.pop_front();
            }
        }
    }
}
