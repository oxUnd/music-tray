# music-tray

一个在 Linux 操作系统下，音乐控制的小工具，它可以把音乐软件正在播放的歌曲的内容显示出来，包括音乐的封面、歌名、以及播放进度，包含上一曲、下一曲、暂停与播放的按钮。

## 功能特性

- 🎵 显示当前播放的音乐信息（歌名、艺术家、专辑）
- ⏯️ 音乐控制（播放/暂停、上一曲、下一曲）
- 📊 播放进度条显示
- 🎨 美观的 TUI 界面
- 🔄 实时更新音乐状态
- 🔌 MPRIS 集成框架（支持 Spotify、VLC、Rhythmbox 等）
- 🎛️ D-Bus 连接管理

## 约束条件
- 使用 https://github.com/ratatui/ratatui 来实现交互界面
- 使用 rust 语言开发

## 构建和运行

### 构建项目
```bash
cargo build --release
```

### 运行应用
```bash
cargo run
```

### 控制键
- `SPACE` - 播放/暂停
- `N` - 下一曲
- `P` - 上一曲
- `Q` - 退出应用

## 技术实现

- **TUI 框架**: ratatui (基于 crossterm)
- **音乐控制**: MPRIS (Media Player Remote Interfacing Specification)
- **异步运行时**: tokio
- **D-Bus 通信**: zbus
- **序列化**: serde
- **错误处理**: anyhow

## 实现状态

### ✅ 已完成
- 基础 TUI 界面
- MPRIS 集成框架
- D-Bus 连接管理
- 音乐控制接口
- 播放器检测

### 🔧 当前限制
- 使用模拟数据进行演示
- 需要真正的 D-Bus 查询实现
- 封面艺术显示待完善

## 平台支持

### ✅ Linux
- 完整的 MPRIS 支持
- 所有音乐播放器集成功能

## 项目结构

```
src/
├── main.rs      # 主程序入口和 TUI 界面
└── music.rs     # 音乐播放器集成和 MPRIS 处理
```

## 依赖项

- `ratatui` - TUI 框架
- `crossterm` - 跨平台终端操作
- `tokio` - 异步运行时
- `zbus` - D-Bus 通信
- `serde` - 序列化支持
- `anyhow` - 错误处理
