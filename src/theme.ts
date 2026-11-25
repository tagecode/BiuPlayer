export type ThemeMode = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

/// Resolves a stored theme mode against the OS preference.
export function resolveTheme(mode: string, prefersDark: boolean): ResolvedTheme {
  if (mode === "light" || mode === "dark") return mode;
  return prefersDark ? "dark" : "light";
}

export function storedThemeMode(): ThemeMode {
  const mode = localStorage.getItem("biu.themeMode");
  return mode === "light" || mode === "dark" || mode === "system" ? mode : "system";
}

export function applyTheme(mode: string) {
  const resolved = resolveTheme(
    mode,
    window.matchMedia("(prefers-color-scheme: dark)").matches,
  );
  document.documentElement.dataset.theme = resolved;
  localStorage.setItem("biu.theme", resolved);
  localStorage.setItem("biu.themeMode", mode);
}

export function initTheme() {
  const mode = storedThemeMode();
  applyTheme(mode);
  if (mode === "system") {
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", () => applyTheme("system"));
  }
}
