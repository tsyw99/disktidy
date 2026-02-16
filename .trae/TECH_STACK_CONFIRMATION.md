# DiskTidy 技术选型确认文档

---

## 文档信息

| 项目     | 内容                                |
| -------- | ----------------------------------- |
| 项目名称 | DiskTidy - Windows 存储空间清理工具 |
| 文档版本 | v1.0                                |
| 创建日期 | 2026-02-15                          |
| 所属阶段 | 第二阶段 - 架构设计与技术选型确认   |

---

## 1. 技术选型总表

### 1.1 框架层技术选型

| 技术  | 版本  | 选型理由                                     | 风险评估                     | 状态   |
| ----- | ----- | -------------------------------------------- | ---------------------------- | ------ |
| Tauri | 2.x   | 轻量级（<10MB）、安全、跨平台、原生性能      | API 可能变化，需关注官方更新 | 已确认 |
| Rust  | 1.75+ | 内存安全、高性能、零成本抽象、优秀的并发支持 | 学习曲线陡峭，团队需适应     | 已确认 |

### 1.
2 前端技术选型

| 技术 | 版本 | 选型理由 | 备选方案 | 状态 |
|------|------|----------|----------|------|
| React | 18.x | 生态成熟、组件化开发、Concurrent Features、Suspense支持 | Vue 3.x | 已确认 |
| TypeScript | 5.x | 类型安全、开发体验优秀、IDE支持完善 | - | 已确认 |
| Tailwind CSS | 3.x | 原子化CSS、高效开发、动画效果核心、与reactbits完美集成 | CSS-in-JS | 已确认 |
| Ant Design | 5.x | 企业级组件库、组件丰富、TypeScript支持、主题定制能力强 | shadcn/ui | 已确认 |
| ECharts | 5.x | 图表类型丰富、性能优秀、可定制性强、中文文档完善 | Recharts | 已确认 |
| Zustand | 4.x | 轻量级（<1KB）、API简洁、TypeScript支持、中间件生态丰富 | Jotai | 已确认 |
| reactbits | latest | 99个高质量动画组件、视觉效果优秀、深度融合Tailwind | - | 已确认 |
| Framer Motion | latest | reactbits核心依赖、动画库成熟、性能优秀 | - | 已确认 |
| GSAP | latest | reactbits依赖、高级动画支持、专业级动画库 | - | 已确认 |
| Lucide React | latest | 轻量级图标库、现代设计、Tree-shaking支持 | Heroicons | 已确认 |

### 1.3 后端技术选型

| 技术 | 版本 | 选型理由 | 备选方案 | 状态 |
|------|------|----------|----------|------|
| Tokio | 1.x | 成熟的异步运行时、生态完善、性能优秀 | async-std | 已确认 |
| walkdir | 2.x | 高效目录遍历、错误处理完善、支持过滤 | jwalk | 已确认 |
| windows-rs | 0.52+ | 微软官方维护、API覆盖度高、安全性好 | winapi | 已确认 |
| serde | 1.x | 高性能序列化、派生宏支持、生态完善 | - | 已确认 |
| serde_json | 1.x | JSON处理标准库、性能优秀 | - | 已确认 |
| tracing | 0.1.x | 结构化日志、异步支持、性能开销低 | log | 已确认 |
| tracing-subscriber | 0.3.x | 日志格式化、多层过滤器支持 | - | 已确认 |
| thiserror | 1.x | 自定义错误类型、派生宏简化开发 | - | 已确认 |
| anyhow | 1.x | 错误传播、错误上下文、简化错误处理 | - | 已确认 |
| sha2 | 0.10.x | 文件哈希计算、SHA-256支持 | - | 已确认 |
| uuid | 1.x | 唯一标识符生成、v4支持 | - | 已确认 |

### 1.4 开发工具选型

| 工具 | 版本 | 用途 | 配置文件 | 状态 |
|------|------|------|----------|------|
| Vite | 5.x | 前端构建工具、HMR支持、快速构建 | vite.config.ts | 已确认 |
| ESLint | 8.x | 前端代码检查 | .eslintrc.cjs | 已确认 |
| Prettier | 3.x | 前端代码格式化 | .prettierrc | 已确认 |
| Vitest | 1.x | 前端单元测试、组件测试 | vite.config.ts | 已确认 |
| Clippy | latest | Rust代码检查 | Cargo.toml | 已确认 |
| rustfmt | latest | Rust代码格式化 | rustfmt.toml | 已确认 |
| Cargo | latest | Rust构建工具、包管理 | Cargo.toml | 已确认 |

---

## 2. 依赖版本清单

### 2.1 前端依赖 (package.json)

```json
{
  "name": "disktidy",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "lint": "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
    "format": "prettier --write \"src/**/*.{ts,tsx,css,json}\"",
    "test": "vitest",
    "test:coverage": "vitest --coverage"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.22.0",
    "antd": "^5.14.0",
    "@ant-design/icons": "^5.3.0",
    "echarts": "^5.5.0",
    "echarts-for-react": "^3.0.2",
    "zustand": "^4.5.0",
    "framer-motion": "^11.0.0",
    "gsap": "^3.12.5",
    "lucide-react": "^0.323.0",
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.2.55",
    "@types/react-dom": "^18.2.19",
    "@types/node": "^20.11.0",
    "typescript": "^5.3.3",
    "vite": "^5.1.0",
    "@vitejs/plugin-react": "^4.2.1",
    "tailwindcss": "^3.4.1",
    "postcss": "^8.4.35",
    "autoprefixer": "^10.4.17",
    "eslint": "^8.56.0",
    "@typescript-eslint/eslint-plugin": "^6.21.0",
    "@typescript-eslint/parser": "^6.21.0",
    "eslint-plugin-react": "^7.33.2",
    "eslint-plugin-react-hooks": "^4.6.0",
    "eslint-plugin-react-refresh": "^0.4.5",
    "prettier": "^3.2.5",
    "prettier-plugin-tailwindcss": "^0.5.11",
    "vitest": "^1.2.2",
    "@testing-library/react": "^14.2.1",
    "@testing-library/jest-dom": "^6.4.0"
  }
}
```

### 2.2 后端依赖 (Cargo.toml)

```toml
[package]
name = "disktidy"
version = "1.0.0"
description = "Windows 存储空间清理工具"
authors = ["DiskTidy Team"]
edition = "2021"
rust-version = "1.75"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
# Tauri 核心
tauri = { version = "2.0", features = ["devtools"] }
tauri-plugin-shell = "2.0"

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 异步运行时
tokio = { version = "1.36", features = ["full"] }

# 文件系统
walkdir = "2.4"

# Windows API
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_System_SystemInformation",
    "Win32_System_Memory",
    "Win32_Storage_FileSystem",
    "Win32_Security",
    "Win32_UI_Shell"
]}

# 日志系统
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# 错误处理
thiserror = "1.0"
anyhow = "1.0"

# 哈希计算
sha2 = "0.10"

# UUID生成
uuid = { version = "1.7", features = ["v4", "serde"] }

# 时间处理
chrono = { version = "0.4", features = ["serde"] }

# 正则表达式
regex = "1.10"

[dev-dependencies]
tempfile = "3.10"
criterion = "0.5"

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "z"
strip = true

[profile.dev]
opt-level = 1

[lints.clippy]
pedantic = "warn"
nursery = "warn"
```

---

## 3. reactbits 组件库集成方案

### 3.1 组件库概览

reactbits 提供了 99 个高质量动画组件，分为四大类：

| 分类 | 组件数量 | 用途 |
|------|----------|------|
| Animations | 21 | 动画效果、交互反馈 |
| Backgrounds | 25 | 页面背景、视觉氛围 |
| Components | 33 | 功能组件、UI元素 |
| TextAnimations | 20 | 文字动画、标题效果 |

### 3.2 核心组件选型

#### 3.2.1 导航组件

| 组件 | 用途 | 配置建议 |
|------|------|----------|
| Dock | 主导航（macOS风格底部导航） | magnification: 70, distance: 200 |
| GooeyNav | 次级导航 | 适合少量导航项 |
| FlowingMenu | 下拉菜单 | 配合 Dock 使用 |

#### 3.2.2 背景组件

| 组件 | 适用场景 | 性能建议 |
|------|----------|----------|
| Aurora | 首页背景，营造科技氛围 | 控制颜色数量，建议 2-3 色 |
| DarkVeil | 内容页面背景 | 调整 speed 参数控制动画速度 |
| Particles | 特殊页面背景 | 控制粒子数量，避免过多 |
| Orb | 聚焦页面背景 | 适合单一焦点页面 |

#### 3.2.3 卡片组件

| 组件 | 适用场景 | 特色功能 |
|------|----------|----------|
| SpotlightCard | 信息展示卡片 | 鼠标跟随光效 |
| MagicBento | 首页仪表盘 | 粒子效果 + 磁性交互 |
| PixelCard | 简洁信息卡片 | 像素化边框效果 |
| TiltedCard | 倾斜展示卡片 | 3D 倾斜效果 |

#### 3.2.4 文字动画组件

| 组件 | 适用场景 | 动画效果 |
|------|----------|----------|
| ShinyText | 标题文字 | 光泽流动效果 |
| GradientText | 强调文字 | 渐变色文字 |
| BlurText | 入场动画 | 模糊到清晰 |
| GlitchText | 特殊效果 | 故障风格 |

#### 3.2.5 列表组件

| 组件 | 适用场景 | 特色功能 |
|------|----------|----------|
| AnimatedList | 文件列表 | 交错入场动画 |
| InfiniteScroll | 无限滚动 | 自动加载更多 |
| ScrollStack | 堆叠滚动 | 卡片堆叠效果 |

#### 3.2.6 动画效果组件

| 组件 | 用途 | 性能建议 |
|------|------|----------|
| ClickSpark | 点击特效 | 按钮点击反馈 |
| FadeContent | 内容渐显 | 适用于页面初始加载 |
| GlareHover | 悬停光效 | 卡片悬停效果 |
| Magnet | 磁性效果 | 交互增强 |

### 3.3 组件集成架构

```
src/
├── components/
│   ├── reactbits/                    # reactbits 组件封装
│   │   ├── backgrounds/              # 背景组件
│   │   │   ├── Aurora/
│   │   │   ├── DarkVeil/
│   │   │   └── Particles/
│   │   ├── components/               # 功能组件
│   │   │   ├── Dock/
│   │   │   ├── SpotlightCard/
│   │   │   ├── MagicBento/
│   │   │   ├── Counter/
│   │   │   └── AnimatedList/
│   │   ├── textAnimations/           # 文字动画
│   │   │   ├── ShinyText/
│   │   │   ├── GradientText/
│   │   │   └── BlurText/
│   │   └── animations/               # 动画效果
│   │       ├── ClickSpark/
│   │       ├── FadeContent/
│   │       └── GlareHover/
│   └── ...
```

---

## 4. Ant Design 定制化方案

### 4.1 主题配置

```typescript
// styles/antdTheme.ts
import type { ThemeConfig } from 'antd';

export const antdTheme: ThemeConfig = {
  token: {
    // 主色调
    colorPrimary: '#8b5cf6',
    colorSuccess: '#10b981',
    colorWarning: '#f59e0b',
    colorError: '#ef4444',
    colorInfo: '#3b82f6',
    
    // 圆角
    borderRadius: 8,
    borderRadiusLG: 12,
    borderRadiusSM: 4,
    
    // 字体
    fontFamily: 'Inter, system-ui, -apple-system, sans-serif',
    fontSize: 14,
    
    // 动画
    motionDurationFast: '0.15s',
    motionDurationMid: '0.2s',
    motionDurationSlow: '0.3s',
  },
  components: {
    Button: {
      primaryShadow: '0 4px 14px rgba(139, 92, 246, 0.4)',
      algorithm: true,
    },
    Card: {
      colorBgContainer: 'rgba(26, 26, 46, 0.8)',
      borderRadiusLG: 16,
    },
    Modal: {
      colorBgElevated: '#1a1a2e',
      borderRadiusLG: 16,
    },
    Table: {
      headerBg: 'rgba(139, 92, 246, 0.1)',
      rowHoverBg: 'rgba(139, 92, 246, 0.05)',
      borderColor: 'rgba(139, 92, 246, 0.2)',
    },
    Input: {
      colorBgContainer: 'rgba(255, 255, 255, 0.05)',
      borderColor: 'rgba(139, 92, 246, 0.2)',
      hoverBorderColor: 'rgba(139, 92, 246, 0.4)',
      activeBorderColor: '#8b5cf6',
    },
    Select: {
      colorBgContainer: 'rgba(255, 255, 255, 0.05)',
      optionSelectedBg: 'rgba(139, 92, 246, 0.2)',
    },
    Progress: {
      remainingColor: 'rgba(139, 92, 246, 0.1)',
    },
  },
  algorithm: [
    // 暗色主题算法
    theme.darkAlgorithm,
  ],
};
```

### 4.2 禁用组件清单

以下 Ant Design 组件在项目中禁用，使用 reactbits 或自定义组件替代：

| 禁用组件 | 替代方案 | 原因 |
|----------|----------|------|
| Calendar | 自定义日历组件 | 减小打包体积 |
| Carousel | reactbits Carousel | 更好的动画效果 |
| Tree | 自定义树组件 | 减小打包体积 |
| Transfer | 自定义穿梭框 | 减小打包体积 |
| Timeline | 自定义时间线 | 减小打包体积 |
| Comment | 自定义评论组件 | 不需要 |
| Avatar | 自定义头像组件 | 减小打包体积 |
| Badge | 自定义徽标组件 | 减小打包体积 |

### 4.3 按需加载配置

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [
    react(),
    // Ant Design 按需加载通过 vite-plugin-style-import 实现
  ],
  css: {
    preprocessorOptions: {
      less: {
        javascriptEnabled: true,
        modifyVars: {
          '@primary-color': '#8b5cf6',
        },
      },
    },
  },
});
```

---

## 5. Tailwind CSS 配置方案

### 5.1 配置文件

```javascript
// tailwind.config.js
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      // 颜色系统
      colors: {
        primary: {
          DEFAULT: '#8b5cf6',
          light: '#a78bfa',
          dark: '#7c3aed',
        },
        success: '#10b981',
        warning: '#f59e0b',
        error: '#ef4444',
        info: '#3b82f6',
      },
      
      // 动画
      animation: {
        'fade-in': 'fadeIn 0.3s ease-out',
        'fade-out': 'fadeOut 0.3s ease-out',
        'slide-up': 'slideUp 0.4s ease-out',
        'slide-down': 'slideDown 0.4s ease-out',
        'scale-in': 'scaleIn 0.2s ease-out',
        'bounce-in': 'bounceIn 0.5s ease-out',
        'pulse-soft': 'pulseSoft 2s ease-in-out infinite',
      },
      
      // 关键帧
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        fadeOut: {
          '0%': { opacity: '1' },
          '100%': { opacity: '0' },
        },
        slideUp: {
          '0%': { transform: 'translateY(20px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        slideDown: {
          '0%': { transform: 'translateY(-20px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        scaleIn: {
          '0%': { transform: 'scale(0.95)', opacity: '0' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
        bounceIn: {
          '0%': { transform: 'scale(0.3)', opacity: '0' },
          '50%': { transform: 'scale(1.05)' },
          '70%': { transform: 'scale(0.9)' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
        pulseSoft: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.7' },
        },
      },
      
      // 过渡时间
      transitionDuration: {
        fast: '150ms',
        normal: '200ms',
        slow: '300ms',
      },
      
      // 阴影
      boxShadow: {
        'glow': '0 0 20px rgba(139, 92, 246, 0.3)',
        'glow-lg': '0 0 40px rgba(139, 92, 246, 0.4)',
        'card': '0 4px 20px rgba(139, 92, 246, 0.15)',
        'card-hover': '0 12px 40px rgba(139, 92, 246, 0.25)',
      },
      
      // 背景渐变
      backgroundImage: {
        'gradient-primary': 'linear-gradient(135deg, #6366f1 0%, #8b5cf6 50%, #a855f7 100%)',
        'gradient-bg': 'linear-gradient(180deg, #0f0f23 0%, #1a1a2e 50%, #16213e 100%)',
        'gradient-card': 'linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(139, 92, 246, 0.1) 100%)',
      },
      
      // 字体
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
      },
    },
  },
  plugins: [],
};
```

---

## 6. 风险评估

### 6.1 技术风险

| 风险项 | 可能性 | 影响 | 应对措施 | 责任人 |
|--------|--------|------|----------|--------|
| Tauri 2.x API 变化 | 中 | 高 | 关注官方更新日志，预留适配时间，锁定稳定版本 | 后端开发 |
| reactbits 组件兼容性 | 低 | 中 | 版本锁定，定期更新评估，封装隔离层 | 前端开发 |
| Windows 版本兼容性 | 中 | 中 | 充分测试 Win10/Win11，使用条件编译 | 后端开发 |
| 扫描性能瓶颈 | 中 | 中 | 多线程优化，增量扫描，性能监控 | 后端开发 |
| 内存占用过高 | 低 | 中 | 流式处理，分页加载，内存池复用 | 全栈开发 |
| Rust 学习曲线 | 中 | 中 | 团队培训，代码审查，文档沉淀 | 团队负责人 |

### 6.2 依赖风险

| 风险项 | 可能性 | 影响 | 应对措施 |
|--------|--------|------|----------|
| 依赖版本冲突 | 低 | 中 | 使用 pnpm 锁定版本，定期更新依赖 |
| 第三方库停止维护 | 低 | 高 | 选择活跃维护的库，准备备选方案 |
| 安全漏洞 | 低 | 高 | 定期安全审计，及时更新修复版本 |

### 6.3 性能风险

| 风险项 | 可能性 | 影响 | 应对措施 |
|--------|--------|------|----------|
| 扫描大文件系统耗时 | 高 | 中 | 异步处理，进度反馈，可中断设计 |
| UI 渲染性能 | 中 | 中 | 虚拟列表，懒加载，GPU 加速 |
| 打包体积过大 | 中 | 低 | 按需加载，Tree Shaking，代码分割 |

---

## 7. 验收标准

### 7.1 技术可行性验收

| 验收项 | 验收标准 | 验收方式 | 状态 |
|--------|----------|----------|------|
| Tauri 2.x 可行性 | 应用体积 < 10MB，启动时间 < 2秒 | 实际构建测试 | 待验证 |
| React 18.x 可行性 | Concurrent Features 正常工作 | 功能测试 | 待验证 |
| reactbits 集成 | 核心组件正常渲染，动画流畅 | 视觉测试 | 待验证 |
| Rust 异步处理 | Tokio 异步运行正常 | 性能测试 | 待验证 |
| Windows API 调用 | 系统 API 调用正常 | 功能测试 | 待验证 |

### 7.2 版本兼容性验收

| 验收项 | 验收标准 | 验收方式 | 状态 |
|--------|----------|----------|------|
| 前端依赖兼容 | npm install 无冲突 | 安装测试 | 待验证 |
| 后端依赖兼容 | cargo build 无错误 | 编译测试 | 待验证 |
| TypeScript 类型 | 严格模式无类型错误 | 编译测试 | 待验证 |
| Rust 编译 | Clippy 无警告 | 静态检查 | 待验证 |

### 7.3 性能指标验收

| 指标 | 目标值 | 验收方式 | 状态 |
|------|--------|----------|------|
| 应用启动时间 | ≤ 3 秒 | 性能测试 | 待验证 |
| 正常运行内存占用 | ≤ 200MB | 内存监控 | 待验证 |
| 快速扫描时间 | ≤ 30 秒 (1TB 硬盘) | 性能测试 | 待验证 |
| 深度扫描时间 | ≤ 3 分钟 (1TB 硬盘) | 性能测试 | 待验证 |
| UI 操作响应时间 | ≤ 300ms | 性能测试 | 待验证 |
| 打包体积 | ≤ 15MB | 构建测试 | 待验证 |

### 7.4 文档完整性验收

| 文档 | 完成状态 | 状态 |
|------|----------|------|
| 技术选型确认文档 | 完整 | 已完成 |
| 依赖版本清单 | 完整 | 已完成 |
| 风险评估报告 | 完整 | 已完成 |
| reactbits 组件集成方案 | 完整 | 已完成 |
| Ant Design 定制化方案 | 完整 | 已完成 |
| Tailwind CSS 配置方案 | 完整 | 已完成 |

---

## 8. 技术选型确认清单

### 8.1 框架层确认

- [x] Tauri 2.x 框架确认
  - [x] 版本选择：Tauri 2.x
  - [x] 特性评估：应用体积 < 10MB，启动 < 2秒
  - [x] 兼容性：Windows 10/11 支持
  - [x] API 稳定性：核心 API 稳定

- [x] Rust 版本确认
  - [x] 版本选择：Rust 1.75+
  - [x] 特性需求：async/await、错误处理、并发安全

### 8.2 前端技术确认

- [x] React 框架确认
  - [x] 版本：React 18.x
  - [x] Concurrent Features 支持
  - [x] Suspense 支持
  - [x] 组件库兼容性

- [x] UI 组件库确认
  - [x] 主选：Ant Design 5.x
  - [x] 定制能力评估
  - [x] TypeScript 支持
  - [x] 主题定制能力

- [x] 样式方案确认
  - [x] 主选：Tailwind CSS 3.x
  - [x] 与 reactbits 集成
  - [x] 构建性能

- [x] 图表库确认
  - [x] 主选：ECharts 5.x
  - [x] 图表类型丰富度
  - [x] 性能表现

- [x] 状态管理确认
  - [x] 主选：Zustand 4.x
  - [x] 轻量级优势
  - [x] TypeScript 支持

- [x] reactbits 组件库确认
  - [x] 99 个组件可用
  - [x] 四大分类覆盖全面
  - [x] 与 Tailwind CSS 完美集成

### 8.3 后端技术确认

- [x] 异步运行时确认
  - [x] 主选：Tokio 1.x
  - [x] 性能表现
  - [x] 生态成熟度

- [x] 文件系统操作库确认
  - [x] 主选：walkdir 2.x
  - [x] 遍历性能
  - [x] 错误处理

- [x] Windows API 交互确认
  - [x] 主选：windows-rs
  - [x] API 覆盖度
  - [x] 安全性

- [x] 序列化库确认
  - [x] 主选：serde 1.x
  - [x] 性能表现
  - [x] 派生宏支持

- [x] 日志系统确认
  - [x] 主选：tracing
  - [x] 结构化日志
  - [x] 异步支持

- [x] 错误处理确认
  - [x] 主选：thiserror + anyhow
  - [x] 自定义错误类型
  - [x] 错误传播

### 8.4 开发工具确认

- [x] 构建工具确认
  - [x] 前端：Vite 5.x
  - [x] 后端：Cargo

- [x] 代码质量工具确认
  - [x] 前端：ESLint + Prettier
  - [x] 后端：Clippy + rustfmt
  - [x] TypeScript 严格模式

- [x] 测试工具确认
  - [x] 前端：Vitest
  - [x] 后端：Cargo test

---

## 9. 附录

### 9.1 参考文档

| 文档名称 | 链接 |
|----------|------|
| Tauri 2.x 官方文档 | https://tauri.app/v2/guide/ |
| React 官方文档 | https://react.dev/ |
| Tailwind CSS 文档 | https://tailwindcss.com/docs |
| Ant Design 文档 | https://ant.design/docs/react/introduce-cn |
| reactbits 组件库 | https://www.reactbits.dev/ |
| Rust 官方文档 | https://doc.rust-lang.org/ |
| windows-rs 文档 | https://microsoft.github.io/windows-rs/ |
| Zustand 文档 | https://docs.pmnd.rs/zustand/getting-started/introduction |
| ECharts 文档 | https://echarts.apache.org/zh/index.html |

### 9.2 版本更新记录

| 版本 | 日期 | 更新内容 | 作者 |
|------|------|----------|------|
| v1.0 | 2026-02-15 | 初始版本，完成技术选型确认 | DiskTidy Team |

---

**文档状态：已完成**

**确认人：** DiskTidy 开发团队

**确认日期：** 2026-02-15
