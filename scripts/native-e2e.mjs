/**
 * Native WebDriver smoke test (Windows) for the real app window.
 *
 * Launches the debug binary through `tauri-driver` + msedgedriver, then drives
 * the real WebView2 window: window title, player bar, theme toggle and in-app
 * navigation to the library. This complements the Playwright suite, which runs
 * in a plain browser with mocked Tauri IPC.
 *
 * Debug builds load the frontend from the Vite dev server, so this script
 * starts `vite` on port 1420 when it is not already running.
 *
 * Run: pnpm test:e2e:native
 * Requires: `cargo install tauri-driver` and a matching msedgedriver in
 * tools/native/msedgedriver.exe (major version must match the WebView2 runtime).
 */
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, copyFileSync, writeFileSync, rmSync, openSync } from "node:fs";
import { resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const root = process.cwd();
const exe = resolve(root, "src-tauri/target/debug/biuplayer.exe");
const driverPath = resolve(root, "tools/native/msedgedriver.exe");
const runDir = resolve(root, ".native-e2e/run");
const runExe = resolve(runDir, "biuplayer.exe");
const tempDir = resolve(root, ".native-e2e/tmp");
const devUrl = "http://localhost:1420";
const port = Number(process.env.TAURI_DRIVER_PORT ?? 4444);

if (!existsSync(exe)) {
  console.error(`missing ${exe}; run: cargo build --bin biuplayer (in src-tauri)`);
  process.exit(1);
}
if (!existsSync(driverPath)) {
  console.error(`missing ${driverPath}; download msedgedriver matching the WebView2 runtime`);
  process.exit(1);
}

mkdirSync(tempDir, { recursive: true });
// Portable mode keeps the test profile and library out of %APPDATA%.
rmSync(runDir, { recursive: true, force: true });
mkdirSync(runDir, { recursive: true });
copyFileSync(exe, runExe);
writeFileSync(resolve(runDir, "portable.flag"), "");

const children = [];
function killChildren() {
  for (const { child } of children) {
    try {
      child.kill();
    } catch {
      /* already exited */
    }
  }
}
process.on("exit", killChildren);

function startProcess(name, command, args, logName, extraEnv = {}) {
  const log = openSync(resolve(root, `.native-e2e/${logName}`), "a");
  const child = spawn(command, args, {
    cwd: root,
    stdio: ["ignore", log, log],
    windowsHide: true,
    env: { ...process.env, ...extraEnv },
  });
  children.push({ name, child });
  return child;
}

async function reachable(url) {
  try {
    const response = await fetch(url, { signal: AbortSignal.timeout(1500) });
    await response.arrayBuffer();
    return true;
  } catch {
    return false;
  }
}

async function ensureFrontend() {
  if (await reachable(devUrl)) return;
  startProcess("vite", "pnpm", ["exec", "vite", "--port", "1420", "--strictPort"], "vite.log");
  const started = Date.now();
  while (Date.now() - started < 30000) {
    if (await reachable(devUrl)) return;
    await sleep(300);
  }
  throw new Error(`vite dev server did not start at ${devUrl}`);
}

await ensureFrontend();

startProcess("tauri-driver", "tauri-driver", ["--port", String(port), "--native-driver", driverPath], "tauri-driver.log", {
  TEMP: tempDir,
  TMP: tempDir,
});

let sessionId = null;

async function wd(method, path, body) {
  const response = await fetch(`http://127.0.0.1:${port}${path}`, {
    method,
    headers: { "content-type": "application/json" },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await response.text();
  let json;
  try {
    json = text ? JSON.parse(text) : {};
  } catch {
    json = { raw: text };
  }
  if (!response.ok) {
    const error = new Error(`${method} ${path} -> ${response.status}: ${text}`);
    error.payload = json;
    throw error;
  }
  return json.value;
}

async function waitFor(fn, label, timeoutMs = 30000) {
  const started = Date.now();
  let last;
  while (Date.now() - started < timeoutMs) {
    try {
      const value = await fn();
      if (value) return value;
    } catch (error) {
      last = error;
    }
    await sleep(250);
  }
  throw new Error(`timeout waiting for ${label}${last ? `: ${last.message}` : ""}`);
}

const ELEMENT_KEY = "element-6066-11e4-a52e-4f735466cecf";

async function find(selector) {
  const value = await wd("POST", `/session/${sessionId}/element`, {
    using: "css selector",
    value: selector,
  });
  return value[ELEMENT_KEY] ?? value.ELEMENT;
}

async function findOrNull(selector) {
  try {
    return await find(selector);
  } catch {
    return null;
  }
}

async function execute(script, args = []) {
  return wd("POST", `/session/${sessionId}/execute/sync`, { script, args });
}

const checks = [];
function check(name, ok, detail = "") {
  checks.push({ name, ok, detail });
  console.log(`${ok ? "ok  " : "FAIL"} ${name}${detail ? ` — ${detail}` : ""}`);
}

try {
  await waitFor(async () => {
    try {
      const status = await wd("GET", "/status");
      return status?.ready !== false;
    } catch {
      return false;
    }
  }, "tauri-driver status");

  const created = await wd("POST", "/session", {
    capabilities: {
      alwaysMatch: {
        "tauri:options": { application: runExe },
        "ms:edgeOptions": { args: ["--disable-gpu"] },
      },
    },
  });
  sessionId = created.sessionId;
  check("session created", Boolean(sessionId), sessionId ?? "");

  await waitFor(() => findOrNull(".shell"), "app shell");
  check("native window renders .shell", true);

  const title = await execute("return document.title");
  check("document title", String(title).includes("BiuPlayer"), String(title));

  const player = await findOrNull(".player-meta strong");
  const playerText = player
    ? await wd("GET", `/session/${sessionId}/element/${player}/text`)
    : "";
  check("player bar shows idle state", playerText.trim().length > 0, playerText);

  const layout = await execute(`
    const doc = document.documentElement;
    const offenders = Array.from(document.querySelectorAll("body *"))
      .filter((el) => el.getBoundingClientRect().right > window.innerWidth + 1)
      .slice(0, 5)
      .map((el) => (el.className || el.tagName) + "@" + Math.round(el.getBoundingClientRect().right));
    return { inner: window.innerWidth, scroll: doc.scrollWidth, height: doc.scrollHeight, offenders };
  `);
  check(
    "no horizontal overflow at default size",
    layout.scroll <= layout.inner + 1,
    `inner=${layout.inner} scroll=${layout.scroll} offenders=${layout.offenders.join(", ")}`,
  );

  const before = await execute("return document.documentElement.dataset.theme");
  const toggle = await find(".theme-toggle");
  await wd("POST", `/session/${sessionId}/element/${toggle}/click`, {});
  const changed = await waitFor(async () => {
    const now = await execute("return document.documentElement.dataset.theme");
    return now !== before ? now : null;
  }, "theme toggle", 5000);
  check("theme toggle switches data-theme", changed !== before, `${before} -> ${changed}`);

  const libraryHref = await findOrNull('a[href="#/library"]');
  if (libraryHref) {
    await wd("POST", `/session/${sessionId}/element/${libraryHref}/click`, {});
  } else {
    await execute("location.hash = '#/library'");
  }
  await waitFor(async () => {
    const hash = await execute("return location.hash");
    return String(hash).includes("/library");
  }, "library route");
  check("navigates to library route", true, await execute("return location.hash"));

  await sleep(500);
  const heading = await execute(
    "const h = document.querySelector('.page-head h2'); return h ? h.textContent : ''",
  );
  check("library page heading present", String(heading).trim().length > 0, String(heading));

  const png = await wd("GET", `/session/${sessionId}/screenshot`);
  writeFileSync(resolve(root, ".native-e2e/window.png"), Buffer.from(png, "base64"));
  console.log("screenshot: .native-e2e/window.png");
} catch (error) {
  console.error(error.message);
  if (error.payload) console.error(JSON.stringify(error.payload, null, 2));
  check("native run completed", false, error.message);
} finally {
  if (sessionId) {
    try {
      await wd("DELETE", `/session/${sessionId}`);
    } catch {
      /* driver may already be gone */
    }
  }
  killChildren();
  writeFileSync(resolve(root, ".native-e2e/checks.json"), JSON.stringify(checks, null, 2));
}

const failed = checks.filter((item) => !item.ok);
console.log(
  failed.length === 0
    ? `native e2e: ${checks.length} checks passed`
    : `native e2e: ${failed.length} failed`,
);
process.exit(failed.length === 0 && checks.length > 0 ? 0 : 1);
