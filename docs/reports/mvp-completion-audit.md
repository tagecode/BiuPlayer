# BiuPlayer MVP 完成度审计

- 审计对象：`docs/Mvp.md` §3 的 51 项 P0，对照提交 `71803f8`（2026-11-25）
- 审计方式：只读代码 + 本机执行构建/静态检查/测试；未修改任何源码
- 结论：**MVP 功能主体已就位，但发布门禁未通过**。51 项中 14 项达成、35 项部分达成、2 项未达成

## 0. 本机实测结果

| 命令 | 结果 |
| --- | --- |
| `pnpm exec tsc` / `vite build` | 通过（dist 447.51 kB JS / 24.96 kB CSS） |
| `cargo fmt --all --check` | 通过 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 |
| `cargo test --workspace` | 43 个测试通过（audio 9 / core 13 / library 18 / lyrics 3；library 需把 TEMP 指到仓库内目录，默认 `%TEMP%` 会 `PermissionDenied`）；`biuplayer` 测试二进制启动失败 `STATUS_ENTRYPOINT_NOT_FOUND (0xc0000139)`，该 crate 的 18 个测试未执行 |
| `pnpm test:e2e` | 5/5 通过（本机缺 chromium-1255 且下载受阻，改用已安装 Chrome `channel` 复跑；用例本身未改） |
| `pnpm typecheck` / `pnpm lint` / `pnpm test` | 均报 `ERR_PNPM_NO_SCRIPT`，脚本不存在 |

> 修正：审计过程中曾把 `index.html` 标题显示为“鍗氶煶”判为乱码，实际是 PowerShell 控制台按 GBK 解码 UTF-8 的显示问题，文件内容为正常的“博音 BiuPlayer”。

## 1. 51 项 P0 矩阵

状态：✅ 达成；🟡 部分达成；❌ 未达成。

### 播放 AUD

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| AUD-01 七格式解码 | ✅ | `crates/biu-audio/tests/formats.rs` 单测一次解码 mp3/flac/wav/aac/alac/ogg/aiff 全通过 | 无真实音乐文件/长音频验证 |
| AUD-02 损坏文件容错 | ✅ | `probe_errors.rs` 5 例（空/文本/截断/错扩展名/元数据一致）；`commands/player.rs:563-586` 失败日志含 path+error 并自动下一首 | 无音频端到端坏文件用例 |
| AUD-04 设备切换/拔插回退 | 🟡 | 设备枚举 `Settings.tsx:40`；切换续播 `player.rs:171-197`；`device_lost` 2s 回退 `player.rs:456-482,537-549`；提示 `NowPlaying.tsx:188` | 无真机拔插验证；**启动时未应用已保存的输出设备**（`lib.rs:181` 用 `AudioPlayerState::default()`） |
| AUD-11 播放控制 ≤100ms | 🟡 | 播放/暂停/停止/上下首/seek/音量/静音命令齐全（`player.rs`） | 无时延测量；**取消静音固定恢复 100%，丢失原音量**（`player.rs:308-315`）；音量不持久化；NowPlaying 页无静音键 |
| AUD-16 四模式+重启保持 | ✅ | `queue.rs` PlayMode 四种；`set_mode` 即 `save()`；`persists_queue_for_restart` 测试 | 该单测本机无法加载，仅静态确认 |
| AUD-20 连续坏文件不卡死 | 🟡 | `player.rs:563-586` 失败日志含 path/error；failures≥3 停止并 `tracing::error!` | **failures 按 250ms tick 采样且会被 playing/loading 重置，不是“按文件”计数**；无自动化用例 |

### 列表 PL

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| PL-01 多列表 CRUD/排序/持久化 | ✅ | `playlists` 表 + `sort_order`（v3）；`tests/playlists.rs` 4 例；`PlaylistDetail` 拖拽/复制/批量删除 | — |
| PL-02 文件/文件夹/拖放/右键/命令行添加+去重 | 🟡 | 拖放导入 `Shell.tsx:248-282`；右键加入列表（`Library.tsx:552-585`）；`single-instance` 转发路径；`add_tracks` 按路径去重 | 无 `tauri-plugin-dialog`，**没有系统文件/文件夹选择器**，只能拖放或手输路径；多文件夹拖入只取 `folders[0]`；重复项只显示一行提示文字 |
| PL-03 拖拽排序/多选/删除/批量/撤销 | 🟡 | `PlaylistDetail.tsx` drag、selected、history、批量移除；`library_move_track` 写 `manual_order` | 1 万条 ≥55fps 未测且该页全量渲染，场景实际不成立；**批删后 `selected` 未清空（可误删）**；undo 依赖全量 id 集一致，列表一变即失败 |
| PL-05 七列排序+恢复手动 | 🟡 | `TrackSort` 类型含 added；后端 `list_tracks_sorted` 支持 title/artist/album/duration/added/path/manual（`lib.rs:1086-1129`） | UI 排序按钮只有 title/artist/album/duration/manual（`Library.tsx:434-452`），**缺 added/path**；手动排序只有 ↑↓，无拖拽；稳定排序无测试 |
| PL-06 即时过滤+拼音 | 🟡 | 200ms 防抖后 `librarySearch`（`Library.tsx:76-89`）；pinyin 四变体；高亮；`scan.rs:122-163` 覆盖 qingtian/qt/zjl 等 | 结果上限 50；1 万条 ≤1s 无基准；**按字面 `zd` 不命中“周杰伦”（首字母为 zjl），按验收口径可能判失败** |
| PL-08 队列查看/插播/调整/清空/重启恢复 | 🟡 | `Shell.tsx:489-551` 队列抽屉 + 拖动调序；`player_queue_play_next/move/remove/clear`；session 持久化 items/index/mode | **`insert_next`（`queue.rs:111-119`）未 `rebuild_shuffle`**，随机模式下“下一首播放”未必真下一首；无独立“插播”；不保存播放进度，见 §2.2 |
| PL-11 10 万虚拟滚动 ≥55fps | 🟡 | `Library.tsx:107` 自实现虚拟列表（ROW_HEIGHT/OVERSCAN） | 仅库页虚拟化，`PlaylistDetail` 用普通 `TrackTable`；10 万 fps 无实测 |

### 库 LIB

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| LIB-01 多目录/排除/隐藏/重扫 | ✅ | `scan_folder_controlled_filtered`；`scan_skips_hidden_and_excluded_folders_and_removes_folder` 测试；设置页 excludes 编辑 | — |
| LIB-02 全量+增量/进度/暂停/取消/继续/断点 | 🟡 | `begin_scan`、scan_status scanned/inserted/skipped、pause/resume/cancel；`resume_from=scan_state`；`scan_pauses_and_resumes_without_duplicates` 测试 | 1 万 ≤60s 无固定基准（qa-100k 仅 10 万空文件 70s/1s） |
| LIB-04 六视图+聚合+占位封面 | 🟡 | albums/artists/genres/years/folders 命令与页面；`albums_and_artists_aggregate_tracks` 测试；`AlbumCover` | 各列表页未分页/未虚拟化，大库会全量渲染 |
| LIB-06 搜索+拼音+100ms+高亮 | 🟡 | FTS5 v2 + `build_search_text` 四变体；path 列可搜文件名；HighlightText | 10 万 ≤100ms 只有 DB 直查 p95 9.95ms（qa-100k），未覆盖 IPC+渲染 |
| LIB-11 WAL/迁移/索引/kill -9 | 🟡 | WAL+synchronous=NORMAL；v1-v4 各自事务；迁移前 `.bak` 整库备份；`tests/migrations.rs` 4 例含 legacy 备份与幂等 | 无 kill -9 恢复测试；迁移失败只置 library=None 并记日志（`lib.rs:185-194`），**不自动从 .bak 还原** |

### 标签 TAG

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| TAG-01 多格式标签+中文+文件名回退 | 🟡 | `biu-library/src/lib.rs:200-230` lofty 读 title/artist/album/track/disk/genre/year/pictures；无标签回退 `file_stem`（:909-917） | 无中文/日文标签 fixture 与测试；WAV ID3、APEv2 未单独验证 |

### 歌词 LRC

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| LRC-01 标准/增强/多时间/offset/<200ms | 🟡 | `biu-lyrics` parse 支持 offset、多时间标签、增强标签剥离；3 个单测 | 无 <200ms 同步精度测量 |
| LRC-03 同名 .lrc+GBK/UTF-8+占位 | ✅ | `find_sidecar` .lrc/.LRC；decode GBK/UTF-16/BOM；内嵌歌词回退；`decodes_gbk` 测试；无歌词显示占位 | 内嵌歌词只读（符合 §2.3） |
| LRC-05 逐行滚动/高亮/点击跳转 | ✅ | `NowPlaying.tsx:99-101` scrollIntoView；`:215-226` 行按钮 `playerSeek` | — |

### 界面 UI

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| UI-01 三栏/折叠/调宽/持久化/800×600 | ✅ | `Shell.tsx:79-157` 宽/折叠写 settings；min 760×520；`shell.css` 响应式 | 800×600 无自动化 |
| UI-03 明暗+跟随系统+无白闪 | 🟡 | `theme.ts` applyTheme/initTheme；跟随系统监听 | **全仓仅 1 条 `[data-theme="light"]` 规则**（`shell.css:1`）；`App.css:35-39` body 深色径向渐变写死；窗口 backgroundColor 固定深色；无内联主题脚本；Shell 快捷切换只写 `biu.theme` 不写 `biu.themeMode`，重启会回退 |
| UI-08 无边框/拖拽/置顶/透明度/记忆 | 🟡 | `windowing.rs`；`window-state` 插件；`Shell.tsx` 拖拽区与按钮 | **`SettingsPatch.frameless` 被 `settings_update` 静默丢弃**（`commands/settings.rs:99` 有字段、无处理分支）→ 重启丢失；默认带边框；双屏 150% 未验证 |
| UI-09 托盘/关闭到托盘/状态同步 | ✅ | `tray.rs` 六项菜单、左键、tooltip 与播放态同步；CloseRequested→hide；`lib.rs:217-219` | 托盘文案硬编码中文；无“关闭到托盘”开关 |
| UI-12 拖放+高亮 | ✅ | `Shell.tsx:317-329` 监听；`shell.css:791-804` 高亮 | 多文件夹只取第一个 |
| UI-15 60fps/CPU≤3% | ❌ | 仅 4 处 transition/animation | 无 fps/CPU 基准脚本、无 CI 门槛、无可复核报告 |

### 系统 SYS

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| SYS-02 单实例+参数转交 | ✅ | `lib.rs:132-144`；`startupArgs` 命令；前端 `Shell.tsx:327-329` 处理裸路径 | 无文件关联注册 |
| SYS-10 便携模式+拷机恢复 | 🟡 | `paths.rs:25-33` `portable.flag`→exe/data；队列/DB/settings 均在 data 下 | **无创建 portable.flag 的入口/UI**；exe 目录只读无提示；跨机拷贝未验证 |

### 数据 DATA

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| DATA-01 三平台目录+便携覆盖 | ✅ | `paths.rs:63-98` Win/macOS/Linux + 单测；音乐文件只读不迁移 | 卸载不删音乐仅静态成立（未实装卸载钩子） |
| DATA-02 settings 原子写+损坏回退 | 🟡 | tmp+fsync+rename；保存前 copy `.bak`；损坏重命名后回默认（`settings.rs:244-248,295-321`） | **Windows 分支先 `remove_file` 再 rename（:316-321），非原子，失败即丢配置**；损坏文件命名 `settings.corrupt.<ts>.json` 而非验收要求的 `.bak` |
| DATA-03 DB 迁移+回滚+备份 | 🟡 | v1-v4 事务；迁移前整库 `.bak`；版本过新拒绝；4 个迁移测试 | 迁移失败不自动还原 `.bak`；无 v1→v2 真实老库样本 |

### 设置 SET

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| SET-01 8 分类+仅显示已实现+定位+即时+重启标注 | 🟡 | 8 分节齐全（`Settings.tsx:17-26`）；左侧定位滚动 | **暴露 gapless/crossfade（明确非目标）**；list_density/reduce_motion/桌面歌词字号颜色可存但无行为；无“需重启”标注 |
| SET-02 主题/音量/设备即时 | 🟡 | `Settings.tsx:83-109` applyTheme/applyOpacity/setAlwaysOnTop；设备切换即时 | 侧栏主题/语言只写 localStorage，不回写 settings.json，便携拷贝后丢失 |
| SET-06 关于页完整 | 🟡 | 版本/os/arch/Tauri/license/Releases 外链（`Settings.tsx:346-379`） | 无构建 commit/hash、无第三方许可清单、无致谢；版本串硬编码 “v0.1.0 · M0”（`i18n.tsx:9`）；应用内不请求已满足 |

### 国际化 I18N

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| I18N-01 三语运行时切换 | 🟡 | `i18n.tsx` 三套 Record 强制键齐全；React 页面无中文残留（语言名除外）；E2E 覆盖切换持久化 | **托盘 6 条中文硬编码**（`tray.rs`）；`index.html lang="zh-CN"` 固定；启动语言只读 `localStorage.biu.lang`（`i18n.tsx:936-939`），忽略 `settings.general.language` |

### 网络 NET

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| NET-04 零遥测/零出站 | ✅ | 无 http/updater 插件、无 fetch/WebSocket；仅点击外链 | 仅静态核查；CSP 为 null；无 24h 抓包报告 |

### 发布 REL

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| REL-01 Actions 产三平台包 | 🟡 | `release.yml` 矩阵 Win NSIS / mac x64+arm64 dmg / Linux appimage+deb+rpm，`v*` 触发 | **仓库 0 个 tag**，workflow 从未按发布路径实跑；产物无成功记录 |
| REL-02 Win 绿色版 | ✅ | `release.yml:97` 打包 exe+portable.flag+WebView2Loader | 版本硬编码；无真机验证 |
| REL-03 NSIS 安装/升级/卸载 | 🟡 | `bundle.targets=all` 含 nsis | 无 NSIS 配置项、无干净机安装/升级/卸载证据；`scripts/smoke-package.sh` 不存在 |
| REL-04 dmg 双架构未签名 | 🟡 | 矩阵含 x64/arm64 dmg | 无法本机验证；README 自承未真机；无 Gatekeeper 说明以外的验证 |
| REL-05 AppImage/deb/rpm | 🟡 | 三格式构建；CI Xvfb X11 冒烟 | Ubuntu 24.04/Wayland 未测；`INSTALL.md` 却声称已验证 |
| REL-06 统一命名 | ❌ | 绿色包 `BiuPlayer_0.1.0_x64-portable.zip`（`release.yml:97,114` 硬编码版本） | **命名缺 `<os>` 段、版本写死**，升级即断；rpm 为 `BiuPlayer-<ver>-1.<arch>` |
| REL-07 SHA256+来源证明 | 🟡 | checksums 作业生成 `SHA256SUMS.txt` | 无 attestation/SBOM/构建来源证明 |
| REL-08 tag→draft→确认→发布+CHANGELOG | 🟡 | `draft: true` + `generate_release_notes` | 正文不含 CHANGELOG；手动 dispatch 不建 Release；无回滚演练、无 tag |
| REL-10 CI 质量门 | 🟡 | fmt/clippy -D warnings/test/build + Linux 冒烟 + E2E | **无 lint/typecheck/test 前端脚本**；Windows CI `--exclude biuplayer` 跳过应用 crate 测试 |

### 质量 QA

| 项 | 状态 | 证据 | 缺口 |
| --- | --- | --- | --- |
| QA-01 日志+诊断包+脱敏 | 🟡 | daily 滚动日志；`diagnostics.rs` zip 含 system/settings/5 份日志/README；一键导出 UI | **无任何脱敏**：dataDir/logDir 绝对路径、settings、日志原样打包；无日志保留上限 |
| QA-02 panic 捕获+转储+下次提示 | 🟡 | panic hook 写 `crash-<ts>.log` + Backtrace；`latest_crash` 前端横幅；可导出 | 无自动化测试；同秒多次 panic 文件名冲突 |
| QA-03 单测+覆盖率 | 🟡 | 43 个 Rust 测试 + 5 E2E | 覆盖率门槛（Arch.md 70/70/60/50）无工具/CI 强制；无前端单测；`biuplayer` crate 测试在本机无法启动 |
| QA-08 文档 | 🟡 | README/INSTALL/RELEASE/CHANGELOG | README 正文陈旧（仍写“M0 验证台”、下一步列已完成项）；无用户手册；P1 标注只在 CHANGELOG 一行 |

## 2. 关键缺口（按影响排序）

### 2.1 发布门禁不成立
`docs/Mvp.md` §7.5 要求 51/51 有验收记录、三平台安装包实跑、S1/S2=0、8h 长稳。现状：0 个 tag、0 个 Release，`docs/reports/` 42 份全是功能快照，**没有验收矩阵、没有隐私抓包、没有 8h 长稳/CPU 报告**。

### 2.2 断点续播未实现（§2.3 使能项）
`queue.rs` 的 session 只存 items/index/mode/muted，不存 position；`Shell.tsx:200-220` 启动只回填队列快照，不自动播放；`resume_on_start` 设置全仓无消费代码。重启后队列在、进度与播放状态丢失。

### 2.3 无边框设置静默丢失（UI-08）
`commands/settings.rs:99` 声明 `patch.frameless`，`settings_update`（:110-235）没有任何 frameless 分支，前端切换后立即生效但永不落盘；`window.ts` 注释“设置持久化在 Rust 侧”与事实不符。

### 2.4 浅色主题不完整（UI-03）
全仓仅 1 条 `[data-theme="light"]` 规则；`App.css:35-39` body 径向渐变写死深色；窗口背景写死 `#0d0f14`；无首帧内联主题脚本，500ms 无白闪无法成立。

### 2.5 settings 写入在 Windows 非原子（DATA-02）
`settings.rs:316-321` 先删后 rename，崩溃窗口内会丢配置；损坏文件命名为 `settings.corrupt.<ts>.json`，与验收要求的 `.bak` 不符。

### 2.6 设置页展示未实现能力（SET-01）
gapless/crossfade 属 §1 非目标却可调；list_density/reduce_motion/桌面歌词字号颜色可存但无行为；无“需重启”标注。这违反“仅显示已实现”。

### 2.7 托盘与语言（I18N-01）
托盘菜单/tooltip 全中文硬编码；语言只存 localStorage 不读 settings，切语言后托盘不变、便携拷贝丢语言。

### 2.8 列表能力缺口（PL-02/PL-05/PL-11）
无系统文件选择器（只能拖放/手输）；排序缺 added/path 两列；播放列表详情页非虚拟化，1 万条会全量渲染。

### 2.9 质量门缺失（REL-10/QA-03）
`pnpm lint/typecheck/test` 三个验收命令直接不存在；覆盖率无强制；Windows 上应用 crate 测试被 CI 排除且本机也因 `STATUS_ENTRYPOINT_NOT_FOUND` 跑不起来。

### 2.10 队列与播放列表的操作语义缺陷
随机模式下 `insert_next` 不重建 shuffle 顺序，“下一首播放”可能不生效（`queue.rs:111-119`）；没有真正的“插播”，只有“插入到下一首”。`PlaylistDetail` 批量删除后不清空选中集，索引错位可误删；undo 要求全量 id 集完全一致，列表有变动即失效。

### 2.11 输出设备设置启动不生效
`lib.rs:181` 启动时用 `AudioPlayerState::default()`，已保存的 `audio.device_id` 不会在建流时应用；用户上次选的声卡重启后被重置为系统默认，需再手动切一次。

### 2.12 文档与实现漂移
`INSTALL.md:9` 说绿色版写 `%APPDATA%`，实现是写 exe 旁 `data/`；`INSTALL.md:72` 声称 24.04 已验证，报告里是未测；README 目录与“下一步”停留在 M0。

## 3. 发布门禁逐条判定

| 门禁 | 判定 |
| --- | --- |
| 51/51 P0 有验收记录 | ❌ 无矩阵/归档，本次审计为 14 ✅ / 35 🟡 / 2 ❌ |
| 7 格式 + 10 万库 + 8h 长稳 | 🟡 7 格式有测试；10 万仅空文件扫描与 DB 直查；8h 长稳无报告 |
| 三平台安装包 | ❌ 无 tag、无产物，仅 workflow 定义 |
| 零出站/隐私说明 | 🟡 静态零出站成立；无抓包证据与隐私说明 |
| S1/S2 = 0 | ❓ 无缺陷分级清单，无法判定 |
| 新人可构建/回滚演练 | ❌ README 可构建；无回滚演练，且 lint/typecheck 命令缺失 |

## 4. 建议修复顺序

1. **补质量门**：加 `typecheck`/`lint`（eslint+prettier）/`test` 脚本并接入 CI；修 `biuplayer` 测试二进制加载失败（或显式说明 Windows 限制）。
2. **修功能 bug**：frameless 持久化；settings Windows 原子写 + `.bak` 命名；Shell 主题切换写 `biuMode` 并与 settings 同步；启动应用已保存输出设备；`insert_next` 重建 shuffle；批删清空选中集。
3. **补断点续播**：session 存 position/playing，启动恢复并受 `resume_on_start` 控制。
4. **收敛设置页**：隐藏/禁用 gapless、crossfade、density、reduceMotion 等未实现项，加“需重启”标注。
5. **补浅色主题**：把 body/窗口背景改为变量，加首帧内联主题脚本。
6. **托盘与语言**：tray 文案走 i18n，语言写入 settings 并在启动时应用。
7. **列表**：加 added/path 排序、原生文件选择器、播放列表虚拟化。
8. **发布工程**：命名去硬编码（版本+os+arch）、Release 正文附 CHANGELOG、打首个 tag 做一次全矩阵实跑。
9. **验收证据**：建立 51 项矩阵归档 + 隐私抓包 + 8h 长稳 + CPU/fps 基准，更新 README/INSTALL。

## 5. 修复后状态（2026-11-25，第二轮）

本轮先处理 §4 第 1–5 项（全部落地并有可复跑证据），随后按用户要求继续第 6–9 项（见 5.2）。发布门禁（§3）仍不成立，原因是没有 tag/产物实跑、性能与长稳数据缺失。

| # | 项 | 状态 | 关键改动 | 证据 |
| --- | --- | --- | --- | --- |
| 1 | 质量门 + `biuplayer` 测试加载 | ✅ | 新增 `pnpm typecheck`（`tsc --noEmit`）、`pnpm lint`（ESLint flat config + typescript-eslint）、`pnpm test`（Vitest，`src/theme.test.ts` 4 用例）；CI 三平台在 build 前执行三步；修复 Windows 测试二进制 `STATUS_ENTRYPOINT_NOT_FOUND`：主/开发 `tauri` 依赖均关默认 feature，去掉 `common-controls-v6`，测试目标不再静态导入缺失的 `comctl32!TaskDialogIndirect`；CI 删除 `--exclude biuplayer` | 见 5.1；`cargo test -p biuplayer --lib` 10 passed |
| 2 | frameless / settings 原子写 / 主题同步 | ✅ | `settings_update` 补 `frameless` 分支；`save_settings` 改 `replace_file`（rename 失败时旁路旧文件并回滚），损坏配置按验收要求备份为 `settings.json.bak`；Shell 主题开关写回 `settings.json` 并广播 `biu:theme`；启动时应用已保存输出设备（同时关闭 §2.11） | `settings.rs` 单测 5 项、`settings_update` 字段测试；手动改设置后重启保留 |
| 3 | 断点续播 | ✅ | `queue.rs` 增 `Progress{path,position_ms,playing}` + `ProgressStore`（路径/播放态变化或 ≥1s 节流落盘）；启动 `restore_last_session` 按队列路径恢复 index、输出设备、seek 与播放态，受 `resume_on_start` 控制 | `progress_store_roundtrip` 通过；`cargo test --workspace` 全绿 |
| 4 | 设置页收敛 | ✅ | 删除未实现的 Gapless、交叉淡化、列表密度、减少动效四个控件；“启动时最小化”加“需重启”徽标（三语） | 设置页 E2E 导航仍通过；`pnpm lint` 0 |
| 5 | 浅色主题 | ✅ | `index.html` 首帧内联脚本按 `biu.themeMode` + 系统偏好设置 `data-theme` 与窗口底色；`App.css` 引入 `--border/--hover/--surface-*/--glow-*/--cover-*` 等语义变量并替换硬编码深色；`shell.css` 浅色块覆盖全部变量；紧凑布局下主题按钮改为图标，避免 800px 宽度竖向折行 | 截图：`docs/reports/assets/mvp-fixes/light-1280.png`、`light-800.png`；`html[data-theme]` 切换由 E2E 断言 |

### 5.1 本轮验证记录

| 命令 | 结果 |
| --- | --- |
| `pnpm typecheck` | 0（无输出） |
| `pnpm lint` | 0 |
| `pnpm test` | 1 file / 4 tests passed |
| `pnpm build` | 0（dist 449.14 kB JS / 26.81 kB CSS） |
| `pnpm test:e2e` | 6/6 passed（11s，Playwright Chromium） |
| `cargo fmt --all --check` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --workspace` | 0，全部通过（62 个测试 + doc-tests，含 `biuplayer` 10 个单元测试） |
| `cargo build --bin biuplayer` | 0 |
| `pnpm check:version` | 0（0.1.0 三处一致） |

环境注意：本机 `%TEMP%` 的 ACL 会让 esbuild/link.exe 失败，验证时把 `TMP/TEMP` 指向 `src-tauri/target/tmp`；这与产品无关，CI 不受影响。

### 5.2 第三轮：§4 第 6–9 项（2026-11-25）

| # | 项 | 状态 | 关键改动 | 证据 / 剩余 |
| --- | --- | --- | --- | --- |
| 6 | 托盘 i18n + 启动语言 | ✅ | `tray.rs` 用 `Labels` 生成三语文案，`apply_language` 在设置变更时重建菜单/tooltip；`i18n.tsx` 启动读 `settings.language` | 三语菜单、切换即生效；语言以 `settings.json` 为真源 |
| 7 | 列表能力 | ✅ | `added_at` 进 `TrackListItem`/DTO/绑定；库列表 7 列排序 + 添加时间/路径列；原生目录选择器（`tauri-plugin-dialog`）；播放列表详情窗口化 | 类型检查/单测通过；10 万行 fps 仍无基准 |
| 8 | 发布工程 | 🟡 | `release.yml` 统一产物名 `BiuPlayer_<v>_<os>_<arch>`、版本从 `package.json` 注入、Release 正文取 CHANGELOG 对应段、tag/版本一致性校验；新增 `pnpm check:version` 并接入 CI | 无 tag、未实跑矩阵 |
| 9 | 验收证据 | 🟡 | 新增 `docs/reports/mvp-acceptance-matrix.md`（51 项：17 ✅ / 33 🟡 / 1 ⛔）、`docs/reports/privacy-egress.md`、`scripts/soak.ps1`；README/INSTALL 产物名与便携数据目录纠偏 | 无抓包、无 8h 实测、无 fps/CPU 数据 |

### 5.3 仍未闭环

- **发布门禁**：0 个 tag、0 个 Release；三平台安装包与绿色版未在干净机实跑。
- **UI-15 性能证据**：10 万行滚动 fps、播放 CPU ≤3%、8 小时长稳均未实测（脚本已备：`powershell -ExecutionPolicy Bypass -File scripts\soak.ps1 -Hours 8`）。
- **QA-01/QA-02/QA-03**：诊断包未脱敏；崩溃恢复无自动化用例；覆盖率工具与阈值未接入 CI。
- **LIB-04 / TAG-01**：无专辑艺术家视图；ID3v1/APEv2/Vorbis/MP4 atoms/WAV-ID3 无 fixture 验证。
- **LRC-01**：当前行误差未测量，NowPlaying 未应用歌词偏移。
- 浅色主题仅静态截图与 `data-theme` 断言，500ms 无白闪需真机冷启动录屏。
- 断点续播的恢复链路（真实音频设备 + 重启）未在真机执行。
