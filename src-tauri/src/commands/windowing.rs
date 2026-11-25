use tauri::WebviewWindow;

/// 应用原生窗口不透明度。返回是否由平台原生实现支持。
///
/// - macOS：`NSWindow.alphaValue`
/// - Windows：`WS_EX_LAYERED` + `SetLayeredWindowAttributes`
/// - Linux：返回 false，由前端 CSS 降级
#[tauri::command]
#[specta::specta]
pub fn set_window_opacity(window: WebviewWindow, opacity: f64) -> Result<bool, String> {
    let opacity = opacity.clamp(0.2, 1.0);

    #[cfg(target_os = "macos")]
    {
        let handle = window.ns_window().map_err(|error| error.to_string())?;
        if handle.is_null() {
            return Ok(false);
        }
        // SAFETY: Tauri 返回的是当前 Window 的 NSWindow 指针，调用发生在 UI 线程命令中。
        // `setAlphaValue:` 是 NSWindow 的公开 KVC setter，参数为 CGFloat(f64)。
        unsafe {
            let ns_window = handle as *mut objc2::runtime::AnyObject;
            let _: () = objc2::msg_send![ns_window, setAlphaValue: opacity];
        }
        Ok(true)
    }

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetLayeredWindowAttributes, SetWindowLongW, GWL_EXSTYLE, LWA_ALPHA,
            WS_EX_LAYERED,
        };
        let hwnd = window.hwnd().map_err(|error| error.to_string())?.0;
        let style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
        unsafe {
            SetWindowLongW(hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED as i32);
        }
        let alpha = (opacity * 255.0).round() as u8;
        let applied = unsafe { SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA) };
        Ok(applied != 0)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = (window, opacity);
        tracing::info!("native window opacity unsupported; falling back to CSS");
        Ok(false)
    }
}

#[tauri::command]
#[specta::specta]
pub fn set_window_frameless(window: WebviewWindow, frameless: bool) -> Result<(), String> {
    window
        .set_decorations(!frameless)
        .map_err(|error| error.to_string())?;
    tracing::info!(frameless, "window decorations updated");
    Ok(())
}

/// 为无边框模式提供窗口控制，避免前端直接依赖平台差异。
#[tauri::command]
#[specta::specta]
pub fn window_minimize(window: WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|error| error.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn window_toggle_maximize(window: WebviewWindow) -> Result<(), String> {
    if window.is_maximized().map_err(|error| error.to_string())? {
        window.unmaximize().map_err(|error| error.to_string())
    } else {
        window.maximize().map_err(|error| error.to_string())
    }
}

#[tauri::command]
#[specta::specta]
pub fn window_close(window: WebviewWindow) -> Result<(), String> {
    window.close().map_err(|error| error.to_string())
}
