# Router 状态管理示例

本文档演示如何在 Page 中实现状态管理。

## 基本概念

Page trait 支持在 struct 中存储状态变量，当状态改变时自动触发 UI 重建。

### 方式一：使用内置脏标记

最简单的方式是维护一个 `needs_rebuild` 标志：

```rust
use hoshimi_ui::prelude::*;
use std::cell::Cell;

#[derive(Debug)]
struct CounterPage {
    count: i32,
    title: String,
    needs_rebuild: Cell<bool>, // 使用 Cell 允许在 &self 方法中修改
}

impl CounterPage {
    fn new() -> Self {
        Self {
            count: 0,
            title: "Counter Example".to_string(),
            needs_rebuild: Cell::new(false),
        }
    }
    
    pub fn increment(&mut self) {
        self.count += 1;
        self.needs_rebuild.set(true);
    }
    
    pub fn decrement(&mut self) {
        self.count -= 1;
        self.needs_rebuild.set(true);
    }
    
    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.needs_rebuild.set(true);
    }
}

impl Page for CounterPage {
    fn route_name(&self) -> &str {
        "counter"
    }
    
    fn build(&self) -> Box<dyn Widget> {
        Box::new(
            Center::new(
                Column::new()
                    .child(Text::new(&self.title))
                    .child(Text::new(&format!("Count: {}", self.count)))
                    .with_spacing(20.0)
            )
        )
    }
    
    fn needs_rebuild(&self) -> bool {
        self.needs_rebuild.get()
    }
    
    fn mark_rebuilt(&mut self) {
        self.needs_rebuild.set(false);
    }
    
    impl_page_common!();
}
```

### 方式二：RefCell 内部可变性

对于更复杂的状态，可以使用 `RefCell`：

```rust
use std::cell::RefCell;

#[derive(Debug)]
struct TodoPage {
    state: RefCell<TodoState>,
}

#[derive(Debug)]
struct TodoState {
    todos: Vec<String>,
    input: String,
    needs_rebuild: bool,
}

impl TodoPage {
    fn new() -> Self {
        Self {
            state: RefCell::new(TodoState {
                todos: Vec::new(),
                input: String::new(),
                needs_rebuild: false,
            }),
        }
    }
    
    pub fn add_todo(&self, text: String) {
        let mut state = self.state.borrow_mut();
        state.todos.push(text);
        state.needs_rebuild = true;
    }
    
    pub fn remove_todo(&self, index: usize) {
        let mut state = self.state.borrow_mut();
        if index < state.todos.len() {
            state.todos.remove(index);
            state.needs_rebuild = true;
        }
    }
    
    pub fn set_input(&self, input: String) {
        let mut state = self.state.borrow_mut();
        state.input = input;
        state.needs_rebuild = true;
    }
}

impl Page for TodoPage {
    fn route_name(&self) -> &str {
        "todo"
    }
    
    fn build(&self) -> Box<dyn Widget> {
        let state = self.state.borrow();
        
        let mut column = Column::new()
            .child(Text::new("Todo List"));
        
        for (i, todo) in state.todos.iter().enumerate() {
            column = column.child(
                Row::new()
                    .child(Text::new(todo))
                    .child(Text::new(&format!(" [{}]", i)))
            );
        }
        
        Box::new(Center::new(column))
    }
    
    fn needs_rebuild(&self) -> bool {
        self.state.borrow().needs_rebuild
    }
    
    fn mark_rebuilt(&mut self) {
        self.state.borrow_mut().needs_rebuild = false;
    }
    
    impl_page_common!();
}
```

### 方式三：消息驱动状态更新

对于更复杂的应用，可以使用消息模式：

```rust
#[derive(Debug)]
enum GameMessage {
    StartGame,
    PauseGame,
    UpdateScore(i32),
    PlayerDied,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Debug)]
struct GamePage {
    state: GameState,
    score: i32,
    lives: i32,
    needs_rebuild: bool,
}

impl GamePage {
    fn new() -> Self {
        Self {
            state: GameState::Menu,
            score: 0,
            lives: 3,
            needs_rebuild: false,
        }
    }
    
    pub fn handle_message(&mut self, msg: GameMessage) {
        match msg {
            GameMessage::StartGame => {
                self.state = GameState::Playing;
                self.score = 0;
                self.lives = 3;
            }
            GameMessage::PauseGame => {
                if self.state == GameState::Playing {
                    self.state = GameState::Paused;
                }
            }
            GameMessage::UpdateScore(points) => {
                self.score += points;
            }
            GameMessage::PlayerDied => {
                self.lives -= 1;
                if self.lives <= 0 {
                    self.state = GameState::GameOver;
                }
            }
        }
        self.needs_rebuild = true;
    }
}

impl Page for GamePage {
    fn route_name(&self) -> &str {
        "game"
    }
    
    fn build(&self) -> Box<dyn Widget> {
        let content = match self.state {
            GameState::Menu => {
                Column::new()
                    .child(Text::new("Game Menu"))
                    .child(Text::new("Press Start"))
            }
            GameState::Playing => {
                Column::new()
                    .child(Text::new(&format!("Score: {}", self.score)))
                    .child(Text::new(&format!("Lives: {}", self.lives)))
            }
            GameState::Paused => {
                Column::new()
                    .child(Text::new("PAUSED"))
                    .child(Text::new(&format!("Score: {}", self.score)))
            }
            GameState::GameOver => {
                Column::new()
                    .child(Text::new("GAME OVER"))
                    .child(Text::new(&format!("Final Score: {}", self.score)))
            }
        };
        
        Box::new(Center::new(content))
    }
    
    fn needs_rebuild(&self) -> bool {
        self.needs_rebuild
    }
    
    fn mark_rebuilt(&mut self) {
        self.needs_rebuild = false;
    }
    
    impl_page_common!();
}
```

## 在应用中使用

### 访问和修改页面状态

```rust
// 创建 router
let mut router = Router::new();

// 推入页面
let counter = CounterPage::new();
router.push(counter);

// 获取当前页面并修改状态
if let Some(page) = router.current_page_mut() {
    // 需要向下转型到具体的页面类型
    if let Some(counter) = page.as_any_mut().downcast_mut::<CounterPage>() {
        counter.increment();
        // Router 会在下一帧 tick() 时自动检测并重建 UI
    }
}

// 主循环
loop {
    let delta = get_delta_time();
    
    // tick 会自动检查并重建需要更新的页面
    router.tick(delta);
    router.paint(&mut painter);
}
```

### 通过事件处理修改状态

```rust
// 在事件处理中修改状态
match event {
    InputEvent::MouseButtonDown { button, position } => {
        if let Some(page) = router.current_page_mut() {
            if let Some(game) = page.as_any_mut().downcast_mut::<GamePage>() {
                game.handle_message(GameMessage::StartGame);
            }
        }
    }
    InputEvent::KeyDown { key_code } => {
        if key_code == KeyCode::Space {
            if let Some(page) = router.current_page_mut() {
                if let Some(game) = page.as_any_mut().downcast_mut::<GamePage>() {
                    game.handle_message(GameMessage::PauseGame);
                }
            }
        }
    }
    _ => {}
}

// 或者可以手动请求重建
router.rebuild_current_page();
```

## 最佳实践

### 1. 最小化重建

只在真正需要时设置 `needs_rebuild`，避免不必要的 UI 重建：

```rust
pub fn update_score(&mut self, new_score: i32) {
    if self.score != new_score {
        self.score = new_score;
        self.needs_rebuild = true; // 只在值变化时才标记
    }
}
```

### 2. 使用 Cell/RefCell 实现内部可变性

对于需要在 `&self` 方法中修改的状态，使用 `Cell` 或 `RefCell`：

```rust
#[derive(Debug)]
struct MyPage {
    // 简单类型用 Cell
    counter: Cell<i32>,
    // 复杂类型用 RefCell
    data: RefCell<Vec<String>>,
    // 标志用 Cell
    needs_rebuild: Cell<bool>,
}
```

### 3. 分离状态和逻辑

将状态结构和页面逻辑分离：

```rust
#[derive(Debug, Clone)]
struct AppState {
    user_name: String,
    is_logged_in: bool,
    notifications: Vec<String>,
}

#[derive(Debug)]
struct HomePage {
    state: RefCell<AppState>,
    needs_rebuild: Cell<bool>,
}

impl HomePage {
    pub fn update_state<F>(&self, f: F)
    where
        F: FnOnce(&mut AppState),
    {
        let mut state = self.state.borrow_mut();
        f(&mut *state);
        drop(state);
        self.needs_rebuild.set(true);
    }
}
```

### 4. 避免在 build() 中修改状态

`build()` 方法应该是纯函数，只读取状态而不修改：

```rust
// ❌ 错误
fn build(&self) -> Box<dyn Widget> {
    self.counter += 1; // 不要在 build 中修改状态
    Box::new(Text::new(&format!("{}", self.counter)))
}

// ✅ 正确
fn build(&self) -> Box<dyn Widget> {
    // 只读取状态
    Box::new(Text::new(&format!("{}", self.counter)))
}
```

## 性能考虑

- **脏标记机制**：只有标记为需要重建的页面才会重建 UI
- **Widget 复用**：Router 内部使用 diff 算法，相同部分的 Widget 会被复用
- **惰性重建**：重建只在 `tick()` 时发生，不是立即重建
- **过渡期间**：页面过渡时不会检查重建，避免过渡动画被打断

## 总结

Hoshimi UI 的 Router 提供了灵活的页面级状态管理：

1. ✅ **在 struct 中存储状态变量**
2. ✅ **提供方法修改状态**
3. ✅ **通过脏标记触发 UI 重建**
4. ✅ **自动检测并重建（在 tick 中）**
5. ✅ **支持内部可变性（Cell/RefCell）**
6. ✅ **支持复杂的状态管理模式**

这种设计兼顾了灵活性和性能，适合各种规模的应用开发。
