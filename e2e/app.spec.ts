import { test, expect } from "@playwright/test";
import { fileURLToPath } from "node:url";

const mockPath = fileURLToPath(new URL("./mock-tauri.js", import.meta.url));

test.beforeEach(async ({ page }) => {
  await page.addInitScript({ path: mockPath });
});

test("shell loads with navigation and idle player", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".player-meta strong")).toHaveText("未在播放");
  await expect(page.getByRole("link", { name: "媒体库" })).toBeVisible();
  await expect(page.getByRole("link", { name: "播放列表" })).toBeVisible();
});

test("library search filters and play starts playback", async ({ page }) => {
  await page.goto("/#/library");
  await expect(page.getByRole("heading", { name: "曲目" })).toBeVisible();
  await expect(page.locator(".track-row")).toHaveCount(3);

  const search = page.getByPlaceholder("搜索标题、艺术家、专辑或拼音");
  await search.fill("晴天");
  await expect(page.locator(".track-table tbody tr")).toHaveCount(1, { timeout: 5_000 });

  await page.locator(".track-table tbody tr").first().getByRole("button", { name: "播放" }).click();
  await expect(page.locator(".player-meta strong")).toHaveText("晴天");
  await expect(page.locator(".player-meta")).toContainText("周杰伦");
});

test("settings section navigation updates URL and highlights", async ({ page }) => {
  await page.goto("/#/settings/general");
  await expect(page.getByRole("heading", { name: "设置" })).toBeVisible();

  await page.getByRole("button", { name: "音频" }).click();
  await expect(page).toHaveURL(/#\/settings\/audio/);
  await expect(page.locator("#settings-audio")).toBeVisible();
  await expect(page.getByRole("button", { name: "音频" })).toHaveClass(/active/);

  await page.getByRole("button", { name: "媒体库" }).click();
  await expect(page).toHaveURL(/#\/settings\/library/);
  await expect(page.locator("#settings-library")).toBeVisible();
});

test("language switch translates shell and persists", async ({ page }) => {
  await page.goto("/#/settings/general");
  await page.locator("select").first().selectOption("en");
  await expect(page.getByRole("link", { name: "Library" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Audio" })).toBeVisible();
});

test("playlist detail shows seeded playlist and tracks", async ({ page }) => {
  await page.goto("/#/playlists");
  await expect(page.getByText("通勤")).toBeVisible();
  await page.getByText("通勤").click();
  await expect(page.locator(".track-table tbody tr")).toHaveCount(2);
});

test("theme toggle flips data-theme and persists the mode", async ({ page }) => {
  await page.goto("/");
  const html = page.locator("html");
  const initial = await html.getAttribute("data-theme");
  await page.locator(".theme-toggle").click();
  await expect(html).toHaveAttribute("data-theme", initial === "light" ? "dark" : "light");
  const mode = await page.evaluate(() => localStorage.getItem("biu.themeMode"));
  expect(mode).toBe(initial === "light" ? "dark" : "light");

  await page.locator(".theme-toggle").click();
  await expect(html).toHaveAttribute("data-theme", initial ?? "dark");
});

test("album artist list opens a filtered detail page", async ({ page }) => {
  await page.goto("/#/library/album-artists");
  const link = page.getByRole("link", { name: /周杰伦/ });
  await expect(link).toBeVisible();
  await expect(link).toContainText("2");
  await link.click();
  await expect(page).toHaveURL(/album-artists/);
  await expect(page.locator(".track-table tbody tr")).toHaveCount(2);
});
