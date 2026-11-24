# 音频输出设备切换

## 实现

- `biu-audio::Player::with_device(Option<&str>)`：按 cpal 设备名匹配输出设备，名称大小写不敏感；找不到时回退系统默认设备。
- `Player::new()` 保持为使用默认设备的兼容入口。
- `AudioPlayerState` 增加 `device: Mutex<Option<String>>`；播放器首次创建时读取该设置。
- 新增 `player_set_device(device)` 命令：
  - 保存设备名；
  - 丢弃旧播放器；
  - 若当前有曲目，按新设备重建播放器，恢复路径、位置和播放/暂停状态；
  - 无曲目时保持懒创建，下次播放生效。
- `settings.audio.device_id` 通过 `SettingsDto.audioDevice` / `SettingsPatch.audioDevice` 暴露。
- 设置页“音频”卡片提供“系统默认 + 设备列表”下拉框；切换后调用 `playerSetDevice`。

## 热插拔

- 已有 `device_lost` 标记与 2 秒重建逻辑；重建时使用当前配置设备名，设备不存在会自动回退默认设备。
- 配置设备被拔出后不会自动改回 `device_id`，但播放会回退默认设备；下次播放仍尝试配置设备。

## 验证

- `cargo test --workspace`：51 个测试通过；clippy/fmt 通过。
- `pnpm build` 通过：JS 381.73 kB / gzip 118.61 kB。
- 未做真实设备切换/拔出的人工验收。

## 限制

- 位完美/独占模式未实现，仍为共享模式。
- 设备名来自 cpal；同名设备可能无法区分。
- 切换设备会重建音频流，当前曲目会有短暂中断。
