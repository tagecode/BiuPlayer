# Windows 真实音频输出冒烟

日期：2026-11-27（本机 Windows 11，默认输出设备：扬声器 Realtek）
范围：AUD-01 解码/输出链路、AUD-04 设备枚举的最低限度运行时验证。

## 做法

新增 `src-tauri/crates/biu-audio/tests/output_smoke.rs`，用 `#[ignore]` 标记，
只在有可用声卡的机器上手动执行：

```powershell
cd src-tauri
cargo test -p biu-audio --test output_smoke -- --ignored --nocapture
```

断言：

1. `list_output_devices()` 至少返回一个设备，且有且仅有一个 `is_default`；
2. `play_sine(None, 48_000, 0.3, 440.0)` 在默认设备上成功建立并播放流；
3. 返回的 `sample_rate >= 8_000`、`channels >= 1`、`seconds ≈ 0.3`。

## 结果与修复

首次运行失败：

```text
play sine on default device: Output("unsupported sample format: U8")
```

根因是 `crates/biu-audio/src/output.rs` 的 `select_config`：它取“第一个双声道且
覆盖请求采样率的配置”，而本机设备把 `U8` 配置排在最前，`play_sine` 只实现了
`F32/I16/U16` 三种写入器，于是直接报错。

修复：选择配置时只接受 `F32/I16/U16`，并按 `F32 > I16 > U16` 优先级取最优；
`default_output_config()` 回退路径也会检查格式，无法支持时给出明确错误。

修复后：`test result: ok. 1 passed`。

## 局限

- 该测试只能证明“流建立并播放”，无法证明扬声器实际发声；听感验证仍需人工。
- 设备拔插/切换（AUD-04 的热插拔回退）依赖真实硬件事件，未自动化；
  应用内已有 `device_lost` 检测与默认设备重建逻辑（`commands/player.rs`），
  本次未在真机上断开设备验证。
- 独占模式（bit-perfect）属 P1，不在本测试范围。
