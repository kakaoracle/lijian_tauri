import assert from "node:assert/strict";

const port = Number(process.env.MOYU_CDP_PORT || 9333);
const pdfPath = Buffer.from(process.argv[2] || "", "base64").toString("utf8");
assert(pdfPath, "A base64-encoded PDF path is required");

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
    const result = await this.send("Runtime.evaluate", {
      expression,
      awaitPromise: true,
      returnByValue: true
    });
    if (result.exceptionDetails) throw new Error(result.exceptionDetails.text || "Runtime evaluation failed");
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
await client.evaluate("location.hash = '/pdf-reader'; location.reload(); true");
await waitFor(client, "document.body.classList.contains('pdf-reader')", "PDF reader route");
await client.evaluate(`window.dispatchEvent(new CustomEvent('local:pdf-opened', { detail: ${JSON.stringify(pdfPath)} })); true`);

const loaded = await waitFor(client, `(() => {
  const viewer = document.querySelector('.viewer-wrap');
  const pages = document.querySelectorAll('.pdf-page-shell').length;
  if (!viewer || pages < 2) return null;
  return {
    pages,
    page: Number(document.body.dataset.pdfPage || 0),
    clientHeight: viewer.clientHeight,
    scrollHeight: viewer.scrollHeight,
    scrollTop: viewer.scrollTop,
    maxScrollTop: viewer.scrollHeight - viewer.clientHeight
  };
})()`, "PDF layout");

assert(loaded.clientHeight > 0, "PDF viewer must have a visible viewport");
assert(loaded.maxScrollTop > 0, "PDF viewer must have a real scroll range");
assert(loaded.clientHeight < loaded.scrollHeight, "PDF content must not expand the viewer viewport");

const afterNext = await client.evaluate(`(async () => {
  const viewer = document.querySelector('.viewer-wrap');
  const before = viewer.scrollTop;
  document.dispatchEvent(new KeyboardEvent('keydown', {
    key: 'PageDown',
    bubbles: true,
    cancelable: true
  }));
  await new Promise((resolve) => setTimeout(resolve, 600));
  return { before, after: viewer.scrollTop, page: Number(document.body.dataset.pdfPage) };
})()`);
assert(afterNext.after > afterNext.before, "PageDown must advance the scroll position");
assert.equal(afterNext.page, loaded.page + 1, "PageDown must advance exactly one page");

const bounds = await client.evaluate(`(() => {
  const rect = document.querySelector('.viewer-wrap').getBoundingClientRect();
  return { x: Math.round(rect.left + rect.width / 2), y: Math.round(rect.top + rect.height / 2), top: document.querySelector('.viewer-wrap').scrollTop };
})()`);
await client.send("Input.dispatchMouseEvent", { type: "mouseWheel", x: bounds.x, y: bounds.y, deltaX: 0, deltaY: 320 });
await delay(600);
const afterWheel = await client.evaluate("document.querySelector('.viewer-wrap').scrollTop");
assert(afterWheel > bounds.top, "Mouse wheel must advance the PDF scroll position");

await client.send("Input.dispatchMouseEvent", { type: "mousePressed", x: bounds.x, y: bounds.y, button: "left", buttons: 1, clickCount: 1 });
await client.send("Input.dispatchMouseEvent", { type: "mouseMoved", x: bounds.x, y: bounds.y - 140, button: "left", buttons: 1 });
await client.send("Input.dispatchMouseEvent", { type: "mouseReleased", x: bounds.x, y: bounds.y - 140, button: "left", buttons: 0, clickCount: 1 });
await delay(600);
const afterDrag = await client.evaluate("document.querySelector('.viewer-wrap').scrollTop");
assert(afterDrag > afterWheel, "Pointer drag must advance the PDF scroll position");

console.log(JSON.stringify({ loaded, afterNext, afterWheel, afterDrag }));
client.close();
