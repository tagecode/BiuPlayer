export type ThemeMode = "system" | "light" | "dark";

export function applyTheme(mode: string) {
  const resolved =
    mode === "system"
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light"
      : mode;
  document.documentElement.dataset.theme = resolved;
  localStorage.setItem("biu.theme", resolved);
  localStorage.setItem("biu.themeMode", mode);
}

export function initTheme() {
  const mode = localStorage.getItem("biu.themeMode") ?? "system";
  applyTheme(mode);
  if (mode === "system") {
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", () => applyTheme("system"));
  }
}
