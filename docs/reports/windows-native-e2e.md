# Windows native WebDriver E2E

Date: 2026-11-26
Scope: Windows 11, WebView2 158.0.4370.33, debug build (`cargo build --bin biuplayer`)

The Playwright suite (`pnpm test:e2e`) runs the frontend in Chromium with mocked
Tauri IPC. It cannot see the real window, the real WebView2 runtime, the native
menu bar, or app startup. This native smoke test closes that gap for the window
layer.

## What it does

`scripts/native-e2e.mjs` (no extra npm dependencies):

1. Copies `src-tauri/target/debug/biuplayer.exe` into `.native-e2e/run/` and
   writes an empty `portable.flag`, so the test profile and library stay inside
   the run directory instead of `%APPDATA%`.
2. Starts `vite` on port 1420 when it is not already running. Debug builds load
   the frontend from `devUrl`, so the dev server is required.
3. Starts `tauri-driver` with `tools/native/msedgedriver.exe` (EdgeDriver major
   version must match the installed WebView2 runtime).
4. Creates a W3C session with the real executable, then asserts:
   - the session is created and `.shell` is rendered by WebView2;
   - `document.title` contains `BiuPlayer`;
   - the player bar shows a track title (`未在播放` when idle);
   - the document does not overflow horizontally at the default 960×680 window;
   - clicking `.theme-toggle` flips `data-theme`;
   - the sidebar link navigates to `#/library` and the page heading renders.
5. Saves `.native-e2e/window.png` and `.native-e2e/checks.json`, then terminates
   the app, driver and dev server.

## Result

```
ok   session created
ok   native window renders .shell
ok   document title — 博音 BiuPlayer
ok   player bar shows idle state — 未在播放
ok   no horizontal overflow at default size — inner=960 scroll=960
ok   theme toggle switches data-theme — light -> dark
ok   navigates to library route — #/library
ok   library page heading present — 媒体库
native e2e: 8 checks passed
```

Screenshot: [window.png](assets/native-e2e/window.png)

The first run failed the overflow check (`scrollWidth 1049 > 960`): the player
bar grid demanded 160 + 220 px minimum columns plus the mode/volume controls, so
it pushed the window content past the viewport. Fixed in `src/shell.css`
(`.playerbar` now uses `minmax(0, 1.1fr) auto minmax(110px, 1fr) auto`, smaller
gap/padding, 64 px volume slider); `e2e/app.spec.ts` gained a 960×680 viewport
regression test so CI catches the same issue in Chromium.

## Prerequisites (not installed by the repo)

```powershell
cargo install tauri-driver --locked
# Download the driver matching the WebView2 runtime major version:
# https://msedgedriver.microsoft.com/<version>/edgedriver_win64.zip
# Extract msedgedriver.exe to tools/native/msedgedriver.exe
pnpm test:e2e:native
```

`tools/native/` and `.native-e2e/` are git-ignored; the driver is a platform
binary, not a product dependency.

## Limits

- Windows-only: `tauri-driver` requires EdgeDriver; Linux uses WebKitWebDriver.
- Covers window/WebView/startup only. Tray menus, file dialogs, drag-and-drop,
  real audio output and device hot-plug still need manual or OS-level checks.
- Not wired into CI yet: the runner would need EdgeDriver plus a WebView2
  runtime version match on every run.
