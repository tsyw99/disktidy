# DiskTidy 技术架构文档

## 1. 系统架构概述

### 1.1 整体架构

DiskTidy 采用 Tauri 2.x 框架构建，实现前后端分离的桌面应用架构：

```
┌─────────────────────────────────────────────────────────────────┐
│                        用户界面层 (UI Layer)                      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    React 18 + TypeScript                 │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │ 系统信息 │ │ 磁盘扫描 │ │ 文件清理 │ │ 设置中心 │       │   │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘       │   │
│  │       └───────────┴───────────┴───────────┘             │   │
│  │                      │                                   │   │
│  │              ┌───────┴───────┐                          │   │
│  │              │ Zustand Store │                          │   │
│  │              └───────┬───────┘                          │   │
│  └──────────────────────┼──────────────────────────────────┘   │
│                         │                                       │
│                    Tauri IPC                                    │
│                         │                                       │
├─────────────────────────┼───────────────────────────────────────┤
│                         │                                       │
│  ┌──────────────────────┼──────────────────────────────────┐   │
│  │              Tauri Commands Layer                        │   │
│  │              ┌───────┴───────┐                          │   │
│  │              │ Command Router │                          │   │
│  │              └───────┬───────┘                          │   │
│  └──────────────────────┼──────────────────────────────────┘   │
│                         │                                       │
│                        后端层 (Rust)                             │
│  ┌──────────────────────┼──────────────────────────────────┐   │
│  │              Core Modules                                 │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │   │
│  │  │ System Info │ │ Disk Scanner │ │   Cleaner   │       │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘       │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │   │
│  │  │File Analyzer│ │Rule Engine  │ │  Scheduler  │       │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘       │   │
│  └──────────────────────────────────────────────────────────┘   │
│                         │                                       │
│  ┌──────────────────────┼──────────────────────────────────┐   │
│  │              System Integration                          │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │   │
│  │  │  Windows API│ │ File System │ │   Win32 API │       │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘       │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 技术栈详情

| 层级       | 技术          | 版本   | 职责                          |
| ---------- | ------------- | ------ | ----------------------------- |
| UI 框架    | React         | 18.x   | 构建用户界面                  |
| 动画组件库 | reactbits     | latest | 动画组件、视觉效果            |
| UI 组件库  | Ant Design    | 5.x    | 提供基础 UI 组件（需定制化）  |
| 样式方案   | Tailwind CSS  | 3.x    | 原子化 CSS 样式、动画效果核心 |
| 数据可视化 | ECharts       | 5.x    | 图表展示                      |
| 状态管理   | Zustand       | 4.x    | 全局状态管理                  |
| 动画库     | Framer Motion | latest | reactbits 依赖，动画支持      |
| 高级动画   | GSAP          | latest | reactbits 依赖，复杂动画      |
| 类型系统   | TypeScript    | 5.x    | 类型安全                      |
| 桌面框架   | Tauri         | 2.x    | 跨平台桌面应用                |
| 后端语言   | Rust          | 1.75+  | 核心业务逻辑                  |
| 异步运行时 | Tokio         | 1.x    | 异步任务处理                  |
| 文件遍历   | walkdir       | 2.x    | 高效目录遍历                  |
| 系统交互   | windows-rs    | latest | Windows API 调用              |

### 1.3 reactbits 组件库集成架构

reactbits 组件库是本项目的核心视觉组件来源，提供 99 个高质量动画组件，分为四大类：

```
┌─────────────────────────────────────────────────────────────────┐
│                    reactbits 组件库集成架构                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Backgrounds (背景组件)                  │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │ Aurora  │ │DarkVeil │ │ Particles│ │   Orb   │       │   │
│  │  │ 首页背景 │ │内容页背景│ │ 特殊页面 │ │ 聚焦页面 │       │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Components (功能组件)                   │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │  Dock   │ │Spotlight│ │MagicBento│ │ Counter │       │   │
│  │  │ 导航组件 │ │  Card   │ │ 仪表盘   │ │ 数字动画 │       │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │Animated │ │Infinite │ │  Stepper │ │ Carousel│       │   │
│  │  │  List   │ │ Scroll  │ │  步骤器  │ │  轮播   │       │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                 TextAnimations (文字动画)                  │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │ShinyText│ │Gradient │ │ BlurText │ │GlitchTxt│       │   │
│  │  │ 标题文字 │ │  Text   │ │ 入场动画 │ │ 特效文字 │       │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Animations (动画效果)                    │   │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │   │
│  │  │ClickSpark│ │FadeCntnt│ │ GlareHov│ │ Magnet  │       │   │
│  │  │ 点击特效 │ │ 内容渐显 │ │ 悬停光效 │ │ 磁性效果 │       │   │
│  │  └─────────┘ └─────────┘ └─────────┘ └─────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 1.4 组件选型优先级

| 优先级 | 组件来源   | 使用场景             | 说明               |
| ------ | ---------- | -------------------- | ------------------ |
| 1      | reactbits  | 动画、视觉效果、导航 | 优先使用现有组件   |
| 2      | Ant Design | 表单、表格、对话框   | 需定制化样式       |
| 3      | 自定义组件 | 特殊业务需求         | 仅在无法满足时开发 |

---

## 2. 前端架构

### 2.1 目录结构

```
src/
├── components/                    # React组件
│   ├── reactbits/                # reactbits组件封装
│   │   ├── Dock/                 # 底部导航Dock组件
│   │   │   ├── index.tsx
│   │   │   ├── Dock.css
│   │   │   └── types.ts
│   │   ├── SpotlightCard/        # 聚光卡片组件
│   │   │   ├── index.tsx
│   │   │   └── SpotlightCard.css
│   │   ├── MagicBento/           # Bento布局组件
│   │   │   ├── index.tsx
│   │   │   └── MagicBento.css
│   │   ├── Counter/              # 数字动画组件
│   │   │   ├── index.tsx
│   │   │   └── Counter.css
│   │   ├── AnimatedList/         # 动画列表组件
│   │   │   ├── index.tsx
│   │   │   └── AnimatedList.css
│   │   ├── backgrounds/          # 背景组件
│   │   │   ├── Aurora/
│   │   │   ├── DarkVeil/
│   │   │   └── Particles/
│   │   └── textAnimations/       # 文字动画组件
│   │       ├── ShinyText/
│   │       ├── GradientText/
│   │       └── BlurText/
│   ├── common/                   # 通用组件
│   │   ├── Button/
│   │   │   ├── index.tsx
│   │   │   └── styles.module.css
│   │   ├── Card/
│   │   ├── Modal/
│   │   ├── Progress/
│   │   └── Table/
│   ├── layout/                   # 布局组件
│   │   ├── MainLayout/           # 主布局（含Dock导航）
│   │   ├── Header/
│   │   ├── Sidebar/
│   │   └── Footer/
│   ├── system/                   # 系统信息组件
│   │   ├── SystemInfo/           # 使用SpotlightCard
│   │   ├── DiskList/             # 使用AnimatedList
│   │   └── DiskChart/            # ECharts图表
│   ├── scan/                     # 扫描相关组件
│   │   ├── ScanConfig/
│   │   ├── ScanProgress/         # 使用Counter动画
│   │   └── ScanResult/           # 使用AnimatedList
│   ├── clean/                    # 清理相关组件
│   │   ├── CleanPreview/
│   │   ├── CleanConfirm/
│   │   └── CleanReport/
│   └── settings/                 # 设置组件
│       ├── GeneralSettings/
│       ├── RuleSettings/
│       └── ScheduleSettings/
├── hooks/                        # 自定义Hooks
│   ├── useSystemInfo.ts
│   ├── useDiskScan.ts
│   ├── useFileClean.ts
│   └── useSettings.ts
├── stores/                       # Zustand状态管理
│   ├── systemStore.ts
│   ├── scanStore.ts
│   ├── cleanStore.ts
│   └── settingsStore.ts
├── services/                     # Tauri命令调用
│   ├── systemService.ts
│   ├── diskService.ts
│   ├── cleanService.ts
│   └── settingsService.ts
├── types/                        # TypeScript类型定义
│   ├── system.ts
│   ├── disk.ts
│   ├── file.ts
│   └── common.ts
├── utils/                        # 工具函数
│   ├── format.ts
│   ├── validation.ts
│   └── constants.ts
├── styles/                       # 全局样式
│   ├── global.css
│   ├── variables.css
│   └── animations.css            # Tailwind动画扩展
├── App.tsx                       # 根组件
├── main.tsx                      # 入口文件
└── router.tsx                    # 路由配置
```

### 2.2 reactbits 组件集成设计

#### 2.2.1 Dock 导航组件集成

应用导航系统使用 reactbits 的 Dock 组件实现 macOS 风格的底部导航：

```typescript
// components/reactbits/Dock/index.tsx
import Dock from "./Dock";
import type { DockItem } from "./types";

export interface AppDockItem extends DockItem {
  route: string;
  icon: React.ReactNode;
  label: string;
}

const dockConfig = {
  spring: { mass: 0.1, stiffness: 150, damping: 12 },
  magnification: 70,
  distance: 200,
  panelHeight: 68,
  dockHeight: 256,
  baseItemSize: 50,
};

const navItems: AppDockItem[] = [
  {
    route: "/system",
    icon: <DashboardIcon />,
    label: "系统概览",
    onClick: () => navigate("/system"),
  },
  {
    route: "/scan",
    icon: <ScanIcon />,
    label: "磁盘扫描",
    onClick: () => navigate("/scan"),
  },
  {
    route: "/clean",
    icon: <TrashIcon />,
    label: "垃圾清理",
    onClick: () => navigate("/clean"),
  },
  {
    route: "/settings",
    icon: <SettingsIcon />,
    label: "设置",
    onClick: () => navigate("/settings"),
  },
];
```

#### 2.2.2 系统概览页面组件映射

| 功能区域 | reactbits 组件 | 用途说明                  |
| -------- | -------------- | ------------------------- |
| 页面背景 | Aurora         | 动态极光背景，营造科技感  |
| 页面标题 | ShinyText      | 光泽流动效果的标题文字    |
| 磁盘卡片 | SpotlightCard  | 鼠标跟随光效的信息卡片    |
| 仪表盘   | MagicBento     | 粒子效果+磁性交互的仪表盘 |
| 统计数字 | Counter        | 动态数字滚动展示          |
| 文件列表 | AnimatedList   | 交错入场动画的列表        |

#### 2.2.3 SpotlightCard 组件封装

```typescript
// components/reactbits/SpotlightCard/index.tsx
import SpotlightCard from "./SpotlightCard";

export interface DiskCardProps {
  disk: DiskInfo;
  onScan?: (diskPath: string) => void;
  className?: string;
}

export const DiskCard: React.FC<DiskCardProps> = ({
  disk,
  onScan,
  className,
}) => {
  return (
    <SpotlightCard
      className={`disk-card ${className}`}
      spotlightColor="rgba(139, 92, 246, 0.25)"
    >
      <div className="p-6">
        <h3 className="text-xl font-semibold text-gray-100">{disk.letter}</h3>
        <p className="text-gray-400">{disk.label}</p>
        <div className="mt-4">
          <Counter
            value={disk.usagePercent}
            fontSize={36}
            textColor="#8b5cf6"
          />
          <span className="text-gray-400 ml-2">已使用</span>
        </div>
        <button
          onClick={() => onScan?.(disk.letter)}
          className="mt-4 btn-primary"
        >
          开始扫描
        </button>
      </div>
    </SpotlightCard>
  );
};
```

#### 2.2.4 背景组件使用策略

```typescript
// components/layout/MainLayout.tsx
import { Aurora } from "../reactbits/backgrounds/Aurora";
import { DarkVeil } from "../reactbits/backgrounds/DarkVeil";

export const MainLayout: React.FC = ({ children }) => {
  const location = useLocation();
  const isHomePage = location.pathname === "/system";

  return (
    <div className="relative min-h-screen">
      {isHomePage ? (
        <Aurora
          colorStops={["#6366f1", "#8b5cf6", "#a855f7"]}
          amplitude={1.0}
          blend={0.5}
        />
      ) : (
        <DarkVeil hueShift={0} noiseIntensity={0.02} speed={0.3} />
      )}
      <div className="relative z-10">{children}</div>
      <Dock items={navItems} {...dockConfig} />
    </div>
  );
};
```

### 2.3 状态管理设计

使用 Zustand 进行状态管理，采用模块化设计：

```typescript
// stores/systemStore.ts
import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { SystemInfo, DiskInfo } from "../types/system";

interface SystemState {
  systemInfo: SystemInfo | null;
  diskList: DiskInfo[];
  isLoading: boolean;
  error: string | null;

  fetchSystemInfo: () => Promise<void>;
  fetchDiskList: () => Promise<void>;
  refreshAll: () => Promise<void>;
}

export const useSystemStore = create<SystemState>()(
  persist(
    (set, get) => ({
      systemInfo: null,
      diskList: [],
      isLoading: false,
      error: null,

      fetchSystemInfo: async () => {
        set({ isLoading: true, error: null });
        try {
          const info = await systemService.getInfo();
          set({ systemInfo: info, isLoading: false });
        } catch (error) {
          set({ error: String(error), isLoading: false });
        }
      },

      fetchDiskList: async () => {
        try {
          const disks = await diskService.getList();
          set({ diskList: disks });
        } catch (error) {
          set({ error: String(error) });
        }
      },

      refreshAll: async () => {
        await Promise.all([get().fetchSystemInfo(), get().fetchDiskList()]);
      },
    }),
    {
      name: "system-storage",
    }
  )
);
```

### 2.4 组件设计规范

**组件分类**：

| 类型           | 说明           | 示例                        |
| -------------- | -------------- | --------------------------- |
| 页面组件       | 路由对应的页面 | SystemPage, ScanPage        |
| 容器组件       | 包含业务逻辑   | ScanContainer               |
| 展示组件       | 纯 UI 展示     | DiskCard, FileList          |
| reactbits 组件 | 动画视觉效果   | Dock, SpotlightCard, Aurora |
| 通用组件       | 可复用组件     | Button, Modal               |

**组件接口规范**：

```typescript
// 组件Props接口定义
interface DiskCardProps {
  disk: DiskInfo;
  onScan?: (diskPath: string) => void;
  className?: string;
}

// 组件实现 - 使用reactbits SpotlightCard
export const DiskCard: React.FC<DiskCardProps> = ({
  disk,
  onScan,
  className,
}) => {
  return (
    <SpotlightCard
      className={className}
      spotlightColor="rgba(139, 92, 246, 0.25)"
    >
      {/* 组件内容 */}
    </SpotlightCard>
  );
};
```

### 2.5 Ant Design 定制化规范

所有 Ant Design 组件必须进行主题定制，确保与项目整体设计风格一致：

```typescript
// styles/antdTheme.ts
import type { ThemeConfig } from "antd";

export const antdTheme: ThemeConfig = {
  token: {
    colorPrimary: "#8b5cf6",
    colorSuccess: "#10b981",
    colorWarning: "#f59e0b",
    colorError: "#ef4444",
    colorInfo: "#3b82f6",
    borderRadius: 8,
    fontFamily: "Inter, system-ui, sans-serif",
  },
  components: {
    Button: {
      primaryShadow: "0 4px 14px rgba(139, 92, 246, 0.4)",
    },
    Card: {
      colorBgContainer: "rgba(26, 26, 46, 0.8)",
    },
    Modal: {
      colorBgElevated: "#1a1a2e",
    },
    Table: {
      headerBg: "rgba(139, 92, 246, 0.1)",
      rowHoverBg: "rgba(139, 92, 246, 0.05)",
    },
  },
};
```

**禁用的 Ant Design 组件清单**：

| 禁用组件 | 替代方案           |
| -------- | ------------------ |
| Calendar | 自定义日历组件     |
| Carousel | reactbits Carousel |
| Tree     | 自定义树组件       |
| Transfer | 自定义穿梭框       |
| Timeline | 自定义时间线       |
| Comment  | 自定义评论组件     |
| Avatar   | 自定义头像组件     |
| Badge    | 自定义徽标组件     |

### 2.6 路由设计

```typescript
// router.tsx
import { createBrowserRouter } from "react-router-dom";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <MainLayout />,
    children: [
      {
        index: true,
        element: <Navigate to="/system" replace />,
      },
      {
        path: "system",
        element: <SystemPage />,
      },
      {
        path: "scan",
        element: <ScanPage />,
      },
      {
        path: "clean",
        element: <CleanPage />,
      },
      {
        path: "settings",
        element: <SettingsPage />,
      },
    ],
  },
]);
```

---

## 3. 后端架构

### 3.1 目录结构

```
src-tauri/
├── src/
│   ├── commands/                 # Tauri命令
│   │   ├── mod.rs
│   │   ├── system.rs            # 系统信息命令
│   │   ├── disk.rs              # 磁盘相关命令
│   │   ├── scan.rs              # 扫描相关命令
│   │   ├── clean.rs             # 清理相关命令
│   │   └── settings.rs          # 设置相关命令
│   ├── modules/                  # 核心模块
│   │   ├── mod.rs
│   │   ├── system_info/         # 系统信息模块
│   │   │   ├── mod.rs
│   │   │   ├── os_info.rs
│   │   │   └── hardware_info.rs
│   │   ├── disk_scan/           # 磁盘扫描模块
│   │   │   ├── mod.rs
│   │   │   ├── scanner.rs
│   │   │   ├── walker.rs
│   │   │   └── progress.rs
│   │   ├── file_analyzer/       # 文件分析模块
│   │   │   ├── mod.rs
│   │   │   ├── garbage.rs
│   │   │   ├── large_file.rs
│   │   │   ├── duplicate.rs
│   │   │   └── classifier.rs
│   │   ├── cleaner/             # 清理执行模块
│   │   │   ├── mod.rs
│   │   │   ├── executor.rs
│   │   │   ├── recycle_bin.rs
│   │   │   └── safety.rs
│   │   ├── rule_engine/         # 规则引擎模块
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs
│   │   │   └── matcher.rs
│   │   └── scheduler/           # 定时任务模块
│   │       ├── mod.rs
│   │       └── task.rs
│   ├── models/                   # 数据模型
│   │   ├── mod.rs
│   │   ├── system.rs
│   │   ├── disk.rs
│   │   ├── file.rs
│   │   └── scan.rs
│   ├── utils/                    # 工具函数
│   │   ├── mod.rs
│   │   ├── path.rs
│   │   ├── hash.rs
│   │   └── size.rs
│   ├── error.rs                  # 错误处理
│   ├── lib.rs                    # 库入口
│   └── main.rs                   # 程序入口
├── Cargo.toml                    # Rust依赖配置
├── tauri.conf.json              # Tauri配置
└── icons/                        # 应用图标
```

### 3.2 核心模块设计

#### 3.2.1 系统信息模块

```rust
// modules/system_info/mod.rs
use serde::{Deserialize, Serialize};
use windows::Win32::System::SystemInformation::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub os_build: String,
    pub architecture: String,
    pub computer_name: String,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub cores: u32,
    pub logical_processors: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub available: u64,
    pub usage_percent: f32,
}

pub struct SystemInfoModule;

impl SystemInfoModule {
    pub fn get_system_info() -> Result<SystemInfo, crate::error::Error> {
        Ok(SystemInfo {
            os_name: Self::get_os_name()?,
            os_version: Self::get_os_version()?,
            os_build: Self::get_os_build()?,
            architecture: Self::get_architecture()?,
            computer_name: Self::get_computer_name()?,
            cpu_info: Self::get_cpu_info()?,
            memory_info: Self::get_memory_info()?,
        })
    }

    fn get_os_name() -> Result<String, crate::error::Error> {
        // 实现获取操作系统名称
    }

    fn get_memory_info() -> Result<MemoryInfo, crate::error::Error> {
        // 实现获取内存信息
    }
}
```

#### 3.2.2 磁盘扫描模块

```rust
// modules/disk_scan/mod.rs
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub path: PathBuf,
    pub mode: ScanMode,
    pub exclude_paths: Vec<PathBuf>,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanMode {
    Quick,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub scan_id: String,
    pub status: ScanStatus,
    pub files_scanned: u64,
    pub dirs_scanned: u64,
    pub current_path: Option<String>,
    pub total_size: u64,
    pub progress_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanStatus {
    Running,
    Paused,
    Completed,
    Cancelled,
    Error,
}

pub struct DiskScanner {
    scan_id: String,
    options: ScanOptions,
    progress: Arc<RwLock<ScanProgress>>,
    cancel_flag: Arc<RwLock<bool>>,
}

impl DiskScanner {
    pub fn new(options: ScanOptions) -> Self {
        let scan_id = uuid::Uuid::new_v4().to_string();
        Self {
            scan_id,
            options,
            progress: Arc::new(RwLock::new(ScanProgress {
                scan_id: scan_id.clone(),
                status: ScanStatus::Running,
                files_scanned: 0,
                dirs_scanned: 0,
                current_path: None,
                total_size: 0,
                progress_percent: 0.0,
            })),
            cancel_flag: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start_scan(&self) -> Result<ScanResult, crate::error::Error> {
        let mut result = ScanResult::default();

        for entry in WalkDir::new(&self.options.path)
            .max_depth(self.options.max_depth.unwrap_or(usize::MAX))
            .into_iter()
            .filter_entry(|e| !self.should_skip(e.path()))
        {
            if *self.cancel_flag.read().await {
                self.update_status(ScanStatus::Cancelled).await;
                break;
            }

            match entry {
                Ok(entry) => {
                    self.process_entry(&entry, &mut result).await?;
                    self.update_progress(&entry).await?;
                }
                Err(e) => {
                    // 记录错误但继续扫描
                    continue;
                }
            }
        }

        self.update_status(ScanStatus::Completed).await;
        Ok(result)
    }

    pub async fn pause(&self) {
        self.update_status(ScanStatus::Paused).await;
    }

    pub async fn resume(&self) {
        self.update_status(ScanStatus::Running).await;
    }

    pub async fn cancel(&self) {
        *self.cancel_flag.write().await = true;
    }

    async fn update_progress(&self, entry: &walkdir::DirEntry) -> Result<(), crate::error::Error> {
        let mut progress = self.progress.write().await;
        if entry.file_type().is_file() {
            progress.files_scanned += 1;
        } else if entry.file_type().is_dir() {
            progress.dirs_scanned += 1;
        }
        progress.current_path = Some(entry.path().to_string_lossy().to_string());
        Ok(())
    }

    async fn update_status(&self, status: ScanStatus) {
        self.progress.write().await.status = status;
    }

    fn should_skip(&self, path: &std::path::Path) -> bool {
        self.options.exclude_paths.iter().any(|p| path.starts_with(p))
    }

    async fn process_entry(&self, entry: &walkdir::DirEntry, result: &mut ScanResult) -> Result<(), crate::error::Error> {
        // 处理文件条目，更新结果
        Ok(())
    }
}
```

#### 3.2.3 文件分析模块

```rust
// modules/file_analyzer/mod.rs
use std::path::PathBuf;
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileAnalysisResult {
    pub garbage_files: Vec<GarbageFile>,
    pub large_files: Vec<LargeFile>,
    pub duplicate_groups: Vec<DuplicateGroup>,
    pub category_stats: HashMap<FileCategory, CategoryStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GarbageFile {
    pub path: PathBuf,
    pub size: u64,
    pub category: GarbageCategory,
    pub safe_to_delete: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GarbageCategory {
    SystemTemp,
    RecycleBin,
    BrowserCache,
    AppCache,
    LogFile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LargeFile {
    pub path: PathBuf,
    pub size: u64,
    pub modified_time: u64,
    pub accessed_time: u64,
    pub file_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<PathBuf>,
}

pub struct FileAnalyzer {
    large_file_threshold: u64,
    duplicate_min_size: u64,
}

impl FileAnalyzer {
    pub fn new() -> Self {
        Self {
            large_file_threshold: 100 * 1024 * 1024, // 100MB
            duplicate_min_size: 1024 * 1024, // 1MB
        }
    }

    pub fn analyze(&self, files: &[FileInfo]) -> FileAnalysisResult {
        FileAnalysisResult {
            garbage_files: self.find_garbage_files(files),
            large_files: self.find_large_files(files),
            duplicate_groups: self.find_duplicates(files),
            category_stats: self.calculate_category_stats(files),
        }
    }

    fn find_garbage_files(&self, files: &[FileInfo]) -> Vec<GarbageFile> {
        files.iter()
            .filter(|f| self.is_garbage_file(f))
            .map(|f| GarbageFile {
                path: f.path.clone(),
                size: f.size,
                category: self.get_garbage_category(f),
                safe_to_delete: self.is_safe_to_delete(f),
            })
            .collect()
    }

    fn find_large_files(&self, files: &[FileInfo]) -> Vec<LargeFile> {
        files.iter()
            .filter(|f| f.size >= self.large_file_threshold)
            .map(|f| LargeFile {
                path: f.path.clone(),
                size: f.size,
                modified_time: f.modified_time,
                accessed_time: f.accessed_time,
                file_type: self.get_file_type(&f.path),
            })
            .collect()
    }

    fn find_duplicates(&self, files: &[FileInfo]) -> Vec<DuplicateGroup> {
        let mut size_groups: HashMap<u64, Vec<&FileInfo>> = HashMap::new();

        for file in files.iter().filter(|f| f.size >= self.duplicate_min_size) {
            size_groups.entry(file.size).or_default().push(file);
        }

        let mut duplicates = Vec::new();
        for (_, group) in size_groups.iter().filter(|(_, g)| g.len() > 1) {
            let hash_groups = self.group_by_hash(group);
            for (_, files) in hash_groups.into_iter().filter(|(_, f)| f.len() > 1) {
                duplicates.push(DuplicateGroup {
                    hash: files[0].hash.clone(),
                    size: files[0].size,
                    files: files.iter().map(|f| f.path.clone()).collect(),
                });
            }
        }

        duplicates
    }

    fn calculate_file_hash(&self, path: &PathBuf) -> Result<String, std::io::Error> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn is_garbage_file(&self, file: &FileInfo) -> bool {
        // 判断是否为垃圾文件
        false
    }

    fn get_garbage_category(&self, file: &FileInfo) -> GarbageCategory {
        // 获取垃圾文件类别
        GarbageCategory::SystemTemp
    }

    fn is_safe_to_delete(&self, file: &FileInfo) -> bool {
        // 判断是否安全删除
        true
    }

    fn get_file_type(&self, path: &PathBuf) -> String {
        // 获取文件类型
        String::new()
    }

    fn group_by_hash(&self, files: &[&FileInfo]) -> HashMap<String, Vec<&FileInfo>> {
        HashMap::new()
    }

    fn calculate_category_stats(&self, files: &[FileInfo]) -> HashMap<FileCategory, CategoryStats> {
        HashMap::new()
    }
}
```

#### 3.2.4 清理执行模块

```rust
// modules/cleaner/mod.rs
use std::path::PathBuf;
use tokio::fs;
use windows::Win32::Storage::FileSystem::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanOptions {
    pub move_to_recycle_bin: bool,
    pub secure_delete: bool,
    pub secure_pass_count: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanResult {
    pub total_files: u64,
    pub cleaned_files: u64,
    pub failed_files: u64,
    pub total_size: u64,
    pub cleaned_size: u64,
    pub errors: Vec<CleanError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanError {
    pub path: PathBuf,
    pub error: String,
}

pub struct Cleaner {
    options: CleanOptions,
    protected_paths: Vec<PathBuf>,
}

impl Cleaner {
    pub fn new(options: CleanOptions) -> Self {
        Self {
            options,
            protected_paths: Self::get_protected_paths(),
        }
    }

    pub async fn clean(&self, files: Vec<PathBuf>) -> Result<CleanResult, crate::error::Error> {
        let mut result = CleanResult {
            total_files: files.len() as u64,
            cleaned_files: 0,
            failed_files: 0,
            total_size: 0,
            cleaned_size: 0,
            errors: Vec::new(),
        };

        for path in files {
            if self.is_protected(&path) {
                result.failed_files += 1;
                result.errors.push(CleanError {
                    path: path.clone(),
                    error: "受保护的文件".to_string(),
                });
                continue;
            }

            match self.clean_file(&path).await {
                Ok(size) => {
                    result.cleaned_files += 1;
                    result.cleaned_size += size;
                }
                Err(e) => {
                    result.failed_files += 1;
                    result.errors.push(CleanError {
                        path: path.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(result)
    }

    async fn clean_file(&self, path: &PathBuf) -> Result<u64, crate::error::Error> {
        let metadata = fs::metadata(path).await?;
        let size = metadata.len();

        if self.options.move_to_recycle_bin {
            self.move_to_recycle_bin(path)?;
        } else if self.options.secure_delete {
            self.secure_delete(path).await?;
        } else {
            fs::remove_file(path).await?;
        }

        Ok(size)
    }

    fn move_to_recycle_bin(&self, path: &PathBuf) -> Result<(), crate::error::Error> {
        // 使用Windows API移动到回收站
        Ok(())
    }

    async fn secure_delete(&self, path: &PathBuf) -> Result<(), crate::error::Error> {
        // 安全删除：多次覆写后删除
        let metadata = fs::metadata(path).await?;
        let size = metadata.len();

        for _ in 0..self.options.secure_pass_count {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(path)
                .await?;

            let zeros = vec![0u8; size as usize];
            use tokio::io::AsyncWriteExt;
            file.write_all(&zeros).await?;
            file.sync_all().await?;
        }

        fs::remove_file(path).await?;
        Ok(())
    }

    fn is_protected(&self, path: &PathBuf) -> bool {
        self.protected_paths.iter().any(|p| path.starts_with(p))
    }

    fn get_protected_paths() -> Vec<PathBuf> {
        vec![
            PathBuf::from(r"C:\Windows\System32"),
            PathBuf::from(r"C:\Windows\SysWOW64"),
            PathBuf::from(r"C:\Program Files"),
            PathBuf::from(r"C:\Program Files (x86)"),
        ]
    }
}
```

### 3.3 后端模块依赖关系图

```
┌─────────────────────────────────────────────────────────────────┐
│                      后端模块依赖关系                             │
└─────────────────────────────────────────────────────────────────┘

                        ┌─────────────────┐
                        │   Tauri App     │
                        │   (main.rs)     │
                        └────────┬────────┘
                                 │
                                 ▼
                        ┌─────────────────┐
                        │  Commands Layer │
                        │   (commands/)   │
                        └────────┬────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        │                        │                        │
        ▼                        ▼                        ▼
┌───────────────┐      ┌───────────────┐      ┌───────────────┐
│ system_info   │      │  disk_scan    │      │   cleaner     │
│   Module      │      │   Module      │      │   Module      │
└───────┬───────┘      └───────┬───────┘      └───────┬───────┘
        │                      │                      │
        │                      ▼                      │
        │              ┌───────────────┐              │
        │              │ file_analyzer │              │
        │              │   Module      │              │
        │              └───────┬───────┘              │
        │                      │                      │
        │              ┌───────┴───────┐              │
        │              ▼               ▼              │
        │      ┌───────────────┐ ┌───────────────┐    │
        │      │  rule_engine  │ │   scheduler   │    │
        │      │   Module      │ │   Module      │    │
        │      └───────────────┘ └───────────────┘    │
        │                                            │
        └────────────────────┬───────────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │     Models      │
                    │   (models/)     │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │     Utils       │
                    │   (utils/)      │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │  System APIs    │
                    │ (Windows/FS)    │
                    └─────────────────┘
```

### 3.4 模块职责说明

| 模块名称      | 职责描述                               | 主要接口                                                  |
| ------------- | -------------------------------------- | --------------------------------------------------------- |
| system_info   | 获取系统基本信息、硬件信息             | `get_system_info`, `get_disk_info`                        |
| disk_scan     | 文件系统遍历、扫描进度管理             | `start_scan`, `pause_scan`, `cancel_scan`, `get_progress` |
| file_analyzer | 垃圾文件识别、大文件分析、重复文件检测 | `analyze_junk`, `analyze_large_files`, `find_duplicates`  |
| cleaner       | 文件删除、回收站操作                   | `clean_files`, `move_to_trash`, `get_clean_report`        |
| rule_engine   | 清理规则管理、规则匹配                 | `get_rules`, `add_rule`, `remove_rule`                    |
| scheduler     | 定时扫描、定时清理                     | `create_task`, `update_task`, `delete_task`, `get_tasks`  |

### 3.5 Tauri 命令层

```rust
// commands/mod.rs
pub mod system;
pub mod disk;
pub mod scan;
pub mod clean;
pub mod settings;

// commands/system.rs
use tauri::State;
use crate::modules::system_info::SystemInfoModule;
use crate::models::SystemInfo;

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    SystemInfoModule::get_system_info()
        .map_err(|e| e.to_string())
}

// commands/disk.rs
use crate::modules::disk_scan::{DiskScanner, ScanOptions, ScanMode, ScanProgress};
use crate::models::DiskInfo;
use std::collections::HashMap;
use tauri::AppHandle;

#[tauri::command]
pub async fn get_disk_list() -> Result<Vec<DiskInfo>, String> {
    // 获取磁盘列表
    Ok(vec![])
}

#[tauri::command]
pub async fn start_scan(
    path: String,
    mode: ScanMode,
    app: AppHandle,
) -> Result<String, String> {
    let scanner = DiskScanner::new(ScanOptions {
        path: path.into(),
        mode,
        exclude_paths: vec![],
        max_depth: None,
    });

    let scan_id = scanner.scan_id.clone();

    tokio::spawn(async move {
        let result = scanner.start_scan().await;
        // 发送扫描完成事件
        app.emit("scan-complete", result).ok();
    });

    Ok(scan_id)
}

#[tauri::command]
pub async fn get_scan_progress(scan_id: String) -> Result<ScanProgress, String> {
    // 获取扫描进度
    Ok(ScanProgress::default())
}

#[tauri::command]
pub async fn pause_scan(scan_id: String) -> Result<(), String> {
    // 暂停扫描
    Ok(())
}

#[tauri::command]
pub async fn cancel_scan(scan_id: String) -> Result<(), String> {
    // 取消扫描
    Ok(())
}

// commands/clean.rs
use crate::modules::cleaner::{Cleaner, CleanOptions, CleanResult};
use std::path::PathBuf;

#[tauri::command]
pub async fn clean_files(
    files: Vec<String>,
    options: CleanOptions,
) -> Result<CleanResult, String> {
    let cleaner = Cleaner::new(options);
    let paths: Vec<PathBuf> = files.into_iter().map(PathBuf::from).collect();

    cleaner.clean(paths)
        .await
        .map_err(|e| e.to_string())
}
```

### 3.4 错误处理

```rust
// error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("系统错误: {0}")]
    System(String),

    #[error("扫描错误: {0}")]
    Scan(String),

    #[error("清理错误: {0}")]
    Clean(String),

    #[error("权限不足: {0}")]
    Permission(String),

    #[error("未知错误: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

---

## 4. 数据模型

### 4.1 前端类型定义

```typescript
// types/system.ts
export interface SystemInfo {
  osName: string;
  osVersion: string;
  osBuild: string;
  architecture: string;
  computerName: string;
  cpuInfo: CpuInfo;
  memoryInfo: MemoryInfo;
}

export interface CpuInfo {
  name: string;
  cores: number;
  logicalProcessors: number;
}

export interface MemoryInfo {
  total: number;
  available: number;
  usagePercent: number;
}

export interface DiskInfo {
  letter: string;
  label: string;
  totalSize: number;
  usedSize: number;
  freeSize: number;
  usagePercent: number;
  fileSystem: string;
  isSystem: boolean;
}

// types/scan.ts
export interface ScanOptions {
  path: string;
  mode: ScanMode;
  excludePaths: string[];
  maxDepth?: number;
}

export type ScanMode = "quick" | "deep";

export interface ScanProgress {
  scanId: string;
  status: ScanStatus;
  filesScanned: number;
  dirsScanned: number;
  currentPath?: string;
  totalSize: number;
  progressPercent: number;
}

export type ScanStatus =
  | "running"
  | "paused"
  | "completed"
  | "cancelled"
  | "error";

// types/file.ts
export interface FileInfo {
  path: string;
  name: string;
  size: number;
  modifiedTime: number;
  accessedTime: number;
  createdTime: number;
  isDirectory: boolean;
  extension?: string;
}

export interface GarbageFile {
  path: string;
  size: number;
  category: GarbageCategory;
  safeToDelete: boolean;
}

export type GarbageCategory =
  | "systemTemp"
  | "recycleBin"
  | "browserCache"
  | "appCache"
  | "logFile";

export interface LargeFile {
  path: string;
  size: number;
  modifiedTime: number;
  accessedTime: number;
  fileType: string;
}

export interface DuplicateGroup {
  hash: string;
  size: number;
  files: string[];
}

export interface CleanOptions {
  moveToRecycleBin: boolean;
  secureDelete: boolean;
  securePassCount: number;
}

export interface CleanResult {
  totalFiles: number;
  cleanedFiles: number;
  failedFiles: number;
  totalSize: number;
  cleanedSize: number;
  errors: CleanError[];
}

export interface CleanError {
  path: string;
  error: string;
}
```

### 4.2 数据流图

```
用户操作
    │
    ▼
┌─────────────┐
│ React组件   │
└──────┬──────┘
       │ dispatch action
       ▼
┌─────────────┐
│ Zustand Store│
└──────┬──────┘
       │ invoke Tauri command
       ▼
┌─────────────┐
│ Tauri IPC   │
└──────┬──────┘
       │ call Rust function
       ▼
┌─────────────┐
│ Rust Module │
└──────┬──────┘
       │ system call
       ▼
┌─────────────┐
│ Windows API │
└─────────────┘
```

### 4.3 跨层通信设计

#### 4.3.1 IPC 通信机制

Tauri IPC 是前后端通信的核心机制，支持同步和异步调用：

```typescript
// 前端调用示例
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// 同步调用（返回Promise）
const systemInfo = await invoke<SystemInfo>("get_system_info");

// 带参数调用
const scanId = await invoke<string>("start_scan", {
  path: "C:\\",
  mode: "quick",
});

// 监听后端事件
const unlisten = await listen<ScanProgress>("scan-progress", (event) => {
  console.log("扫描进度:", event.payload);
});

// 取消监听
unlisten();
```

```rust
// 后端命令定义
#[tauri::command]
async fn get_system_info() -> Result<SystemInfo, String> {
    SystemInfoModule::get_system_info()
        .map_err(|e| e.to_string())
}

// 发送事件到前端
app.emit("scan-progress", &progress).ok();
```

#### 4.3.2 数据序列化规范

**Rust 端序列化**：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub os_build: String,
    pub architecture: String,
    pub computer_name: String,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
}
```

**TypeScript 端类型定义**：

```typescript
// 使用camelCase命名
export interface SystemInfo {
  osName: string;
  osVersion: string;
  osBuild: string;
  architecture: string;
  computerName: string;
  cpuInfo: CpuInfo;
  memoryInfo: MemoryInfo;
}
```

**类型映射表**：

| Rust 类型     | TypeScript 类型 | 说明            |
| ------------- | --------------- | --------------- |
| String        | string          | 字符串          |
| u64           | number          | 64 位无符号整数 |
| u32           | number          | 32 位无符号整数 |
| f32           | number          | 32 位浮点数     |
| bool          | boolean         | 布尔值          |
| Vec<T>        | T[]             | 数组            |
| HashMap<K, V> | Record<K, V>    | 键值对          |
| Option<T>     | T \| null       | 可选值          |
| PathBuf       | string          | 路径            |

#### 4.3.3 错误传递机制

**错误码定义**：

| 错误码 | 类型       | 说明         |
| ------ | ---------- | ------------ |
| E001   | IO         | 文件读写错误 |
| E002   | Permission | 权限不足     |
| E003   | Scan       | 扫描过程错误 |
| E004   | Clean      | 清理过程错误 |
| E005   | System     | 系统调用错误 |
| E006   | Validation | 参数验证错误 |

**前端错误处理**：

```typescript
// services/errorHandler.ts
export class AppError extends Error {
  constructor(
    public code: string,
    public message: string,
    public details?: unknown
  ) {
    super(message);
    this.name = "AppError";
  }
}

export function handleTauriError(error: unknown): AppError {
  if (typeof error === "string") {
    const [code, ...messageParts] = error.split(":");
    return new AppError(code.trim(), messageParts.join(":").trim());
  }
  return new AppError("UNKNOWN", "未知错误", error);
}
```

**后端错误格式**：

```rust
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "E001: IO错误 - {}", e),
            Error::Permission(msg) => write!(f, "E002: 权限不足 - {}", msg),
            Error::Scan(msg) => write!(f, "E003: 扫描错误 - {}", msg),
            Error::Clean(msg) => write!(f, "E004: 清理错误 - {}", msg),
            Error::System(msg) => write!(f, "E005: 系统错误 - {}", msg),
            Error::Unknown(msg) => write!(f, "E006: 未知错误 - {}", msg),
        }
    }
}
```

#### 4.3.4 大数据传输优化

对于大量文件列表等大数据传输场景：

```rust
// 分批返回数据
#[tauri::command]
async fn get_file_list(
    scan_id: String,
    offset: usize,
    limit: usize,
) -> Result<FileListPage, String> {
    // 返回分页数据
    Ok(FileListPage {
        items: files[offset..offset+limit].to_vec(),
        total: files.len(),
        has_more: offset + limit < files.len(),
    })
}
```

```typescript
// 前端分页加载
async function loadFiles(scanId: string, page: number, pageSize: number) {
  const result = await invoke<FileListPage>("get_file_list", {
    scanId,
    offset: page * pageSize,
    limit: pageSize,
  });
  return result;
}
```

---

## 5. 安全设计

### 5.1 安全架构总览

```
┌─────────────────────────────────────────────────────────────────┐
│                       安全架构层次                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    用户确认层                            │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │   │
│  │  │ 删除确认弹窗 │ │ 风险提示    │ │ 操作撤销    │       │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    文件保护层                            │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │   │
│  │  │ 系统目录保护 │ │ 敏感文件识别 │ │ 扩展名过滤  │       │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    权限管理层                            │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │   │
│  │  │ 管理员检测   │ │ UAC提权     │ │ 权限不足处理 │       │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    数据安全层                            │   │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐       │   │
│  │  │ 配置加密     │ │ 日志脱敏    │ │ 临时文件清理 │       │   │
│  │  └─────────────┘ └─────────────┘ └─────────────┘       │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 文件保护机制

#### 5.2.1 受保护目录列表

| 目录路径                             | 保护级别 | 说明              |
| ------------------------------------ | -------- | ----------------- |
| C:\Windows\System32                  | 严格     | 系统核心文件      |
| C:\Windows\SysWOW64                  | 严格     | 64 位系统兼容文件 |
| C:\Program Files                     | 高       | 已安装程序        |
| C:\Program Files (x86)               | 高       | 32 位程序         |
| C:\ProgramData                       | 中       | 应用程序数据      |
| C:\Users\*\AppData\Local\Microsoft   | 中       | 微软应用数据      |
| C:\Users\*\AppData\Roaming\Microsoft | 中       | 微软漫游数据      |

#### 5.2.2 受保护扩展名

| 扩展名     | 保护级别 | 说明         |
| ---------- | -------- | ------------ |
| .sys       | 严格     | 系统驱动文件 |
| .dll       | 严格     | 动态链接库   |
| .exe       | 高       | 可执行文件   |
| .bat, .cmd | 高       | 批处理脚本   |
| .reg       | 高       | 注册表文件   |

#### 5.2.3 安全检查器实现

```rust
// modules/cleaner/safety.rs
pub struct SafetyChecker {
    protected_paths: Vec<PathBuf>,
    protected_extensions: Vec<String>,
    sensitive_patterns: Vec<String>,
}

impl SafetyChecker {
    pub fn new() -> Self {
        Self {
            protected_paths: vec![
                PathBuf::from(r"C:\Windows\System32"),
                PathBuf::from(r"C:\Windows\SysWOW64"),
                PathBuf::from(r"C:\Program Files"),
                PathBuf::from(r"C:\Program Files (x86)"),
                PathBuf::from(r"C:\ProgramData"),
            ],
            protected_extensions: vec![
                ".sys".to_string(),
                ".dll".to_string(),
                ".exe".to_string(),
                ".bat".to_string(),
                ".cmd".to_string(),
                ".reg".to_string(),
            ],
            sensitive_patterns: vec![
                "password".to_string(),
                "credential".to_string(),
                "key".to_string(),
            ],
        }
    }

    pub fn check_file(&self, path: &PathBuf) -> SafetyCheckResult {
        if self.is_protected_path(path) {
            return SafetyCheckResult::ProtectedPath;
        }
        if self.is_protected_extension(path) {
            return SafetyCheckResult::ProtectedExtension;
        }
        if self.is_sensitive_file(path) {
            return SafetyCheckResult::SensitiveFile;
        }
        SafetyCheckResult::Safe
    }

    pub fn is_safe_to_delete(&self, path: &PathBuf) -> bool {
        !self.is_protected_path(path) && !self.is_protected_extension(path)
    }

    fn is_protected_path(&self, path: &PathBuf) -> bool {
        self.protected_paths.iter().any(|p| path.starts_with(p))
    }

    fn is_protected_extension(&self, path: &PathBuf) -> bool {
        path.extension()
            .map(|ext| self.protected_extensions.contains(&ext.to_string_lossy().to_lowercase()))
            .unwrap_or(false)
    }

    fn is_sensitive_file(&self, path: &PathBuf) -> bool {
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        self.sensitive_patterns.iter().any(|p| file_name.contains(p))
    }
}

pub enum SafetyCheckResult {
    Safe,
    ProtectedPath,
    ProtectedExtension,
    SensitiveFile,
}
```

### 5.3 权限管理

#### 5.3.1 管理员权限检测

```rust
// utils/permission.rs
use windows::Win32::Security::*;
use windows::Win32::Foundation::*;

pub fn check_admin_privileges() -> bool {
    unsafe {
        let mut sid: PSID = PSID::default();
        let mut admin_token: HANDLE = HANDLE::default();

        // 创建管理员SID
        let result = AllocateAndInitializeSid(
            &SECURITY_NT_AUTHORITY,
            2,
            SECURITY_BUILTIN_DOMAIN_RID,
            DOMAIN_ALIAS_RID_ADMINS,
            0, 0, 0, 0, 0, 0,
            &mut sid,
        );

        if !result.as_bool() {
            return false;
        }

        // 检查当前进程是否具有管理员权限
        let mut is_admin = FALSE;
        CheckTokenMembership(HANDLE::default(), sid, &mut is_admin);

        FreeSid(sid);
        is_admin.as_bool()
    }
}

pub fn request_admin_privileges() -> Result<(), Error> {
    // 使用ShellExecute请求UAC提权
    // 重新启动应用并请求管理员权限
    Ok(())
}
```

#### 5.3.2 权限不足处理策略

| 场景             | 处理方式               |
| ---------------- | ---------------------- |
| 扫描系统目录     | 跳过并记录日志         |
| 清理受保护文件   | 提示用户需要管理员权限 |
| 访问其他用户目录 | 提示权限不足           |

### 5.4 数据安全

#### 5.4.1 配置文件加密

```rust
// utils/crypto.rs
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::{Engine as _, engine::general_purpose};

pub fn encrypt_config(data: &str, key: &[u8; 32]) -> Result<String, Error> {
    // AES-256-GCM加密
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(b"unique nonce");
    let ciphertext = cipher.encrypt(nonce, data.as_bytes())
        .map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(general_purpose::STANDARD.encode(&ciphertext))
}

pub fn decrypt_config(encrypted: &str, key: &[u8; 32]) -> Result<String, Error> {
    let ciphertext = general_purpose::STANDARD.decode(encrypted)
        .map_err(|e| Error::Crypto(e.to_string()))?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(b"unique nonce");
    let plaintext = cipher.decrypt(nonce, ciphertext.as_slice())
        .map_err(|e| Error::Crypto(e.to_string()))?;
    Ok(String::from_utf8_lossy(&plaintext).to_string())
}
```

#### 5.4.2 日志脱敏

```rust
// utils/logger.rs
pub fn sanitize_path(path: &str) -> String {
    // 隐藏用户名
    path.replace_regex(r"C:\\Users\\[^\\]+", "C:\\Users\\***")
}

pub fn sanitize_log_message(msg: &str) -> String {
    let mut result = msg.to_string();
    // 隐藏可能的敏感信息
    result = result.replace_regex(r"password=\S+", "password=***");
    result = result.replace_regex(r"token=\S+", "token=***");
    result
}
```

### 5.5 删除确认机制

```typescript
// 前端删除确认流程
interface DeleteConfirmation {
  fileCount: number;
  totalSize: number;
  riskLevel: "low" | "medium" | "high";
  categories: CategorySummary[];
  warnings: string[];
}

async function confirmDelete(files: FileInfo[]): Promise<boolean> {
  const confirmation = await analyzeDeleteRisk(files);

  if (confirmation.riskLevel === "high") {
    return showHighRiskDialog(confirmation);
  }

  return showNormalConfirmDialog(confirmation);
}
```

---

## 6. 性能优化策略

### 6.1 扫描优化

- **多线程并行扫描**：使用 Tokio 异步运行时
- **增量扫描**：记录上次扫描结果
- **智能跳过**：跳过系统目录和虚拟目录
- **批量处理**：批量收集文件信息

### 6.2 内存优化

- **流式处理**：避免一次性加载所有数据
- **分页加载**：大列表分页展示
- **及时释放**：处理完成后释放资源

### 6.3 UI 优化

- **虚拟列表**：大列表虚拟滚动
- **懒加载**：组件按需加载
- **防抖节流**：频繁操作优化

---

## 7. 测试策略

### 7.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info() {
        let info = SystemInfoModule::get_system_info().unwrap();
        assert!(!info.os_name.is_empty());
    }

    #[test]
    fn test_file_hash() {
        let analyzer = FileAnalyzer::new();
        // 测试文件哈希计算
    }
}
```

### 7.2 集成测试

- 前后端集成测试
- API 接口测试
- 文件操作测试

### 7.3 性能测试

- 扫描性能测试
- 内存占用测试
- UI 响应测试

---

## 8. 架构评审与优化

### 8.1 架构评审清单

| 评审项         | 状态    | 说明                           |
| -------------- | ------- | ------------------------------ |
| 前后端职责分离 | ✅ 通过 | React 前端与 Rust 后端职责清晰 |
| 模块边界定义   | ✅ 通过 | 各模块职责单一、边界清晰       |
| 数据流清晰     | ✅ 通过 | IPC 通信机制明确，数据流向清晰 |
| 安全机制完善   | ✅ 通过 | 多层安全保护机制               |
| 性能优化策略   | ✅ 通过 | 多维度性能优化方案             |
| 可扩展性       | ✅ 通过 | 模块化设计，易于扩展           |
| reactbits 集成 | ✅ 通过 | 组件库深度融合，视觉效果优秀   |

### 8.2 潜在风险与应对

| 风险                 | 可能性 | 影响 | 应对措施                   |
| -------------------- | ------ | ---- | -------------------------- |
| Tauri 2.x API 变化   | 中     | 高   | 关注官方更新，预留适配时间 |
| reactbits 组件兼容性 | 低     | 中   | 版本锁定，定期更新评估     |
| Windows 版本兼容性   | 中     | 中   | 充分测试 Win10/Win11       |
| 扫描性能瓶颈         | 中     | 中   | 多线程优化，增量扫描       |
| 内存占用过高         | 低     | 中   | 流式处理，分页加载         |

### 8.3 优化建议

#### 8.3.1 前端优化

1. **组件懒加载**：使用 React.lazy 实现路由级懒加载
2. **状态持久化**：关键状态使用 localStorage 持久化
3. **动画性能**：使用 GPU 加速，避免重排重绘
4. **虚拟列表**：大文件列表使用虚拟滚动

#### 8.3.2 后端优化

1. **并行扫描**：使用 Tokio 多任务并行处理
2. **缓存机制**：扫描结果缓存，避免重复计算
3. **内存池**：大文件处理使用内存池复用
4. **增量更新**：支持增量扫描和清理

#### 8.3.3 通信优化

1. **批量传输**：大数据分批传输
2. **事件节流**：进度更新使用节流机制
3. **压缩传输**：大数据使用压缩传输

### 8.4 架构演进方向

```
┌─────────────────────────────────────────────────────────────────┐
│                     架构演进路线图                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  v1.0 (当前)                                                    │
│  ├── 基础架构搭建                                                │
│  ├── reactbits组件集成                                          │
│  └── 核心功能实现                                                │
│                                                                 │
│  v1.1                                                           │
│  ├── 插件系统支持                                                │
│  ├── 自定义主题                                                  │
│  └── 多语言支持                                                  │
│                                                                 │
│  v2.0                                                           │
│  ├── 云端同步                                                    │
│  ├── 智能推荐                                                    │
│  └── 高级分析                                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 9. 附录

### 9.1 技术栈版本清单

| 技术          | 版本   | 说明       |
| ------------- | ------ | ---------- |
| React         | 18.x   | 前端框架   |
| TypeScript    | 5.x    | 类型系统   |
| Tailwind CSS  | 3.x    | 样式方案   |
| Ant Design    | 5.x    | UI 组件库  |
| reactbits     | latest | 动画组件库 |
| Framer Motion | latest | 动画库     |
| GSAP          | latest | 高级动画   |
| ECharts       | 5.x    | 数据可视化 |
| Zustand       | 4.x    | 状态管理   |
| Tauri         | 2.x    | 桌面框架   |
| Rust          | 1.75+  | 后端语言   |
| Tokio         | 1.x    | 异步运行时 |

### 9.2 参考文档

- [Tauri 2.x 官方文档](https://tauri.app/v2/guide/)
- [React 官方文档](https://react.dev/)
- [Tailwind CSS 文档](https://tailwindcss.com/docs)
- [Ant Design 文档](https://ant.design/docs/react/introduce-cn)
- [reactbits 组件库](https://www.reactbits.dev/)
- [Rust 官方文档](https://doc.rust-lang.org/)
- [windows-rs 文档](https://microsoft.github.io/windows-rs/)
