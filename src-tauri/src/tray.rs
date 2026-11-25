use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

/// Localized strings for the tray menu (Mvp I18N-01). Rust cannot reach the
/// frontend dictionary, so the three supported languages are mirrored here.
struct Labels {
    show: &'static str,
    play: &'static str,
    pause: &'static str,
    prev: &'static str,
    next: &'static str,
    quit: &'static str,
    app: &'static str,
}

fn labels(language: &str) -> Labels {
    match language {
        "en" => Labels {
            show: "Show / Hide",
            play: "Play",
            pause: "Pause",
            prev: "Previous",
            next: "Next",
            quit: "Quit BiuPlayer",
            app: "BiuPlayer",
        },
        "zh-TW" => Labels {
            show: "顯示 / 隱藏",
            play: "播放",
            pause: "暫停",
            prev: "上一首",
            next: "下一首",
            quit: "結束 BiuPlayer",
            app: "博音 BiuPlayer",
        },
        _ => Labels {
            show: "显示 / 隐藏",
            play: "播放",
            pause: "暂停",
            prev: "上一首",
            next: "下一首",
            quit: "退出 BiuPlayer",
            app: "博音 BiuPlayer",
        },
    }
}

#[derive(Default, Clone)]
struct PlaybackInfo {
    playing: bool,
    title: String,
    artist: String,
}

pub struct TrayState {
    tray: TrayIcon,
    show: MenuItem<tauri::Wry>,
    play: MenuItem<tauri::Wry>,
    prev: MenuItem<tauri::Wry>,
    next: MenuItem<tauri::Wry>,
    quit: MenuItem<tauri::Wry>,
    language: std::sync::Mutex<String>,
    info: std::sync::Mutex<PlaybackInfo>,
}

fn language_of(app: &AppHandle) -> String {
    app.try_state::<crate::commands::settings::SettingsState>()
        .map(|state| {
            let guard = state
                .settings
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            guard.general.language.clone()
        })
        .unwrap_or_else(|| "zh-CN".to_string())
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let language = language_of(app);
    let text = labels(&language);
    let show = MenuItem::with_id(app, "show", text.show, true, None::<&str>)?;
    let play = MenuItem::with_id(app, "play_pause", text.play, true, None::<&str>)?;
    let prev = MenuItem::with_id(app, "prev", text.prev, true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", text.next, true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", text.quit, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &play, &prev, &next, &separator, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => toggle_window(app),
            "quit" => app.exit(0),
            action @ ("play_pause" | "prev" | "next") => {
                let _ = app.emit("tray:action", action);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_window(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    let tray = builder.build(app)?;
    app.manage(TrayState {
        tray,
        show,
        play,
        prev,
        next,
        quit,
        language: std::sync::Mutex::new(language),
        info: std::sync::Mutex::new(PlaybackInfo::default()),
    });
    sync_tooltip(app);
    Ok(())
}

/// Re-labels the tray after the user switches language (Mvp I18N-01).
pub fn apply_language(app: &AppHandle, language: &str) {
    let Some(state) = app.try_state::<TrayState>() else {
        return;
    };
    let text = labels(language);
    let _ = state.show.set_text(text.show);
    let _ = state.prev.set_text(text.prev);
    let _ = state.next.set_text(text.next);
    let _ = state.quit.set_text(text.quit);
    *state
        .language
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = language.to_string();
    sync_tooltip(app);
    let info = state
        .info
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let _ = state
        .play
        .set_text(if info.playing { text.pause } else { text.play });
}

fn sync_tooltip(app: &AppHandle) {
    let Some(state) = app.try_state::<TrayState>() else {
        return;
    };
    let language = state
        .language
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let text = labels(&language);
    let info = state
        .info
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let tooltip = if info.title.is_empty() {
        text.app.to_string()
    } else if info.artist.is_empty() {
        format!("{} - {}", info.title, text.app)
    } else {
        format!("{} - {}", info.title, info.artist)
    };
    let _ = state.tray.set_tooltip(Some(tooltip.as_str()));
}

/// Mirrors playing state and the current track into the tray tooltip/menu.
pub fn sync_state(app: &AppHandle, payload: &str) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) else {
        return;
    };
    let state_name = value.get("state").and_then(|v| v.as_str()).unwrap_or("");
    let title = value.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let artist = value.get("artist").and_then(|v| v.as_str()).unwrap_or("");

    let Some(tray_state) = app.try_state::<TrayState>() else {
        return;
    };
    {
        let mut info = tray_state
            .info
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        info.playing = state_name == "playing";
        info.title = title.to_string();
        info.artist = artist.to_string();
    }
    sync_tooltip(app);

    let language = tray_state
        .language
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    let text = labels(&language);
    let _ = tray_state.play.set_text(if state_name == "playing" {
        text.pause
    } else {
        text.play
    });
}

fn toggle_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    match window.is_visible() {
        Ok(true) => {
            let _ = window.hide();
        }
        _ => {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
