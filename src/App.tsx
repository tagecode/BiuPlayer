import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { commands } from "./bindings";
import type { AppInfo, AppPathsInfo, AudioDevice, PingReply, TestTick } from "./bindings";
import "./App.css";
import { useI18n } from "./i18n";
import { formatError } from "./errors";

const inTauri = "__TAURI_INTERNALS__" in window;

function App() {
  const { t } = useI18n();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [infoError, setInfoError] = useState("");
  const [paths, setPaths] = useState<AppPathsInfo | null>(null);
  const [diagPath, setDiagPath] = useState("");
  const [diagError, setDiagError] = useState("");
  const [pingInput, setPingInput] = useState("hello biu");
  const [pingReply, setPingReply] = useState<PingReply | null>(null);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [deviceError, setDeviceError] = useState("");
  const [burst, setBurst] = useState(0);
  const [received, setReceived] = useState(0);
  const [missing, setMissing] = useState(0);
  const [eventStatus, setEventStatus] = useState(t("m0.waiting"));
  const expectedRef = useRef(1);
  const receivedRef = useRef(0);

  useEffect(() => {
    if (!inTauri) {
      setInfoError(t("m0.ipcUnavailable"));
      return;
    }
    commands
      .appInfo()
      .then(setInfo)
      .catch((err) => setInfoError(formatError(err, t)));
    commands
      .appPaths()
      .then(setPaths)
      .catch((err) => setInfoError(formatError(err, t)));
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
        echo: t("m0.callFailed", { message: formatError(err, t) }),
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
    setEventStatus(t("m0.sending"));
    try {
      const sent = await commands.emitTestEvents(count);
      setEventStatus(t("m0.sent", { count: sent }));
    } catch (err) {
      setEventStatus(t("m0.sendFailed", { message: formatError(err, t) }));
    }
  }

  async function runDiagnostics() {
    setDiagError("");
    try {
      setDiagPath(await commands.exportDiagnostics());
    } catch (err) {
      setDiagError(formatError(err, t));
    }
  }

  async function loadDevices() {
    setDeviceError("");
    try {
      setDevices(await commands.audioDevices());
    } catch (err) {
      setDeviceError(formatError(err, t));
    }
  }

  const complete = received > 0 && received === burst && missing === 0;

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <span className="brand-mark">Biu</span>
          <div>
            <h1>{t("m0.title")}</h1>
            <p>{t("m0.subtitle")}</p>
          </div>
        </div>
        <div className="chips">
          <span className={`chip ${inTauri ? "ok" : "warn"}`}>
            {inTauri ? t("m0.connected") : t("m0.preview")}
          </span>
          <span className="chip">v{info?.version ?? "0.1.0"}</span>
        </div>
      </header>

      <main className="grid">
        <section className="card span-2">
          <div className="card-head">
            <h2>{t("m0.env")}</h2>
            <span className="muted">app_info</span>
          </div>
          {infoError && <p className="error">{infoError}</p>}
          <dl className="kv">
            <div>
              <dt>{t("m0.app")}</dt>
              <dd>{info?.name ?? "—"}</dd>
            </div>
            <div>
              <dt>Tauri</dt>
              <dd>{info?.tauri_version ?? "—"}</dd>
            </div>
            <div>
              <dt>{t("m0.platform")}</dt>
              <dd>
                {info ? `${info.os} / ${info.arch}` : "—"}
              </dd>
            </div>
            <div className="wide">
              <dt>{t("m0.logDir")}</dt>
              <dd className="mono">{paths?.log_dir ?? info?.log_dir ?? "—"}</dd>
            </div>
            <div className="wide">
              <dt>{t("m0.dataDir")}</dt>
              <dd className="mono">{paths?.data_dir ?? "—"}</dd>
            </div>
            <div>
              <dt>{t("m0.portable")}</dt>
              <dd>{paths ? (paths.portable ? t("m0.enabled") : t("m0.disabled")) : "—"}</dd>
            </div>
          </dl>
          <div className="inline" style={{ marginTop: 12 }}>
            <button onClick={runDiagnostics} disabled={!inTauri}>
              {t("m0.exportDiag")}
            </button>
            <span className="muted mono">{diagPath || t("m0.diagHint")}</span>
          </div>
          {diagError && <p className="error">{diagError}</p>}
        </section>

        <section className="card">
          <div className="card-head">
            <h2>{t("m0.ping")}</h2>
            <span className="muted">app_ping</span>
          </div>
          <div className="inline">
            <input
              value={pingInput}
              onChange={(event) => setPingInput(event.currentTarget.value)}
              placeholder={t("m0.pingPlaceholder")}
            />
            <button onClick={runPing} disabled={!inTauri}>
              {t("m0.send")}
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
            <h2>{t("m0.events")}</h2>
            <span className="muted">test:tick</span>
          </div>
          <p className="hint">
            {t("m0.eventsHint")}
          </p>
          <div className="metrics">
            <div>
              <strong>{received}</strong>
              <span>{t("m0.received")}</span>
            </div>
            <div>
              <strong>{burst}</strong>
              <span>{t("m0.target")}</span>
            </div>
            <div>
              <strong>{missing}</strong>
              <span>{t("m0.missing")}</span>
            </div>
          </div>
          <button className="primary block" onClick={runBurst} disabled={!inTauri}>
            {t("m0.send1000")}
          </button>
          <p className={`status ${complete ? "ok" : ""}`}>
            {complete ? t("m0.pass") : eventStatus}
          </p>
        </section>

        <section className="card span-2">
          <div className="card-head">
            <h2>{t("m0.devices")}</h2>
            <div className="inline">
              <span className="muted">cpal</span>
              <button onClick={loadDevices} disabled={!inTauri}>
                {t("m0.enumerate")}
              </button>
            </div>
          </div>
          {deviceError && <p className="error">{deviceError}</p>}
          {devices.length === 0 ? (
            <p className="hint">{t("m0.devicesHint")}</p>
          ) : (
            <ul className="devices">
              {devices.map((device) => (
                <li key={device.name}>
                  <div>
                    <strong>{device.name}</strong>
                    {device.is_default && <span className="badge">{t("m0.default")}</span>}
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
            <h2>{t("m0.checklist")}</h2>
            <span className="muted">docs/reports</span>
          </div>
          <ul className="checklist">
            <li className="done">
              <span>{t("m0.decode")}</span>
              <p>{t("m0.decodeDesc")}</p>
            </li>
            <li className="done">
              <span>{t("m0.output")}</span>
              <p>{t("m0.outputDesc")}</p>
            </li>
            <li className={complete ? "done" : ""}>
              <span>IPC</span>
              <p>{t("m0.ipcDesc")}</p>
            </li>
            <li className={info?.log_dir ? "done" : ""}>
              <span>{t("m0.log")}</span>
              <p>{t("m0.logDesc")}</p>
            </li>
            <li className="done">
              <span>{t("m0.scan")}</span>
              <p>{t("m0.scanDesc")}</p>
            </li>
          </ul>
        </section>
      </main>

      <footer className="footer">
        <span>{t("m0.footer")}</span>
        <span className="mono">biuplayer 0.1.0</span>
      </footer>
    </div>
  );
}

export default App;
