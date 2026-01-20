# Hoshimi UI 开发指南 (UI DSL Syntax Guide)

本文档定义了 Hoshimi 引擎用于构建用户界面的 **.hui (Hoshimi UI)** 文件的语法规范。

## 1. 设计理念 (Design Philosophy)

Hoshimi UI DSL (Domain Specific Language) 旨在提供一种**声明式 (Declarative)**、**结构化**且**易于阅读**的方式来定义游戏界面。

*   **类 Rust/KDL 语法**：采用类似 Rust 结构体初始化的嵌套语法，去除冗余的符号（如过多的逗号），保持整洁。
*   **组件化**：一切皆组件 (Widget)，通过组合基础组件构建复杂界面。
*   **布局与逻辑分离**：`.hui` 负责结构与样式，Lua 脚本负责交互逻辑。

## 2. 基础语法 (Basic Syntax)

### 2.1 文件结构
UI 文件以 `.hui` 为扩展名。每个文件通常定义一个根节点（通常是 `Screen` 或 `Component`）。

```rust
// main_menu.hui

Screen "MainMenu" {
    // 属性定义
    background: "ui/bg_menu"
    
    // 子节点
    VBox {
        align_x: "center"
        align_y: "center"
        spacing: 20.0

        // 嵌套组件
        Image {
            src: "ui/logo"
            width: 400
            height: 150
        }

        Button "BtnStart" {
            text: "开始游戏"
            on_click: "System.startGame"
            width: 200
            height: 60
        }
    }
}
```

### 2.2 语法规则
1.  **节点 (Node)**: 由 `类型名 [ID] { ... }` 组成。ID 是可选的，用于在脚本中查找节点。
2.  **属性 (Property)**: `key: value` 格式。
    *   字符串使用双引号 `"..."`。
    *   数字直接书写（支持整数和浮点数）。
    *   布尔值: `true`, `false`.
    *   颜色: Hex 格式 `"#RRGGBB"` 或 `"#RRGGBBAA"`.
3.  **注释**: 支持单行注释 `//` 和块注释 `/* ... */`。

## 3. 核心布局系统 (Layout System)

UI 引擎基于简化版的 **Flexbox** 模型。

### 3.1 容器组件

*   **VBox (Vertical Box)**: 子元素垂直排列。
*   **HBox (Horizontal Box)**: 子元素水平排列。
*   **ZStack**: 子元素层叠排列（后者覆盖前者），常用于背景图与内容叠加。

### 3.2 布局属性

| 属性名 | 类型 | 描述 |
| :--- | :--- | :--- |
| `width` / `height` | Number / String | 尺寸。支持像素数字 (100) 或百分比字符串 ("50%") 或自适应 ("auto")。 |
| `padding` | Number / List | 内边距。`10` 或 `[10, 20]` (上下, 左右) 或 `[10, 20, 10, 20]` (上, 右, 下, 左)。 |
| `margin` | Number / List | 外边距。格式同 padding。 |
| `spacing` | Number | (仅 Box) 子元素之间的间距。 |
| `align_x` | String | 水平对齐: `"left"`, `"center"`, `"right"`, `"stretch"`. |
| `align_y` | String | 垂直对齐: `"top"`, `"center"`, `"bottom"`, `"stretch"`. |

```rust
HBox {
    height: 100
    width: "100%"
    align_y: "center" // 垂直居中
    spacing: 15

    Image { width: 50; height: 50; src: "icon" }
    Text { text: "Item Name"; color: "#FFFFFF" }
}
```

## 4. 基础组件库 (Standard Widgets)

### 4.1 视觉组件
*   **Text**: 文本显示。
    *   `text`: 内容字符串。
    *   `font_size`: 字号。
    *   `color`: 字体颜色。
    *   `font_family`: 字体名称（可选）。
*   **Image**: 图片显示。
    *   `src`: 资源路径（相对于 `assets/`）。
    *   `scale_mode`: `"fit"`, `"fill"`, `"stretch"`.

### 4.2 交互组件
*   **Button**: 按钮。可以是纯文本按钮，也可以包含子元素（作为容器）。
    *   `on_click`: 点击触发的 Lua 函数名或事件名。
    *   `on_hover`: 悬停触发的 Lua 函数。
*   **Input**: 输入框 (Plan)。
*   **Slider**: 滑动条 (Plan)。

## 5. 样式与复用 (Styling & Reusability)

为了避免属性重复，DSL 支持定义**样式表**或**预设**。

### 5.1 Style 语法
可以在 `Screen` 内部或单独的 `.style` 文件中定义样式块。

```rust
Style "btn_primary" {
    width: 200
    height: 60
    background: "#3366FF"
    border_radius: 8
}

// 使用样式
Button {
    style: "btn_primary"
    text: "Confirm"
}
```

## 6. 脚本绑定 (Scripting Integration)

### 6.1 事件绑定
属性如果是 `on_` 开头，则被视为事件绑定。值通常是一个 Lua 全局函数路径。

```rust
Button {
    // 调用 Lua 中定义的 UI_Handlers.onBack()
    on_click: "UI_Handlers.onBack" 
}
```

### 6.2 数据绑定 (Data Binding)
支持简单的单向数据绑定，使用 `${var}` 语法绑定到全局 Lua 变量或 UI Context 变量。

```rust
Text {
    // 自动监听 Lua 变量 Player.gold 的变化
    text: "Gold: ${Player.gold}"
    color: "#FFD700"
}
```

## 7. 逻辑控制与动态渲染 (Control Flow & Dynamic Rendering)

为了缩小与 Ren'Py 等脚本化 UI 系统的差距，DSL 引入了专门的控制流节点，允许 UI 根据 Lua 状态进行真正的动态结构变化（Adding/Removing Nodes），而不仅仅是隐藏。

### 7.1 条件渲染 (Conditional Rendering)
使用 `If` 节点包裹需要动态显示的组件。

*   `condition`: 必需。一个返回布尔值的 Lua 表达式字符串。

```rust
VBox {
    // 只有当条件为 true 时，内部组件才会被挂载到渲染树
    If {
        condition: "${Player.magic_power} == 1"
        
        Button {
            text: "Cast Spell"
            on_click: "Actions.castSpell"
        }
    } Else If {
        condition: "${Player.magic_power} == 2"

        Text {
            text: "You need magic power to cast spells."
            color: "#888888"
        }
    } Else {
        // Pass
    }
}
```

### 7.2 循环渲染 (Loop Rendering)
使用 `For` 节点遍历 Lua 数组或列表，动态生成子组件。

*   `each`: 必需。定义在循环体内使用的局部变量名。
*   `in`: 必需。一个 Lua 数组或可迭代对象。

```rust
// 假设 Lua 数据:
// save_slots = [
//   { id: 1, date: "2023-01-01", info: "Chapter 1" },
//   { id: 2, date: "2023-01-02", info: "Chapter 2" }
// ]

VBox {
    spacing: 10

    For {
        each: "slot"
        in: "${Global.save_slots}"

        // 循环体模板：会对数组中每个元素实例化一次
        Button {
            width: "100%"
            height: 80
            // 支持访问迭代变量的字段
            text: "Save ${slot.id}: ${slot.info} (${slot.date})"
            
            // 将数据传递给回调
            on_click: "System.loadGame(slot.id)"
        }
    }
}
```

### 7.3 局部作用域与性能
*   **作用域 (Scope)**: `For` 循环创建的 `each` 变量仅在 `For` 节点的子层级中有效，不会污染全局环境。
*   **Diff 更新**: 引擎会监听 `condition` 和 `in` 绑定的源数据。当数据变化时，会尝试进行最小化的 Diff 更新（例如只新增一个列表项），而不是销毁重建整个列表。

## 8. 示例：设置菜单 (Example)

```rust
Screen "SettingsMenu" {
    background: "#000000AA" // 半透明遮罩
    
    ZStack {
        // 居中的设置面板
        VBox {
            width: 600
            height: 400
            background: "ui/panel_bg"
            align_x: "center"
            align_y: "center"
            padding: 40
            spacing: 20

            Text { 
                text: "Settings"
                font_size: 32
                align_x: "center"
            }

            // 音量控制行
            HBox {
                width: "100%"
                align_y: "center"
                Text { text: "BGM Volume"; width: 150 }
                // Slider 组件
                Slider "SliderBGM" {
                    width: "stretch"
                    value: "${Settings.bgm_volume}"
                    on_change: "Settings.setBGM"
                }
            }

            // 底部按钮栏
            HBox {
                align_x: "right"
                spacing: 20
                margin: [40, 0, 0, 0] // Top margin

                Button {
                    text: "Cancel"
                    on_click: "UI.closeCurrent"
                    width: 120
                }
                
                Button {
                    text: "Apply"
                    style: "btn_primary"
                    on_click: "Settings.save"
                    width: 120
                }
            }
        }
    }
}
```
