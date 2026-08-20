# DSH Desktop

DSH Desktop 是一个基于 Tauri 的轻量桌面托盘壳，用于在桌面窗口中运行用户本地安装的 DeepSeek Harness（`dsh`）Web 服务。

> [!IMPORTANT]
> 本项目不内置 `dsh`，也不会自动下载或更新它。使用前请先按照 [DeepSeek Harness 官方教程](https://deepseek.com/harness/) 安装命令行工具，并确保可以在终端执行 `dsh`。

## 注意事项

> [!NOTE]
> DSH Desktop 启动时会执行 `dsh web --port 0`，由系统自动分配可用端口；不会占用固定端口。

- `dsh` 不在当前环境中时，应用会提示用户打开官方安装页面，然后退出。
- 关闭主窗口只会销毁 WebView，托盘和 DSH 服务仍会保持运行；可从托盘菜单重新打开窗口。
- 使用托盘菜单中的 **Quit** 才会退出应用并停止由当前应用启动的 DSH 服务。
- Linux 托盘显示依赖桌面环境；GNOME 等环境可能需要 AppIndicator 支持。
- AppImage 不作为发布格式提供，以避免 Ubuntu 打包的 WebKitGTK 与 Arch Linux、Wayland、Mesa 运行库产生兼容性问题。

## 发布包

| 平台 | 文件 |
| --- | --- |
| Debian / Ubuntu | `.deb` |
| Arch Linux | `.pkg.tar.zst` |
| Windows | 可直接运行的 `.exe` 或 `.msi` 安装包 |

## 从源码运行

需要安装 Bun、Rust，以及当前平台的 Tauri 依赖。

```bash
bun install
bun tauri dev
```

生成发布包的流程由 GitHub Actions 完成。维护者可以使用以下命令预览或创建版本发布：

```bash
bun run release -- --dry-run
bun run release -- --push
```
