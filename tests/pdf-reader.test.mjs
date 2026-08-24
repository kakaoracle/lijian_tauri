import assert from "node:assert/strict";
import fs from "node:fs";

const appSource = fs.readFileSync(new URL("../dist/app.js", import.meta.url), "utf8");
const styles = fs.readFileSync(new URL("../dist/styles.css", import.meta.url), "utf8");
const rustSource = fs.readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

const viewerRule = styles.match(/\.viewer-wrap\s*\{([^}]*)\}/)?.[1] || "";
assert.match(viewerRule, /height:\s*100%\s*;/, "PDF viewer must be constrained to the window height");
assert.match(viewerRule, /max-height:\s*100%\s*;/, "PDF viewer must not grow to the full document height");
assert.match(viewerRule, /overflow:\s*auto\s*;/, "PDF viewer must own scrolling");

assert.doesNotMatch(
  appSource,
  /Promise\.all\(Array\.from\(\{\s*length:\s*totalPages[\s\S]{0,250}?pdfDoc\.getPage/,
  "PDF loading must not fetch every page before the reader becomes interactive"
);
assert.match(appSource, /function\s+ensurePageLoaded\s*\(/, "PDF pages must load incrementally");
assert.match(appSource, /viewerWrap\.scrollTop\s*=\s*targetTop/, "Page navigation must move the scroll container");
assert.match(appSource, /debugPdf\("navigate"/, "Page navigation must leave diagnostic evidence");
assert.doesNotMatch(appSource, /debugPdf\("drag-move"/, "Pointer-move logging must not flood the UI bridge");
assert.doesNotMatch(appSource, /pdf-topbar|page-input|page-select|prev-page|next-page|zoom-indicator/, "PDF must not render a second toolbar below the native menu");
assert.match(appSource, /document\.body\.dataset\.pdfPage\s*=\s*String\(currentPage\)/, "Current page must remain available as internal state");
assert.match(appSource, /document\.body\.dataset\.pdfZoom\s*=\s*String\(Math\.round\(currentZoom \* 100\)\)/, "Current zoom must remain available as internal state");
assert.doesNotMatch(styles, /\.pdf-topbar|--pdf-menu-offset|\.page-input|\.page-select|\.zoom-indicator/, "Removed PDF toolbar styles must not leave layout offsets behind");
const pdfRouteSource = appSource.slice(appSource.indexOf("async function renderPdfReader"), appSource.indexOf("const ROUTES"));
assert.doesNotMatch(pdfRouteSource, /page-tools|toolbar|titlebar/, "PDF route must not render a competing top bar");

assert.match(rustSource, /MENU_SETTINGS_ZOOM_IN/, "Native settings menu must define Zoom In");
assert.match(rustSource, /MENU_SETTINGS_ZOOM_OUT/, "Native settings menu must define Zoom Out");
assert.match(rustSource, /emit_native_menu_action\(app, "pdf_reader", "pdf-zoom-in"\)/);
assert.match(rustSource, /emit_native_menu_action\(app, "pdf_reader", "pdf-zoom-out"\)/);
assert.match(rustSource, /local:native-menu-action/, "Native menu actions must have a direct WebView dispatch path");
assert.match(appSource, /window\.addEventListener\("local:native-menu-action"/, "Business windows must handle direct native menu dispatch");

const zoomStep = Number(appSource.match(/const ZOOM_STEP\s*=\s*([0-9.]+)/)?.[1]);
assert.equal(zoomStep, 0.15, "Each menu zoom command should change PDF scale by a visible 15%");
assert.match(appSource, /let pendingZoomTarget = null/, "Zoom requests must be queueable while a page is rendering");
assert.match(appSource, /function flushZoomRequest\s*\(/, "Queued zoom must flush after rendering");
assert.match(appSource, /function requestZoomBy\s*\(/, "Repeated menu commands must accumulate");
assert.doesNotMatch(appSource, /function bumpRenderVersion[\s\S]{0,400}?state\.renderPromise\s*=\s*null/, "A visual-mode change must not release an in-flight canvas render early");
assert.match(appSource, /if \(version !== renderVersion\) scheduleVisibleRender\(\)/, "A stale render must schedule the current visual version after it finishes");
assert.match(appSource, /handleNativeMenuAction/, "Native menu and runtime verification must share one action dispatcher");
assert.match(appSource, /debugPdf\("zoom:request"/, "Zoom requests must be logged");
assert.match(appSource, /debugPdf\("zoom:apply"/, "Applied zoom must be logged");
assert.doesNotMatch(appSource, /ZOOM_ANIM_STEP|ZOOM_FRAME_MS|animateZoomTo/, "Zoom must not rerender every animation frame");
assert.match(appSource, /function pdfRenderScale\(viewportWidth, viewportHeight\)/, "PDF render density must account for the current page size");
assert.match(appSource, /qualityBoost = clamp\(1\.28 - Math\.max\(0, currentZoom - 0\.85\) \* 0\.18, 1\.05, 1\.28\)/, "Higher zoom levels must avoid fixed excessive oversampling");
assert.match(appSource, /12_000_000/, "PDF canvases must respect a per-page pixel budget");
assert.doesNotMatch(appSource, /ratio \* 1\.65|clamp\([^\n]*1\.8, 2\.6\)/, "PDF rendering must not retain the old fixed sharpening-prone scale");
assert.match(appSource, /softenedLuminance = 128 \+ \(luminance - 128\) \* 0\.92/, "Opaque monochrome rendering must preserve antialiased edge gray levels");
assert.match(appSource, /darknessCoverage = clamp\(\(250 - luminance\) \/ 242/, "Transparent monochrome rendering must preserve edge coverage");
assert.match(appSource, /context\.imageSmoothingQuality = "high"/, "Canvas image resampling must use high-quality smoothing");
assert.match(appSource, /canvas\.dataset\.renderScale = renderScale\.toFixed\(3\)/, "Completed PDF renders must expose their actual density for diagnostics");
assert.match(styles, /image-rendering:\s*auto/, "PDF canvas CSS must not request pixelated or crisp-edge rendering");

assert.match(appSource, /<option value="night">/, "Global settings must expose a night theme");
assert.match(appSource, /\["mist", "paper", "ink", "pine", "night"\]/, "Frontend theme validation must accept night");
assert.match(rustSource, /\["mist", "paper", "ink", "pine", "night"\]/, "Rust theme validation must accept night");
assert.match(styles, /body\[data-theme="night"\]\s*\{/, "Night theme CSS variables must exist");

assert.match(rustSource, /night_mode:\s*bool/, "PDF night override must be persisted by Rust");
assert.match(rustSource, /\("pdfReader", "nightMode"\)/, "PDF night override must be accepted by the settings command");
assert.match(rustSource, /trim_margins:\s*bool/, "PDF horizontal margin trimming must be persisted by Rust");
assert.match(rustSource, /\("pdfReader", "trimMargins"\)/, "PDF horizontal margin trimming must be accepted by the settings command");
assert.match(appSource, /trimMargins:\s*true/, "PDF horizontal margin trimming must default to enabled");
assert.match(appSource, /function detectHorizontalCrop\s*\(/, "PDF pages must detect horizontal white margins");
assert.match(appSource, /cropWidth < width \* 0\.6/, "Margin trimming must protect unusually narrow content");
assert.match(appSource, /padding = Math\.round\(12 \* renderScale\)/, "Margin trimming must preserve a safe content gutter");
assert.match(appSource, /canvas\.dataset\.trimmedMargins/, "Completed PDF renders must expose the trimmed percentage for diagnostics");
assert.match(appSource, /theme === "night"/, "Global night theme must force PDF night rendering");
assert.match(styles, /body\.pdf-reader\.mode-dark \.pdf-page-canvas\s*\{[^}]*background:\s*#000[^}]*filter:\s*invert/s);
assert.match(appSource, /hideImages/, "Night mode must not remove the independent image visibility setting");

console.log("pdf-reader tests passed");
