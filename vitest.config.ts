import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    exclude: ["e2e/**", "node_modules/**"],
    coverage: {
      provider: "v8",
      reporter: ["text", "json-summary", "html"],
      reportsDirectory: "coverage",
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/**/*.test.{ts,tsx}", "src/bindings.ts", "src/main.tsx"],
      // QA-03: the pure modules that carry lyric timing and theming behaviour
      // must stay covered; page components are exercised by Playwright instead.
      thresholds: {
        "src/theme.ts": { lines: 85, functions: 85, statements: 85, branches: 75 },
        "src/lyricsTiming.ts": { lines: 90, functions: 90, statements: 90, branches: 85 },
      },
    },
  },
});
