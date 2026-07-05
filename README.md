# DiskTidy - 磁盘清理工具

基于 Tauri + React 的 Windows 桌面磁盘清理工具，内置 AI Agent 智能助手，可自然语言驱动磁盘扫描与清理。

## 功能特性

- **AI Agent 智能助手** - 基于 Rig 框架，支持自然语言交互，自动调用工具完成磁盘扫描、分析与清理
- **系统概览** - 实时展示磁盘分区信息、使用率和存储状态
- **智能扫描** - 扫描大文件、垃圾文件、应用缓存、软件残留
- **文件分析** - 按类型分类统计，可视化展示文件分布
- **安全清理** - 支持回收站与永久删除，内置系统目录保护

## 技术栈

- **前端**：React 18 + TypeScript + Tailwind CSS + Zustand + ECharts
- **后端**：Rust + Tauri 2.x + Rig 0.39
- **Agent 框架**：[Rig](https://www.rig.rs/) — Rust 原生 LLM 应用框架，提供 Agent 循环、工具调用、流式输出等能力

## Agent 架构

DiskTidy Agent 是对现有磁盘管理工具的 LLM 封装层，让用户通过自然语言完成磁盘操作。

### 整体架构

```
前端 (React)
  │  Tauri invoke / events
  ▼
Tauri 命令层
  │
  ▼
AgentManager ────────────────────────────────────────────
  │                                                     │
  ├─ Config        LLM 提供商/模型/API Key 配置          │
  ├─ Context       对话历史管理（VecDeque<Message>）      │
  ├─ Error         统一错误处理与重试                     │
  ├─ StreamBridge  流式事件 → Tauri 前端推送             │
  │                                                     │
  └─ Tools (9 个 Rig Tool)                               │
       ├─ disk_scan               磁盘扫描                │
       ├─ file_classifier         文件分类                │
       ├─ large_file_scanner      大文件扫描              │
       ├─ garbage_analyzer        垃圾文件分析            │
       ├─ cleaner                 文件清理（含安全确认）   │
       ├─ app_cache_scanner       应用缓存扫描            │
       ├─ software_residue_scanner 软件残留扫描           │
       ├─ file_search             文件搜索                │
       └─ file_delete             文件删除                │
```

### Agent 工作流程

```
用户输入 → AgentManager.chat()
             │
             ├─ 非流式: agent.prompt().with_history() → 文本响应
             └─ 流式:   agent.stream_prompt().with_history() → StreamBridge → 前端事件
             │
             ▼
          Rig Agent 循环
             │
             ├─ 模型返回文本 → 响应结束
             └─ 模型请求工具 → 执行工具 → 结果回传 → 继续循环
             │
             ▼
          追加到 ConversationContext（自动截断旧消息）
```

### 关键设计

| 模块 | 职责 | 路径 |
|------|------|------|
| AgentManager | Agent 生命周期管理、LLM 客户端创建、对话调度 | `agent/agent_manager.rs` |
| AgentConfig | 多提供商配置（DeepSeek/GLM/Kimi）及验证 | `agent/config.rs` |
| ConversationContext | 手动管理对话历史，按轮数自动截断 | `agent/context.rs` |
| AgentError | 分类错误类型，支持可重试判断和错误码 | `agent/error.rs` |
| StreamBridge | Rig 流式事件 → Tauri `agent-stream-event` 事件 | `agent/stream_bridge.rs` |
| Tools | 9 个 Rig Tool 实现，将现有模块封装为 LLM 可调用工具 | `agent/tools/*.rs` |

### 对话模式

- **非流式**：`prompt()` + `with_history()`，返回完整文本。需手动追加历史到 Context
- **流式**：`stream_prompt()` + `with_history()`，通过 Tauri 事件实时推送 `text_delta`、`tool_call_start`、`tool_result`、`done` 到前端

### 错误处理

- 自动重试：LLM/网络/超时/速率限制错误，最多重试 2 次（指数退避）
- 连续失败保护：连续 3 次失败自动停止
- 工具错误：Rig 自动将错误消息反馈给模型，模型可自行修正重试

## 项目结构

```
DiskTidy/
├── src/                         # React 前端
│   ├── components/              # 通用组件
│   ├── pages/                   # 页面
│   ├── services/                # Tauri 命令调用
│   ├── stores/                  # Zustand 状态
│   └── types/                   # TS 类型
├── src-tauri/
│   └── src/
│       ├── commands.rs          # Tauri 命令
│       └── modules/
│           ├── agent/           # AI Agent 模块
│           │   ├── agent_manager.rs
│           │   ├── config.rs
│           │   ├── context.rs
│           │   ├── error.rs
│           │   ├── stream_bridge.rs
│           │   ├── prompts/     # 系统提示词
│           │   └── tools/       # Rig Tool 实现 (9 个)
│           ├── cleaner/         # 文件清理
│           ├── file_analyzer/   # 文件分析（大文件/垃圾/重复）
│           ├── disk_scan.rs     # 磁盘扫描
│           ├── large_file_scanner.rs
│           ├── app_cache/       # 应用缓存
│           ├── software_residue/# 软件残留
│           └── settings/        # 设置管理
├── .trae/documents/             # 技术文档
└── package.json
```

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.75+
- Windows 10/11

```bash
npm install
npm run tauri dev
```

## 安全说明

- 系统关键目录受保护，防止误删
- 永久删除需二次确认
- 优先推荐移至回收站

## 许可证

本软件采用 **CC BY-NC 4.0**（知识共享署名-非商业性使用 4.0 国际许可协议）

- ✅ 个人使用、学习研究、修改源码、非商业分发
- ❌ 商业用途、销售本软件或衍生作品

Copyright (c) 2025 DiskTidy
