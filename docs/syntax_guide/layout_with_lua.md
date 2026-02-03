## Hoshimi UI声明语言 文档

### 前言

本文档面向 Hoshimi 引擎使用者，详细介绍引擎的 Lua UI 声明语言的使用方法。该语言采用 Flutter 风格的函数式组件设计，支持全组件背景贴图、动态内容插入、游戏变量联动，兼顾语义化可读性与灵活扩展性，适配 GAL 游戏以贴图为主、逻辑与UI分离的开发需求。

核心优势：

- 函数式组件：语法简洁、语义清晰，层级关系直观，告别繁琐的 JSON 式嵌套。

- 全组件贴图支持：所有UI组件均可设置背景图片，适配 GAL 游戏贴图化 UI 设计习惯。

- 游戏变量联动：直接读写引擎全局游戏变量（GameVars），轻松实现UI状态动态切换。

- UI/内容/逻辑分离：通过Slot组件动态注入内容，降低开发耦合度。

- 基础富文本支持：兼容简易 Markdown 语法，满足剧情文本、提示文本的样式需求。

前置说明：

- 引擎为所有的以`.ui.lua`结尾的脚本内置所有组件构造函数，无需额外定义，可直接在Lua脚本中调用。

- Lua 脚本最终返回的 UI 结构会被引擎解析为渲染结构体，使用者无需关注底层渲染细节，只需按规范编写脚本。

- 全局游戏变量 GameVars 由引擎自动注入，可直接读写，用于关联 UI 状态与游戏逻辑。

### 第一章 基础概念

#### 1.1 核心设计原则

##### 1.1.1 函数式声明

采用`组件函数(属性表, 子组件列表)`的Flutter风格语法，通过函数名直接标识组件类型，子组件列表作为参数传入，层级关系一目了然。

示例：

```lua
vbox(
    { 
        x = 400, 
        y = 200, 
        spacing = 30 
    }, -- 属性表
    { 
        text({ 
			text="标题", 
            text_size=36 
        }) 
    } -- 子组件列表
)

-- 或者，更加简单的写法
vbox { 
    x = 400, 
    y = 200, 
    spacing = 30 
}, -- 属性表
{ 
    text { 
        text="标题", 
        text_size=36 
    } 
} -- 子组件列表
```

##### 1.1.2 UI/内容/逻辑分离

- UI层：通过组件函数定义布局、样式、贴图，负责显示什么。

- 内容层：通过 Slot 组件预留占位，引擎动态注入富文本、图片等内容，负责显示哪些具体内容。

- 逻辑层：通过绑定 Lua 函数处理交互事件、读写 GameVars，负责点击/状态变化后做什么。

#### 1.2 全局游戏变量 GameVars

GameVars 是引擎注入的全局Lua表，用于存储用户自定义的游戏变量，UI 可直接通过 Lua 表达式读取或修改，实现状态联动。同时，在存档或者读档时，GameVars 中的所有的内容都会被写进存档中或者从存档中读出。

> TODO：如果你需要使用例如剧情进度等变量，你需要去读取 EngineVars 中的值，但是这部分我还没有想好怎么规划。。

##### 1.2.1 基础用法

```lua

-- 读取变量：根据剧情进度显示不同文本
text({
    text = GameVars.btn_count >= 5 and "解锁隐藏剧情" or "继续推进剧情"
})

-- 修改变量：点击按钮更新选中状态
button({
    events = {
        click = function()
            GameVars.btn_count = GameVars.btn_count + 1 -- 修改全局选中状态
        end
    }
})
```

#### 1.3 组件通用规则

##### 1.3.1 组件结构规范

所有组件均遵循`构造函数+属性表+子组件列表`的结构，其中：

- 属性表（必填）：存储组件的位置、尺寸、样式、贴图等配置，是一个Lua表。

- 子组件列表（可选）：仅布局组件（vbox/hbox）、按钮组件支持，传入子组件的数组，默认空表。

示例：

```lua

-- 布局组件（支持子组件）
vbox(属性表, 子组件列表)

-- 普通组件（无子女组件）
text(属性表)
image(属性表)
```

##### 1.3.2 全组件通用属性

所有组件的属性表中，均支持以下基础属性（可选，不设置则使用引擎默认值）：

|属性名|类型|说明|默认值|
|---|---|---|---|
|id|string|组件唯一标识，用于关联逻辑、更新组件状态|自动生成随机标识|
|x|number|组件左上角X坐标（相对于父组件左上角）|0|
|y|number|组件左上角Y坐标（相对于父组件左上角）|0|
|width|number|组件宽度|自适应内容/父组件宽度|
|height|number|组件高度|自适应内容/父组件高度|
|bg_color|table|背景色，格式RGBA（每个值0-1），如{1,1,1,1}为纯白不透明|透明（{0,0,0,0}）|
##### 1.3.3 全组件背景图片属性

所有组件均支持背景图片配置，通过以下4个属性实现（可选，不设置则不显示背景图）：

|属性名|类型|说明|默认值|
|---|---|---|---|
|bg_image|string|背景图片资源路径，需遵循引擎资源约定|nil（不显示背景图）|
|bg_image_alpha|number|背景图片透明度（0-1），0完全透明，1完全不透明|1.0|
|bg_image_mode|string|图片显示模式：stretch（拉伸填充）、cover（完全覆盖，可能裁剪）、contain（容纳，可能留白）、origin（原分辨率呈现）|origin|
|bg_image_offset|table|背景图片偏移量，格式{x,y}，相对于组件左上角|{0,0}|

提示：背景图片与背景色可共存，图片透明区域会显示背景色。

### 第二章 组件详解

引擎内置7类核心组件，涵盖布局、交互、内容、占位等游戏UI需求，以下按使用频率排序详细介绍。

#### 2.1 布局组件

用于管理子组件的排版，支持嵌套使用，是构建UI层级的基础，核心有2种：垂直布局（vbox）、水平布局（hbox）。

##### 2.1.1 垂直布局组件 vbox

功能：子组件从上到下垂直排列，支持设置间距、对齐方式、内边距等。

###### 构造函数

```lua
vbox(props, children)
```

###### 专属属性（补充通用属性）

|属性名|类型|说明|默认值|
|---|---|---|---|
|spacing|number|子组件之间的垂直间距（像素）|20|
|align|string|子组件水平对齐方式：left（左对齐）、center（居中）、right（右对齐）|center|
|padding|number|布局内边距（上下左右均生效），子组件与布局边框的距离|0|
###### 使用示例（带背景贴图）

```lua
-- 主菜单垂直布局，带框架贴图，子组件垂直排列
vbox(
    {
        id = "main_menu_vbox",
        x = 540,
        y = 250,
        width = 200,
        height = 300,
        spacing = 40,
        align = "center",
        padding = 20,
        bg_color = {0,0,0,0.5}, -- 半透明黑色背景
        -- 背景贴图：菜单框架
        bg_image = "assets/layout/menu_frame.png",
        bg_image_alpha = 0.9,
        bg_image_mode = "stretch"
    },
    {
        text({text="星空下的约定", text_size=36}), -- 子组件1：标题
        button({text="开始游戏"}), -- 子组件2：按钮
        button({text="继续游戏"}) -- 子组件3：按钮
    }
)
```

##### 2.1.2 水平布局组件 hbox

功能：子组件从左到右水平排列，属性与vbox基本一致，仅排版方向不同。

###### 构造函数

```lua
hbox(props, children)
```

###### 专属属性（补充通用属性）

|属性名|类型|说明|默认值|
|---|---|---|---|
|spacing|number|子组件之间的水平间距（像素）|20|
|align|string|子组件垂直对齐方式：top（上对齐）、center（居中）、bottom（下对齐）|center|
|padding|number|布局内边距（上下左右均生效）|0|
###### 使用示例（底部工具栏）

```lua
-- 底部水平布局，放置存档、设置、退出按钮
hbox(
    {
        x = 0,
        y = 650,
        width = 1280,
        height = 70,
        spacing = 80,
        align = "center",
        bg_image = "assets/layout/bottom_bar.png",
        bg_image_mode = "stretch"
    },
    {
        button({text="存档", width=100, height=50}),
        button({text="设置", width=100, height=50}),
        button({text="退出", width=100, height=50})
    }
)
```

#### 2.2 交互组件（核心）

##### 2.2.1 按钮组件 button

功能：支持点击交互，可设置常态/选中态/禁用态贴图切换，绑定点击事件，是GAL游戏菜单、选项的核心组件。

###### 构造函数

```lua
button(props, children)
```

###### 专属属性（补充通用属性+背景图片属性）

|属性名|类型|说明|默认值|
|---|---|---|---|
|text|string|按钮上的文本内容，不设置则为空|""|
|text_size|number|按钮文本字号|24|
|text_color|table|文本颜色，RGBA格式|{1,1,1,1}（纯白）|
|border_width|number|按钮边框宽度，0则不显示边框|0|
|border_color|table|按钮边框颜色，RGBA格式|{1,1,1,1}|
|disabled|boolean|是否禁用按钮（禁用后无法点击，文本变灰）|false|
|events|table|交互事件绑定，目前仅支持click（点击事件）|nil|
###### 事件绑定详解

通过 events.click 绑定Lua函数，点击按钮时触发，函数可读写GameVars、调用引擎指令（如切换UI、加载存档）。

> TODO：引擎指令会通过直接写入全局表的方式来进行提供调用，但是这块我还没有设计。

###### 使用示例（状态切换+事件绑定）

```lua
-- 开始游戏按钮，支持选中/常态/禁用态贴图切换
button({
    id = "btn_start",
    width = 200,
    height = 60,
    text = "开始游戏",
    text_size = 24,
    text_color = {1,1,1,1},
    -- 背景贴图：根据GameVars切换状态
    bg_image = GameVars.story_progress <= 0 
        and "assets/btn/disabled.png" -- 禁用态（无剧情进度）
        or (GameVars.menu_selected == 1 
            and "assets/btn/selected.png" -- 选中态
            or "assets/btn/normal.png"), -- 常态
    disabled = GameVars.story_progress <= 0, -- 无进度时禁用
    events = {
        click = function()
            GameVars.menu_selected = 1 -- 更新选中状态
        end
    }
})
```

#### 2.3 内容组件

##### 2.3.1 文本组件 text

功能：显示静态文本，支持简易Markdown样式，可设置背景贴图。这套组件同样会被使用在引擎的剧情页面中，意味着在剧本文件里面，您同样可以使用这套样式代码。

> TODO：介绍一下剧情文件

###### 构造函数

```lua
text(props)
```

###### 专属属性（补充通用属性）

|属性名|类型|说明|默认值|
|---|---|---|---|
|text|string|文本内容，必填|""|
|text_size|number|文本字号|24|
|base_color|table|文本颜色，RGBA格式|{1,1,1,1}|
|line_space|number|多行文本的行距选项|1.0|
|shadow_color|table|文本阴影颜色，RGBA格式，不设置则无阴影|nil|
|shadow_offset|table|文本阴影偏移量，格式{x,y}|{2,2}|
###### 支持的Markdown样式

| 语法             | 效果                                                        |
| ---------------- | ----------------------------------------------------------- |
| \**文本**        | 粗体                                                        |
| \*文本*          | 斜体                                                        |
| [[#颜色值 文本]] | 文本染色（颜色值为十六进制）                                |
| 文本<br />文本   | 换行（直接换行即可）                                        |
| \[文本](标识)    | 剧情跳转点击链接（标识指的是在剧本文件中的`#标识`跳转锚点） |

###### 使用示例（标题文本）

```lua
text({
    id = "story_text",
    x = 100,
    y = 500,
    width = 1080,
    height = 180,
    text = "**清晨的阳光透过窗户**，*洒在桌面上*。[[#00ff00 你睁开眼睛，看到桌上放着一封邀请函]]。\n[查看邀请函](check_letter)",
    text_size = 20,
    line_spacing = 1.5,
    base_color = {1,1,1,1},
    -- 背景贴图：剧情文本框
    bg_image = "assets/text_box/story_bg.png",
    bg_image_mode = "stretch",
    bg_image_alpha = 0.8
})
```

##### 2.3.3 图片组件 image

功能：显示图片（如背景图、立绘、图标），支持透明度调节，是GAL游戏贴图化UI的基础组件。

###### 构造函数

```lua
image(props)
```

###### 专属属性（补充通用属性）

|属性名|类型|说明|默认值|
|---|---|---|---|
|src|string|图片资源路径，必填（遵循资源约定）|nil|
|alpha|number|图片整体透明度（0-1）|1.0|
|mode|string|图片显示模式：stretch、cover、contain、origin|origin|
###### 使用示例（背景图+立绘）

```lua
-- 1. 全屏背景图
image({
    src = "assets/backgrounds/room_bg.jpg",
    x = 0,
    y = 0,
    width = 1280,
    height = 720,
    alpha = 0.9
})

-- 2. 角色立绘（居左显示，不拉伸）
image({
    src = "assets/characters/girl_01.png",
    x = 50,
    y = 200,
    mode = "center", -- 居中显示，保持原图尺寸
    alpha = 1.0
})
```

#### 2.4 占位组件

##### 2.4.1 Slot组件 slot

功能：预留内容占位符，引擎动态注入文本、图片、组件列表等内容，实现UI模板与内容分离，适合动态剧情提示、可变菜单内容等场景。

核心逻辑：Slot仅定义占位位置、尺寸、样式，不定义具体内容；引擎通过slot_id匹配对应的内容，无对应内容时显示默认内容。

###### 构造函数

```lua
slot(props)
```

###### 专属属性（补充通用属性+背景图片属性）

|属性名|类型|说明|默认值|
|---|---|---|---|
|slot_id|string|占位标识，引擎通过该标识匹配内容，必填|nil|
|default_content|组件|无匹配内容时显示的默认组件（通常为text）|nil|
|spacing|number|注入多个子组件时的间距|10|
|align|string|注入内容的对齐方式：left/center/right|center|
###### 使用示例（剧情提示占位）

```lua
-- 剧情提示Slot，动态注入提示内容
slot({
    id = "story_tip_slot",
    slot_id = "main_menu_story_tip", -- 引擎匹配内容的标识
    x = 100,
    y = 650,
    width = 1080,
    height = 60,
    align = "center",
    -- 背景贴图：提示框
    bg_image = "assets/slot/tip_bg.png",
    bg_image_mode = "stretch",
    bg_image_alpha = 0.9,
    -- 默认内容（无动态内容时显示）
    default_content = text({
        text = "**提示**：完成新手教程可解锁[[#00ff00 隐藏角色]]",
        text_size = 16,
        base_color = {0.9,0.9,0.9,1}
    })
})
```

提示：引擎注入内容的逻辑由引擎配置，使用者只需确保slot_id与内容标识一致即可。

### 第三章 组件内状态管理

#### 2.1 功能定位

新增组件内私有状态，用于管理组件自身的临时交互状态（如按钮悬浮态、点击计数、弹窗显隐等），与全局状态（GameVars）分工明确，避免全局变量污染，同时提升UI渲染性能。

两种状态的核心区别：

| 状态类型       | 作用域       | 适用场景                                        | 持久化（存档/读档）               |
| -------------- | ------------ | ----------------------------------------------- | --------------------------------- |
| 组件内私有状态 | 单个组件实例 | 按钮hover态、点击计数、输入框临时内容、弹窗显隐 | 否（页面销毁/组件卸载后自动清空） |
| 全局GameVars   | 整个游戏     | 剧情进度、玩家选择、存档信息、角色属性          | 是（自动纳入游戏存档体系）        |
#### 2.2 实现步骤

##### 2.2.1 声明组件私有状态

在组件配置中通过`state`字段（Lua表格式）声明私有状态，语法简洁，类似变量声明，示例如下：

```lua
-- 示例：按钮组件声明私有状态
button({
    id = "counter_btn",
    text = "点击计数",
    -- 组件私有状态声明（支持多种数据类型）
    state = {
        count = 1,          -- 数值类型：点击计数
        is_hover = false,   -- 布尔类型：是否悬浮
        tip = "初始状态"     -- 字符串类型：提示文本
    },
    -- 动态属性：根据状态切换按钮背景图
    bg_image = function(self)
        -- 通过self.xxx直接访问私有状态
        return self.is_hover 
            and "assets/btn/hover.png" 
            or "assets/btn/normal.png"
    end
})
```

##### 2.2.2 更新组件私有状态

必须通过`self:setState(new_state)`方法更新状态（禁止直接赋值，如`self.count = 2`），该方法会自动触发组件局部重绘，提升性能，示例如下：

```lua
-- 示例：事件中更新组件状态
button({
    id = "counter_btn",
    state = { count = 1, is_hover = false },
    events = {
        -- 点击事件：更新计数与提示文本
        click = function(self)
            self:setState({
                count = self.count + 1,
                tip = "当前计数：" .. (self.count + 1)
            })
        end,
        -- 悬浮事件：更新悬浮状态
        mouse_enter = function(self)
            self:setState({ is_hover = true })
        end,
        -- 离开事件：重置悬浮状态
        mouse_leave = function(self)
            self:setState({ is_hover = false })
        end
    }
})
```

#### 2.3 全局与局部状态协同

组件内可同时访问私有状态与全局状态，实现两者联动，示例如下：

```lua
-- 示例：混合使用局部状态与全局状态
button({
    id = "story_btn",
    state = { is_selected = false },
    events = {
        click = function(self)
            -- 更新局部状态：切换选中态
            self:setState({ is_selected = not self.is_selected })
            -- 同步全局状态：记录选中的按钮ID
            GameVars.selected_btn = self.id
        end
    },
    -- 动态绑定：结合局部与全局状态判断样式
    bg_image = function(self)
        return (self.is_selected or GameVars.highlight_btn == self.id)
            and "assets/btn/selected.png"
            or "assets/btn/normal.png"
    end
})
```

### 第四章 UI路由系统

#### 3.1 功能定位

通过全局`router`表管理所有UI页面的加载、切换、销毁与缓存，无需手动处理布局文件读取与组件内存管理（引擎自动完成），适配视觉小说游戏多页面切换场景（如主菜单→角色选择→剧情界面）。

#### 3.2 核心概念

- 路由ID：每个UI页面的唯一标识，与UI脚本`meta.id`建议保持一致，用于页面切换与定位；

- 路由映射：通过`router.register`方法注册“路由ID→UI文件路径”的对应关系，引擎自动读取；

- 路由栈：引擎自动维护的页面切换历史，支持返回上一页操作。

#### 3.3 实操步骤

##### 3.3.1 注册UI路由

在游戏入口脚本（如`main.lua`）中，通过`router.register`方法统一注册所有页面，示例如下：

```lua
-- 示例：全局UI路由注册（main.lua）
-- 注册格式：路由ID → UI文件路径
router.register({
    main_menu = "ui/main_menu.ui.lua",          -- 主菜单
    character_select = "ui/character.ui.lua",   -- 角色选择界面
    story_interface = "ui/story.ui.lua",        -- 剧情界面
    settings = "ui/settings.ui.lua"             -- 设置界面
})

-- 初始加载主菜单（游戏启动时执行）
router.push("main_menu")
```

注意：注册时需确保布局文件路径正确，引擎会自动校验文件是否存在，路径错误会触发报错提示。

##### 3.3.2 页面切换操作

通过`router`提供的核心方法实现页面切换，支持基础跳转、带参跳转、返回上一页等操作，常用方法如下：

###### （1）基础跳转：router.push(ui_id)

切换到指定UI页面，使用这种方法进行页面跳转原页面不会被释放，会一直在路由栈内，示例如下：

```lua
-- 示例：主菜单按钮跳转至角色选择界面
button({
    id = "to_char_btn",
    text = "进入角色选择",
    events = {
        click = function()
            -- 跳转至角色选择页面（路由ID：character_select）
            router.push("character_select")
        end
    }
})
```

###### （2）带参跳转：router.push(ui_id, params)

切换页面时传递初始化参数，目标页面可通过`router.get_params()`读取参数，实现跨页面数据传递，示例如下：

```lua
-- 示例1：带参跳转（主菜单→角色选择）
button({
    id = "to_char_btn",
    events = {
        click = function()
            -- 传递参数：玩家名称、当前章节
            router.push("character_select", { 
                player_name = "旅行者", 
                chapter_id = 1 
            })
        end
    }
})

-- 示例2：目标页面（角色选择）读取参数
-- character.ui.lua 中的logic.on_load方法
logic = {
    on_load = function()
        -- 读取路由传递的参数
        local params = router.get_params()
        -- 可将参数同步到全局状态，供组件使用
        GameVars.player_name = params.player_name or "默认名称"
        GameVars.current_chapter = params.chapter_id or 1
    end
}
```

###### （3）返回上一页：router.pop()

返回路由栈中的上一个页面，适用于“返回主菜单”“返回上一级界面”场景，示例如下：

```lua
-- 示例：角色选择界面返回主菜单
button({
    id = "back_btn",
    text = "返回主菜单",
    events = {
        click = function()
            router.pop() -- 自动返回上一页（主菜单）
        end
    }
})
```

###### （4）非缓存跳转：精细释放内存

 若你希望在非缓存跳转时，主动控制页面的内存占用与路由栈记录，可使用以下两个方法，满足不同场景需求： 

1. `router.release(ui_id)`：仅释放内存，保留路由栈记录    

   - 功能：释放目标页面的所有内存（组件实例、私有状态等），但该页面的路由ID仍保留在路由栈中（挂牌名字），不会被删除；    

   - 适用场景：暂时不用该页面，但后续可能需要重新加载（如剧情分支页面、临时弹窗），既节省内存，又保留路由追溯能力；    

   - 注意：后续通过 `router.pop()`/`router.push(ui_id)` 访问该页面时，引擎会重新读取UI文件、重新初始化页面（与首次加载一致，无原有状态）；

   - 示例：      

     ```lua
     -- 跳转后释放页A的内存，保留路由栈记录      
     router.push("page_b")      
     router.release("page_a")
     ```

2. `router.destroy(ui_id)`：释放内存 + 删除路由栈记录

   - 功能：释放目标页面的所有内存，同时将该页面的路由 ID 从路由栈中彻底删除；

   - 适用场景：永久无需返回的页面（如新手引导、角色创建、结局页面），彻底清理资源，避免路由栈冗余；

   - 注意：操作后无法通过 `router.back()`/`router.pop()` 返回该页面，需重新通过 `router.push(ui_id)` 注册并加载；

   - 示例：

     ```lua
     -- 跳转后彻底销毁新手引导页面
     router.push("main_menu")
     router.destroy("guide")
     ```

#### 3.4 路由与状态的协同规则

- 状态隔离：不同UI页面的组件私有状态完全独立，互不干扰；

- 状态销毁：非缓存页面切换时，原页面的所有组件私有状态会自动销毁，缓存页面保留状态；

- 全局状态保留：页面切换时，`GameVars`全局状态不会被销毁，始终保持一致。

### 第五章 完整UI脚本示例

以下是GAL游戏主菜单的完整Lua UI脚本，整合所有组件、状态联动、背景贴图、逻辑绑定，可直接复制到引擎中使用（需替换资源路径）。

```lua
-- GAL游戏主菜单 UI脚本
-- 说明：引擎自动注入GameVars，组件构造函数已内置，无需额外定义

-- 1. 初始化游戏变量（防止变量未定义）
local function init_game_vars()
    if not GameVars.menu_selected then GameVars.menu_selected = 1 end
    if not GameVars.story_progress then GameVars.story_progress = 0 end
    if not GameVars.player_name then GameVars.player_name = "旅行者" end
    if not GameVars.last_save_slot then GameVars.last_save_slot = 1 end
end

-- 2. 定义UI核心结构（最终返回给引擎解析）
local main_menu_ui = {
    -- 元信息（UI标识、尺寸，供引擎管理）
    meta = {
        id = "main_menu", -- UI唯一标识，切换/更新时需用到
        width = 1280,     -- 屏幕宽度（适配引擎默认窗口）
        height = 720,     -- 屏幕高度
        style = "gal_main_menu" -- 可选，全局样式标识
    },
    
    -- 3. UI布局（函数式组件，全组件带贴图）
    content = {
        -- 3.1 全屏背景图
        image({
            src = "assets/backgrounds/menu_bg.jpg",
            x = 0,
            y = 0,
            width = 1280,
            height = 720,
            alpha = 0.9
        }),
        
        -- 3.2 主垂直布局（菜单容器，带框架贴图）
        vbox(
            {
                id = "main_menu_vbox",
                x = 540,
                y = 250,
                width = 200,
                height = 300,
                spacing = 40,
                align = "center",
                padding = 20,
                bg_color = {0,0,0,0.5},
                bg_image = "assets/layout/menu_frame.png",
                bg_image_alpha = 0.9,
                bg_image_mode = "stretch"
            },
            {
                -- 标题文本
                text({
                    text = "星空下的约定",
                    text_size = 36,
                    text_color = {1, 0.9, 0.7, 1},
                    shadow_color = {0,0,0,0.8},
                    shadow_offset = {2,2}
                }),
                
                -- 开始游戏按钮（状态切换）
                button({
                    id = "btn_start",
                    width = 200,
                    height = 60,
                    text = "开始游戏",
                    text_size = 24,
                    text_color = {1,1,1,1},
                    bg_image = GameVars.menu_selected == 1 
                        and "assets/btn/selected.png" 
                        or "assets/btn/normal.png",
                    bg_image_mode = "stretch",
                    events = {
                        click = function()
                            return main_menu_ui.logic.on_menu_click(1)
                        end
                    }
                }),
                
                -- 继续游戏按钮（禁用态切换）
                button({
                    id = "btn_continue",
                    width = 200,
                    height = 60,
                    text = "继续游戏",
                    text_size = 24,
                    text_color = GameVars.story_progress > 0 and {1,1,1,1} or {0.5,0.5,0.5,1},
                    bg_image = GameVars.story_progress <= 0 
                        and "assets/btn/disabled.png"
                        or (GameVars.menu_selected == 2 
                            and "assets/btn/selected.png" 
                            or "assets/btn/normal.png"),
                    bg_image_mode = "stretch",
                    disabled = GameVars.story_progress <= 0,
                    events = {
                        click = function()
                            if GameVars.story_progress > 0 then
                                return main_menu_ui.logic.on_menu_click(2)
                            end
                            return nil
                        end
                    }
                }),
                
                -- 退出游戏按钮
                button({
                    id = "btn_quit",
                    width = 200,
                    height = 60,
                    text = "退出游戏",
                    text_size = 24,
                    text_color = {1,1,1,1},
                    bg_image = GameVars.menu_selected == 3 
                        and "assets/btn/selected.png" 
                        or "assets/btn/normal.png",
                    bg_image_mode = "stretch",
                    events = {
                        click = function()
                            return main_menu_ui.logic.on_menu_click(3)
                        end
                    }
                })
            }
        ),
        
        -- 3.3 剧情提示Slot（动态注入内容）
        slot({
            id = "story_tip_slot",
            slot_id = "main_menu_story_tip",
            x = 100,
            y = 650,
            width = 1080,
            height = 60,
            align = "center",
            bg_image = "assets/slot/tip_bg.png",
            bg_image_mode = "stretch",
            bg_image_alpha = 0.9,
            default_content = text({
                text = "**提示**：完成新手教程可解锁[[#00ff00 隐藏角色]]",
                text_size = 16,
                base_color = {0.9,0.9,0.9,1}
            })
        })
    },
    
    -- 4. 逻辑与事件（纯逻辑，不涉及渲染）
    logic = {
        -- 4.1 UI加载时执行（初始化变量）
        on_load = function()
            init_game_vars()
            print("主菜单加载完成，玩家：" .. GameVars.player_name)
        end,
        
        -- 4.2 菜单按钮点击回调（处理交互逻辑）
        on_menu_click = function(index)
            GameVars.menu_selected = index -- 更新选中状态
            -- 返回引擎指令，触发对应操作
            if index == 1 then
                -- 开始游戏：切换到角色选择UI
                return { cmd = "switch_ui", target_ui_id = "character_select" }
            elseif index == 2 then
                -- 继续游戏：加载最后一次存档
                return { cmd = "load_save", save_slot = GameVars.last_save_slot }
            elseif index == 3 then
                -- 退出游戏：返回引擎主界面
                return { cmd = "quit_game" }
            end
            return nil
        end,
        
        -- 4.3 定时更新（引擎每帧调用，监听变量变化）
        on_update = function(delta_time)
            -- 示例：剧情进度达到5时，更新提示内容
            if GameVars.story_progress >= 5 then
                return { cmd = "update_slot", slot_id = "main_menu_story_tip" }
            end
            return nil
        end
    }
}

-- 5. 暴露逻辑到全局（供组件事件调用）
_G.ui_logic = main_menu_ui.logic

-- 6. 返回UI结构，供引擎解析渲染
return main_menu_ui
```

### 第六章 资源约定与常见问题

#### 4.1 资源路径约定

为避免资源加载失败，所有图片资源需遵循以下路径规范，引擎会自动从assets目录下查找：

- 背景图：backgrounds/xxx.png/jpg

- 角色立绘：characters/xxx.png

- 按钮贴图：btn/xxx.svg

- 布局框架：layout/xxx.webp

- 文本框/提示框：assets/text_box/xxx.png、assets/slot/xxx.png

提示：图片格式支持png、jpg、svg、webp，建议使用png格式（支持透明通道，适配GAL游戏贴图需求）。
