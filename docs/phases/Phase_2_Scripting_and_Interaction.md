# Phase 2: 脚本交互与热重载 (Scripting & Interaction)

## 1. 阶段目标
本阶段主要引入 Lua 脚本层，接管游戏状态管理，并实现 Hoshimi 专用脚本格式 (`.hmd`) 的解析与核心的“开发即预览”体验（热重载）。

## 2. 核心任务拆解

### 2.1 脚本解析器增强 (HMD Parser)
- [ ] **剧本格式解析**
  - 实现 `.hmd` 解析器，区分普通 Markdown 与游戏指令。
  - **角色识别**: 实现“加粗即角色” (`**Name**:`) 策略，将其他段落视为旁白。
  - **指令解析**: 解析 `!{ key: value }` 行级注解（Annotation）。
- [ ] **解析内联变量**
  - 支持在对话文本中通过 `${GameVars.flag}` 语法进行动态插值。

### 2.2 Lua 绑定 (Lua Integration)
- [ ] **集成 mlua**
  - 配置 `mlua` (使用 Lua 5.4 或 LuaJIT)。
  - 创建全局 Lua 虚拟机实例。
- [ ] **变量管理系统 (Variable System)**
  - 在 Lua 中维护 `GameVars` 全局表。
  - 实现 Rust 访问 Lua 变量的接口。

### 2.3 逻辑控制 (Logic Control)
- [ ] **逻辑钩子 (Logic Hooks)**
  - 解析 `<< code >>` 语法块，直接执行内部的 Lua 代码。
  - 实现 `<< if condition >> ... << else >> ... << end >>` 的条件分支控制流。
- [ ] **选项系统 (Choices)**
  - 识别无序列表中的**链接项** (`- [Text](link)`) 为交互选项。
  - 支持解析列表项下的缩进注解 (`!{ ... }`) 来配置选项属性（如音效、消耗）。

### 2.4 基础热重载 (Hot Reloading)
- [ ] **文件监听器**
  - 使用 `notify` crate 监听 `assets/` 目录。
- [ ] **重载策略**
  - **HMD 变更**: 不重启引擎，重新解析当前 `.hmd` 文件，刷新上下文。
  - **Lua 变更**: 重新加载 Lua 文件 (`dofile`)，但需要设计“热状态保留”机制 (Hot State Preservation)，确保 `GameVars` 不会被重置。
  - **验证**: 运行时在 VS Code 修改剧本台词，游戏窗口内即时更新。

## 3. 技术难点与注意
- **解析器稳定性**: 处理不规范的 Markdown 语法（如忘记空行），提供友好的错误提示。
- **状态一致性**: 热重载导致脚本行号变化时，尽量保留当前对话进度（Smart Resume），或者回退到最近的 Label/Block。
