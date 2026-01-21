# Hoshimi 插件系统开发文档 (Plugin System Development Guide)

本文档详细描述了 Hoshimi 引擎插件系统的设计规范、生命周期钩子暴露机制以及开发方法。

## 1. 系统架构 (Architecture)

Hoshimi 的插件系统旨在让开发者通过 **Lua 脚本** 动态扩展引擎功能，而无需重新编译 Rust 核心代码。

插件系统基于 **Rust (Host) - Lua (Guest)** 的互操作模型：
1.  **Rust 核心层**: 负责定义 `Plugin` Trait，管理插件注册表 (Registry)，并在主循环 (Main Loop) 的特定阶段分发调用。
2.  **Lua 插件层**: 实现特定的生命周期回调函数 (Callbacks)，通过 `Engine.register_plugin` 注册到系统中。

### 1.1 核心设计目标
*   **非侵入式**: 插件不应修改引擎核心逻辑，而是“挂载”在核心流程之上。
*   **高性能渲染**: 直接暴露 Skia Canvas 给 Lua，利用 Luau/LuaJIT 的高性能实现每帧绘制。
*   **层级控制**: 允许插件指定自己在渲染管线中的层级 (Layer Order)。

## 2. 插件生命周期 (Lifecycle Hooks)

插件本质上是一个包含特定函数的 Lua Table。引擎会在特定的时间点调用这些函数。

### 2.1 基础结构
一个标准的插件文件 (`plugins/my_plugin.lua`) 结构如下：

```lua
local MyPlugin = {
    id = "com.user.weather_effects",
    version = "1.0.0",
    layer = "foreground", -- 指定渲染层级
    active = true
}

-- 初始化
function MyPlugin:on_init()
    -- 加载资源，初始化变量
end

-- 事件处理
function MyPlugin:on_event(event)
    -- 处理键盘、鼠标事件
    -- 返回 true 表示事件已消费，不再传递
    return false 
end

-- 逻辑更新
function MyPlugin:on_update(dt)
    -- dt: 自上一帧以来的时间差 (秒)
end

-- 渲染
function MyPlugin:on_render(canvas)
    -- canvas: Skia Canvas 对象
end

-- 销毁
function MyPlugin:on_destroy()
    -- 清理资源
end

return MyPlugin
```

### 2.2 钩子详解

#### `on_init()`
*   **触发时机**: 插件被 `PluginManager` 加载并注册成功后及第一帧开始前。
*   **用途**: 加载图片、字体资源，初始化内部状态变量。

#### `on_update(dt)`
*   **触发时机**: 每一帧逻辑更新阶段 (Update Phase)。
*   **参数**: `dt` (number) - Delta Time，单位秒。
*   **用途**: 更新粒子位置、计算动画插值、处理计时器。

#### `on_render(canvas)`
*   **触发时机**: 每一帧渲染阶段 (Render Phase)。
*   **参数**: `canvas` (UserData) - 绑定的 Skia Canvas 对象。
*   **用途**: 发出绘图指令。这是插件最强大的部分。

#### `on_event(event)`
*   **触发时机**: 当 SDL2 接收到输入事件时。
*   **参数**: `event` (Table) - 包含 `type`, `key`, `x`, `y` 等字段。
*   **返回值**: `boolean` - 如果返回 `true`，引擎将停止向后续系统（如 UI 或 路由）传递该事件（事件吞噬）。

#### `on_destroy()`
*   **触发时机**: 引擎关闭或插件被卸载时。
*   **用途**: 释放非托管资源（通常 Rust 会自动管理内存，此钩子主要用于保存数据或清理临时文件）。

## 3. 渲染注入系统 (Render Injection System)

为了让插件既能绘制背景（如天气效果），又能绘制前景（如 Debug 浮层），我们定义了严格的渲染层级。

### 3.1 渲染管线层级
引擎的渲染循环按以下顺序执行：

| 层级名称 (Enum) | 说明 | 对应常量 | 典型应用 |
| :--- | :--- | :--- | :--- |
| **Background** | 最底层，在场景背景图之前 | `RenderLayer.Background` | 动态天空盒、纯色背景 |
| **Scene_Bottom** | 场景层底部，背景图之后，立绘之前 | `RenderLayer.SceneBottom` | 背景上的动态物体 |
| **Scene_Top** | 场景层顶部，立绘之后，UI 之前 | `RenderLayer.SceneTop` | 雨/雪/雾天气效果、全屏滤镜 |
| **UI_Bottom** | UI 层底部，在标准 UI 绘制前 | `RenderLayer.UIBottom` | 复杂的自定义 UI 背景 |
| **UI_Top** | 最顶层，在标准 UI 绘制后 | `RenderLayer.UITop` | 鼠标特效、Debug 信息、FPS 计数器 |

### 3.2 指定层级
在 Lua 插件表中通过 `layer` 字段指定：
```lua
MyPlugin.layer = "scene_top"
```

## 4. 核心 API 暴露设计 (Rust -> Lua)

为了支撑上述功能，Rust 端需要通过 `mlua` 向 Lua 暴露以下核心对象。

### 4.1 Canvas API (Skia Binding)
这是 `on_render` 的核心。

```rust
// Rust 伪代码：Canvas 方法绑定
impl UserData for Canvas {
    fn add_methods(methods: &mut UserDataMethods<Self>) {
        methods.add_method("draw_image", |_, this, (img, x, y)| { ... });
        methods.add_method("draw_rect", |_, this, (x, y, w, h, paint)| { ... });
        methods.add_method("draw_text", |_, this, (text, x, y, font, paint)| { ... });
        methods.add_method("save", ...);
        methods.add_method("restore", ...);
        methods.add_method("translate", ...);
    }
}
```

**Lua 使用示例**:
```lua
function MyPlugin:on_render(canvas)
    local paint = Skia.Paint.new()
    paint:setColor("#FF0000")
    canvas:drawRect(10, 10, 100, 100, paint)
end
```

### 4.2 Event API (SDL2 Event Binding)
`on_event` 接收的事件结构。

```lua
-- Lua 接收到的 event 表结构示例
{
    type = "keydown", -- 或 "keyup", "mousedown", "mousemotion" ...
    keycode = "Escape",
    scancode = "Esc",
    x = 0, -- 鼠标事件特有
    y = 0
}
```

## 5. 开发示例：简易雨水效果

下面是一个完整的插件示例，展示了如何在立绘层之上绘制下落的雨滴。

```lua
-- plugins/rain_effect.lua

local RainPlugin = {
    id = "builtin.effect.rain",
    layer = "scene_top", -- 渲染在立绘之上，UI 之下
    drops = {} -- 存储雨滴数据
}

local DROP_COUNT = 100
local SCREEN_W = 1920
local SCREEN_H = 1080

function RainPlugin:on_init()
    -- 初始化雨滴
    for i = 1, DROP_COUNT do
        table.insert(self.drops, {
            x = math.random(0, SCREEN_W),
            y = math.random(0, SCREEN_H),
            speed = math.random(10, 20),
            length = math.random(10, 30)
        })
    end
    
    -- 预创建画笔，避免每帧创建（性能优化）
    self.paint = Skia.Paint.new()
    self.paint:setColor("#Aaddddff") -- 半透明淡蓝
    self.paint:setStrokeWidth(2)
    self.paint:setAntiAlias(true)
end

function RainPlugin:on_update(dt)
    -- 更新每一滴雨的位置
    for _, drop in ipairs(self.drops) do
        drop.y = drop.y + drop.speed * (dt * 60) -- 基于时间增量移动
        
        -- 超出屏幕复位
        if drop.y > SCREEN_H then
            drop.y = -drop.length
            drop.x = math.random(0, SCREEN_W)
        end
    end
end

function RainPlugin:on_render(canvas)
    for _, drop in ipairs(self.drops) do
        -- 绘制线条模拟雨滴
        canvas:drawLine(drop.x, drop.y, drop.x, drop.y + drop.length, self.paint)
    end
end

return RainPlugin
```

## 6. 注册与加载

### 6.1 加载流程
1.  引擎启动时，扫描 `plugins/` 目录下的所有 `.lua` 文件。
2.  执行 Lua 文件，获取返回的 Table。
3.  Rust 端调用 `plugin_table.on_init()`。
4.  将插件加入 `PluginManager` 的活跃列表。

### 6.2 手动注册 (Debug console)
在开发过程中，可以在控制台手动加载：
```lua
Engine.load_plugin("plugins/test_effect.lua")
```
