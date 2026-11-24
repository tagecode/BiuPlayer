# M0-02 音频输出报告

- 日期：2026-11-24
- 环境：macOS 26.5.1（x86_64）、cpal 0.15.3、CoreAudio
- 结论：**默认设备与 44.1k/48k 共享模式播放均成功**；设备枚举可用。
- 范围：仅 macOS 本机；Windows WASAPI、Linux ALSA/PulseAudio 未运行，待 CI/真机补测。

## 1. 设备枚举

`cargo run -p biu-audio --example output_probe -- 44100` 输出：

| 设备 | 默认 | 声道 | 采样率 | 采样格式 |
|---|---|---:|---|---|
| MacBook Pro 扬声器 | 是 | 2 | 44100 / 48000 / 88200 / 96000 | F32 |
| BlackHole 2ch | 否 | 2 | 8000–768000 | F32 |
| Soundflower (2ch) | 否 | 2 | 44100–192000 | F32 |
| Soundflower (64ch) | 否 | 64 | 44100–192000 | F32 |
| ZoomAudioDevice | 否 | 2 | 48000 | F32 |

说明：设备名按系统本地化显示；上表为测试机实际名称，代码不依赖具体名称。

## 2. 播放验证

| 请求采样率 | 实际设备 | 声道 | 格式 | 时长 | 结果 |
|---:|---|---:|---|---:|---|
| 44100 | MacBook Pro 扬声器 | 2 | F32 | 1 s | PLAYED |
| 48000 | MacBook Pro 扬声器 | 2 | F32 | 1 s | PLAYED |

实现要点：

- `play_sine` 以共享模式打开默认/指定设备，优先选择 `F32`，回退 `I16`/`U16`；按设备支持的采样率打开，不支持的请求率由设备返回实际配置。
- 正弦波按交错样本写满一个回调缓冲；回调返回 `Ok`，不阻塞 UI。
- 仅验证“能出声/无错误”，未做位完美、独占模式、ASIO/WASAPI Exclusive、Hog Mode；这些在 A1-05/A1-06。

## 3. 验收口径对照

- [x] 能枚举系统输出设备，标出默认设备与支持采样率。
- [x] 44.1kHz 与 48kHz 正弦可播放，无 cpal 错误。
- [ ] 三平台真实出声：仅 macOS。
- [ ] 波形/录音截图：本机扬声器输出未录制，报告以 `PLAYED` 行与设备信息为证；正式 A1-05 用 BlackHole 回环录音补波形。

## 4. 复现

```bash
cd src-tauri
cargo run -p biu-audio --example output_probe -- 44100
cargo run -p biu-audio --example output_probe -- 48000
```

## 5. 风险

- 无可用输出设备/设备被独占时，当前返回错误但不弹窗；A1-06 做设备丢失与回退。
- 采样率转换尚未接入；非 44.1/48 文件在 M0 不处理。
