import { test, expect } from "@playwright/test";
import { fileURLToPath } from "node:url";

const mockPath = fileURLToPath(new URL("./mock-tauri.js", import.meta.url));

/** UI-15: the library grid must stay at 55fps or better while scrolling a
 * 100k-track library. The IPC layer is stubbed so the measurement only covers
 * rendering/virtualisation, which is what the requirement is about. */
test("library scroll holds 55fps with 100k tracks", async ({ page }) => {
  await page.addInitScript({ path: mockPath });
  await page.addInitScript(() => {
    const total = 100_000;
    const make = (index: number) => ({
      id: index + 1,
      title: `Track ${index + 1}`,
      artist: `Artist ${index % 50}`,
      album: `Album ${index % 500}`,
      duration_ms: 180_000 + index,
      format: "MP3",
      path: `/music/${index}.mp3`,
      added_at: null,
    });
    type Internals = {
      invoke: (command: string, args?: unknown, options?: unknown) => Promise<unknown>;
    };
    const internals = (window as unknown as { __TAURI_INTERNALS__: Internals })
      .__TAURI_INTERNALS__;
    const original = internals.invoke.bind(internals);
    internals.invoke = async (command, args, options) => {
      if (command === "library_track_count") return total;
      if (command === "library_tracks") {
        const { limit, offset } = args as { limit: number; offset: number };
        return Array.from({ length: limit }, (_, index) => make(offset + index));
      }
      return original(command, args, options);
    };
  });

  await page.goto("/#/library");
  await expect(page.locator(".track-row").first()).toBeVisible();

  const measured = await page.evaluate(async () => {
    const scroller = document.querySelector<HTMLElement>(".track-scroll");
    if (!scroller) throw new Error(".track-scroll not found");
    const sample = async (move: boolean) => {
      const frames: number[] = [];
      let rows = 0;
      const started = performance.now();
      await new Promise<void>((resolve) => {
        const tick = (now: number) => {
          frames.push(now);
          if (move) scroller.scrollTop += 240;
          rows = Math.max(rows, document.querySelectorAll(".track-row").length);
          if (now - started < 3_000) {
            requestAnimationFrame(tick);
          } else {
            resolve();
          }
        };
        requestAnimationFrame(tick);
      });
      const elapsed = frames[frames.length - 1] - frames[0];
      return { fps: ((frames.length - 1) / elapsed) * 1_000, rows, elapsed };
    };
    const baseline = await sample(false);
    const scrolling = await sample(true);
    return {
      fps: scrolling.fps,
      baselineFps: baseline.fps,
      frames: Math.round((scrolling.fps * scrolling.elapsed) / 1_000),
      elapsed: scrolling.elapsed,
      maxRows: Math.max(baseline.rows, scrolling.rows),
      scrollTop: scroller.scrollTop,
    };
  });

  // Attach the numbers so `pnpm test:perf` output doubles as evidence.
  await test.info().attach("scroll-fps.json", {
    body: JSON.stringify(measured, null, 2),
    contentType: "application/json",
  });

  console.log(
    `UI-15: ${measured.fps.toFixed(1)}fps scrolling (idle ${measured.baselineFps.toFixed(1)}fps) ` +
      `over ${measured.elapsed.toFixed(0)}ms, max ${measured.maxRows} rows in DOM, ` +
      `scrolled ${measured.scrollTop}px`,
  );

  expect(measured.maxRows, "virtual list must not mount 100k rows").toBeLessThan(80);
  expect(measured.scrollTop, "scroll must actually move").toBeGreaterThan(1_000);
  expect(measured.fps).toBeGreaterThanOrEqual(55);
});
