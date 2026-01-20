# Phase 4: 插件与多媒体 (Plugins & Multimedia)

## 1. 阶段目标
增强引擎的扩展性与视听表现力，支持视频播放和第三方插件接入。

## 2. 核心任务拆解

### 4.1 插件系统架构 (Plugin Architecture)
- [ ] **Plugin Trait 定义**
  - 定义 Rust 侧的标准接口:
    ```rust
    trait Plugin {
        fn name(&self) -> &str;
        fn on_init(&mut self, ctx: &mut Context);
        fn on_render_layer(&mut self, canvas: &Canvas, layer: LayerOrder);
    }
    ```
- [ ] **Lua 插件注册**
  - 允许 Lua 脚本注册 Table 作为一个逻辑插件。
  - 引擎在渲染管线的特定 Hook 点 (Pre-Background, Post-UI 等) 回调插件方法。

### 4.2 视频播放系统 (Video Player)
- [ ] **FFmpeg 集成**
  - 引入 `ffmpeg-next`。
  - 实现视频解码线程，解码出 RGB/YUV 数据。
- [ ] **纹理流 (Texture Streaming)**
  - 实现 `VideoTexture`，每帧将解码数据上传到 GPU。
  - 处理音画同步 (A/V Sync)，基于音频时钟调整视频帧渲染。

### 4.3 音频增强 (Audio)
- [ ] **多轨道混合**
  - 实现 Voice, BGM, SE 独立音量控制。
  - 支持音频淡入淡出 (Fade In/Out) 效果。

## 3. 技术难点与注意
- **FFmpeg 编译**: 跨平台编译 FFmpeg 是个大坑，建议为不同平台准备预编译的动态库。
- **插件安全性**: 限制插件访问文件系统的权限，防止恶意插件破坏用户数据 (沙盒化)。
