import { useEffect, useState } from "react";
import { commands, type SettingsDto, type SettingsPatch_Deserialize } from "../bindings";
import { useI18n, type Lang } from "../i18n";
import { applyTheme } from "../theme";

export function Settings() {
  const { t, lang, setLang } = useI18n();
  const [settings, setSettings] = useState<SettingsDto | null>(null);
  const [message, setMessage] = useState("");

  useEffect(() => {
    commands
      .settingsGet()
      .then((value) => {
        setSettings(value);
        applyTheme(value.theme);
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

  async function update(patch: SettingsPatch_Deserialize) {
    try {
      setMessage("");
      const next = await commands.settingsUpdate(patch);
      setSettings(next);
      if (patch.theme) applyTheme(String(patch.theme));
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
