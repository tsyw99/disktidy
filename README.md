# DiskTidy - 磁盘清理工具

一款基于 Tauri + React 开发的 Windows 桌面磁盘清理工具，帮助用户快速扫描、分析和清理磁盘空间。

## 功能特性

- **系统概览** - 实时展示磁盘分区信息、使用率和存储状态
- **智能扫描** - 快速扫描大文件、垃圾文件和应用缓存
- **文件分析** - 按类型分类统计文件分布，可视化展示
- **安全清理** - 支持移至回收站和永久删除，内置系统目录保护
- **软件残留清理** - 扫描已卸载软件的残留文件和注册表项
- **驱动管理** - 查看和管理系统驱动程序

## 技术栈

- **前端**: React 18 + TypeScript + Tailwind CSS
- **后端**: Rust + Tauri 2.x
- **状态管理**: Zustand
- **图表**: ECharts
- **动画**: Framer Motion

## 快速开始

### 环境要求

- Node.js 18+
- Rust 1.75+
- Windows 10/11

### 安装依赖

```bash
npm install
```

### 开发运行

```bash
npm run dev
npm run tauri dev
```

### 构建应用

```bash
npm run build
cd src-tauri
cargo build --release
```

## 项目结构

```
DiskTidy/
├── src/                    # 前端源码
│   ├── components/         # React 组件
│   ├── pages/              # 页面组件
│   ├── services/           # Tauri 命令调用
│   ├── stores/             # Zustand 状态管理
│   ├── types/              # TypeScript 类型定义
│   └── utils/              # 工具函数
├── src-tauri/              # Tauri 后端源码
│   ├── src/                # Rust 源码
│   └── Cargo.toml          # Rust 依赖
└── package.json            # 前端依赖
```

## 安全说明

- 系统关键目录受保护，防止误删
- 删除操作需二次确认
- 支持移至回收站，可恢复

## 许可证

本软件采用 **CC BY-NC 4.0**（知识共享署名-非商业性使用 4.0 国际许可协议）

### 许可范围

- ✅ **允许**

  - 个人使用
  - 学习研究
  - 修改源代码
  - 非商业性分发

- ❌ **禁止**
  - 商业用途
  - 销售本软件或其衍生作品
  - 将本软件用于盈利目的

### 版权声明

Copyright (c) 2025 DiskTidy

详细许可条款请参阅 [CC BY-NC 4.0](https://creativecommons.org/licenses/by-nc/4.0/)
