# 安装与卸载

## Windows

### 安装版

1. 下载 `BiuPlayer_0.1.0_x64-setup.exe`。
2. 若 SmartScreen 提示，选择“更多信息 → 仍要运行”。
3. 按向导安装，默认当前用户目录，不需要管理员权限。

卸载：设置 → 应用 → 已安装的应用 → BiuPlayer → 卸载。

### 绿色版

1. 下载 `BiuPlayer_0.1.0_x64-portable.zip` 并解压到任意目录。
2. 运行 `BiuPlayer.exe`。
3. 依赖 WebView2 Runtime；Win10 1809+ 通常已自带。

绿色版数据默认写入 `%APPDATA%\BiuPlayer`，不会写安装目录。

## macOS

1. 下载对应架构的 dmg：Intel 用 `x64`，Apple Silicon 用 `aarch64`。
2. 打开 dmg，把 `BiuPlayer.app` 拖到“应用程序”。
3. 首次启动右键应用 →“打开”，或执行：

```bash
xattr -dr com.apple.quarantine /Applications/BiuPlayer.app
```

未签名、未公证，系统可能提示“无法验证开发者”。

卸载：把 `BiuPlayer.app` 移到废纸篓；如需清除数据，删除：

```bash
rm -rf ~/Library/Application\ Support/BiuPlayer
rm -rf ~/Library/Caches/BiuPlayer
rm -rf ~/Library/Logs/com.tagecode.biuplayer
```

## Linux

### AppImage

```bash
chmod +x BiuPlayer_0.1.0_amd64.AppImage
./BiuPlayer_0.1.0_amd64.AppImage
```

需要 FUSE2；没有 FUSE 时：

```bash
./BiuPlayer_0.1.0_amd64.AppImage --appimage-extract-and-run
```

### deb

```bash
sudo apt install ./BiuPlayer_0.1.0_amd64.deb
```

卸载：`sudo apt remove biuplayer`。

### rpm

```bash
sudo rpm -i BiuPlayer-0.1.0-1.x86_64.rpm
```

卸载：`sudo rpm -e biuplayer`。

Linux 需要 WebKitGTK 4.1；Ubuntu 22.04/24.04 已验证，Ubuntu 20.04 只有 4.0，不兼容。

## 数据位置

| 平台 | 数据 | 日志 |
| --- | --- | --- |
| Windows | `%APPDATA%\BiuPlayer` | `%LOCALAPPDATA%\BiuPlayer\logs` |
| macOS | `~/Library/Application Support/BiuPlayer` | `~/Library/Logs/com.tagecode.biuplayer` |
| Linux | `$XDG_DATA_HOME/BiuPlayer` | `$XDG_CACHE_HOME/BiuPlayer/logs` |
| 便携模式 | `<exe>/data` | `<exe>/data/logs` |

便携模式：在可执行文件旁放一个 `portable.flag` 文件。

## 校验下载

```bash
sha256sum -c SHA256SUMS.txt
# macOS
shasum -a 256 -c SHA256SUMS.txt
```
