use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub struct TrayState {
    tray: TrayIcon,
    play: MenuItem<tauri::Wry>,
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示 / 隐藏", true, None::<&str>)?;
    let play = MenuItem::with_id(app, "play_pause", "播放 / 暂停", true, None::<&str>)?;
    let prev = MenuItem::with_id(app, "prev", "上一首", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", "下一首", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出 BiuPlayer", true, None::<&str>)?;
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
    app.manage(TrayState { tray, play });
    Ok(())
}

/// 把播放状态同步到托盘：tooltip 显示当前曲目，播放菜单显示“播放/暂停”。
pub fn sync_state(app: &AppHandle, payload: &str) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) else {
        return;
    };
    let state = value.get("state").and_then(|v| v.as_str()).unwrap_or("");
    let title = value.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let artist = value.get("artist").and_then(|v| v.as_str()).unwrap_or("");
    let tooltip = if title.is_empty() {
        "博音 BiuPlayer".to_string()
    } else if artist.is_empty() {
        format!("{title} - 博音 BiuPlayer")
    } else {
        format!("{title} - {artist}")
    };

    let Some(tray_state) = app.try_state::<TrayState>() else {
        return;
    };
    let _ = tray_state.tray.set_tooltip(Some(tooltip.as_str()));
    let _ = tray_state.play.set_text(if state == "playing" {
        "暂停"
    } else {
        "播放"
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
