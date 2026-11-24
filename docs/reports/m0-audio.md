# M0-01 解码与探针报告

- 日期：2026-11-24
- 环境：macOS 26.5.1（x86_64）、Rust 1.99.1、symphonia 0.5.5
- 结论：**7/7 格式通过**，MVP 的 7 种格式均可完整解码为 f32 交错样本。
- 验证方式：`cargo test -p biu-audio --test formats`；`cargo run -p biu-audio --example probe -- ../fixtures/audio/m0/*`

## 1. 结果

| 文件 | 容器/格式 | 解码器 | 采样率 | 声道 | 时长 | 解码帧数 | 峰值 |
|---|---|---|---|---:|---:|---:|---:|---:|
| tone.mp3 | mp3 | mp3 | 44100 | 2 | 2000 ms | 88200 | -18.3 dBFS |
| tone.flac | flac | flac | 44100 | 2 | 2000 ms | 88200 | -18.1 dBFS |
| tone.wav | wav | pcm | 44100 | 2 | 2000 ms | 88200 | -18.1 dBFS |
| tone-aac.m4a | aac | aac | 44100 | 2 | 2023 ms | 90112 | -14.4 dBFS |
| tone-alac.m4a | alac | alac | 44100 | 2 | 2000 ms | 88200 | -18.1 dBFS |
| tone.ogg | ogg | vorbis | 44100 | 2 | 2000 ms | 88200 | -17.9 dBFS |
| tone.aiff | aiff | pcm | 44100 | 2 | 2000 ms | 88200 | -18.1 dBFS |

夹具为 2 秒双声道正弦（左 440 Hz、右 660 Hz），带中文标题/艺术家/专辑元数据，位于 `fixtures/audio/m0/`。

## 2. 关键发现

1. **AIFF 可用**：symphonia 0.5.5 的 `aiff` feature 走 `symphonia-format-riff`，无需额外解码器；此前文档中的“AIFF 依赖 aiff feature”已在代码中落实。
2. **AAC 有时长偏移**：AAC 解码得到 90112 帧（2023 ms），比源文件多 1024 帧编码器延迟；当前探针未做 gapless trim。正式播放需在 A1-01/A1-04 用 iTunSMPB/编码延迟信息裁剪，M0 不作为失败。
3. **ALAC/AAC 的容器元数据不完整**：isomp4 轨道在 `codec_params` 中可能缺 `channels`/`sample_rate`，实现改为回退到首个解码帧的 `SignalSpec`，避免误判为“无法播放”。
4. **Opus 不在 MVP**：symphonia 0.5 无内置 Opus 解码器；Opus 属 P1 扩展格式，按 Arch 方案走 libopus 绑定或可选组件。
5. **WAV/AIFF 的 codec 名显示为 `pcm/other`**：这是 M0 探针的粗粒度映射，正式解码矩阵会区分 `pcm_s16le`/`pcm_s16be` 等。

## 3. 复现

```bash
cd src-tauri
cargo test -p biu-audio --test formats
cargo run -p biu-audio --example probe -- ../fixtures/audio/m0/*
```

夹具生成命令（ffmpeg 7.1.1）：

```bash
ffmpeg -f lavfi -i "sine=frequency=440:duration=2:sample_rate=44100" \
       -f lavfi -i "sine=frequency=660:duration=2:sample_rate=44100" \
       -filter_complex "[0:a][1:a]amerge=inputs=2[a]" -map "[a]" \
       -c:a pcm_s16le fixtures/audio/m0/tone.wav
ffmpeg -i fixtures/audio/m0/tone.wav -c:a flac fixtures/audio/m0/tone.flac
ffmpeg -i fixtures/audio/m0/tone.wav -c:a libmp3lame -b:a 192k fixtures/audio/m0/tone.mp3
ffmpeg -i fixtures/audio/m0/tone.wav -c:a aac -b:a 192k fixtures/audio/m0/tone-aac.m4a
ffmpeg -i fixtures/audio/m0/tone.wav -c:a alac fixtures/audio/m0/tone-alac.m4a
ffmpeg -i fixtures/audio/m0/tone.wav -c:a libvorbis -q:a 6 fixtures/audio/m0/tone.ogg
ffmpeg -i fixtures/audio/m0/tone.wav -c:a pcm_s16be fixtures/audio/m0/tone.aiff
```

## 4. 未覆盖 / 后续

- 未测 24bit/96kHz、5.1、损坏文件、扩展名与实际格式不符；这些属 A1-01/A1-02。
- 未做 seek、gapless、重采样；M0 仅验证“能完整解码”。
- 未在 Windows/Linux 运行；CI 已加三平台测试，待 Actions 结果补录。
