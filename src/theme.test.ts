import { describe, expect, it } from "vitest";

import { resolveTheme } from "./theme";

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
