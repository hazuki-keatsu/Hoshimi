# Hoshimi 引擎开发文档

## 1. 项目简介
Hoshimi 是一款基于 Rust 语言开发的高性能、跨平台 GalGame/Visual Novel 引擎。
本项目核心理念为 **内容驱动 (Content-Driven)**，旨在通过 **Markdown-First** 的开发流最大限度降低创作者的编码负担。引擎内置了路由自动生成、资源自动分段加载等智能化功能，同时提供强大的插件系统和 Lua 脚本支持，兼顾易用性与高拓展性。

## 2. 技术架构

### 2.1 核心技术栈
本项目采用 **Rust + Skia + SDL2 + Lua** 的技术组合：

- **开发语言**: Rust (内存安全、高性能、零 GC)
- **渲染引擎**: Skia (Google 维护的高性能 2D 图形库，Chrome/Flutter 核心)
- **窗口/输入/音频**: SDL2 (成熟的跨平台媒体抽象层)
- **脚本语言**: Lua 5.x (通过 mlua 绑定，负责变量管理与逻辑控制)
- **UI 系统**: 三层架构 (Markdown 注解 -> DSL -> Lua API)
- **视频支持**: FFmpeg (通过 ffmpeg-next 绑定)
- **资源管理**: 自动化资源分段与懒加载

### 2.2 架构分层
引擎整体架构自底向上分为四层：

1.  **平台适配层 (HAL)**: SDL2 窗口管理、输入事件泵、OpenGL 上下文。
2.  **引擎核心层 (Core)**: 
    - **Renderer**: Skia 渲染封装，支持插件注入渲染指令。
    - **Plugin System**: 插件生命周期管理（Init, Update, Render Hooks）。
    - **Router**: 路由表管理器，负责解析 Markdown 生成场景图。
3.  **脚本与数据层 (Scripting)**:
    - **Lua VM**: 管理 Global Variables (Flag)，处理复杂逻辑判断(Luau API 风格)。
    - **Resource Manager**: 基于路由表自动预测并分段加载/卸载资源。
4.  **内容表现层 (Content)**:
    - **Markdown Scripts**: 游戏剧本与基础演出。
    - **DSL UI Types**: 组件化 UI 描述。

## 3. 详细模块设计

### 3.1 内容驱动核心 (Content-Driven Core)
引擎不再强制要求编写大量初始化代码，而是以 Markdown 文件为单一事实来源 (Single Source of Truth)。

- **自动路由表 (Auto-Routing)**: 
    - 引擎启动时扫描 `assets/scripts/*.md`。
    - 根据文件名和 Frontmatter 生成路由表 (Route Table)。
    - **跳转实现**: 开发者仅需在 Markdown 中书写 `[Next Chapter](chapter2.md)`，引擎自动处理场景切换与旧资源释放。
- **自动分段加载 (Auto-Segmentation)**:
    - 解析 Markdown 内容，按章节 (Chapter) 或 场景 (Scene) 切分资源组。
    - 给每一个 Markdown 最小的剧情片段添加一个指针，然后根据用户的配置文件去自动释放这个指针前多少的资源，和自动加载后多少的资源，这些都是异步加载进入内存的。
    - 既然路由表已知，引擎可实现 **预加载 (Pre-load)** 下一个可能跳转的场景资源，并自动 **GC (Garbage Configure)** 远离当前路径的资源。

### 3.2 三层 UI 系统 (Tri-Layer UI System)
为了平衡易用性与灵活性，UI 系统被设计为三个层级：

#### **Level 1: Markdown 注解 UI (Annotation UI)**
*面向编剧与非程序开发人员。*
通过扩展 Markdown 语法快速构建标准 AVG 界面。
- **背景**: `!{bg: "school_day"}` -> 自动创建全屏背景层。
- **立绘**: `!{char: "hoshimi", pos: center, face: smile}` -> 自动管理立绘图层与位置。
- **对话**: 普通文本段落自动渲染到默认对话框中。

#### **Level 2: 组件式 DSL UI (Component DSL)**
*面向 UI 设计师。*
为了避免 XML 的繁琐，采用类 Rust 结构体声明的 DSL (Domain Specific Language) 或简化版 JSON/TOML 定义不规则 UI 布局。
```lua
-- layouts/title_screen.ui
Screen {
    id = "title_main",
    children = {
        Image { src = "logo.png", align = "top-center", listen = "hover_anim" },
        VBox {
            y = 500,
            Button { text = "Start Game", action = "route:chapter1" },
            Button { text = "Load", action = "sys:load_menu" }
        }
    }
}
```
引擎将 DSL 解析为 UI Tree，自动处理布局计算。

#### **Level 3: Luau 绘图 API UI (Imperative API)**
*面向高级程序员/插件开发者。*
暴露 Skia 的底层 Canvas 绘图能力给 Lua，用于实现极度复杂的自定义控件或特效。
```lua
function on_render(canvas)
    local paint = Paint.new()
    paint:setBlur(5.0)
    canvas:drawCircle(100, 100, 50, paint)
end
```

### 3.3 插件扩展模块 (Plugin System)
插件模块是引擎扩展性的基石。

- **设计思路**: 基于 **生命周期钩子 (Lifecycle Hooks)**。
- **实现方式**:
    1.  Rust 端定义 `Plugin` Trait，包含 `on_init`, `on_update`, `on_render`, `on_event` 等方法。
    2.  Lua 端提供 `Engine.register_plugin(plugin_table)` 接口。
- **注入渲染 (Render Injection)**:
    - 渲染管线分为：`Background` -> `Character` -> `Plugin_Layer_Bottom` -> `UI` -> `Plugin_Layer_Top`。
    - 插件可以在注册时指定 `layer_order`，决定绘图指令插入到哪一层。

### 3.4 脚本与变量管理 (Lua Scripting)
虽然流程控制由 Markdown 接管，但 Lua 依然负责核心状态管理。
- **全局状态 (Global State)**: `GameVars` 表存储好感度、物品栏等，自动随存档序列化。
- **逻辑回调**: Markdown 中可嵌入 Lua 钩子，例如 `[Check]{ if GameVars.money < 100 then return "bad_end" end }`。

### 3.5 热重载 (Hot Reload System)
结合路由表实现极其快速的开发迭代：
1.  **文件监听**: 监听 Markdown、Lua、DSL 文件变动，然后用户手动通过命令行工具或者是后文提到的 Debug Overlay 来对指定的修改部分进行重载。
2.  **即时重绘**: 
    - 若修改了 **UI DSL**：仅重构 UI 树，保持游戏状态不变。
    - 若修改了 **Markdown**：引擎根据当前路由锚点，重新解析当前段落，刷新显示。
    - 若修改了 **Lua 逻辑**：热替换函数定义，保持 `GameVars` 数据不丢失。
3.  **开发工具**: 提供一个 Debug Overlay，显示当前路由路径，支持一键跳转任意节点。

## 4. 开发环境搭建

### 4.1 前置依赖
1.  **Rust Toolchain**: 安装最新 stable 版本。
2.  **C++ 编译环境** (Skia 构建依赖):
    - Windows: Visual Studio Build Tools (MSVC)
    - macOS: Xcode Command Line Tools
    - Linux: clang, build-essential
3.  **库依赖** (需确保系统路径中存在):
    - SDL2 development libraries
    - Python 3 (构建 Skia 脚本需要)

### 4.2 构建步骤
```bash
# 克隆项目
git clone https://github.com/hazuki-keatsu/Hoshimi.git
cd Hoshimi

# 下载并编译依赖 (首次运行较慢，需编译 Skia)
# 注意：确保网络环境能连接到 Google 仓库或配置了 Skia 镜像
cargo build

# 运行 Demo
cargo run
```

## 5. 项目路线图 (Roadmap)
- [ ] **Phase 1: 核心与路由** [查看详细文档](phases/Phase_1_Core_and_Routing.md)
    - 搭建 Rust + Skia + SDL2 基础窗口。
    - 实现 Markdown 解析器与自动路由表生成。
    - 实现 Level 1 注解 UI (背景/立绘显示)。
- [ ] **Phase 2: 脚本与交互** [查看详细文档](phases/Phase_2_Scripting_and_Interaction.md)
    - 集成 mlua，挂载变量管理系统。
    - 实现基础的热重载 (Markdown 修改即时刷新)。
- [ ] **Phase 3: UI 系统增强** [查看详细文档](phases/Phase_3_UI_System.md)
    - 研发组件 DSL 解析器，摆脱硬编码 UI。
    - 实现 Level 3 Lua 绘图接口。
- [ ] **Phase 4: 插件与多媒体** [查看详细文档](phases/Phase_4_Plugins_and_Multimedia.md)
    - 设计插件 Registry 与渲染钩子注入。
    - 接入 FFmpeg 视频层。
- [ ] **Phase 5: 工具化与发布** [查看详细文档](phases/Phase_5_Tooling_and_Publishing.md)
    - 完善自动资源分段/打包算法。
    - 制作 Debug Overlay 路由跳转器。

## 6. 项目避坑点

1. Skia 的 OpenGL 版本不匹配：在 SDL2 初始化时，固定设置OpenGL 3.3 Core，这个版本是所有平台的兼容黄金版本，Windows/macOS/Linux/Android 都完美支持，Skia 的 OpenGL 后端对这个版本做了极致优化。
2. Skia 的纹理内存泄漏：GalGame 的立绘 / 背景都是「按需加载、用完释放」，用 Rust 的智能指针（Arc\<Image\>）封装图片资源，配合纹理池复用内存，Skia 的 Image 实现了 Drop trait，Rust 会自动释放内存，无需手动管理。
3. SDL2 的音频和渲染帧不同步：用 SDL2 的主循环驱动音频，不要用异步音频库，SDL2 的mixer模块和事件循环是同源的，帧率固定 60 帧后，音画同步率 100%，GalGame 的音画同步问题完美解决。
4. Skia 的文本渲染乱码 / 中日文显示异常：编译 Skia 时开启 textlayout 特性；加载带中日文的 TTF 字体（比如思源黑体、方正兰亭），不要用系统默认字体；Rust 的字符串是UTF-8，直接传给 Skia 即可，无需转码，完美支持中日文。
5. 跨平台编译时 SDL2/Skia 链接失败： Rust 的 cargo 的静态链接特性，静态链接后，编译出的二进制文件不依赖任何系统库，全平台直接运行，无需安装运行时。
