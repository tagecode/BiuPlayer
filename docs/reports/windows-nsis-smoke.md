# Windows NSIS 安装包冒烟（构建通过，安装验证受阻）

日期：2026-11-27
范围：REL-03（NSIS 安装/升级/卸载无残留）、REL-02（绿色版数据随目录）。

## 构建

`pnpm tauri build --bundles nsis` 首次失败在 `Downloading nsis-3.13.zip` 超时。
手动补齐 bundler 期望的工具链后构建成功：

- NSIS 3.13 解压 `%LOCALAPPDATA%\tauri\NSIS`（`makensis.exe` 必须在根层）；
- `nsis_tauri_utils.dll` 0.5.3 放到 `NSIS\Plugins\x86-unicode\additional\`；
- 产物：`src-tauri/target/release/bundle/nsis/BiuPlayer_0.1.0_x64-setup.exe`（4.51 MiB）。

## 冒烟脚本

`scripts/nsis-smoke.ps1`（Windows PowerShell 5.1 兼容）流程：

1. 静默安装 `/S /D=<临时目录>`，等待安装器退出；
2. 断言 `biuplayer.exe`、`uninstall.exe`、开始菜单快捷方式、`HKCU\...\Uninstall` 键存在；
3. 在安装目录写 `portable.flag`，启动 6 秒，断言 `data\logs` 被创建（便携模式数据随程序目录）；
4. 静默卸载，断言 exe/快捷方式/注册表键均被移除。

脚本本身已验证可执行（语法、路径、断言逻辑），但在本机无法得到有效的安装结果，原因如下。

## 本机阻塞

宿主 `%TEMP%`（`C:\Users\ADMINI~1\AppData\Local\Temp`）ACL 异常：普通进程无法写入，
此前 esbuild、link.exe、msedgedriver 都因此失败。NSIS 安装器同样受影响：

- 默认环境下 `/S` 立即失败（exit=2），UI 模式弹出
  **“NSIS Error — Error writing temporary file. Make sure your temp folder is valid.”**；
- 把 `TEMP`/`TMP` 指到可写目录后，安装器不再报错，但 `/S` 返回 0 且**不创建任何文件、
  不写卸载注册表键**（UI 向导可以打开，标题 “BiuPlayer Setup”，但自动点按无法稳定完成）。

已排除的因素：安装模式 `currentUser`（不需要提权）、WebView2 已安装（HKLM WOW6432Node 158.x）、
`/D=` 显式目录、`CRCCheck off` 重编译、最小 NSIS 安装器对照组（对照组在本机可正常安装）。

结论：**安装包本身构建成功，静默安装行为需在干净 Windows 机器上复核**；
本机结果不能作为 REL-03 的通过证据，也不是产品缺陷的证明。

## 干净机执行方式

```powershell
# 管理员或普通用户均可（currentUser 安装）
powershell -ExecutionPolicy Bypass -File scripts\nsis-smoke.ps1 `
  -Installer src-tauri\target\release\bundle\nsis\BiuPlayer_0.1.0_x64-setup.exe
```

若脚本通过，把结果补进 `docs/reports/mvp-acceptance-matrix.md` 的 REL-03 行。

## 副产物

调试期间把 `%LOCALAPPDATA%\BiuPlayer` 改名为 `BiuPlayer-dev-cache` 以排除旧数据干扰，
验证后已改回；本机 `%TEMP%` 的 ACL 问题属于宿主环境，未修改系统设置。
