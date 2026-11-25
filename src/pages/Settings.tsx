import { useEffect, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { applyFrameless, applyOpacity } from "../window";
import {
  commands,
  type AppInfo,
  type AudioDevice,
  type SettingsDto,
  type SettingsPatch_Deserialize,
} from "../bindings";
import { useI18n, type Lang } from "../i18n";
import { applyTheme } from "../theme";
import { formatError } from "../errors";

export function Settings() {
  const { t, lang, setLang } = useI18n();
  const [settings, setSettings] = useState<SettingsDto | null>(null);
  const [message, setMessage] = useState("");
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [devices, setDevices] = useState<AudioDevice[]>([]);
  const [excludeDraft, setExcludeDraft] = useState("");

  useEffect(() => {
    commands.appInfo().then(setAppInfo).catch(() => {});
    commands.audioDevices().then(setDevices).catch(() => {});
  }, []);

  useEffect(() => {
    commands
      .settingsGet()
      .then((value) => {
        setSettings(value);
        setExcludeDraft((value.libraryExcludes ?? []).join("\n"));
        applyTheme(value.theme);
        void applyOpacity(value.opacity ?? 1);
        void applyFrameless(Boolean(value.frameless));
        void getCurrentWindow().setAlwaysOnTop(Boolean(value.alwaysOnTop)).catch(() => {});
      })
      .catch((error) => setMessage(formatError(error, t)));
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
      setMessage(t("settings.deviceChanged"));
    } catch (error) {
      setMessage(formatError(error, t));
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
        void applyOpacity(patch.opacity ?? 1);
      }
      if (patch.frameless !== undefined) {
        void applyFrameless(Boolean(patch.frameless));
        window.dispatchEvent(
          new CustomEvent("biu:frameless", { detail: Boolean(patch.frameless) }),
        );
      }
      if (patch.sidebarCollapsed !== undefined) {
        window.dispatchEvent(
          new CustomEvent("biu:sidebar-collapsed", { detail: Boolean(patch.sidebarCollapsed) }),
        );
      }
    } catch (error) {
      setMessage(formatError(error, t));
    }
  }

  if (!settings) {
    return (
      <section className="page">
        <h1>{t("nav.settings")}</h1>
        <p className="muted">{message || t("settings.loading")}</p>
      </section>
    );
  }

  return (
    <section className="page settings-page">
      <header className="page-head">
        <div>
          <p className="page-kicker">P-13</p>
          <h1>{t("nav.settings")}</h1>
          <p className="muted">{t("settings.savedNote")}</p>
        </div>
      </header>
      {message && <p className="notice">{message}</p>}

      <div className="settings-grid">
        <div className="settings-card">
          <h3>{t("settings.section.general")}</h3>
          <label className="setting-row">
            <span>{t("settings.language")}</span>
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
            <span>{t("settings.theme")}</span>
            <select value={settings.theme} onChange={(event) => update({ theme: event.target.value })}>
              <option value="system">{t("settings.theme.system")}</option>
              <option value="dark">{t("settings.theme.dark")}</option>
              <option value="light">{t("settings.theme.light")}</option>
            </select>
          </label>
          <label className="setting-row">
            <span>{t("settings.startMinimized")}</span>
            <input
              type="checkbox"
              checked={settings.startMinimized}
              onChange={(event) => update({ startMinimized: event.target.checked })}
            />
          </label>
        </div>

        <div className="settings-card">
          <h3>{t("settings.section.audio")}</h3>
          <label className="setting-row">
            <span>{t("settings.outputDevice")}</span>
            <select
              value={settings.audioDevice || "default"}
              onChange={(event) => selectDevice(event.target.value)}
            >
              <option value="default">{t("settings.defaultDevice")}</option>
              {devices.map((device) => (
                <option key={device.name} value={device.name}>
                  {device.name}
                  {device.is_default ? t("settings.defaultTag") : ""}
                </option>
              ))}
            </select>
          </label>
          <p className="muted">
            {t("settings.deviceHint")}
          </p>
          <p className="muted">{t("settings.bitPerfectHint")}</p>
        </div>

        <div className="settings-card">
          <h3>{t("settings.section.playback")}</h3>
          <label className="setting-row">
            <span>{t("settings.resumeOnStart")}</span>
            <input
              type="checkbox"
              checked={settings.resumeOnStart}
              onChange={(event) => update({ resumeOnStart: event.target.checked })}
            />
          </label>
          <label className="setting-row">
            <span>{t("settings.gapless")}</span>
            <input
              type="checkbox"
              checked={settings.gapless}
              onChange={(event) => update({ gapless: event.target.checked })}
            />
          </label>
          <label className="setting-row">
            <span>{t("settings.crossfade")}</span>
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
            <span>{t("settings.defaultMode")}</span>
            <select value={settings.mode} onChange={(event) => update({ mode: event.target.value })}>
              <option value="sequential">{t("settings.mode.sequential")}</option>
              <option value="list_loop">{t("settings.mode.listLoop")}</option>
              <option value="single_loop">{t("settings.mode.singleLoop")}</option>
              <option value="shuffle">{t("settings.mode.shuffle")}</option>
            </select>
          </label>
        </div>

        <div className="settings-card">
          <h3>{t("settings.section.lyrics")}</h3>
          <label className="setting-row">
            <span>{t("settings.lyricsOffset")}</span>
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
            <span>{t("settings.desktopLyricsSize")}</span>
            <input
              type="number"
              min={16}
              max={96}
              value={settings.desktopLyricsFontSize ?? 32}
              onChange={(event) => update({ desktopLyricsFontSize: Number(event.target.value) })}
            />
          </label>
          <label className="setting-row">
            <span>{t("settings.desktopLyricsColor")}</span>
            <input
              type="color"
              value={settings.desktopLyricsColor ?? "#f5f7fa"}
              onChange={(event) => update({ desktopLyricsColor: event.target.value })}
            />
          </label>
        </div>

        <div className="settings-card">
          <h3>{t("settings.section.interface")}</h3>
          <label className="setting-row">
            <span>{t("settings.density")}</span>
            <select
              value={settings.listDensity}
              onChange={(event) => update({ listDensity: event.target.value })}
            >
              <option value="comfortable">{t("settings.density.comfortable")}</option>
              <option value="compact">{t("settings.density.compact")}</option>
            </select>
          </label>
          <label className="setting-row">
            <span>{t("settings.sidebarCollapsed")}</span>
            <input
              type="checkbox"
              checked={settings.sidebarCollapsed}
              onChange={(event) => update({ sidebarCollapsed: event.target.checked })}
            />
          </label>
          <label className="setting-row">
            <span>{t("settings.reduceMotion")}</span>
            <input
              type="checkbox"
              checked={settings.reduceMotion}
              onChange={(event) => update({ reduceMotion: event.target.checked })}
            />
          </label>
          <label className="setting-row">
            <span>{t("settings.alwaysOnTop")}</span>
            <input
              type="checkbox"
              checked={settings.alwaysOnTop}
              onChange={(event) => update({ alwaysOnTop: event.target.checked })}
            />
          </label>
          <label className="setting-row">
            <span>{t("settings.opacity")}</span>
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
          <h3>{t("settings.section.about")}</h3>
          <p>
            {t("app.name")} BiuPlayer {appInfo?.version ?? ""}
          </p>
          <p className="muted">
            {appInfo ? `${appInfo.os} / ${appInfo.arch} · Tauri ${appInfo.tauri_version}` : ""}
          </p>
          <p className="muted">{t("settings.licenseLine")}</p>
          <div className="setting-row">
            <button
              type="button"
              className="ghost"
              onClick={() =>
                openUrl("https://github.com/tagecode/BiuPlayer/releases").catch((error) =>
                  setMessage(formatError(error, t)),
                )
              }
            >
              {t("settings.checkUpdates")}
            </button>
            <button
              type="button"
              className="ghost"
              onClick={() =>
                openUrl("https://github.com/tagecode/BiuPlayer/blob/main/LICENSE").catch(
                  (error) => setMessage(formatError(error, t)),
                )
              }
            >
              {t("settings.openLicense")}
            </button>
          </div>
        </div>

        <div className="settings-card">
          <h3>{t("settings.section.library")}</h3>
          <p className="muted">{t("settings.libraryExcludeHint")}</p>
          <textarea
            className="settings-textarea"
            rows={4}
            value={excludeDraft}
            placeholder={"**/@eaDir/**\n/path/to/skip"}
            onChange={(event) => setExcludeDraft(event.target.value)}
            onBlur={() => update({ libraryExcludes: excludeDraft.split(/\r?\n/) })}
          />
          <p className="muted">{t("settings.libraryExcludeNote")}</p>
        </div>

        <div className="settings-card">
          <h3>{t("settings.section.data")}</h3>
          <p className="muted">
            {t("settings.dataNote")}
          </p>
          <p className="muted mono">{t("settings.schema")}{settings.schemaVersion}</p>
        </div>
      </div>
    </section>
  );
}
