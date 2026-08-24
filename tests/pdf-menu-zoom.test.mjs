import assert from "node:assert/strict";

const port = Number(process.env.MOYU_CDP_PORT || 0);
const pdfPath = Buffer.from(process.argv[2] || "", "base64").toString("utf8");

if (!port || !pdfPath) {
  console.log("pdf-menu-zoom test skipped: requires MOYU_CDP_PORT and a base64 PDF path");
  process.exit(0);
}

const delay = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));

async function findPageTarget() {
  for (let attempt = 0; attempt < 60; attempt += 1) {
    try {
      const targets = await fetch(`http://127.0.0.1:${port}/json`).then((response) => response.json());
      const target = targets.find((item) => item.type === "page" && item.webSocketDebuggerUrl);
      if (target) return target;
    } catch {}
    await delay(250);
  }
  throw new Error(`WebView2 debugging endpoint did not open on port ${port}`);
}

class CdpClient {
  constructor(url) {
    this.nextId = 1;
    this.pending = new Map();
    this.socket = new WebSocket(url);
  }

  async connect() {
    await new Promise((resolve, reject) => {
      this.socket.addEventListener("open", resolve, { once: true });
      this.socket.addEventListener("error", reject, { once: true });
    });
    this.socket.addEventListener("message", (event) => {
      const message = JSON.parse(event.data);
      const pending = this.pending.get(message.id);
      if (!pending) return;
      this.pending.delete(message.id);
      if (message.error) pending.reject(new Error(message.error.message));
      else pending.resolve(message.result);
    });
  }

  send(method, params = {}) {
    const id = this.nextId++;
    this.socket.send(JSON.stringify({ id, method, params }));
    return new Promise((resolve, reject) => this.pending.set(id, { resolve, reject }));
  }

  async evaluate(expression) {
    const result = await this.send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
    if (result.exceptionDetails) {
      throw new Error(result.exceptionDetails.exception?.description || result.exceptionDetails.text || "Runtime evaluation failed");
    }
    return result.result.value;
  }

  close() {
    this.socket.close();
  }
}

async function waitFor(client, expression, label) {
  for (let attempt = 0; attempt < 120; attempt += 1) {
    const value = await client.evaluate(expression).catch(() => null);
    if (value) return value;
    await delay(250);
  }
  throw new Error(`Timed out waiting for ${label}`);
}

const target = await findPageTarget();
const client = new CdpClient(target.webSocketDebuggerUrl);
await client.connect();
const launcherSettings = await client.evaluate("window.__TAURI__.core.invoke('app_get_settings')");
assert.equal(launcherSettings.launcherCompactMigrated, true, "Launcher compact-size migration must be recorded");
assert(launcherSettings.windowStates.launcher.width >= 820, "Launcher width must respect the compact minimum");
assert(launcherSettings.windowStates.launcher.height >= 560, "Launcher height must respect the compact minimum");
await client.evaluate("location.hash = '/pdf-reader'; location.reload(); true");
await waitFor(client, "document.body.classList.contains('pdf-reader')", "PDF reader route");
await client.evaluate(`window.dispatchEvent(new CustomEvent('local:pdf-opened', { detail: ${JSON.stringify(pdfPath)} })); true`);
await waitFor(client, "document.querySelectorAll('.pdf-page-shell').length >= 2", "PDF layout");

const initialSettings = await client.evaluate("window.__TAURI__.core.invoke('app_get_settings')");
await client.evaluate(`window.__TAURI__.core.invoke('app_set_module_setting', { moduleKey: 'pdfReader', settingKey: 'trimMargins', enabled: true }).then((settings) => {
  window.dispatchEvent(new CustomEvent('local:module-settings', { detail: settings.moduleSettings }));
  return true;
})`);
const trimmedMargins = Number(await waitFor(client, `(() => {
  const canvas = Array.from(document.querySelectorAll('.pdf-page-canvas')).find((item) => Number(item.dataset.trimmedMargins) > 0);
  return canvas?.dataset.trimmedMargins || null;
})()`, "trimmed PDF margins"));
assert(trimmedMargins > 0 && trimmedMargins <= 40, `PDF margin trimming is unsafe: ${trimmedMargins}%`);
const initialZoom = Number(await client.evaluate("document.body.dataset.pdfZoom"));
const initialRenderScale = Number(await waitFor(client, `(() => {
  const canvas = Array.from(document.querySelectorAll('.pdf-page-canvas')).find((item) => item.dataset.renderScale);
  return canvas?.dataset.renderScale || null;
})()`, "adaptive PDF render scale"));
assert(initialRenderScale >= 1 && initialRenderScale <= 2.4, `PDF canvas render density is out of range: ${initialRenderScale}`);
const expectedZoomIn = Math.min(250, initialZoom + 15);

await client.evaluate("window.dispatchEvent(new CustomEvent('local:native-menu-action', { detail: { action: 'pdf-zoom-in' } })); true");
await waitFor(client, `document.body.dataset.pdfZoom === '${expectedZoomIn}'`, "menu zoom in");

await client.evaluate("window.dispatchEvent(new CustomEvent('local:native-menu-action', { detail: { action: 'pdf-zoom-out' } })); window.dispatchEvent(new CustomEvent('local:native-menu-action', { detail: { action: 'pdf-zoom-out' } })); true");
const expectedZoomOut = Math.max(50, expectedZoomIn - 30);
await waitFor(client, `document.body.dataset.pdfZoom === '${expectedZoomOut}'`, "accumulated menu zoom out");

try {
  await client.evaluate(`window.__TAURI__.core.invoke('app_set_theme', { theme: 'night' }).then((settings) => {
    window.dispatchEvent(new CustomEvent('local:settings-changed', { detail: settings }));
    return true;
  })`);
  const globalNight = await waitFor(client, `document.body.dataset.theme === 'night' && document.body.classList.contains('mode-dark') && (() => {
    const viewer = getComputedStyle(document.querySelector('.viewer-wrap')).backgroundColor;
    const filter = getComputedStyle(document.querySelector('.pdf-page-canvas')).filter;
    return { viewer, filter };
  })()`, "global night PDF styling");
  assert.notEqual(globalNight.viewer, "rgba(0, 0, 0, 0)");
  assert.notEqual(globalNight.filter, "none");

  await client.evaluate(`window.__TAURI__.core.invoke('app_set_theme', { theme: 'mist' }).then((settings) => {
    window.dispatchEvent(new CustomEvent('local:settings-changed', { detail: settings }));
    return true;
  })`);
  await client.evaluate(`window.__TAURI__.core.invoke('app_set_module_setting', { moduleKey: 'pdfReader', settingKey: 'nightMode', enabled: true }).then((settings) => {
    window.dispatchEvent(new CustomEvent('local:settings-changed', { detail: settings }));
    window.dispatchEvent(new CustomEvent('local:module-settings', { detail: settings.moduleSettings }));
    return true;
  })`);
  const persistedNight = await waitFor(client, `document.body.classList.contains('mode-dark') && window.__TAURI__.core.invoke('app_get_settings').then((settings) => settings.moduleSettings.pdfReader.nightMode === true)`, "persisted PDF night override");
  assert.equal(persistedNight, true);
} finally {
  await client.evaluate(`window.__TAURI__.core.invoke('app_set_module_setting', { moduleKey: 'pdfReader', settingKey: 'nightMode', enabled: ${Boolean(initialSettings.moduleSettings?.pdfReader?.nightMode)} }).then((settings) => {
    window.dispatchEvent(new CustomEvent('local:module-settings', { detail: settings.moduleSettings }));
    return true;
  })`);
  await client.evaluate(`window.__TAURI__.core.invoke('app_set_theme', { theme: ${JSON.stringify(initialSettings.theme || "mist")} }).then((settings) => {
    window.dispatchEvent(new CustomEvent('local:settings-changed', { detail: settings }));
    return true;
  })`);
  await client.evaluate(`window.__TAURI__.core.invoke('app_set_module_setting', { moduleKey: 'pdfReader', settingKey: 'trimMargins', enabled: ${Boolean(initialSettings.moduleSettings?.pdfReader?.trimMargins)} }).then((settings) => {
    window.dispatchEvent(new CustomEvent('local:module-settings', { detail: settings.moduleSettings }));
    return true;
  })`);
}

console.log(JSON.stringify({ launcher: launcherSettings.windowStates.launcher, initialZoom, initialRenderScale, trimmedMargins, expectedZoomIn, expectedZoomOut, night: "ok" }));
client.close();
