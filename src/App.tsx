import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { commands } from "./bindings";
import type { AppInfo, AudioDevice, PingReply, TestTick } from "./bindings";
import "./App.css";

const inTauri = "__TAURI_INTERNALS__" in window;

function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [infoError, setInfoError] = useState("");
  const [pingInput, setPingInput] = useState("hello biu");
  const [pingReply, setPingReply] = useState<PingReply | null>(null);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [deviceError, setDeviceError] = useState("");
  const [burst, setBurst] = useState(0);
  const [received, setReceived] = useState(0);
  const [missing, setMissing] = useState(0);
  const [eventStatus, setEventStatus] = useState("等待测试");
  const expectedRef = useRef(1);
  const receivedRef = useRef(0);

  useEffect(() => {
    if (!inTauri) {
      setInfoError("浏览器预览模式：IPC 不可用");
      return;
    }
    commands
      .appInfo()
      .then(setInfo)
      .catch((err) => setInfoError(String(err)));
  }, []);

  useEffect(() => {
    if (!inTauri) return;
    let unlisten: UnlistenFn | undefined;
    listen<TestTick>("test:tick", (event) => {
      receivedRef.current += 1;
      setReceived(receivedRef.current);
      if (event.payload.seq !== expectedRef.current) {
        setMissing((value) => value + 1);
      }
      expectedRef.current = event.payload.seq + 1;
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  async function runPing() {
    try {
      setPingReply(await commands.appPing(pingInput));
    } catch (err) {
      setPingReply({
        payload: pingInput,
        echo: `调用失败: ${String(err)}`,
        server_time_ms: null,
      });
    }
  }

  async function runBurst() {
    const count = 1000;
    receivedRef.current = 0;
    expectedRef.current = 1;
    setReceived(0);
    setMissing(0);
    setBurst(count);
    setEventStatus("发送中…");
    try {
      const sent = await commands.emitTestEvents(count);
      setEventStatus(`已发送 ${sent} 个事件，等待接收`);
    } catch (err) {
      setEventStatus(`发送失败: ${String(err)}`);
    }
  }

  async function loadDevices() {
    setDeviceError("");
    try {
      setDevices(await commands.audioDevices());
    } catch (err) {
      setDeviceError(String(err));
    }
  }

  const complete = received > 0 && received === burst && missing === 0;

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="brand-mark">Biu</span>
          <div>
            <h1>博音 BiuPlayer</h1>
            <p>M0 技术验证台 · 本地播放器</p>
          </div>
        </div>
        <div className="chips">
          <span className={`chip ${inTauri ? "ok" : "warn"}`}>
            {inTauri ? "Tauri 已连接" : "浏览器预览"}
          </span>
          <span className="chip">v{info?.version ?? "0.1.0"}</span>
        </div>
      </header>

      <main className="grid">
        <section className="card span-2">
          <div className="card-head">
            <h2>运行环境</h2>
            <span className="muted">app_info</span>
          </div>
          {infoError && <p className="error">{infoError}</p>}
          <dl className="kv">
            <div>
              <dt>应用</dt>
              <dd>{info?.name ?? "—"}</dd>
            </div>
            <div>
              <dt>Tauri</dt>
              <dd>{info?.tauri_version ?? "—"}</dd>
            </div>
            <div>
              <dt>平台</dt>
              <dd>
                {info ? `${info.os} / ${info.arch}` : "—"}
              </dd>
            </div>
            <div className="wide">
              <dt>日志目录</dt>
              <dd className="mono">{info?.log_dir ?? "—"}</dd>
            </div>
          </dl>
        </section>

        <section className="card">
          <div className="card-head">
            <h2>IPC 往返</h2>
            <span className="muted">app_ping</span>
          </div>
          <div className="inline">
            <input
              value={pingInput}
              onChange={(event) => setPingInput(event.currentTarget.value)}
              placeholder="输入一段文本"
            />
            <button onClick={runPing} disabled={!inTauri}>
              发送
            </button>
          </div>
          <div className="result">
            <span className="label">echo</span>
            <code>{pingReply?.echo ?? "—"}</code>
          </div>
          <div className="result">
            <span className="label">server_time</span>
            <code>{pingReply?.server_time_ms ?? "—"}</code>
          </div>
        </section>

        <section className="card">
          <div className="card-head">
            <h2>事件通道</h2>
            <span className="muted">test:tick</span>
          </div>
          <p className="hint">
            发送 1000 个事件并校验 seq 是否连续；用于验证事件不丢失、不乱序。
          </p>
          <div className="metrics">
            <div>
              <strong>{received}</strong>
              <span>已接收</span>
            </div>
            <div>
              <strong>{burst}</strong>
              <span>目标</span>
            </div>
            <div>
              <strong>{missing}</strong>
              <span>乱序/缺口</span>
            </div>
          </div>
          <button className="primary block" onClick={runBurst} disabled={!inTauri}>
            发送 1000 事件
          </button>
          <p className={`status ${complete ? "ok" : ""}`}>
            {complete ? "通过：1000/1000，无缺失" : eventStatus}
          </p>
        </section>

        <section className="card span-2">
          <div className="card-head">
            <h2>音频输出设备</h2>
            <div className="inline">
              <span className="muted">cpal</span>
              <button onClick={loadDevices} disabled={!inTauri}>
                枚举
              </button>
            </div>
          </div>
          {deviceError && <p className="error">{deviceError}</p>}
          {devices.length === 0 ? (
            <p className="hint">点击“枚举”读取系统输出设备与支持的采样率。</p>
          ) : (
            <ul className="devices">
              {devices.map((device) => (
                <li key={device.name}>
                  <div>
                    <strong>{device.name}</strong>
                    {device.is_default && <span className="badge">默认</span>}
                  </div>
                  <span className="muted">
                    {device.channels}ch · {device.sample_formats.join("/")} ·{" "}
                    {device.sample_rates.join(" / ")} Hz
                  </span>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="card span-2">
          <div className="card-head">
            <h2>M0 验证清单</h2>
            <span className="muted">docs/reports</span>
          </div>
          <ul className="checklist">
            <li className="done">
              <span>解码</span>
              <p>symphonia 7/7：MP3、FLAC、WAV、AAC、ALAC、OGG、AIFF</p>
            </li>
            <li className="done">
              <span>输出</span>
              <p>cpal 44.1k / 48k 正弦波，默认设备与设备枚举</p>
            </li>
            <li className={complete ? "done" : ""}>
              <span>IPC</span>
              <p>command 往返、类型绑定、1000 事件连续性</p>
            </li>
            <li className={info?.log_dir ? "done" : ""}>
              <span>日志</span>
              <p>tracing 落盘到应用日志目录</p>
            </li>
            <li className="done">
              <span>扫描</span>
              <p>SQLite 建表、FTS5 查询、10k 文件扫描基准</p>
            </li>
          </ul>
        </section>
      </main>

      <footer className="footer">
        <span>M0-01 / M0-02 / M0-03 / M0-04 验证台</span>
        <span className="mono">biuplayer 0.1.0</span>
      </footer>
    </div>
  );
}

export default App;
