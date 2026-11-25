import { beforeEach, describe, expect, it, vi } from "vitest";

import { applyTheme, initTheme, resolveTheme, storedThemeMode } from "./theme";

type MediaListener = () => void;

function stubMatchMedia(prefersDark: boolean) {
  const listeners = new Set<MediaListener>();
  const media = {
    matches: prefersDark,
    addEventListener: (_: string, listener: MediaListener) => listeners.add(listener),
    removeEventListener: (_: string, listener: MediaListener) => listeners.delete(listener),
  };
  vi.stubGlobal("matchMedia", () => media);
  return {
    media,
    emit(prefersDarkNow: boolean) {
      media.matches = prefersDarkNow;
      listeners.forEach((listener) => listener());
    },
  };
}

beforeEach(() => {
  localStorage.clear();
  delete document.documentElement.dataset.theme;
  vi.unstubAllGlobals();
});

describe("resolveTheme", () => {
  it("keeps an explicit light theme even when the OS prefers dark", () => {
    expect(resolveTheme("light", true)).toBe("light");
  });

  it("keeps an explicit dark theme even when the OS prefers light", () => {
    expect(resolveTheme("dark", false)).toBe("dark");
  });

  it("follows the OS preference in system mode", () => {
    expect(resolveTheme("system", true)).toBe("dark");
    expect(resolveTheme("system", false)).toBe("light");
  });

  it("treats unknown or missing values as system mode", () => {
    expect(resolveTheme("", true)).toBe("dark");
    expect(resolveTheme("sepia", false)).toBe("light");
  });
});

describe("storedThemeMode", () => {
  it("defaults to system when nothing is stored", () => {
    expect(storedThemeMode()).toBe("system");
  });

  it("returns a stored explicit mode", () => {
    localStorage.setItem("biu.themeMode", "light");
    expect(storedThemeMode()).toBe("light");
  });

  it("falls back to system for invalid values", () => {
    localStorage.setItem("biu.themeMode", "sepia");
    expect(storedThemeMode()).toBe("system");
  });
});

describe("applyTheme", () => {
  it("writes the resolved theme and the raw mode", () => {
    stubMatchMedia(true);
    applyTheme("light");
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(localStorage.getItem("biu.theme")).toBe("light");
    expect(localStorage.getItem("biu.themeMode")).toBe("light");
  });

  it("resolves system against the current OS preference", () => {
    stubMatchMedia(false);
    applyTheme("system");
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(localStorage.getItem("biu.themeMode")).toBe("system");
  });
});

describe("initTheme", () => {
  it("applies the stored mode on boot", () => {
    stubMatchMedia(true);
    localStorage.setItem("biu.themeMode", "light");
    initTheme();
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("re-applies system theme when the OS preference changes", () => {
    const media = stubMatchMedia(true);
    localStorage.setItem("biu.themeMode", "system");
    initTheme();
    expect(document.documentElement.dataset.theme).toBe("dark");

    media.emit(false);
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(localStorage.getItem("biu.theme")).toBe("light");
  });
});
