# BiuPlayer MVP 验收矩阵（51 项 P0）

> 归档日期：2026-11-25（第四轮：代码类缺口闭环）
> 对应文档：`docs/Mvp.md` §3 验收口径、§7.5 发布门禁  
> 状态口径：✅ 达标（有自动化或本地复验证据）；🟡 部分达标（代码完成或仅静态验证，仍有明确缺口）；⛔ 未达标

## 汇总

| 模块 | 项数 | ✅ | 🟡 | ⛔ |
| --- | ---: | ---: | ---: | ---: |
| AUD 播放 | 6 | 1 | 5 | 0 |
| PL 列表/队列 | 7 | 1 | 6 | 0 |
| LIB 媒体库 | 5 | 1 | 4 | 0 |
| TAG 标签 | 1 | 1 | 0 | 0 |
| LRC 歌词 | 3 | 3 | 0 | 0 |
| UI 界面 | 6 | 3 | 3 | 0 |
| SYS 系统 | 2 | 1 | 1 | 0 |
| DATA 数据 | 3 | 3 | 0 | 0 |
| SET 设置 | 3 | 2 | 1 | 0 |
| I18N 国际化 | 1 | 1 | 0 | 0 |
| NET 隐私 | 1 | 0 | 1 | 0 |
| REL 发布 | 9 | 2 | 7 | 0 |
| QA 质量 | 4 | 3 | 1 | 0 |
| **合计** | **51** | **22** | **29** | **0** |

## 明细

| ID | 验收要求（摘要） | 状态 | 证据 / 复验方式 | 剩余缺口 |
| --- | --- | :---: | --- | --- |
| AUD-01 | 7 格式解码 | ✅ | `cargo test -p biu-audio --test formats`（7/7 probe 通过） | 无声卡环境下仅解码路径 |
| AUD-02 | 文件头探测 / 损坏容错 | 🟡 | `tests/probe_errors.rs` 5 例；播放失败日志含 path/error | 无真实截断文件端到端自动跳曲用例 |
| AUD-04 | 设备枚举/切换/拔出 2s 回退 | 🟡 | `player_set_device`、`recover_output`、前端 `device_lost` 提示 | USB 真机拔插未测 |
| AUD-11 | 播放控制 ≤100ms | 🟡 | 命令齐全（播放/暂停/停止/上下首/seek/音量/静音） | 延迟未基准测试 |
| AUD-16 | 4 模式 + 重启保持 | 🟡 | `queue.rs` 模式单测；会话持久化 mode | 重启后不自动播放（需 `resume_on_start` 开关） |
| AUD-20 | 连续 3 坏文件不卡死 | 🟡 | 失败计数 + `stopped after three consecutive playback failures` 日志 | 无自动化用例 |
| PL-01 | 多列表 CRUD/排序/持久化 | 🟡 | `playlist_*` 命令 + `tests/playlists.rs` | 列表排序 UI 未做 |
| PL-02 | 文件/文件夹/拖放/右键/命令行添加 | 🟡 | 拖放、文件夹扫描、原生目录选择器、曲目右键加入播放列表（`Library.tsx:492`）、`--playlist` 命令行、单实例转交 | 多文件夹拖入只取第一个；无单文件选择器 |
| PL-03 | 拖拽排序/多选/批量/撤销 | 🟡 | `PlaylistDetail` drag/drop、多选、undo（内存 20 步） | 1 万条 55fps 未实测 |
| PL-05 | 7 列排序 + 恢复手动 | 🟡 | 标题/艺术家/专辑/添加时间/时长/路径/手动，后端 `list_tracks_sorted` | 窄屏隐藏添加时间/路径列 |
| PL-06 | 即时过滤 + 拼音 | ✅ | `librarySearch` 即时触发；`bench_100k` p50 32ms/p95 44ms | — |
| PL-08 | 队列下一首/插播/调整/清空/重启恢复 | 🟡 | `player_queue_*` 命令 + `player-session.json` | 队列外显性能未压测 |
| PL-11 | 10 万虚拟滚动 ≥55fps | 🟡 | 库页面与播放列表详情均已窗口化（`Library.tsx`、`PlaylistDetail.tsx`） | 未跑 10 万行 fps 基准 |
| LIB-01 | 多目录/递归/排除/隐藏/重扫 | 🟡 | `scan_folder_controlled_filtered`、excludes、隐藏点文件 | Windows 隐藏属性不识别；无目录监听 |
| LIB-02 | 全量+增量/进度/暂停/继续/断点 | 🟡 | `scan_bench` 1 万 11.35s、重扫 0.54s；暂停/继续单测 | 每文件一个事务；取消后需重输路径继续 |
| LIB-04 | 7 视图 + 分页/虚拟 + 占位封面 | 🟡 | 歌曲/专辑/艺术家/专辑艺术家/流派/年代/文件夹路由；封面占位；专辑艺术家由 schema v5 `album_artist` 聚合 | 除歌曲外的列表页未虚拟化，命令层无 limit/offset |
| LIB-06 | 搜索 + 拼音 + 10 万 ≤100ms + 高亮 | 🟡 | FTS5 + bm25，`bench_100k` max 56ms；高亮仅字面匹配 | 拼音查询不高亮；搜索行曾缺 artist/album（已修 DTO） |
| LIB-11 | WAL/迁移/幂等/崩溃可恢复 | ✅ | `tests/migrations.rs`；强杀进程后 integrity ok | — |
| TAG-01 | ID3v1/v2/APEv2/Vorbis/MP4/WAV | ✅ | `cargo test -p biu-library --test tag_formats`：6 类容器写入后扫描回读（含中文标题/艺术家/专辑与 albumartist）；ID3v1 用 Latin-1 ASCII | ID3v1 无 albumartist、无法存中文（格式限制） |
| LRC-01 | 标准/增强/多标签/offset | ✅ | 解析单测 + `[offset:-200]` → 800ms；`activeLineIndex` 边界/偏移单测；播放页 100ms 插值，offset 生效 | 误差上界由 100ms 插值决定，未在真机音频上抽测 |
| LRC-03 | 同名 .lrc + GBK/UTF-8 | ✅ | `biu-lyrics` 3 单测 + 占位不阻塞 | — |
| LRC-05 | 滚动/高亮/点击跳转 | ✅ | `scrollIntoView` + 行按钮 `playerSeek` | 无 GUI 点击自动化 |
| UI-01 | 侧栏/播放栏/折叠/调宽/持久化 | ✅ | `Shell.tsx` + `settingsUpdate` | 800×600 无截图基线 |
| UI-03 | 浅/深主题 + 跟随系统 + 无白闪 | 🟡 | 浅色变量补齐 + 首帧内联脚本 + E2E 断言 | 500ms 白闪未仪器化 |
| UI-08 | 无边框/置顶/透明度/尺寸记忆 | 🟡 | frameless 已持久化；window-state | 双屏 150% 未测 |
| UI-09 | 托盘菜单/关闭到托盘/状态同步 | ✅ | `tray.rs` 六项菜单 + tooltip 同步 + 三语 | — |
| UI-12 | 文件/文件夹拖放 + 高亮 | ✅ | `onDragDropEvent` + `.drop-overlay` | 多文件夹只扫第一个 |
| UI-15 | 动画 60fps、CPU ≤3% | 🟡 | `pnpm test:perf`（生产构建、10 万行）：滚动 60fps、DOM 行数 <80 | 播放 CPU ≤3% 与 8h 长稳需真机音频与时间，尚无数据 |
| SYS-02 | 单实例 + 参数转交 | ✅ | single-instance 插件 + `startupArgs` | — |
| SYS-10 | 便携模式 + 拷机恢复 | 🟡 | `portable.flag` → `<exe>/data` | 无创建入口；跨机拷贝未验证 |
| DATA-01 | 三平台目录 + 卸载不删音乐 | ✅ | `paths.rs` + 单测 | — |
| DATA-02 | settings 原子写 + 损坏备份 .bak | ✅ | `replace_file` + `settings.json.bak` + 单测 | — |
| DATA-03 | 迁移 + 回滚 + 迁移前备份 | ✅ | 各版本事务 + `library.db.bak` + 4 测试 | 迁移失败不自动还原 |
| SET-01 | 8 分类 + 仅显示已实现 + 需重启标注 | ✅ | 移除 gapless/crossfade/density/reduceMotion；“需重启”badge | — |
| SET-02 | 主题/音量/设备即时生效 | ✅ | 即时 `applyTheme`/`settingsUpdate` | — |
| SET-06 | 关于页版本/构建/许可/检查更新 | 🟡 | 版本/os/arch/Tauri/license 外链 | 无构建 commit、无第三方许可清单/致谢 |
| I18N-01 | 三语运行时切换、无硬编码 | ✅ | React + Rust 托盘三语；启动读 `settings.language` | — |
| NET-04 | 零遥测、默认零出站 | 🟡 | 无 http/updater 插件、无 fetch；见 `privacy-egress.md` | 未做 24h 抓包 |
| REL-01 | v* tag 产出四平台 | 🟡 | release.yml 矩阵齐全 | 0 个 tag，未实跑 |
| REL-02 | Win 绿色 zip 配置随目录 | ✅ | `portable.flag` + `paths.rs` | 文档 INSTALL 已纠偏 |
| REL-03 | NSIS 安装/升级/卸载无残留 | 🟡 | bundle nsis | 无干净机证据 |
| REL-04 | dmg 双架构未签名可打开 | 🟡 | 矩阵含 x64/arm64 | 未真机验证 |
| REL-05 | AppImage/deb/rpm + 22.04/24.04 | 🟡 | 三包 + CI Xvfb 冒烟 | 24.04/Wayland 未测 |
| REL-06 | `BiuPlayer_<version>_<os>_<arch>` | ✅ | 产物统一重命名 + `pnpm check:version` | — |
| REL-07 | SHA256SUMS + 来源证明 | 🟡 | SHA256SUMS.txt | 无 attestation |
| REL-08 | tag→draft→人工确认→发布+CHANGELOG | 🟡 | draft + CHANGELOG 正文 + tag/版本校验 | 无实际发布演练 |
| REL-10 | CI 质量门 fmt/clippy/test/lint/typecheck/build | ✅ | ci.yml 增加 typecheck/lint/test/version | E2E 仅在 ubuntu 作业 |
| QA-01 | 滚动日志 + 诊断包 + 脱敏 | ✅ | 按天滚动 + zip 诊断包；`Redactor` 脱敏 data/cache/log 目录、用户名、家目录（含 `/` 与 JSON 转义变体）；测试断言 zip 内无原始路径 | 无日志保留期上限 |
| QA-02 | panic 捕获/转储/下次提示 | ✅ | `install_panic_hook` 写 `crash-<millis>.log` + `latest_crash` 横幅；新增 catch_unwind 自动化用例 | 崩溃后自动恢复播放未做 |
| QA-03 | 单测 + 覆盖率达标 | ✅ | Rust 62 + Vitest 16 + Playwright 7；`pnpm test:coverage` 对 `theme.ts`/`lyricsTiming.ts` 设 100% 阈值；CI 跑 `cargo llvm-cov`（audio/library/lyrics/core，实测行覆盖 66.44%，阈值 45） | 应用壳 crate 不做插桩（tauri 插桩测试会挂起），阈值按基线设定 |
| QA-08 | README/构建指南/用户手册 | 🟡 | README/INSTALL/RELEASE 已更新 | 无独立用户手册 |

## 发布门禁（§7.5）判定

| 门禁项 | 判定 | 说明 |
| --- | --- | --- |
| 51/51 P0 有验收记录 | 🟡 | 本表覆盖 51 项；33 项仍为部分达标 |
| 7 格式 / 10 万库 / 8h 长稳 | 🟡 | 前两项有本地数据；8h 长稳脚本已备，未实跑 |
| 三平台安装包 | ⛔ | 未打 tag、未实跑 release 矩阵 |
| 默认零出站 / 隐私说明 | 🟡 | 静态审计 + 说明已归档，无抓包 |
| S1/S2 = 0 | 🟡 | 审计未发现 S1；S2 见本表 ⛔/🟡 项 |
| 新人可构建 / 回滚演练 | 🟡 | 构建文档已更新；回滚未演练 |
