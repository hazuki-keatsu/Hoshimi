<img src="./logos/logo.png" alt="Logo" align="left" style="zoom:7%;" />

## Hoshimi

A **high-performance**, **content-driven**, and **easy-to-use** visual novel engine.

## 📂 Project Structure

This project adopts a **Layered Architecture** similar to the Flutter Engine, separating the low-level shell from the high-level framework logic.

```
Hoshimi/
├── examples/               # Example projects for testing the engine
│   └── hello_world/        # A standard boilerplate project
│       ├── assets/         # Game content (Images, Scripts, UI)
│       └── config.toml     # Project configuration
├── docs/                   # Documentation
├── plugins/                # Core Lua plugins
├── src/                    # Source Code (The Engine)
│   ├── shell/              # Platform Embedder (SDL2, Window, Input Loop)
│   ├── foundation/         # Base utilites (Math, Logger, Filesystem)
│   ├── painting/           # Graphics Abstraction (Skia wrappers, TextLayout)
│   ├── rendering/          # Render Object Tree (Layout calculations)
│   ├── widgets/            # Widget Layer (DSL Parsers -> Element Tree)
│   ├── scripting/          # Lua VM binding & State Management
│   └── main.rs             # Entry point
├── tools/                  # Build scripts
└── Cargo.toml
```

### Layer Details

1.  **Shell**: The interface with the operating system. Handles window creation, OpenGL context, and raw event polling using SDL2.
2.  **Foundation**: Low-level shared utilities used by all other layers.
3.  **Painting**: Wraps the skira (Skia) library to provide a clean 2D drawing API.
4.  **Rendering**: The layout engine. Implements the Flexbox algorithm and manages the RenderTree (dirty checking, painting order).
5.  **Widgets**: The high-level UI system. Contains the parsers for `.hui` files and the component logic.
6.  **Scripting**: The "Brain". Bridges the Lua VM with the Rust application state.
