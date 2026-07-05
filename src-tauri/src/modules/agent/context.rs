use std::collections::VecDeque;

use rig_core::completion::message::Message;

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

    /// 清空上下文
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// 获取当前消息数量
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// 自动截断旧消息，保留最近的 max_turns 轮
    fn truncate_if_needed(&mut self) {
        let max_messages = self.max_turns * 2;
        while self.messages.len() > max_messages {
            self.messages.pop_front();
        }
    }
}
