# 隐私与出站请求审计（NET-04）

> 归档日期：2026-11-25  
> 结论：应用代码默认零出站、无遥测、无账号体系；唯一网络行为是用户主动点击外链时交给系统浏览器。

## 审计方法

静态审计覆盖三处可能发起网络请求的地方：

1. 前端（`src/**`）：检查 `fetch(`、`XMLHttpRequest`、`WebSocket`、`sendBeacon`、`new Image()` 等。
2. Rust 依赖与源码（`src-tauri/**`）：检查 `reqwest`、`hyper`、`ureq`、`tauri-plugin-http`、`tauri-plugin-updater`、`sentry`、`telemetry`。
3. npm 依赖（`package.json`）：检查 `analytics`、`sentry`、`http`、`updater` 相关包。

复核命令（PowerShell）：

```powershell
Select-String -Path src\*.ts,src\*.tsx,src\**\*.tsx -Pattern 'fetch\(|WebSocket|XMLHttpRequest|sendBeacon'
Select-String -Path src-tauri\Cargo.toml -Pattern 'reqwest|http|updater|telemetry|sentry'
Select-String -Path package.json -Pattern 'updater|http|analytics|sentry'
```

三条命令均无匹配（2026-11-25 实测）。

## 结论与证据

| 检查项 | 结果 |
| --- | --- |
| 遥测 / 崩溃上报服务 | 无。panic 只写本地 `crash-<ts>.log`，不发送 |
| 账号 / 登录 | 无 |
| 更新检查 | 仅“检查更新”按钮调用 `openUrl` 打开 GitHub Releases 页面，应用内不发请求 |
| 网络请求库 | Rust 依赖无 reqwest/hyper/ureq；Tauri 插件仅 opener / single-instance / window-state / dialog |
| 前端请求 | 无 fetch / XHR / WebSocket；所有数据来自 Tauri IPC |
| 启动行为 | 启动只读本地配置、数据库与队列文件，无网络调用 |
| 外链 | `Settings → 关于`、许可链接、更新说明；由系统默认浏览器打开 |

## 已知边界

- `tauri.conf.json` 的 `security.csp` 为 `null`（未配置 CSP）。当前代码不发起请求，但缺少 CSP 意味着一旦引入远程内容就没有二次防线；建议在 P1 收敛为 `default-src 'self'`。
- WebView2 / WebKitGTK 运行时自身可能产生系统级更新或证书检查流量，不属于本应用代码行为，也不受应用控制。
- 本报告是静态审计；未做 24 小时抓包/DNS 观测（需真机与网络工具）。发布前建议补一次抓包并归档 pcap 摘要。
