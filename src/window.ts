import { commands } from "./bindings";

/** 优先使用原生窗口透明度；平台不支持时退回 WebView CSS 透明度。 */
export async function applyOpacity(value: number): Promise<boolean> {
  let native = false;
  try {
    native = await commands.setWindowOpacity(value);
  } catch {
    native = false;
  }
  document.documentElement.style.setProperty(
    "--ui-opacity",
    native ? "1" : String(value),
  );
  return native;
}

/** 无边框窗口；设置持久化在 Rust 侧。 */
export async function applyFrameless(value: boolean): Promise<void> {
  try {
    await commands.setWindowFrameless(value);
  } catch {
    // 平台不支持时保持系统边框。
  }
}
