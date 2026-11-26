# BiuPlayer MVP 验收矩阵（51 项 P0）

> 归档日期：2026-11-25（第四轮：代码类缺口闭环）
> 复核：2026-11-26（第五轮：关闭剩余 10 个代码缺口，35✅ / 16🟡 / 0⛔）
> 对应文档：`docs/Mvp.md` §3 验收口径、§7.5 发布门禁  
> 状态口径：✅ 达标（有自动化或本地复验证据）；🟡 部分达标（代码完成或仅静态验证，仍有明确缺口）；⛔ 未达标

## 汇总

| 模块 | 项数 | ✅ | 🟡 | ⛔ |
| --- | ---: | ---: | ---: | ---: |
| AUD 播放 | 6 | 3 | 3 | 0 |
| PL 列表/队列 | 7 | 5 | 2 | 0 |
| LIB 媒体库 | 5 | 4 | 1 | 0 |
| TAG 标签 | 1 | 1 | 0 | 0 |
| LRC 歌词 | 3 | 3 | 0 | 0 |
| UI 界面 | 6 | 4 | 2 | 0 |
| SYS 系统 | 2 | 1 | 1 | 0 |
| DATA 数据 | 3 | 3 | 0 | 0 |
| SET 设置 | 3 | 3 | 0 | 0 |
| I18N 国际化 | 1 | 1 | 0 | 0 |
| NET 隐私 | 1 | 0 | 1 | 0 |
| REL 发布 | 9 | 3 | 6 | 0 |
| QA 质量 | 4 | 4 | 0 | 0 |
| **合计** | **51** | **35** | **16** | **0** |

## 明细

| ID | 验收要求（摘要） | 状态 | 证据 / 复验方式 | 剩余缺口 |
| --- | --- | :---: | --- | --- |
| AUD-01 | 7 格式解码 | ✅ | `cargo test -p biu-audio --test formats`（7/7 probe 通过）；`cargo test -p biu-audio --test output_smoke -- --ignored` 默认设备真实播放 0.3s 通过（修复 U8 配置选择，见 `docs/reports/windows-audio-smoke.md`） | 扬声器听感未人工验证 |
| AUD-02 | 文件头探测 / 损坏容错 | 🟡 | `tests/probe_errors.rs` 5 例；播放失败日志含 path/error | 无真实截断文件端到端自动跳曲用例 |
| AUD-04 | 设备枚举/切换/拔出 2s 回退 | 🟡 | `player_set_device`、`recover_output`、前端 `device_lost` 提示；真实设备枚举/播放冒烟通过 | USB 真机拔插未测 |
| AUD-11 | 播放控制 ≤100ms | 🟡 | 命令齐全（播放/暂停/停止/上下首/seek/音量/静音） | 延迟未基准测试 |
| AUD-16 | 4 模式 + 重启保持 | ✅ | `queue.rs` 模式单测；`Session.mode` 持久化，`persists_queue_for_restart` 断言恢复为 `Shuffle` | 重启后自动继续播放由 A1-05 的 `resume_on_start` 控制，属另一项 |
| AUD-20 | 连续 3 坏文件不卡死 | ✅ | `FailureTracker` + 单测 `skips_two_bad_files_and_stops_on_the_third`、`a_successful_track_resets_the_failure_streak`；日志含 `stopped after three consecutive playback failures` | — |
| PL-01 | 多列表 CRUD/排序/持久化 | ✅ | `playlist_*` 命令 + `tests/playlists.rs`；播放列表页「名称 A→Z / Z→A / 默认顺序」排序并写回 `sort_order` | 无按创建时间排序 |
| PL-02 | 文件/文件夹/拖放/右键/命令行添加 | ✅ | 拖放（多目录一次性 `libraryScanStartMany`）、目录选择器、「添加文件…」多选 `libraryImportFiles`、曲目右键加入播放列表、`--playlist` 命令行、单实例转交 | — |
| PL-03 | 拖拽排序/多选/批量/撤销 | 🟡 | `PlaylistDetail` drag/drop、多选、undo（内存 20 步） | 1 万条 55fps 未实测 |
| PL-05 | 7 列排序 + 恢复手动 | ✅ | 标题/艺术家/专辑/添加时间/时长/路径/手动 7 个排序键，后端 `list_tracks_sorted`；手动排序持久化 | 窄屏（<1100px）隐藏添加时间/路径列，属响应式取舍 |
| PL-06 | 即时过滤 + 拼音 | ✅ | `librarySearch` 即时触发；`bench_100k` p50 32ms/p95 44ms | — |
| PL-08 | 队列下一首/插播/调整/清空/重启恢复 | 🟡 | `player_queue_*` 命令 + `player-session.json` | 队列外显性能未压测 |
| PL-11 | 10 万虚拟滚动 ≥55fps | ✅ | `Library.tsx` 与 `PlaylistDetail.tsx` 均窗口化；`pnpm test:perf` 10 万行实测 60.0fps、DOM 26 行 | 播放列表页自身 1 万行未单独跑 fps |
| LIB-01 | 多目录/递归/排除/隐藏/重扫 | 🟡 | `scan_folder_controlled_filtered`、excludes、隐藏点文件 | Windows 隐藏属性不识别；无目录监听 |
| LIB-02 | 全量+增量/进度/暂停/继续/断点 | ✅ | `scan_bench` 1 万 11.35s、重扫 0.54s；暂停/继续单测；取消后 UI 可直接「继续」并从 `session.status.root` 续扫 | 仍为每文件一个事务（1 万条 11.35s 已低于 60s 目标） |
| LIB-04 | 7 视图 + 分页/虚拟 + 占位封面 | ✅ | 歌曲虚拟滚动；专辑/艺术家/专辑艺术家/流派/年代/文件夹列表分页加载（`useLoadMore`，每页 120）；专辑艺术家由 schema v5 `album_artist` 聚合 | 列表命令层无 limit/offset，分页在前端 |
| LIB-06 | 搜索 + 拼音 + 10 万 ≤100ms + 高亮 | ✅ | FTS5 + bm25，`bench_100k` max 56ms；拼音查询整行高亮（`isPinyinQuery`），中文按字面高亮；搜索行已补齐 artist/album | 高亮为整行而非拼音音节定位 |
| LIB-11 | WAL/迁移/幂等/崩溃可恢复 | ✅ | `tests/migrations.rs`；强杀进程后 integrity ok | — |
| TAG-01 | ID3v1/v2/APEv2/Vorbis/MP4/WAV | ✅ | `cargo test -p biu-library --test tag_formats`：6 类容器写入后扫描回读（含中文标题/艺术家/专辑与 albumartist）；ID3v1 用 Latin-1 ASCII | ID3v1 无 albumartist、无法存中文（格式限制） |
| LRC-01 | 标准/增强/多标签/offset | ✅ | 解析单测 + `[offset:-200]` → 800ms；`activeLineIndex` 边界/偏移单测；播放页 100ms 插值，offset 生效 | 误差上界由 100ms 插值决定，未在真机音频上抽测 |
| LRC-03 | 同名 .lrc + GBK/UTF-8 | ✅ | `biu-lyrics` 3 单测 + 占位不阻塞 | — |
| LRC-05 | 滚动/高亮/点击跳转 | ✅ | `scrollIntoView` + 行按钮 `playerSeek` | 无 GUI 点击自动化 |
| UI-01 | 侧栏/播放栏/折叠/调宽/持久化 | ✅ | `Shell.tsx` + `settingsUpdate`；min 760×520；`docs/reports/assets/mvp-fixes/light-800.png` 为 800×600 基线；Windows 原生 WebDriver 在 960×680 实测 `scrollWidth == innerWidth`（`docs/reports/windows-native-e2e.md`），Chromium 侧有 960×680 回归用例 | 800×600 仅截图，无自动化断言 |
| UI-03 | 浅/深主题 + 跟随系统 + 无白闪 | ✅ | 浅色变量补齐 + 首帧内联脚本（记录 `__biuThemeBoot`）；E2E `theme is applied before the bundle paints` 断言首帧为 light 且背景 `#f6f7f9` | 未做 500ms 内逐帧采样 |
| UI-08 | 无边框/置顶/透明度/尺寸记忆 | 🟡 | frameless 已持久化；window-state | 双屏 150% 未测 |
| UI-09 | 托盘菜单/关闭到托盘/状态同步 | ✅ | `tray.rs` 六项菜单 + tooltip 同步 + 三语 | — |
| UI-12 | 文件/文件夹拖放 + 高亮 | ✅ | `onDragDropEvent` + `.drop-overlay`；多目录一次 `libraryScanStartMany` | — |
| UI-15 | 动画 60fps、CPU ≤3% | 🟡 | `pnpm test:perf`（生产构建、10 万行）：滚动 60fps、DOM 行数 <80 | 播放 CPU ≤3% 与 8h 长稳需真机音频与时间，尚无数据 |
| SYS-02 | 单实例 + 参数转交 | ✅ | single-instance 插件 + `startupArgs` | — |
| SYS-10 | 便携模式 + 拷机恢复 | 🟡 | 设置 → 数据可勾选便携模式（`portable_flag_set` 写/删 `portable.flag`），并显示当前数据目录 | 跨机拷贝恢复未实测 |
| DATA-01 | 三平台目录 + 卸载不删音乐 | ✅ | `paths.rs` + 单测 | — |
| DATA-02 | settings 原子写 + 损坏备份 .bak | ✅ | `replace_file` + `settings.json.bak` + 单测 | — |
| DATA-03 | 迁移 + 回滚 + 迁移前备份 | ✅ | 各版本事务 + `library.db.bak` + 4 测试；打开失败自动从 `.bak` 还原（`SchemaTooNew` 除外） | 还原为整库覆盖，非增量修复 |
| SET-01 | 8 分类 + 仅显示已实现 + 需重启标注 | ✅ | 移除 gapless/crossfade/density/reduceMotion；“需重启”badge | — |
| SET-02 | 主题/音量/设备即时生效 | ✅ | 即时 `applyTheme`/`settingsUpdate` | — |
| SET-06 | 关于页版本/构建/许可/检查更新 | ✅ | 关于页显示版本/os/arch/Tauri/构建 commit（`BIU_GIT_COMMIT`）、致谢、`docs/THIRD_PARTY_LICENSES.md` 外链、检查更新外链 | 许可清单为文档级，未内嵌依赖版本 |
| I18N-01 | 三语运行时切换、无硬编码 | ✅ | React + Rust 托盘三语；启动读 `settings.language` | — |
| NET-04 | 零遥测、默认零出站 | 🟡 | 无 http/updater 插件、无 fetch；见 `privacy-egress.md` | 未做 24h 抓包 |
| REL-01 | v* tag 产出四平台 | 🟡 | release.yml 矩阵齐全 | 0 个 tag，未实跑 |
| REL-02 | Win 绿色 zip 配置随目录 | ✅ | `portable.flag` + `paths.rs` | 文档 INSTALL 已纠偏 |
| REL-03 | NSIS 安装/升级/卸载无残留 | 🟡 | bundle nsis：`BiuPlayer_0.1.0_x64-setup.exe` 构建通过；`scripts/nsis-smoke.ps1` 已就绪（静默安装→检查文件/快捷方式/注册表→便携启动→卸载） | 本机 `%TEMP%` ACL 异常导致安装器静默安装为空操作（详见 `docs/reports/windows-nsis-smoke.md`），需干净机执行冒烟脚本 |
| REL-04 | dmg 双架构未签名可打开 | 🟡 | 矩阵含 x64/arm64 | 未真机验证 |
| REL-05 | AppImage/deb/rpm + 22.04/24.04 | 🟡 | 三包 + CI Xvfb 冒烟 | 24.04/Wayland 未测 |
| REL-06 | `BiuPlayer_<version>_<os>_<arch>` | ✅ | 产物统一重命名 + `pnpm check:version` | — |
| REL-07 | SHA256SUMS + 来源证明 | 🟡 | SHA256SUMS.txt | 无 attestation |
| REL-08 | tag→draft→人工确认→发布+CHANGELOG | 🟡 | draft + CHANGELOG 正文 + tag/版本校验 | 无实际发布演练 |
| REL-10 | CI 质量门 fmt/clippy/test/lint/typecheck/build | ✅ | ci.yml 增加 typecheck/lint/test/version | E2E 仅在 ubuntu 作业 |
| QA-01 | 滚动日志 + 诊断包 + 脱敏 | ✅ | 按天滚动 + zip 诊断包；`Redactor` 脱敏 data/cache/log 目录、用户名、家目录（含 `/` 与 JSON 转义变体）；测试断言 zip 内无原始路径 | 无日志保留期上限 |
| QA-02 | panic 捕获/转储/下次提示 | ✅ | `install_panic_hook` 写 `crash-<millis>.log` + `latest_crash` 横幅；新增 catch_unwind 自动化用例 | 崩溃后自动恢复播放未做 |
| QA-03 | 单测 + 覆盖率达标 | ✅ | Rust 67 + Vitest 16 + Playwright 8；`pnpm test:coverage` 对 `theme.ts`/`lyricsTiming.ts` 设 100% 阈值；CI 跑 `cargo llvm-cov`（audio/library/lyrics/core，实测行覆盖 66.44%，阈值 45） | 应用壳 crate 不做插桩（tauri 插桩测试会挂起），阈值按基线设定 |
| QA-08 | README/构建指南/用户手册 | ✅ | README/INSTALL/RELEASE + `docs/USER_GUIDE.md`（导入/播放/搜索/歌词/设置/便携/诊断/FAQ） | 无截图版手册 |

## 发布门禁（§7.5）判定

| 门禁项 | 判定 | 说明 |
| --- | --- | --- |
| 51/51 P0 有验收记录 | 🟡 | 本表覆盖 51 项；16 项仍为部分达标 |
| 7 格式 / 10 万库 / 8h 长稳 | 🟡 | 前两项有本地数据；8h 长稳脚本已备，未实跑 |
| 三平台安装包 | ⛔ | 未打 tag、未实跑 release 矩阵 |
| 默认零出站 / 隐私说明 | 🟡 | 静态审计 + 说明已归档，无抓包 |
| S1/S2 = 0 | 🟡 | 审计未发现 S1；S2 见本表 ⛔/🟡 项 |
| 新人可构建 / 回滚演练 | 🟡 | 构建文档已更新；回滚未演练 |
