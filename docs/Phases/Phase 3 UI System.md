# Phase 3: UI 系统增强 (UI System Enhancement)

## 1. 阶段目标
将 UI 系统从硬编码升级为数据驱动，支持设计师使用 Hoshimi UI DSL (`.hui`) 定义复杂的界面，并实现动态渲染能力。

## 2. 核心任务拆解

### 2.1 UI DSL 解析 (DSL Parser)
- [ ] **DSL 语法定义**
  - 实现基于 KDL/Rust-Struct 风格的 `.hui` 解析器。
  - **核心节点**: `Screen`, `VBox`, `HBox`, `ZStack`, `Image`, `Button`, `Text`.
  - **属性解析**: 支持字符串、数字、布尔值及 Hex 颜色。
- [ ] **UI 树与布局 (Layout Engine)**
  - 构建 UI Node Tree。
  - 实现简化版 Flexbox 布局 (`align_x`, `align_y`, `padding`, `margin`, `spacing`)。

### 2.2 动态渲染与逻辑 (Dynamic Rendering)
- [ ] **流程控制节点**
  - **条件渲染**: 实现 `If { condition: "${...}" }` 节点，根据 Lua 表达式动态挂载/卸载子树。
  - **列表渲染**: 实现 `For { each: "item", in: "${list}" }` 节点，遍历 Lua 数组生成 UI 副本。
- [ ] **数据绑定 (Data Binding)**
  - 实现 Text/Value 属性的单向绑定：`${Player.gold}`。
  - 建立 Rust 端的观察者机制，当 Lua 数据变更时标记 UI 脏区 (Dirty Rect)。

### 2.3 交互事件系统 (Event System)
- [ ] **事件映射**
  - 实现 `on_click`, `on_hover` 属性绑定到 Lua 函数（如 `System.startGame`）。
  - 处理事件冒泡与拦截。

### 2.4 Lua 绘图接口 (Level 3 API)
- [ ] **Canvas 暴露**
  - 将 `skia_safe::Canvas` 包装为 Lua UserData。
  - 暴露 `drawRect`, `drawCircle`, `drawImage`, `drawText` 等基础 API。
- [ ] **Paint 对象封装**
  - 封装 `skia_safe::Paint`，允许 Lua 设置颜色、透明度、混合模式。

## 3. 技术难点与注意
- **Diff 算法**: 对于 `For` 循环列表，当源数据发生变化时，尽量进行增量更新（Diffing），避免销毁重建整个列表以保证性能。
- **Lua 上下文隔离**: `For` 循环中的局部变量 (`each`) 不应污染全局 Lua 环境，需要实现临时的 Scope Stack。
