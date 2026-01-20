# Phase 1: 核心引擎与路由系统 (Core & Routing)

## 1. 阶段目标
本阶段旨在构建引擎的最小可行性产品 (MVP)，实现从启动引擎到解析 Markdown 脚本并显示画面的完整流程。

## 2. 核心任务拆解

### 2.1 基础设施搭建 (Infrastructure)
- [ ] **SDL2 窗口初始化**
  - 初始化 SDL2 Video & Events 子系统。
  - 创建 1280x720 (或其他基准分辨率) 的 Window。
  - 建立 OpenGL 3.3 Core Profile 上下文。
- [ ] **Skia 渲染环境**
  - 集成 `rust-skia`。
  - 创建 `DirectContext` 绑定到 OpenGL 上下文。
  - 创建 `Surface` 并获取 Canvas。
  - **验证**: 能在窗口中绘制一个红色的矩形。

### 2.2 Markdown 解析与路由 (Routing System)
- [ ] **Markdown 解析器**
  - 引入 `pulldown-cmark` 或类似 crate。
  - 定义自定义语法扩展：
    - `!{...}` (注解指令)
    - `[Next](target)` (链接跳转)
- [ ] **自动路由表生成 (Auto-Router)**
  - 启动时递归扫描 `assets/scripts/` 目录。
  - 解析每个 `.md` 文件的 Frontmatter (YAML 头) 获取 ID 和标题。
  - 构建 `HashMap<String, PathBuf>` 映射表 (Route ID -> File Path)。
  - **验证**: 打印出生成的路由表。

### 2.3 基础渲染器 (Basic Renderer)
- [ ] **资源加载基础**
  - 实现简单的同步文件读取。
  - 加载 PNG/JPG 图片为 `skia_safe::Image`。
  - 加载 TTF 字体。
- [ ] **Level 1 注解 UI 实现**
  - 解析 `!{bg: "path"}` -> 渲染全屏背景。
  - 解析 `!{char: "path", pos: ...}` -> 渲染立绘 (支持 center, left, right)。
  - 解析普通文本 -> 渲染到底部对话框 (Dialog Box)。

## 3. 技术难点与注意
- **坐标系转换**: 需要处理 逻辑分辨率 (Design Resolution) 到 物理分辨率 (Window Size) 的映射，实现 `Letterbox` 模式。
- **所有权管理**: 路由表需要在多个模块间共享，考虑使用 `Arc<RwLock<RouteTable>>`。
