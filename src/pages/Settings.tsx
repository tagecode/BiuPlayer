import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  commands,
  type AppInfo,
  type AudioDevice,
  type SettingsDto,
  type SettingsPatch_Deserialize,
} from "../bindings";
import { useI18n, type Lang } from "../i18n";
import { applyTheme } from "../theme";

export function Settings() {
  const { t, lang, setLang } = useI18n();
  const [settings, setSettings] = useState<SettingsDto | null>(null);
  const [message, setMessage] = useState("");
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [devices, setDevices] = useState<AudioDevice[]>([]);

  useEffect(() => {
    commands.appInfo().then(setAppInfo).catch(() => {});
    commands.audioDevices().then(setDevices).catch(() => {});
  }, []);

  useEffect(() => {
    commands
      .settingsGet()
      .then((value) => {
        setSettings(value);
        applyTheme(value.theme);
        document.documentElement.style.setProperty("--ui-opacity", String(value.opacity ?? 1));
        void getCurrentWindow().setAlwaysOnTop(Boolean(value.alwaysOnTop)).catch(() => {});
      })
      .catch((error) => setMessage(String(error)));
  }, []);

  useEffect(() => {
    if (settings?.theme !== "system") return;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const listener = () => applyTheme("system");
    media.addEventListener("change", listener);
    return () => media.removeEventListener("change", listener);
  }, [settings?.theme]);

  async function selectDevice(device: string) {
    try {
      setMessage("");
      const next = await commands.settingsUpdate({ audioDevice: device });
      setSettings(next);
      await commands.playerSetDevice(device === "default" ? null : device);
      setMessage("输出设备已切换");
    } catch (error) {
      setMessage(String(error));
    }
  }

  async function update(patch: SettingsPatch_Deserialize) {
    try {
      setMessage("");
      const next = await commands.settingsUpdate(patch);
      setSettings(next);
      if (patch.theme) applyTheme(String(patch.theme));
      if (patch.alwaysOnTop !== undefined) {
        await getCurrentWindow().setAlwaysOnTop(Boolean(patch.alwaysOnTop)).catch(() => {});
      }
      if (patch.opacity !== undefined) {
        document.documentElement.style.setProperty("--ui-opacity", String(patch.opacity));
      }
    } catch (error) {
      setMessage(String(error));
    }
  }

  if (!settings) {
    return (
      <section className="page">
        <h1>{t("nav.settings")}</h1>
        <p className="muted">{message || "加载设置…"}</p>
      </section>
    );
  }

  return (
    <section className="page settings-page">
      <header className="page-head">
        <div>
          <p className="page-kicker">P-13</p>
          <h1>{t("nav.settings")}</h1>
          <p className="muted">设置写入本地 settings.json，不联网、不上传。</p>
        </div>
      </header>
      {message && <p className="notice">{message}</p>}

      <div className="settings-grid">
        <div className="settings-card">
          <h3>常规</h3>
          <label className="setting-row">
            <span>语言</span>
            <select
              value={lang}
              onChange={(event) => {
                const value = event.target.value as Lang;
                setLang(value);
                update({ language: value });
              }}
            >
              <option value="zh-CN">简体中文</option>
              <option value="zh-TW">繁體中文</option>
              <option value="en">English</option>
            </select>
          </label>
          <label className="setting-row">
            <span>主题</span>
            <select value={settings.theme} onChange={(event) => update({ theme: event.target.value })}>
              <option value="system">跟随系统</option>
              <option value="dark">深色</option>
              <option value="light">浅色</option>
            </select>
          </label>
          <label className="setting-row">
            <span>启动时最小化到托盘</span>
            <input
              type="checkbox"
              checked={settings.startMinimized}
              onChange={(event) => update({ startMinimized: event.target.checked })}
            />
          </label>
        </div>

        <div className="settings-card">
          <h3>音频</h3>
          <label className="setting-row">
            <span>输出设备</span>
            <select
              value={settings.audioDevice || "default"}
              onChange={(event) => selectDevice(event.target.value)}
            >
              <option value="default">系统默认</option>
              {devices.map((device) => (
                <option key={device.name} value={device.name}>
                  {device.name}
                  {device.is_default ? "（默认）" : ""}
                </option>
              ))}
            </select>
          </label>
          <p className="muted">
            切换后立即生效；设备被拔出时播放器会自动回退到系统默认设备。
          </p>
          <p className="muted">位完美/独占输出属于 P1，当前使用系统共享模式。</p>
        </div>

        <div className="settings-card">
          <h3>播放</h3>
          <label className="setting-row">
            <span>启动时恢复上次播放</span>
            <input
              type="checkbox"
              checked={settings.resumeOnStart}
              onChange={(event) => update({ resumeOnStart: event.target.checked })}
            />
          </label>
          <label className="setting-row">
            <span>无缝播放</span>
            <input
              type="checkbox"
              checked={settings.gapless}
              onChange={(event) => update({ gapless: event.target.checked })}
            />
          </label>
          <label className="setting-row">
            <span>淡入淡出（毫秒）</span>
            <input
              type="number"
              min={0}
              max={10000}
              step={100}
              value={settings.crossfadeMs ?? 0}
              onChange={(event) => update({ crossfadeMs: Number(event.target.value) })}
            />
          </label>
          <label className="setting-row">
            <span>默认播放模式</span>
            <select value={settings.mode} onChange={(event) => update({ mode: event.target.value })}>
              <option value="sequential">顺序</option>
              <option value="list_loop">列表循环</option>
              <option value="single_loop">单曲循环</option>
              <option value="shuffle">随机</option>
            </select>
          </label>
        </div>

        <div className="settings-card">
          <h3>歌词</h3>
          <label className="setting-row">
            <span>全局偏移（毫秒）</span>
            <input
              type="number"
              min={-10000}
              max={10000}
              step={50}
              value={settings.lyricsOffsetMs ?? 0}
              onChange={(event) => update({ lyricsOffsetMs: Number(event.target.value) })}
            />
          </label>
          <label className="setting-row">
            <span>桌面歌词字号</span>
            <input
              type="number"
              min={16}
              max={96}
              value={settings.desktopLyricsFontSize ?? 32}
              onChange={(event) => update({ desktopLyricsFontSize: Number(event.target.value) })}
            />
          </label>
          <label className="setting-row">
            <span>桌面歌词颜色</span>
            <input
              type="color"
              value={settings.desktopLyricsColor ?? "#f5f7fa"}
              onChange={(event) => update({ desktopLyricsColor: event.target.value })}
            />
          </label>
        </div>

        <div className="settings-card">
          <h3>界面</h3>
          <label className="setting-row">
            <span>列表密度</span>
            <select
              value={settings.listDensity}
              onChange={(event) => update({ listDensity: event.target.value })}
            >
              <option value="comfortable">舒适</option>
              <option value="compact">紧凑</option>
            </select>
          </label>
          <label className="setting-row">
            <span>减少动效</span>
            <input
              type="checkbox"
              checked={settings.reduceMotion}
              onChange={(event) => update({ reduceMotion: event.target.checked })}
            />
          </label>
          <label className="setting-row">
            <span>窗口置顶</span>
            <input
              type="checkbox"
              checked={settings.alwaysOnTop}
              onChange={(event) => update({ alwaysOnTop: event.target.checked })}
            />
          </label>
          <label className="setting-row">
            <span>界面不透明度</span>
            <input
              type="range"
              min="0.7"
              max="1"
              step="0.05"
              value={settings.opacity ?? 1}
              onChange={(event) => update({ opacity: Number(event.target.value) })}
            />
          </label>
        </div>

        <div className="settings-card">
          <h3>关于</h3>
          <p>
            博音 BiuPlayer {appInfo?.version ?? ""}
          </p>
          <p className="muted">
            {appInfo ? `${appInfo.os} / ${appInfo.arch} · Tauri ${appInfo.tauri_version}` : ""}
          </p>
          <p className="muted">MIT License · 不联网、不采集遥测、不要求账号。</p>
          <div className="setting-row">
            <button
              type="button"
              className="ghost"
              onClick={() =>
                openUrl("https://github.com/tagecode/BiuPlayer/releases").catch((error) =>
                  setMessage(String(error)),
                )
              }
            >
              检查更新
            </button>
            <button
              type="button"
              className="ghost"
              onClick={() =>
                openUrl("https://github.com/tagecode/BiuPlayer/blob/main/LICENSE").catch(
                  (error) => setMessage(String(error)),
                )
              }
            >
              开源许可
            </button>
          </div>
        </div>

        <div className="settings-card">
          <h3>数据</h3>
          <p className="muted">
            设置保存到 settings.json；诊断包只包含设置、最近日志和系统信息，不含音乐与曲库数据库。
          </p>
          <p className="muted mono">schema v{settings.schemaVersion}</p>
        </div>
      </div>
    </section>
  );
}
