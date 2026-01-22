# Hoshimi 剧本开发指南 (Hoshimi Scripting Guide)

Version: 0.1.0

## 1. 文件格式
Hoshimi 引擎使用 **`.hmd`** (Hoshimi Markdown) 作为剧本文件的扩展名。为了提供更好的写作体验，我们设计了一套**定制化语法**，仅部分兼容 Markdown。

- **扩展名**: `.hmd`
- **编码**: UTF-8 (无 BOM)
- **建议编辑器**: VS Code (配合插件)或者任何你想要的编辑器

## 2. 资源组织规范 (Asset Organization)

引擎采用 **约定优于配置 (Convention over Configuration)** 的资源目录结构。

### 2.1 目录结构
所有资源存放于 `assets/` 目录下：

```text
assets/
  ├── scripts/             # .hmd 剧本文件
  ├── backgrounds/         # !{ bg: "..." } 查找路径
  │   └── classroom_day.png
  ├── characters/          # !{ char: "..." } 查找路径
  │   ├── hoshimi/         # 角色名文件夹
  │   │   ├── default.png  # 如果不指定 face，默认加载这个
  │   │   ├── smile.png    # face: "smile"
  │   │   └── angry.png    # face: "angry"
  │   └── mystery_girl/
  └── audio/
      ├── bgm/
      ├── sfx/
      └── voices/
```

- **立绘加载逻辑**: 当脚本调用 `!{ char: "hoshimi", face: "smile" }` 时，引擎会自动寻找 `assets/characters/hoshimi/smile.png` (支持 webp/png/jpg，但打包时会全部转换为 webp)。

### 2.2 角色定义 (Character Definition)
为了将对话中的**显示名称**（如 `[星见]`）与**资源 ID**（如 `hoshimi`）关联起来，需要在 `assets/characters.toml` 中定义角色映射表：

```toml
# assets/characters.toml

[hoshimi]
display_name = "星见"
default_face = "default"

[mystery_girl]
display_name = "???"
default_face = "shadow"
```

*   **自动关联**: 当剧本中出现 `[星见]` 时，引擎会自动查找 `display_name` 为 "星见" 的角色，并在需要时自动高亮或操作其立绘。
*   **无需每次指定**: 如果角色已定义，对话时无需手动调用 `!{ char: ... }`，引擎会根据当前说话者自动显示对应立绘。

## 3. 基础语法 (Basic Syntax)

### 3.1 语句块与换行 (Blocks & Line Breaks)

剧本由一个个**语句块 (Block)** 组成。
*   **分隔符**: 使用**空行**（双换行）来分割不同的语句块。
*   **换行规则**: 语句块内部的**单个换行**会被引擎保留并渲染，无需使用 `<br>`。

### 3.2 对话与旁白 (Dialogue & Narration)

通过行首是否包含 `[角色名]` 来区分对话与旁白。

#### 对话 (Dialogue)
以 `[角色名]` 开头，后跟对话内容。角色名与内容之间可以有空格，也可以换行。

```text
[星见]
你好，指挥官！
今天也是充满元气的一天呢！
```

#### 旁白 (Narration)
没有 `[角色名]` 标记的文本块即为旁白。

```text
阳光透过窗户洒在课桌上。
远处传来了上课铃声。
```

### 3.3 场景指令 (Annotations)

使用 `!{ key: value, ... }` 格式。
*   **位置灵活**: 指令可以出现在语句块的**任何位置**。
*   **触发时机**: 引擎渲染文本流，遇到指令位置时立即触发效果。

```text
!{ bg: "classroom", bgm: "daily" }

[星见]
指挥官？!{ face: "surprise" } 你怎么在这里？
其实... !{ face: "shy" } 我一直在等你。
```

### 3.4 流程控制 (Flow Control)

#### 3.4.1 文件跳转 (Script Transition)
使用 `(文件路径)` 语法进行剧本切换。该语句必须**独占一行**（或者是单独的一个语句块）。
可以在同一行通过指令指定转场效果。

```text
// 跳转到第二章，使用默认转场
(chapter_2.hmd)

// 跳转并指定转场特效
(chapter_2.hmd) !{ transition: "fade", time: 2.0 }
```

#### 3.4.2 场景锚点与跳转 (Anchors & Labels)
*   **定义锚点**: 使用 `# 场景名` (标准 Markdown 标题语法)。
*   **跳转锚点**: 使用 `(#场景名)`。

```text
# scene_start

[星见]
我们要去哪里？

(#scene_forest)

# scene_forest
!{ bg: "forest" }
```

#### 3.4.3 分支选项 (Choices)
使用 Markdown 无序列表语法。
*   **格式**: `- [选项文本](跳转目标)`
*   **附加指令**: 在选项下方**缩进**书写 `!{...}`，用于定义该选项触发时的逻辑或转场。

跳转目标可以是：
1. **文件路径**: `path/to/script.hmd`
2. **场景锚点**: `#scene_name`

```text
面临选择，你决定：

- [使用传送魔法](#scene_magic)
  !{ transition: "swirl", sfx: "teleport", cost: 10 }
- [徒步走过去](scene_walk.hmd)
  !{ transition: "fade", time: 2.0 }
```

## 4. 逻辑扩展 (Logic Extensions)

逻辑拓展借鉴 VN 领域通用的 `<< >>` 符号。

### 4.1 逻辑钩子 (Logic Hooks)
使用 `<< ... >>` 包裹 Lua 代码。

```lua
// 单行逻辑：修改变量
<< GameVars.affinity = GameVars.affinity + 10 >>

// 条件分支块 (Block)
<< if GameVars.has_key then >>
[系统]
你使用钥匙打开了门。
<< else >>
[系统]
门锁着，你打不开。
<< end >>
```

### 4.2 文本内变量 (Inline Variables)
在对话中动态插入变量值，使用 `${var}` 语法。

```text
[店员]
这把剑售价 ${item_price} 金币，你要买吗？
```
