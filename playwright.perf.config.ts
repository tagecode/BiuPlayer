import { defineConfig } from "@playwright/test";

// UI-15 benchmark config: separate from `playwright.config.ts` so the regular
// E2E suite stays fast and deterministic. Run with `pnpm test:perf`.
export default defineConfig({
  testDir: "./e2e",
  testMatch: /perf\.spec\.ts/,
  timeout: 120_000,
  expect: { timeout: 10_000 },
  fullyParallel: false,
  reporter: [["list"]],
  use: {
    baseURL: "http://127.0.0.1:4173",
    headless: true,
    viewport: { width: 1280, height: 800 },
  },
  webServer: {
    // Measure the production bundle: the dev server ships React dev mode and
    // double-renders under StrictMode, which would skew the fps numbers.
    command: "pnpm exec vite preview --host 127.0.0.1 --port 4173 --strictPort",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
