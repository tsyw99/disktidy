use futures::StreamExt;
use tauri::{AppHandle, Emitter};
use serde::Serialize;
use log::{debug, warn, info};
use std::time::Duration;

use rig_core::agent::MultiTurnStreamItem;
use rig_core::agent::StreamingError;
use rig_core::streaming::{StreamedAssistantContent, ToolCallDeltaContent};

/// 流式接收超时时间（秒）— 如果超过此时间没有收到新的流式项，视为流结束
const STREAM_TIMEOUT_SECS: u64 = 30;

/// 检查将 delta 追加到 accumulated 后是否会产生重复模式。
/// 返回去重后的有效增量（可能为空字符串表示完全重复应跳过）。
/// 使用字符级比较（非字节级），安全处理 UTF-8 多字节字符。
fn dedup_delta(accumulated: &str, delta: &str) -> String {
    let acc_chars: Vec<char> = accumulated.chars().collect();
    let delta_chars: Vec<char> = delta.chars().collect();
    let acc_len = acc_chars.len();
    let delta_len = delta_chars.len();

    if delta_len == 0 || acc_len == 0 {
        return delta.to_string();
    }

    // 策略1：delta 完全被 accumulated 末尾覆盖（字符级比较）
    if delta_len <= acc_len && acc_chars[acc_len - delta_len..] == delta_chars[..] {
        debug!("去重: delta 与已有末尾完全重复 ({} chars)", delta_len);
        return String::new();
    }

    // 策略2：检查 delta 中是否存在内部重复模式（如 "我来我来" → 保留 "我来"）
    if delta_len >= 2 && delta_len % 2 == 0 {
        let half = delta_len / 2;
        if delta_chars[..half] == delta_chars[half..] {
            // 确认前半部分也不与 accumulated 末尾重复
            if acc_len >= half && acc_chars[acc_len - half..] == delta_chars[..half] {
                debug!("去重: delta 内部重复且首段与已有末尾重复 ({} chars)", half);
                return String::new();
            }
            // delta 内部重复但与已有末尾不重复 → 取一半
            debug!("去重: delta 内部重复，截取一半 ({} chars → {} chars)", delta_len, half);
            return delta_chars[..half].iter().collect();
        }
    }

    // 策略3：字符级重复检测（如 "   "、",,"、"EE"）
    if delta_len >= 2 {
        let all_same = delta_chars.iter().all(|&c| c == delta_chars[0]);
        if all_same {
            // 单字符重复，只取一个
            if acc_len > 0 && acc_chars[acc_len - 1] == delta_chars[0] {
                debug!("去重: 字符级重复且与末尾相同 '{}'", delta_chars[0]);
                return String::new();
            }
            debug!("去重: 字符级重复，截取一个 '{}'", delta_chars[0]);
            return delta_chars[0].to_string();
        }
    }

    delta.to_string()
}

/// 流式事件负载（发送到前端）
#[derive(Debug, Clone, Serialize)]
pub struct AgentStreamEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub message_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_args: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 将 Rig 流式事件桥接到 Tauri 前端事件
///
/// 处理 MultiTurnStreamItem 流，提取文本增量、工具调用和最终结果，
/// 通过 `app.emit("agent-stream-event", ...)` 推送到前端。
///
/// 返回完整的响应文本。
pub async fn bridge_stream_to_tauri<R>(
    stream: impl futures::Stream<Item = Result<MultiTurnStreamItem<R>, StreamingError>> + Unpin,
    app: &AppHandle,
    message_id: &str,
) -> Result<String, String> {
    let mut full_response = String::new();
    let mut current_tool_name: Option<String> = None;
    let mut current_tool_args = String::new();
    let mut text_delta_count: usize = 0;
    let mut tool_call_count: usize = 0;

    info!("[{message_id}] 开始桥接流式事件");

    // 使用 tokio::pin! 固定 stream
    let mut stream = std::pin::pin!(stream);

    loop {
        // 为每次 stream.next() 添加超时，防止底层 SSE 连接未关闭导致永久阻塞
        let next_item = tokio::time::timeout(
            Duration::from_secs(STREAM_TIMEOUT_SECS),
            stream.next(),
        ).await;

        match next_item {
            Ok(Some(item)) => {
                let item = match item {
                    Ok(i) => i,
                    Err(e) => {
                        warn!("[{message_id}] 流式项错误: {e}");
                        return Err(format!("流式处理错误: {e}"));
                    }
                };

                match item {
                    // 文本增量
                    MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text)) => {
                        let raw_delta = text.text.clone();
                        text_delta_count += 1;

                        // 去重检测：过滤 LLM 流式输出中的重复 token
                        let delta = dedup_delta(&full_response, &raw_delta);

                        if text_delta_count <= 3 || text_delta_count % 50 == 0 {
                            let skipped_chars = raw_delta.chars().count() - delta.chars().count();
                            if skipped_chars > 0 {
                                debug!(
                                    "[{message_id}] 文本增量 #{}: {} chars (跳过 {} chars 重复)",
                                    text_delta_count, delta.chars().count(), skipped_chars
                                );
                            } else {
                                debug!("[{message_id}] 文本增量 #{}: {} chars", text_delta_count, delta.chars().count());
                            }
                        }

                        if delta.is_empty() {
                            continue; // 完全重复，跳过不发送给前端
                        }

                        full_response.push_str(&delta);

                        let payload = AgentStreamEvent {
                            event_type: "text_delta".to_string(),
                            message_id: message_id.to_string(),
                            content: Some(delta),
                            tool_name: None,
                            tool_args: None,
                            tool_result: None,
                            error: None,
                        };
                        app.emit("agent-stream-event", &payload)
                            .map_err(|e| {
                                warn!("[{message_id}] 事件发送失败 (text_delta): {e}");
                                format!("事件发送失败: {e}")
                            })?;
                    }

                    // 完整工具调用
                    MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCall {
                        tool_call,
                        ..
                    }) => {
                        tool_call_count += 1;
                        current_tool_name = Some(tool_call.function.name.clone());
                        current_tool_args = tool_call.function.arguments.to_string();

                        info!(
                            "[{message_id}] 工具调用 #{}: {}",
                            tool_call_count, tool_call.function.name
                        );

                        let payload = AgentStreamEvent {
                            event_type: "tool_call_start".to_string(),
                            message_id: message_id.to_string(),
                            content: None,
                            tool_name: Some(tool_call.function.name.clone()),
                            tool_args: Some(tool_call.function.arguments.to_string()),
                            tool_result: None,
                            error: None,
                        };
                        app.emit("agent-stream-event", &payload)
                            .map_err(|e| {
                                warn!("[{message_id}] 事件发送失败 (tool_call_start): {e}");
                                format!("事件发送失败: {e}")
                            })?;
                    }

                    // 工具调用增量（缓冲）
                    MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ToolCallDelta {
                        content,
                        ..
                    }) => {
                        match content {
                            ToolCallDeltaContent::Name(name) => {
                                current_tool_name = Some(name);
                            }
                            ToolCallDeltaContent::Delta(delta) => {
                                current_tool_args.push_str(&delta);
                            }
                        }
                    }

                    // 工具结果
                    MultiTurnStreamItem::StreamUserItem(user_content) => {
                        let result_text = format!("{:?}", user_content);
                        // 截断过长的结果
                        let result_text = if result_text.len() > 500 {
                            format!("{}...", &result_text[..500])
                        } else {
                            result_text
                        };

                        info!(
                            "[{message_id}] 工具结果: {} ({} chars)",
                            current_tool_name.as_deref().unwrap_or("unknown"),
                            result_text.len()
                        );

                        let payload = AgentStreamEvent {
                            event_type: "tool_result".to_string(),
                            message_id: message_id.to_string(),
                            content: None,
                            tool_name: current_tool_name.clone(),
                            tool_args: None,
                            tool_result: Some(result_text),
                            error: None,
                        };
                        app.emit("agent-stream-event", &payload)
                            .map_err(|e| {
                                warn!("[{message_id}] 事件发送失败 (tool_result): {e}");
                                format!("事件发送失败: {e}")
                            })?;

                        current_tool_name = None;
                        current_tool_args.clear();
                    }

                    // 完成调用（含 token 用量信息，忽略）
                    MultiTurnStreamItem::CompletionCall(_) => {
                        debug!("[{message_id}] 收到 CompletionCall");
                    }

                    // 最终响应 — 立即发送 done 事件并退出，不再等待流结束
                    MultiTurnStreamItem::FinalResponse(_) => {
                        info!(
                            "[{message_id}] 收到 FinalResponse，流式桥接完成: {} 文本增量, {} 工具调用, {} 字符总响应",
                            text_delta_count, tool_call_count, full_response.len()
                        );

                        let done_payload = AgentStreamEvent {
                            event_type: "done".to_string(),
                            message_id: message_id.to_string(),
                            content: Some(full_response.clone()),
                            tool_name: None,
                            tool_args: None,
                            tool_result: None,
                            error: None,
                        };
                        app.emit("agent-stream-event", &done_payload)
                            .map_err(|e| {
                                warn!("[{message_id}] 事件发送失败 (done): {e}");
                                format!("事件发送失败: {e}")
                            })?;

                        return Ok(full_response);
                    }

                    // 推理内容（忽略）
                    MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Reasoning(_)) => {}
                    MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::ReasoningDelta { .. }) => {}
                    // 其他未处理变体
                    other => {
                        debug!("[{message_id}] 忽略未处理的流式项: {:?}", std::any::type_name_of_val(&other));
                    }
                }
            }
            Ok(None) => {
                // 流正常结束（无 FinalResponse，可能是非 multi_turn 场景）
                info!(
                    "[{message_id}] 流正常结束: {} 文本增量, {} 工具调用, {} 字符总响应",
                    text_delta_count, tool_call_count, full_response.len()
                );
                break;
            }
            Err(_) => {
                // 超时：超过 STREAM_TIMEOUT_SECS 秒没有收到新的流式项
                warn!(
                    "[{message_id}] 流式接收超时 ({}s)，视为流结束。已接收: {} 文本增量, {} 字符",
                    STREAM_TIMEOUT_SECS, text_delta_count, full_response.len()
                );
                break;
            }
        }
    }

    // 发送完成事件（仅在流正常结束或超时退出时到达此处）
    let done_payload = AgentStreamEvent {
        event_type: "done".to_string(),
        message_id: message_id.to_string(),
        content: Some(full_response.clone()),
        tool_name: None,
        tool_args: None,
        tool_result: None,
        error: None,
    };
    app.emit("agent-stream-event", &done_payload)
        .map_err(|e| {
            warn!("[{message_id}] 事件发送失败 (done): {e}");
            format!("事件发送失败: {e}")
        })?;

    info!(
        "[{message_id}] 流式桥接完成: {} 文本增量, {} 工具调用, {} 字符总响应",
        text_delta_count, tool_call_count, full_response.len()
    );

    Ok(full_response)
}