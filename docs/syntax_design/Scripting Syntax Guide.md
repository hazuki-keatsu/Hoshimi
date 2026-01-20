# Hoshimi 剧本开发指南 (Hoshimi Scripting Guide)

Version: 0.1.0

## 1. 文件格式
为了区分普通 Markdown 文档与游戏剧本，Hoshimi 引擎使用 **`.hmd`** (Hoshimi Markdown) 作为剧本文件的扩展名。

- **扩展名**: `.hmd`
- **编码**: UTF-8 (无 BOM)
- **建议编辑器**: VS Code (配合官方插件)

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

## 3. 基础语法 (Basic Syntax)

Hoshimi 脚本完全兼容标准 Markdown 语法。可以使用`//`作为注释。

### 3.1 对话与旁白 (Dialogue)
最基础的文本段落会被自动识别为对话或旁白。为了彻底解决符号冲突并提高可读性，引擎采用**“加粗即角色”**的判定策略。

*   **角色对话**：必须以 `**角色名**:` 或 `**角色名**：` 开头（注意**角色名必须加粗**）。此格式之后的内容即为对话文本。
*   **旁白**：不符合上述格式的段落均视为旁白。

同时，各种 Markdown 原生样式语法，均受支持：
- **黑体** (粗体)
- *斜体*
- ~~删除线~~
- 段内换行语法 `<br>`

**书写规则**：
1. 每一段对话或旁白应使用**空行**分割。
2. 解析器会读取 `**角色名**:` 后的所有文本直到下一个空行，这意味着对话可以跨行书写。

```markdown
这是一个普通的旁白段落，即使包含冒号：像这样，也不会被误判。

**星见**:
你好，指挥官！
今天也是充满元气的一天呢！（因为没有空行，这行字仍属于上一句对话）

**???**: 有些事情，还是不知道为好... (同行书写也是允许的)
```

- **旁白**: 直接书写文本。
- **角色对话**: 使用 `**角色名**: 对话内容` 的格式。

### 3.2 场景指令 (Annotations)
使用 `!{ key: value, ... }` 格式的注解语法来控制演出效果。
*   语法类似于 JavaScript 对象字面量，**键名 (Key) 无需加引号**。
*   指令必须占据单独一行。

#### 背景控制 (Background)
```markdown
// transition 和 duration 默认为 "fade" 和 1.0
!{ bg: "bg_classroom_day" }

// 手动设置
!{ bg: "bg_classroom_day", transition: "cut" }
```

其他的 transition:
1. `fade`：淡入淡出
2. `cross-fade`：交叉淡入淡出
3. `dissolve`：溶解过渡
4. `wipe`：擦除过渡(默认向下擦除)
   1. `wipe-up`：向上擦除
   2. `wipe-down`：向下擦除
   3. `wipe-left`：向左擦除
   4. `wipe-right`：向右擦除
5. `slide`：滑动进入(默认从左向右滑动)
   1. `slide-left`：从右向左滑动滑动进入
   2. `slide-right`：从左向右滑动滑动进入
   3. `slide-up`：从下向上滑动滑动进入
   4. `slide-down`：从上向下滑动滑动进入

目前默认只有这些，更多的进入特效可以通过插件系统实现，或者在仓库提 Issue，对于呼声高的，开发组会实现到引擎中。

#### 立绘管理 (Character)
```markdown
!{ char: "hoshimi", face: "smile", pos: "center" }

// 通过百分比来确定位置
!{ char: "hoshimi", face: "smile", pos: "25%" }

// 同时操作多个属性
!{ char: "hoshimi", face: "angry", effect: "shake" }

// 隐藏立绘
!{ char: "hoshimi", mode: "hide" }
```

#### 音频控制 (Audio)
```markdown
// 播放 BGM (循环)
!{ bgm: "daily_life_01", vol: 0.8 }

// 播放 BGM (循环) + Intro
!{ bgm: "daily_life_01", intro: "daily_life_01_intro", vol: 0.8 }

// 播放音效 (单次)
!{ sfx: "phone_ring" }

// 语音 (通常与对话配合，但也支持单独调用)
!{ voice: "hoshimi_001" }
**星见**:
指挥官？
```

### 3.3 流程控制 (Flow Control)

#### 3.3.1 路由跳转 (Jump)
使用标准的 Markdown 链接语法跳转到通过路由表注册的其他 `.hmd` 脚本。

`[]`中的内容将会变成专场时的加载界面标题

```markdown
[第二章](chapter_02.hmd)
```

#### 3.3.2 复杂交互选项 (Complex Choices)
使用 **无序列表** 定义选项。

> **注意**：为了防止与普通文本列表混淆，引擎仅会将 **包含链接 `[]()`** 的列表项识别为交互选项。纯文本列表将作为普通旁白显示。

可以通过 **缩进的注解块 (Nested Block)** 为选项添加附加逻辑（如播放音效、转场动画、执行 Lua 代码）。

```markdown
面临选择，你决定：

- [使用传送魔法](scene_magic.hmd)
  !{ transition: "swirl", sfx: "teleport", cost: 10 }
  
- [徒步走过去](scene_walk.hmd)
  !{ transition: "fade", time: 2.0 }
```
- **设计理念**: 利用 Markdown 的嵌套列表语法，让附加属性自然地依附于选项。

## 4. 逻辑扩展 (Logic Extensions)

逻辑拓展借鉴 VN 领域通用的 `<< >>` 符号。

### 4.1 逻辑钩子 (Logic Hooks)
使用 `<< ... >>` 包裹 Lua 代码。

```lua
// 单行逻辑：修改变量
<< GameVars.affinity = GameVars.affinity + 10 >>

// 条件分支块 (Block)
<< if GameVars.has_key then >>
**系统**:
你使用钥匙打开了门。
<< else >>
**系统**:
门锁着，你打不开。
<< end >>
```

### 4.2 文本内变量 (Inline Variables)
在对话中动态插入变量值，使用 `${var}` 语法。

```markdown
**店员**:
这把剑售价 ${item_price} 金币，你要买吗？
```
