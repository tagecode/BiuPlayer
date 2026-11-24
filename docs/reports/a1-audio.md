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

## 未完成

- A1-03：`play_sine`/设备枚举已有；尚未实现“文件 → 解码 → cpal 流”的流式输出、设备切换、拔卡回退。
- A1-04：播放状态机、seek、音量/静音、队列、`player:state` 事件均未实现。
- 因此 A1 出口（7 格式可播、控制/队列可用）尚未达成；下一步实现流式 `TrackReader` + `Player`。
