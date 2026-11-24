# A1 播放内核进展（A1-01 / A1-02）

日期：2026-11-24
状态：A1-01 完成；A1-02 完成；A1-03/04 未完成

## A1-01 7 格式解码

`src-tauri/crates/biu-audio/src/decode.rs`：

- `probe_path` 全解码为 f32 交错样本，返回采样率/声道/时长/峰值。
- 7/7 夹具：MP3、FLAC、WAV、AAC/M4A、ALAC/M4A、OGG Vorbis、AIFF。
- 测试 `crates/biu-audio/tests/formats.rs` 通过。

## A1-02 探测与容错

新增 `probe_metadata(path)`：只读容器/track 元数据，不整曲解码，用于入队前快速校验。

错误分类：

| 场景 | 结果 |
|---|---|
| 空文件 | `AudioError::Empty` |
| 文本/非音频 | `Unsupported` 或 `Corrupt` |
| 截断容器 | `Corrupt("truncated container")` |
| 扩展名错误 | 按内容嗅探（RIFF/FORM/fLaC/OggS/ID3/ftyp） |
| 正常文件 | 与全解码的采样率/声道/时长一致 |

夹具 `fixtures/audio/bad/`：`empty.mp3`、`text.mp3`、`truncated.flac`、`wrong-ext.mp3`（WAV 内容）。

测试 `crates/biu-audio/tests/probe_errors.rs` 5 个全过。

## A1-03/04 流式播放内核

新增 `crates/biu-audio/src/player.rs`：

- `TrackReader`：symphonia 流式解码，逐包读取并应用 gapless trim；`seek_ms` 使用 `SeekMode::Accurate`。
- `Player`：后台音频线程持有 cpal 共享流，环形缓冲 2 秒；回调只读原子状态 + `try_lock` 缓冲，避免阻塞。
- `PlaybackState`：Idle / Loading / Playing / Paused / Stopped / Error；自动 EOF 停止。
- 支持 `load`、`play`、`pause`、`stop`、`seek_ms`、`set_volume`、`snapshot`；非 44.1k 源用线性相位累加重采样到输出设备采样率。
- 修复 symphonia 0.5.5 对零帧 OGG 包的 `SampleBuffer` panic（`frames == 0` 直接跳过）。

手工冒烟（本机 macOS，cpal 默认输出）：

| 格式 | 首帧状态 | 1 秒后位置 | pause | seek 1000+500ms | stop |
|---|---|---|---|---|---|
| MP3 / FLAC / WAV / ALAC / AIFF | Playing | ~684–696ms | Paused | ~1487–1510ms | Stopped |
| AAC/M4A | Playing | ~673ms | Paused | ~1510ms | Stopped |
| OGG | Playing | ~673ms | Paused | ~1499ms | Stopped |

单元测试：状态码往返、44.1k→48k 重采样帧数、单声道复制到双声道，共 3 个通过。

Tauri 接入：

- `src-tauri/src/commands/player.rs`：`player_open`、`player_play`、`player_pause`、`player_stop`、`player_seek`、`player_set_volume`、`player_snapshot`。
- `AudioPlayerState` 懒创建播放器，应用启动不打开音频设备；无播放器时返回 Idle 快照。
- 后台线程以 4Hz 发 `player:state`，载荷 `PlayerStateDto`（state/path/position_ms/duration_ms/sample_rate/channels/volume/error）。
- `player_open` 校验文件存在与 MVP 扩展名，音频设备错误映射 `E_AUDIO_DEVICE`，参数错误映射 `E_INVALID_ARGUMENT`。
- `src-tauri/tests/ipc.rs` 新增 Idle 快照与缺失文件拒绝 2 个测试，IPC 共 5 个通过；`cargo clippy -D warnings` 通过。

## 未完成

- A1-03：设备切换、热拔插 2 秒回退、独占/位完美未做；当前固定默认输出设备与默认采样率。
- A1-04：音量已支持，但静音/队列未做；前端尚无播放控制 UI（P1 播放页实现）。
- A1 出口尚未关闭；下一步队列与设备回退，再进入 L1 扫描入库。
