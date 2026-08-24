#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{atomic::{AtomicBool, Ordering}, Mutex, OnceLock},
};
use base64::{engine::general_purpose, Engine as _};
use tauri::{
    menu::{MenuBuilder, SubmenuBuilder},
    path::BaseDirectory,
    utils::config::{Color, WebviewUrl},
    webview::WebviewBuilder,
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Webview, Window, WindowBuilder, Wry,
    WebviewWindow, WebviewWindowBuilder,
};
use url::Url;
#[cfg(target_os = "windows")]
use std::ffi::c_void;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Graphics::Dwm::DwmDefWindowProc;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::RECT;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::{
    Input::KeyboardAndMouse::{RegisterHotKey, MOD_ALT, MOD_CONTROL},
    WindowsAndMessaging::{
        CallWindowProcW, DefWindowProcW, GetMessageW, GetSystemMetrics, GetWindowLongPtrW,
        GetWindowRect, SetWindowLongPtrW, GWLP_WNDPROC, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT,
        HTCLIENT, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT, MSG, SM_CXPADDEDBORDER,
        SM_CXSIZEFRAME, SM_CYSIZEFRAME, WM_HOTKEY, WM_NCHITTEST, WNDPROC,
    },
};

const DEFAULT_URL: &str = "https://www.fenbi.com";
const SETTINGS_FILE_NAME: &str = "settings.json";
const PDF_READER_PROGRESS_FILE_NAME: &str = "pdf-reader-progress.json";
const MENTAL_MATH_HISTORY_FILE_NAME: &str = "mental-math-history.json";
const VOCABULARY_STORE_FILE_NAME: &str = "vocabulary-sources.json";
const DEFAULT_PDF_VISUAL_MODE: &str = "transparent";
const MENTAL_MATH_HISTORY_LIMIT: usize = 1000;
const VOCABULARY_SOURCE_LIMIT: usize = 100;
const VOCABULARY_TXT_MAX_BYTES: u64 = 8 * 1024 * 1024;
const FENBI_IMAGE_CACHE_LIMIT: usize = 80;
const FENBI_IMAGE_MAX_BYTES: usize = 3 * 1024 * 1024;
const FENBI_IMAGE_MAX_PIXELS: usize = 2_400_000;
const MENU_FILE_SHOW_LAUNCHER: &str = "menu.file.show-launcher";
const MENU_FILE_OPEN_PDF: &str = "menu.file.open-pdf";
const MENU_FILE_CLOSE_WINDOW: &str = "menu.file.close-window";
const MENU_SETTINGS_OPEN: &str = "menu.settings.open";
const MENU_HELP_CONTACT: &str = "menu.help.contact";
const MENU_SETTINGS_ZOOM_IN: &str = "menu.settings.zoom-in";
const MENU_SETTINGS_ZOOM_OUT: &str = "menu.settings.zoom-out";
const GENERIC_TRANSPARENCY_CSS: &str = r#"
html, body {
  background: transparent !important;
  background-color: transparent !important;
  background-image: none !important;
}
*, *::before, *::after {
  background-color: transparent !important;
  background-image: none !important;
  box-shadow: none !important;
}
img, svg, canvas, video, picture, iframe {
  opacity: 1 !important;
}
"#;

const FENBI_TRANSPARENCY_CSS: &str = r#"
html, body, #app, #root, .app, .container, .layout {
  background: transparent !important;
  background-color: transparent !important;
}
[class*='header'],
[class*='navbar'],
[class*='nav-bar'],
[class*='toolbar'],
[class*='tabbar'],
[class*='shadow'],
[style*='box-shadow'] {
  box-shadow: none !important;
}
"#;

const FENBI_PERSONALIZED_CSS: &str = r#"
body, body * {
  text-shadow: none !important;
}
a,
button,
span,
p,
strong,
b,
em,
label,
li,
div,
h1,
h2,
h3,
h4,
h5,
h6 {
  color: rgba(18, 18, 18, 0.96) !important;
  -webkit-text-fill-color: rgba(18, 18, 18, 0.96) !important;
}
"#;

const FENBI_IMAGE_TRANSPARENCY_SCRIPT: &str = r#"
(() => {
  if (!/(\.|^)fenbi\.com$/i.test(location.hostname)) return;
  const MARK = 'moyuImageTransparency';
  const FALLBACK_CLASS = 'moyu-image-transparent-fallback';
  const REQUEST_PREFIX = '__MOYU_CONTENT_IMAGE__';
  const MAX_PIXELS = 2400000;
  const MAX_EDGE_SAMPLES = 1600;
  const STYLE_ID = '__moyu_fenbi_image_transparency__';
  let logCount = 0;
  let scanCount = 0;
  let requestCount = 0;

  function safeText(value, max = 220) {
    const text = String(value ?? '');
    return text.length > max ? `${text.slice(0, max)}...` : text;
  }

  function debug(event, payload = {}) {
    if (logCount >= 80) return;
    logCount += 1;
    const message = `[fenbi-image-transparency] ${event} ${JSON.stringify(payload)}`;
    try {
      document.title = `__MOYU_CONTENT_LOG__${safeText(message, 700)}`;
    } catch (_) {}
  }

  function imageInfo(img) {
    const src = img.currentSrc || img.src || '';
    let parsed = null;
    try { parsed = src ? new URL(src, location.href) : null; } catch (_) {}
    return {
      src: safeText(src, 220),
      host: parsed?.host || '',
      sameOrigin: parsed ? parsed.origin === location.origin : false,
      width: img.naturalWidth || 0,
      height: img.naturalHeight || 0
    };
  }

  function installStyle() {
    let style = document.getElementById(STYLE_ID);
    if (!style) {
      style = document.createElement('style');
      style.id = STYLE_ID;
      document.documentElement.appendChild(style);
    }
    style.textContent = `
      img.${FALLBACK_CLASS} {
        filter: grayscale(1) contrast(1.85) brightness(0.92) !important;
        mix-blend-mode: multiply !important;
      }
    `;
  }

  function requestRustProcess(img) {
    if (!img || img.dataset.moyuRustRequest === 'busy' || img.dataset.moyuRustRequest === 'done') {
      return false;
    }
    const src = img.currentSrc || img.src || '';
    if (!src.startsWith('https://')) return false;
    requestCount += 1;
    const id = `img-${Date.now()}-${requestCount}`;
    img.dataset.moyuRustRequest = 'busy';
    img.dataset.moyuRustRequestId = id;
    img.classList.add(FALLBACK_CLASS);
    try {
      document.title = `${REQUEST_PREFIX}${JSON.stringify({ id, url: src, referer: location.href })}`;
      debug('rust-request', { id, src: safeText(src, 180) });
      return true;
    } catch (_) {
      img.dataset.moyuRustRequest = 'failed';
      return false;
    }
  }

  window.__moyuFenbiImageTransparencyReceive = function(id, dataUrl, ok) {
    const img = document.querySelector(`img[data-moyu-rust-request-id="${id}"]`);
    if (!img) return;
    if (ok && dataUrl && dataUrl.startsWith('data:image/png;base64,')) {
      img.dataset.moyuOriginalSrc = img.currentSrc || img.src || '';
      img.src = dataUrl;
      img.classList.remove(FALLBACK_CLASS);
      img.dataset[MARK] = 'done';
      img.dataset.moyuRustRequest = 'done';
      debug('rust-success', { id, src: safeText(img.dataset.moyuOriginalSrc || '', 180) });
    } else {
      img.classList.add(FALLBACK_CLASS);
      img.dataset[MARK] = 'done';
      img.dataset.moyuRustRequest = 'failed';
      debug('rust-fallback', { id, src: safeText(img.currentSrc || img.src || '', 180) });
    }
  };

  function canProcess(img, info) {
    if (!img || img.dataset[MARK] === 'done' || img.dataset[MARK] === 'busy') return false;
    const src = img.currentSrc || img.src || '';
    if (!src || src.startsWith('data:') || src.startsWith('blob:')) {
      return false;
    }
    const width = img.naturalWidth || 0;
    const height = img.naturalHeight || 0;
    if (width < 12 || height < 12) {
      return false;
    }
    if (width * height > MAX_PIXELS) {
      debug('skip-large', { pixels: width * height, ...info });
      return false;
    }
    return true;
  }

  function edgeBackground(data, width, height) {
    const samples = [];
    const step = Math.max(1, Math.floor((width * 2 + height * 2) / MAX_EDGE_SAMPLES));
    function add(x, y) {
      const i = (y * width + x) * 4;
      const a = data[i + 3];
      if (a < 220) return;
      samples.push([data[i], data[i + 1], data[i + 2]]);
    }
    for (let x = 0; x < width; x += step) {
      add(x, 0);
      add(x, height - 1);
    }
    for (let y = 0; y < height; y += step) {
      add(0, y);
      add(width - 1, y);
    }
    if (!samples.length) return [255, 255, 255];
    samples.sort((a, b) => (a[0] + a[1] + a[2]) - (b[0] + b[1] + b[2]));
    const bright = samples.slice(Math.floor(samples.length * 0.55));
    const pool = bright.length ? bright : samples;
    const sum = pool.reduce((acc, item) => {
      acc[0] += item[0];
      acc[1] += item[1];
      acc[2] += item[2];
      return acc;
    }, [0, 0, 0]);
    return sum.map((value) => value / pool.length);
  }

  function processPixels(imageData) {
    const data = imageData.data;
    const bg = edgeBackground(data, imageData.width, imageData.height);
    let visible = 0;
    let transparent = 0;
    let faint = 0;
    for (let i = 0; i < data.length; i += 4) {
      const r = data[i];
      const g = data[i + 1];
      const b = data[i + 2];
      const a = data[i + 3];
      if (a < 16) continue;
      const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      const bgDistance = Math.abs(r - bg[0]) + Math.abs(g - bg[1]) + Math.abs(b - bg[2]);
      const isLightBg = luminance > 205 && bgDistance < 92;
      const isNearWhite = luminance > 232 && Math.max(r, g, b) - Math.min(r, g, b) < 34;
      if (isLightBg || isNearWhite) {
        const fade = Math.max(0, Math.min(1, (245 - luminance) / 42));
        data[i + 3] = Math.round(26 * fade);
        if (data[i + 3] <= 4) transparent += 1;
        else faint += 1;
        continue;
      }
      const ink = Math.max(0, Math.min(235, (luminance - 18) * 0.42));
      data[i] = ink;
      data[i + 1] = ink;
      data[i + 2] = ink;
      data[i + 3] = Math.max(a, 236);
      visible += 1;
    }
    return {
      visible,
      transparent,
      faint,
      background: bg.map((value) => Math.round(value))
    };
  }

  async function processImage(img) {
    const baseInfo = imageInfo(img);
    if (!canProcess(img, baseInfo)) return;
    img.dataset[MARK] = 'busy';
    try {
      if (!img.complete) {
        await new Promise((resolve, reject) => {
          img.addEventListener('load', resolve, { once: true });
          img.addEventListener('error', reject, { once: true });
        });
      }
      const loadedInfo = imageInfo(img);
      if (!canProcess(img, loadedInfo) && img.dataset[MARK] !== 'busy') return;
      const width = img.naturalWidth;
      const height = img.naturalHeight;
      const canvas = document.createElement('canvas');
      canvas.width = width;
      canvas.height = height;
      const ctx = canvas.getContext('2d', { willReadFrequently: true });
      if (!ctx) throw new Error('canvas unavailable');
      ctx.drawImage(img, 0, 0, width, height);
      const imageData = ctx.getImageData(0, 0, width, height);
      const stats = processPixels(imageData);
      if (stats.visible < Math.max(12, width * height * 0.002)) {
        throw new Error(`content too sparse visible=${stats.visible}`);
      }
      ctx.putImageData(imageData, 0, 0);
      img.dataset.moyuOriginalSrc = img.currentSrc || img.src || '';
      img.src = canvas.toDataURL('image/png');
      img.classList.remove(FALLBACK_CLASS);
      img.dataset[MARK] = 'done';
      debug('process-success', {
        ...loadedInfo,
        transparentRatio: Number((stats.transparent / Math.max(1, width * height)).toFixed(3)),
        visibleRatio: Number((stats.visible / Math.max(1, width * height)).toFixed(3))
      });
    } catch (error) {
      const message = String(error?.message || error || '');
      const likelyCrossOrigin = /taint|cross-origin|security/i.test(message);
      debug('process-fallback', {
        ...imageInfo(img),
        errorMessage: safeText(message, 220),
        likelyCrossOrigin
      });
      if (!likelyCrossOrigin || !requestRustProcess(img)) {
        img.classList.add(FALLBACK_CLASS);
        img.dataset[MARK] = 'done';
      }
    }
  }

  function scan(root = document) {
    installStyle();
    scanCount += 1;
    const images = Array.from(root.querySelectorAll?.('img') || []);
    if (scanCount <= 5 || scanCount % 20 === 0) {
      const backgroundCandidates = Array.from(document.querySelectorAll('[style*="background"], [class*="image"], [class*="img"], [class*="pic"]'))
        .slice(0, 80)
        .filter((node) => {
          try { return /url\(/i.test(getComputedStyle(node).backgroundImage || ''); } catch (_) { return false; }
        });
      debug('scan-summary', {
        scanCount,
        imageCount: images.length,
        pendingImages: images.filter((img) => img.dataset[MARK] !== 'done' && img.dataset[MARK] !== 'busy').length,
        canvasCount: document.querySelectorAll('canvas').length,
        backgroundImageCandidates: backgroundCandidates.length
      });
    }
    images.forEach((img) => processImage(img));
  }

  if (!window.__moyuFenbiImageTransparencyInstalled) {
    window.__moyuFenbiImageTransparencyInstalled = true;
    debug('install', {
      href: safeText(location.href, 180)
    });
    const observer = new MutationObserver((mutations) => {
      window.requestIdleCallback
        ? window.requestIdleCallback(() => scan(document), { timeout: 800 })
        : window.setTimeout(() => scan(document), 80);
    });
    observer.observe(document.documentElement, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['src', 'srcset', 'style', 'class']
    });
    window.addEventListener('load', () => scan(document), { once: true });
    window.setInterval(() => scan(document), 2500);
  }
  scan(document);
})();
"#;

const UNIVERSAL_SAME_WINDOW_NAVIGATION_SCRIPT: &str = r#"
(() => {
  if (window.__moyuUniversalSameWindowNavigationInstalled) return;
  window.__moyuUniversalSameWindowNavigationInstalled = true;
  const originalOpen = window.open;
  function normalizeTarget(target) {
    return target ? String(target).toLowerCase() : '';
  }
  function shouldReuseCurrentWindow(target) {
    const normalized = normalizeTarget(target);
    return normalized === '' || normalized === '_blank' || normalized === '_new';
  }
  function navigateCurrentWindow(url) {
    if (!url) return null;
    try { window.location.assign(url); } catch (_) { window.location.href = url; }
    return window;
  }
  const locationProxy = {
    assign: (url) => navigateCurrentWindow(url),
    replace: (url) => window.location.replace(url),
    get href() { return window.location.href; },
    set href(url) { navigateCurrentWindow(url); }
  };
  function popupProxy() {
    return new Proxy({}, {
      get(_target, prop) {
        if (prop === 'location') return locationProxy;
        if (prop === 'closed') return false;
        if (prop === 'focus' || prop === 'blur' || prop === 'close') return () => {};
        if (prop === 'opener') return window;
        return window[prop];
      },
      set(_target, prop, value) {
        if (prop === 'location') {
          navigateCurrentWindow(String(value || ''));
          return true;
        }
        return true;
      }
    });
  }
  window.open = function patchedWindowOpen(url, target, features) {
    if (shouldReuseCurrentWindow(target)) {
      if (url) navigateCurrentWindow(url);
      return popupProxy();
    }
    return originalOpen.call(window, url, target, features);
  };
  document.addEventListener('click', (event) => {
    const anchor = event.target && event.target.closest ? event.target.closest('a[href]') : null;
    if (!anchor) return;
    const target = normalizeTarget(anchor.getAttribute('target'));
    if (target !== '_blank' && target !== '_new') return;
    const href = anchor.href;
    if (!href || href.startsWith('javascript:')) return;
    event.preventDefault();
    event.stopPropagation();
    navigateCurrentWindow(href);
  }, true);
  document.addEventListener('submit', (event) => {
    const form = event.target;
    if (!form) return;
    const submitterTarget = normalizeTarget(event.submitter && event.submitter.getAttribute ? event.submitter.getAttribute('formtarget') : '');
    const formTarget = normalizeTarget(form.getAttribute && form.getAttribute('target'));
    const target = submitterTarget || formTarget;
    if (!shouldReuseCurrentWindow(target)) return;
    if (event.submitter && event.submitter.removeAttribute) {
      event.submitter.removeAttribute('formtarget');
    }
    if (form.removeAttribute) {
      form.removeAttribute('target');
    }
  }, true);
})();
"#;

#[derive(Default)]
struct RuntimeState {
    pdf_path: Mutex<Option<String>>,
    content_url: Mutex<Option<String>>,
    content_transparent: Mutex<Option<bool>>,
    content_webview: Mutex<Option<Webview>>,
    all_windows_hidden: Mutex<bool>,
    fenbi_image_cache: Mutex<FenbiImageCache>,
}

#[derive(Default)]
struct FenbiImageCache {
    items: HashMap<String, String>,
    order: VecDeque<String>,
}

#[cfg(target_os = "windows")]
static RESIZE_ZONE_EXPANDED: AtomicBool = AtomicBool::new(true);
#[cfg(target_os = "windows")]
static WINDOW_PROC_MAP: OnceLock<Mutex<HashMap<isize, isize>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowState {
    width: f64,
    height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    zoom_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    default_transparent: bool,
    #[serde(default = "default_theme")]
    theme: String,
    #[serde(default = "default_expanded_resize_zone")]
    expanded_resize_zone: bool,
    #[serde(default)]
    titlebar_transparent_follow: bool,
    #[serde(default = "default_module_settings")]
    module_settings: ModuleSettings,
    #[serde(default = "default_plugin_settings")]
    plugin_settings: PluginSettings,
    #[serde(default)]
    window_states: HashMap<String, WindowState>,
    #[serde(default)]
    launcher_compact_migrated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModuleSettings {
    #[serde(default = "default_content_module_settings")]
    content: ContentModuleSettings,
    #[serde(default = "default_mental_math_module_settings")]
    mental_math: MentalMathModuleSettings,
    #[serde(default = "default_pdf_reader_module_settings")]
    pdf_reader: PdfReaderModuleSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PluginSettings {
    #[serde(default = "default_content_plugin_settings")]
    content: ContentPluginSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentPluginSettings {
    fenbi_personalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentModuleSettings {
    transparent: bool,
    #[serde(default)]
    universal_transparency: bool,
    #[serde(default = "default_content_image_transparency")]
    image_transparency: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MentalMathModuleSettings {
    transparent: bool,
    weakness_focus: bool,
    strong_text: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PdfReaderModuleSettings {
    transparent: bool,
    #[serde(default)]
    monochrome: bool,
    #[serde(default)]
    hide_images: bool,
    #[serde(default)]
    night_mode: bool,
    #[serde(default = "default_true")]
    trim_margins: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PdfProgress {
    bookmarks: HashMap<String, PdfBookmark>,
    visual_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PdfBookmark {
    page: u32,
    total_pages: Option<u32>,
    last_read: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PdfSavePayload {
    file_path: String,
    page: u32,
    total_pages: Option<u32>,
}

#[derive(Debug, Serialize)]
struct PdfReadResult {
    ok: bool,
    data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FenbiImageRequest {
    id: String,
    url: String,
    referer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MentalMathHistory {
    #[serde(default)]
    items: Vec<MentalMathHistoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MentalMathHistoryItem {
    id: String,
    timestamp: String,
    mode_key: String,
    mode_label: String,
    question_text: String,
    correct_answer: String,
    user_answer: String,
    is_correct: bool,
    elapsed_ms: u32,
    weakness_focus_enabled: bool,
    from_weakness_focus: bool,
    #[serde(default)]
    features: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MentalMathHistorySavePayload {
    mode_key: String,
    mode_label: String,
    question_text: String,
    correct_answer: String,
    user_answer: String,
    is_correct: bool,
    elapsed_ms: u32,
    weakness_focus_enabled: bool,
    from_weakness_focus: bool,
    #[serde(default)]
    features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct VocabularyStore {
    #[serde(default)]
    sources: Vec<VocabularySource>,
    #[serde(default)]
    blacklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VocabularySource {
    id: String,
    path: String,
    file_name: String,
    imported_at: String,
    char_count: usize,
    sentence_count: usize,
    #[serde(default)]
    entries: Vec<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VocabularySourcePayload {
    path: String,
    file_name: String,
    char_count: usize,
    sentence_count: usize,
    #[serde(default)]
    entries: Vec<Value>,
}

fn app_data_file(app: &AppHandle, file_name: &str) -> Result<PathBuf, String> {
    app.path()
        .resolve(file_name, BaseDirectory::AppData)
        .map_err(|error| error.to_string())
}

fn read_json<T>(path: &Path, fallback: T) -> T
where
    T: for<'de> Deserialize<'de>,
{
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<T>(&raw).ok())
        .unwrap_or(fallback)
}

fn write_json<T>(path: &Path, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let raw = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    fs::write(path, format!("{raw}\n")).map_err(|error| error.to_string())
}

fn is_allowed_fenbi_image_host(host: &str) -> bool {
    matches!(
        host,
        "fb.fbstatic.cn" | "fb.fenbike.cn" | "nodestatic.fbstatic.cn" | "userprofile.fbstatic.cn"
    )
}

fn cache_get_fenbi_image(app: &AppHandle, url: &str) -> Option<String> {
    let handle = app.clone();
    let state = handle.state::<RuntimeState>();
    state
        .fenbi_image_cache
        .lock()
        .ok()
        .and_then(|cache| cache.items.get(url).cloned())
}

fn cache_put_fenbi_image(app: &AppHandle, url: String, data_url: String) {
    let state = app.state::<RuntimeState>();
    let Ok(mut cache) = state.fenbi_image_cache.lock() else {
        return;
    };
    if !cache.items.contains_key(&url) {
        cache.order.push_back(url.clone());
    }
    cache.items.insert(url.clone(), data_url);
    while cache.order.len() > FENBI_IMAGE_CACHE_LIMIT {
        if let Some(old) = cache.order.pop_front() {
            cache.items.remove(&old);
        }
    }
}

fn clear_fenbi_image_cache(app: &AppHandle) {
    let state = app.state::<RuntimeState>();
    {
        let cache = state.fenbi_image_cache.lock();
        if let Ok(mut cache) = cache {
            cache.items.clear();
            cache.order.clear();
        }
    }
}

fn eval_fenbi_image_result(webview: &Webview, id: &str, data_url: Option<&str>, ok: bool) {
    let id_json = serde_json::to_string(id).unwrap_or_else(|_| "\"\"".to_string());
    let data_json = serde_json::to_string(data_url.unwrap_or("")).unwrap_or_else(|_| "\"\"".to_string());
    let script = format!(
        "window.__moyuFenbiImageTransparencyReceive && window.__moyuFenbiImageTransparencyReceive({id}, {data}, {ok});",
        id = id_json,
        data = data_json,
        ok = if ok { "true" } else { "false" }
    );
    let _ = webview.eval(&script);
}

fn decode_png_rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    let cursor = std::io::Cursor::new(bytes);
    let mut decoder = png::Decoder::new(cursor);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|error| error.to_string())?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).map_err(|error| error.to_string())?;
    let data = &buffer[..info.buffer_size()];
    let pixel_count = info.width as usize * info.height as usize;
    if pixel_count == 0 || pixel_count > FENBI_IMAGE_MAX_PIXELS {
        return Err("image size out of range".to_string());
    }
    let mut rgba = Vec::with_capacity(pixel_count * 4);
    match info.color_type {
        png::ColorType::Rgba => rgba.extend_from_slice(data),
        png::ColorType::Rgb => {
            for chunk in data.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
        }
        png::ColorType::Grayscale => {
            for value in data {
                rgba.extend_from_slice(&[*value, *value, *value, 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for chunk in data.chunks_exact(2) {
                rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
        }
        png::ColorType::Indexed => return Err("indexed png was not expanded".to_string()),
    }
    Ok((info.width, info.height, rgba))
}

fn encode_png_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Best);
        let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
        writer.write_image_data(rgba).map_err(|error| error.to_string())?;
    }
    Ok(output)
}

fn edge_background_from_rgba(rgba: &[u8], width: usize, height: usize) -> [f64; 3] {
    let mut samples: Vec<[u8; 3]> = Vec::new();
    let step = ((width * 2 + height * 2) / 1600).max(1);
    let mut add = |x: usize, y: usize| {
        let i = (y * width + x) * 4;
        if rgba.get(i + 3).copied().unwrap_or(0) < 220 {
            return;
        }
        samples.push([rgba[i], rgba[i + 1], rgba[i + 2]]);
    };
    for x in (0..width).step_by(step) {
        add(x, 0);
        add(x, height - 1);
    }
    for y in (0..height).step_by(step) {
        add(0, y);
        add(width - 1, y);
    }
    if samples.is_empty() {
        return [255.0, 255.0, 255.0];
    }
    samples.sort_by_key(|sample| sample[0] as u16 + sample[1] as u16 + sample[2] as u16);
    let start = samples.len() * 55 / 100;
    let pool = &samples[start..];
    let pool = if pool.is_empty() { &samples[..] } else { pool };
    let mut sum = [0.0, 0.0, 0.0];
    for sample in pool {
        sum[0] += sample[0] as f64;
        sum[1] += sample[1] as f64;
        sum[2] += sample[2] as f64;
    }
    [
        sum[0] / pool.len() as f64,
        sum[1] / pool.len() as f64,
        sum[2] / pool.len() as f64,
    ]
}

fn transparentize_fenbi_rgba(width: u32, height: u32, rgba: &mut [u8]) -> Result<Value, String> {
    let width_usize = width as usize;
    let height_usize = height as usize;
    let background = edge_background_from_rgba(rgba, width_usize, height_usize);
    let mut transparent = 0usize;
    let mut visible = 0usize;
    for pixel in rgba.chunks_exact_mut(4) {
        let r = pixel[0] as f64;
        let g = pixel[1] as f64;
        let b = pixel[2] as f64;
        let a = pixel[3];
        if a < 16 {
            continue;
        }
        let luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        let chroma = r.max(g).max(b) - r.min(g).min(b);
        let bg_distance = (r - background[0]).abs() + (g - background[1]).abs() + (b - background[2]).abs();
        let is_light_background = luminance > 188.0 && bg_distance < 132.0;
        let is_near_white = luminance > 218.0 && chroma < 58.0;
        if is_light_background || is_near_white {
            pixel[3] = 0;
            transparent += 1;
            continue;
        }
        let ink = ((luminance - 22.0) * 0.34).clamp(0.0, 210.0) as u8;
        pixel[0] = ink;
        pixel[1] = ink;
        pixel[2] = ink;
        pixel[3] = a.max(238);
        visible += 1;
    }
    if visible < (width_usize * height_usize).max(1) / 600 {
        return Err("processed image has too little visible content".to_string());
    }
    Ok(json!({
        "transparentRatio": transparent as f64 / (width_usize * height_usize).max(1) as f64,
        "visibleRatio": visible as f64 / (width_usize * height_usize).max(1) as f64
    }))
}

fn process_fenbi_image_url(app: AppHandle, image_url: String, referer: Option<String>) -> Result<Value, String> {
    let parsed = Url::parse(&image_url).map_err(|error| error.to_string())?;
    if parsed.scheme() != "https" {
        return Err("only https images are allowed".to_string());
    }
    let host = parsed.host_str().unwrap_or_default();
    if !is_allowed_fenbi_image_host(host) {
        return Err("image host is not allowed".to_string());
    }
    if let Some(data_url) = cache_get_fenbi_image(&app, &image_url) {
        return Ok(json!({ "ok": true, "dataUrl": data_url, "cached": true }));
    }
    let mut request = minreq::get(&image_url)
        .with_header("User-Agent", "Mozilla/5.0")
        .with_header("Accept", "image/avif,image/webp,image/apng,image/png,image/*,*/*;q=0.8")
        .with_timeout(8);
    if let Some(referer) = referer.as_ref().filter(|value| value.starts_with("https://")) {
        request = request.with_header("Referer", referer);
    }
    let response = request.send().map_err(|error| error.to_string())?;
    let status = response.status_code;
    if !(200..300).contains(&status) {
        return Err(format!("image request failed with status {status}"));
    }
    let bytes = response.as_bytes();
    if bytes.len() > FENBI_IMAGE_MAX_BYTES {
        return Err("image is too large".to_string());
    }
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("only png images are supported for now".to_string());
    }
    let (width, height, mut rgba) = decode_png_rgba(bytes)?;
    let stats = transparentize_fenbi_rgba(width, height, &mut rgba)?;
    let png = encode_png_rgba(width, height, &rgba)?;
    let data_url = format!("data:image/png;base64,{}", general_purpose::STANDARD.encode(png));
    cache_put_fenbi_image(&app, image_url, data_url.clone());
    Ok(json!({
        "ok": true,
        "dataUrl": data_url,
        "cached": false,
        "width": width,
        "height": height,
        "stats": stats
    }))
}

fn default_window_state(key: &str) -> WindowState {
    match key {
        "launcher" => WindowState {
            width: 820.0,
            height: 620.0,
            zoom_factor: None,
        },
        "transparentContent" => WindowState {
            width: 1320.0,
            height: 860.0,
            zoom_factor: Some(1.0),
        },
        "mentalMath" => WindowState {
            width: 760.0,
            height: 620.0,
            zoom_factor: None,
        },
        "pdfReader" => WindowState {
            width: 760.0,
            height: 680.0,
            zoom_factor: None,
        },
        "vocabulary" => WindowState {
            width: 1100.0,
            height: 760.0,
            zoom_factor: None,
        },
        _ => WindowState {
            width: 900.0,
            height: 700.0,
            zoom_factor: None,
        },
    }
}

fn default_theme() -> String {
    "mist".to_string()
}

fn default_expanded_resize_zone() -> bool {
    true
}

fn default_module_settings() -> ModuleSettings {
    ModuleSettings {
        content: default_content_module_settings(),
        mental_math: default_mental_math_module_settings(),
        pdf_reader: default_pdf_reader_module_settings(),
    }
}

fn default_plugin_settings() -> PluginSettings {
    PluginSettings {
        content: default_content_plugin_settings(),
    }
}

fn default_content_plugin_settings() -> ContentPluginSettings {
    ContentPluginSettings {
        fenbi_personalized: true,
    }
}

fn default_content_module_settings() -> ContentModuleSettings {
    ContentModuleSettings {
        transparent: false,
        universal_transparency: false,
        image_transparency: true,
    }
}

fn default_content_image_transparency() -> bool {
    true
}

fn default_mental_math_module_settings() -> MentalMathModuleSettings {
    MentalMathModuleSettings {
        transparent: true,
        weakness_focus: true,
        strong_text: true,
    }
}

fn default_pdf_reader_module_settings() -> PdfReaderModuleSettings {
    PdfReaderModuleSettings {
        transparent: true,
        monochrome: false,
        hide_images: false,
        night_mode: false,
        trim_margins: true,
    }
}

fn default_true() -> bool {
    true
}

fn normalize_plugin_settings(settings: &mut AppSettings) {
    settings.module_settings.content.image_transparency =
        settings.plugin_settings.content.fenbi_personalized;
}

#[cfg(target_os = "windows")]
fn native_resize_proc_map() -> &'static Mutex<HashMap<isize, isize>> {
    WINDOW_PROC_MAP.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(target_os = "windows")]
fn sync_resize_zone_setting(expanded: bool) {
    RESIZE_ZONE_EXPANDED.store(expanded, Ordering::Relaxed);
}

#[cfg(target_os = "windows")]
fn resize_border_metrics() -> (i32, i32) {
    let frame_x = unsafe { GetSystemMetrics(SM_CXSIZEFRAME) };
    let frame_y = unsafe { GetSystemMetrics(SM_CYSIZEFRAME) };
    let padding = unsafe { GetSystemMetrics(SM_CXPADDEDBORDER) };
    let base_x = (frame_x + padding).max(6);
    let base_y = (frame_y + padding).max(6);
    if RESIZE_ZONE_EXPANDED.load(Ordering::Relaxed) {
        (base_x.max(12), base_y.max(12))
    } else {
        (base_x, base_y)
    }
}

#[cfg(target_os = "windows")]
fn point_from_lparam(value: isize) -> (i32, i32) {
    let x = (value & 0xffff) as u16 as i16 as i32;
    let y = ((value >> 16) & 0xffff) as u16 as i16 as i32;
    (x, y)
}

#[cfg(target_os = "windows")]
fn nc_hit_test(hwnd: isize, lparam: isize) -> Option<isize> {
    let mut rect = RECT::default();
    let ok = unsafe { GetWindowRect(hwnd as *mut c_void, &mut rect) };
    if ok == 0 {
        return None;
    }
    let (x, y) = point_from_lparam(lparam);
    let (border_x, border_y) = resize_border_metrics();

    let left = x < rect.left + border_x;
    let right = x >= rect.right - border_x;
    let top = y < rect.top + border_y;
    let bottom = y >= rect.bottom - border_y;

    let hit = match (left, right, top, bottom) {
        (true, _, true, _) => HTTOPLEFT,
        (_, true, true, _) => HTTOPRIGHT,
        (true, _, _, true) => HTBOTTOMLEFT,
        (_, true, _, true) => HTBOTTOMRIGHT,
        (_, _, true, _) => HTTOP,
        (_, _, _, true) => HTBOTTOM,
        (true, _, _, _) => HTLEFT,
        (_, true, _, _) => HTRIGHT,
        _ => HTCLIENT,
    };

    if hit == HTCLIENT {
        None
    } else {
        Some(hit as isize)
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn native_resize_wndproc(
    hwnd: *mut c_void,
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> isize {
    if msg == WM_NCHITTEST {
        let mut hit_result = 0isize;
        if unsafe { DwmDefWindowProc(hwnd, msg, wparam, lparam, &mut hit_result) } != 0 {
            return hit_result;
        }
        if let Some(hit) = nc_hit_test(hwnd as isize, lparam) {
            return hit;
        }
    }

    let original = native_resize_proc_map()
        .lock()
        .ok()
        .and_then(|map| map.get(&(hwnd as isize)).copied())
        .unwrap_or(0);

    if original != 0 {
        unsafe { CallWindowProcW(std::mem::transmute::<isize, WNDPROC>(original), hwnd, msg, wparam, lparam) }
    } else {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }
}

#[cfg(target_os = "windows")]
fn install_native_resize_hit_test(hwnd: isize) {
    let mut map = match native_resize_proc_map().lock() {
        Ok(map) => map,
        Err(_) => return,
    };
    if map.contains_key(&hwnd) {
        return;
    }
    let original = unsafe { GetWindowLongPtrW(hwnd as *mut c_void, GWLP_WNDPROC) };
    if original == 0 {
        return;
    }
    let replaced = unsafe {
        SetWindowLongPtrW(
            hwnd as *mut c_void,
            GWLP_WNDPROC,
            native_resize_wndproc as *const () as usize as isize,
        )
    };
    if replaced != 0 || original != 0 {
        map.insert(hwnd, original);
    }
}

fn default_settings() -> AppSettings {
    let mut window_states = HashMap::new();
    window_states.insert(
        "transparentContent".to_string(),
        default_window_state("transparentContent"),
    );
    window_states.insert("launcher".to_string(), default_window_state("launcher"));
    window_states.insert("mentalMath".to_string(), default_window_state("mentalMath"));
    window_states.insert("pdfReader".to_string(), default_window_state("pdfReader"));
    window_states.insert("vocabulary".to_string(), default_window_state("vocabulary"));
    AppSettings {
        default_transparent: false,
        theme: "mist".to_string(),
        expanded_resize_zone: default_expanded_resize_zone(),
        titlebar_transparent_follow: false,
        module_settings: default_module_settings(),
        plugin_settings: default_plugin_settings(),
        window_states,
        launcher_compact_migrated: true,
    }
}

fn load_settings(app: &AppHandle) -> AppSettings {
    let path = match app_data_file(app, SETTINGS_FILE_NAME) {
        Ok(path) => path,
        Err(_) => return default_settings(),
    };
    let mut settings = read_json(&path, default_settings());
    let mut needs_save = false;
    for key in ["launcher", "transparentContent", "mentalMath", "pdfReader", "vocabulary"] {
        settings
            .window_states
            .entry(key.to_string())
            .or_insert_with(|| default_window_state(key));
    }
    if !settings.launcher_compact_migrated {
        settings
            .window_states
            .insert("launcher".to_string(), default_window_state("launcher"));
        settings.launcher_compact_migrated = true;
        needs_save = true;
    }
    if !["mist", "paper", "ink", "pine", "night"].contains(&settings.theme.as_str()) {
        settings.theme = "mist".to_string();
        needs_save = true;
    }
    normalize_plugin_settings(&mut settings);
    #[cfg(target_os = "windows")]
    sync_resize_zone_setting(settings.expanded_resize_zone);
    if needs_save {
        let _ = save_settings(app, &settings);
    }
    settings
}

fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = app_data_file(app, SETTINGS_FILE_NAME)?;
    write_json(&path, settings)
}

fn load_window_state(app: &AppHandle, key: &str) -> WindowState {
    let settings = load_settings(app);
    let fallback = default_window_state(key);
    let state = settings
        .window_states
        .get(key)
        .cloned()
        .unwrap_or(fallback.clone());
    let (min_width, min_height) = match key {
        "launcher" => (420.0, 380.0),
        "transparentContent" => (120.0, 120.0),
        "mentalMath" => (320.0, 240.0),
        "pdfReader" => (400.0, 300.0),
        _ => (320.0, 240.0),
    };
    WindowState {
        width: state.width.max(min_width),
        height: state.height.max(min_height),
        zoom_factor: state.zoom_factor.or(fallback.zoom_factor),
    }
}

fn save_window_state(
    app: &AppHandle,
    key: &str,
    window: Option<&WebviewWindow>,
    zoom_factor: Option<f64>,
) -> Result<WindowState, String> {
    let mut settings = load_settings(app);
    let mut state = settings
        .window_states
        .get(key)
        .cloned()
        .unwrap_or_else(|| default_window_state(key));
    if let Some(window) = window {
        if let Ok(size) = window.inner_size() {
            state.width = size.width as f64;
            state.height = size.height as f64;
        }
    }
    if let Some(zoom) = zoom_factor {
        state.zoom_factor = Some(zoom.clamp(0.5, 2.0));
    }
    settings.window_states.insert(key.to_string(), state.clone());
    save_settings(app, &settings)?;
    Ok(state)
}

fn save_window_state_for_window(
    app: &AppHandle,
    key: &str,
    window: Option<&Window>,
    zoom_factor: Option<f64>,
) -> Result<WindowState, String> {
    let mut settings = load_settings(app);
    let mut state = settings
        .window_states
        .get(key)
        .cloned()
        .unwrap_or_else(|| default_window_state(key));
    if let Some(window) = window {
        if let Ok(size) = window.inner_size() {
            state.width = size.width as f64 / window.scale_factor().unwrap_or(1.0);
            state.height = size.height as f64 / window.scale_factor().unwrap_or(1.0);
        }
    }
    if let Some(zoom) = zoom_factor {
        state.zoom_factor = Some(zoom.clamp(0.5, 2.0));
    }
    settings.window_states.insert(key.to_string(), state.clone());
    save_settings(app, &settings)?;
    Ok(state)
}

fn default_pdf_progress() -> PdfProgress {
    PdfProgress {
        bookmarks: HashMap::new(),
        visual_mode: DEFAULT_PDF_VISUAL_MODE.to_string(),
    }
}

fn load_pdf_progress(app: &AppHandle) -> PdfProgress {
    let path = match app_data_file(app, PDF_READER_PROGRESS_FILE_NAME) {
        Ok(path) => path,
        Err(_) => return default_pdf_progress(),
    };
    let mut progress = read_json(&path, default_pdf_progress());
    if !["default", "dark", "transparent"].contains(&progress.visual_mode.as_str()) {
        progress.visual_mode = DEFAULT_PDF_VISUAL_MODE.to_string();
    }
    progress
}

fn save_pdf_progress(app: &AppHandle, progress: &PdfProgress) -> Result<(), String> {
    let path = app_data_file(app, PDF_READER_PROGRESS_FILE_NAME)?;
    write_json(&path, progress)
}

fn default_mental_math_history() -> MentalMathHistory {
    MentalMathHistory { items: Vec::new() }
}

fn load_mental_math_history(app: &AppHandle) -> MentalMathHistory {
    let path = match app_data_file(app, MENTAL_MATH_HISTORY_FILE_NAME) {
        Ok(path) => path,
        Err(_) => return default_mental_math_history(),
    };
    let mut history = read_json(&path, default_mental_math_history());
    if history.items.len() > MENTAL_MATH_HISTORY_LIMIT {
        let keep_from = history.items.len() - MENTAL_MATH_HISTORY_LIMIT;
        history.items = history.items.split_off(keep_from);
    }
    history
}

fn save_mental_math_history(app: &AppHandle, history: &MentalMathHistory) -> Result<(), String> {
    let path = app_data_file(app, MENTAL_MATH_HISTORY_FILE_NAME)?;
    write_json(&path, history)
}

fn load_vocabulary_store(app: &AppHandle) -> VocabularyStore {
    let path = match app_data_file(app, VOCABULARY_STORE_FILE_NAME) {
        Ok(path) => path,
        Err(_) => return VocabularyStore::default(),
    };
    let mut store = read_json(&path, VocabularyStore::default());
    remove_blacklisted_entries_from_sources(&mut store);
    store
}

fn save_vocabulary_store(app: &AppHandle, store: &VocabularyStore) -> Result<(), String> {
    let path = app_data_file(app, VOCABULARY_STORE_FILE_NAME)?;
    write_json(&path, store)
}

fn normalize_vocabulary_term(value: &str) -> String {
    value.trim().to_string()
}

fn vocabulary_entry_term(entry: &Value) -> Option<String> {
    entry
        .get("term")
        .and_then(|value| value.as_str())
        .map(normalize_vocabulary_term)
        .filter(|value| !value.is_empty())
}

fn vocabulary_blacklist_set(store: &VocabularyStore) -> std::collections::HashSet<String> {
    store
        .blacklist
        .iter()
        .map(|term| normalize_vocabulary_term(term.as_str()))
        .filter(|term| !term.is_empty())
        .collect()
}

fn filter_vocabulary_entries(
    entries: Vec<Value>,
    blacklist: &std::collections::HashSet<String>,
) -> Vec<Value> {
    entries
        .into_iter()
        .filter(|entry| {
            vocabulary_entry_term(entry)
                .map(|term| !blacklist.contains(&term))
                .unwrap_or(true)
        })
        .collect()
}

fn remove_blacklisted_entries_from_sources(store: &mut VocabularyStore) {
    let blacklist = vocabulary_blacklist_set(store);
    if blacklist.is_empty() {
        return;
    }
    for source in &mut store.sources {
        let entries = std::mem::take(&mut source.entries);
        source.entries = filter_vocabulary_entries(entries, &blacklist);
    }
}

fn decode_txt(bytes: Vec<u8>) -> Result<String, String> {
    if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        return String::from_utf8(bytes[3..].to_vec())
            .map_err(|_| "TXT 文件不是有效的 UTF-8 文本".to_string());
    }
    if bytes.starts_with(&[0xff, 0xfe]) {
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&units)
            .map_err(|_| "TXT 文件不是有效的 UTF-16 LE 文本".to_string());
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        let units = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&units)
            .map_err(|_| "TXT 文件不是有效的 UTF-16 BE 文本".to_string());
    }
    String::from_utf8(bytes)
        .map_err(|_| "TXT 当前支持 UTF-8、UTF-8 BOM 或 UTF-16 编码".to_string())
}

fn parse_color(hex: &str) -> Option<Color> {
    let trimmed = hex.trim_start_matches('#');
    if trimmed.len() != 6 {
        return None;
    }
    let value = u32::from_str_radix(trimmed, 16).ok()?;
    Some(Color(
        ((value >> 16) & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        (value & 0xff) as u8,
        0xff,
    ))
}

fn normalize_url(raw_url: Option<String>) -> String {
    let Some(value) = raw_url.map(|value| value.trim().to_string()) else {
        return DEFAULT_URL.to_string();
    };
    if value.is_empty() {
        return DEFAULT_URL.to_string();
    }
    if let Ok(url) = Url::parse(&value) {
        return url.to_string();
    }
    if let Ok(url) = Url::parse(&format!("https://{value}")) {
        return url.to_string();
    }
    DEFAULT_URL.to_string()
}

fn content_webview_bounds(window: &Window) -> Option<(LogicalPosition<f64>, LogicalSize<f64>)> {
    let size = window.inner_size().ok()?;
    let scale = window.scale_factor().unwrap_or(1.0);
    let logical_width = size.width as f64 / scale;
    let logical_height = size.height as f64 / scale;
    Some((
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(logical_width.max(1.0), logical_height.max(1.0)),
    ))
}

fn resize_content_webview(app: &AppHandle, state: &RuntimeState) {
    let Some(window) = app.get_window("content") else {
        return;
    };
    let Some((content_position, content_size)) = content_webview_bounds(&window) else {
        return;
    };
    if let Ok(slot) = state.content_webview.lock() {
        if let Some(webview) = slot.as_ref() {
            let _ = webview.set_position(content_position);
            let _ = webview.set_size(content_size);
        }
    }
}

fn destroy_content_window(app: &AppHandle, state: &RuntimeState) {
    if let Ok(mut slot) = state.content_webview.lock() {
        if let Some(webview) = slot.take() {
            let _ = webview.close();
        }
    }
    if let Ok(mut slot) = state.content_transparent.lock() {
        *slot = None;
    }
    if let Some(window) = app.get_window("content") {
        let _ = save_window_state_for_window(app, "transparentContent", Some(&window), None);
        let _ = window.destroy();
    }
    clear_fenbi_image_cache(app);
}

fn show_launcher_window(app: &AppHandle) {
    if let Some(launcher) = app.get_webview_window("launcher") {
        let _ = save_window_state(app, "launcher", Some(&launcher), None);
        let _ = launcher.show();
        let _ = launcher.set_focus();
    }
}

fn hide_all_windows(app: &AppHandle) {
    for label in ["launcher", "mental_math", "pdf_reader", "vocabulary"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
    if let Some(window) = app.get_window("content") {
        let _ = window.hide();
    }
}

fn show_default_window(app: &AppHandle) {
    show_launcher_window(app);
}

fn toggle_all_windows_visibility(app: &AppHandle) {
    let state = app.state::<RuntimeState>();
    let mut hidden = match state.all_windows_hidden.lock() {
        Ok(hidden) => hidden,
        Err(_) => return,
    };
    if *hidden {
        show_default_window(app);
        *hidden = false;
    } else {
        hide_all_windows(app);
        *hidden = true;
    }
}

#[cfg(target_os = "windows")]
fn register_global_hotkeys(app: AppHandle) {
    std::thread::spawn(move || unsafe {
        const TOGGLE_ID: i32 = 0x4848;
        let registered = RegisterHotKey(
            std::ptr::null_mut(),
            TOGGLE_ID,
            MOD_CONTROL | MOD_ALT,
            b'H' as u32,
        );
        if registered == 0 {
            let _ = app_debug_log(app.clone(), "global hotkey register failed: Ctrl+Alt+H".to_string());
            return;
        }
        let _ = app_debug_log(app.clone(), "global hotkey registered: Ctrl+Alt+H".to_string());
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if msg.message == WM_HOTKEY && msg.wParam == TOGGLE_ID as usize {
                toggle_all_windows_visibility(&app);
            }
        }
    });
}

#[cfg(not(target_os = "windows"))]
fn register_global_hotkeys(_app: AppHandle) {}

fn reopen_content_window_later(app: AppHandle, target_url: String, transparent: bool) {
    tauri::async_runtime::spawn(async move {
        show_launcher_window(&app);
        std::thread::sleep(std::time::Duration::from_millis(60));
        let state = app.state::<RuntimeState>();
        destroy_content_window(&app, &state);
        std::thread::sleep(std::time::Duration::from_millis(20));
        let _ = viewer_open(app.clone(), state, Some(target_url), Some(transparent)).await;
    });
}

fn content_page_builder(
    app: AppHandle,
    target_url: &str,
    transparent: bool,
    fenbi_personalized: bool,
) -> Result<WebviewBuilder<Wry>, String> {
    let mut builder = WebviewBuilder::new(
        "content_page",
        WebviewUrl::External(Url::parse(target_url).map_err(|error| error.to_string())?),
    );

    if transparent {
        let log_app = app.clone();
        builder = builder
            .transparent(true)
            .background_color(Color(0x01, 0x00, 0x00, 0x00))
            .initialization_script(&content_init_script(target_url, transparent, fenbi_personalized))
            .on_document_title_changed(move |webview, title| {
                const LOG_PREFIX: &str = "__MOYU_CONTENT_LOG__";
                const IMAGE_PREFIX: &str = "__MOYU_CONTENT_IMAGE__";
                if let Some(message) = title.strip_prefix(LOG_PREFIX) {
                    let _ = app_debug_log(log_app.clone(), message.to_string());
                    return;
                }
                if let Some(payload) = title.strip_prefix(IMAGE_PREFIX) {
                    match serde_json::from_str::<FenbiImageRequest>(payload) {
                        Ok(request) => match process_fenbi_image_url(
                            log_app.clone(),
                            request.url.clone(),
                            request.referer.clone(),
                        ) {
                            Ok(result) => {
                                let data_url = result.get("dataUrl").and_then(|value| value.as_str());
                                eval_fenbi_image_result(&webview, &request.id, data_url, data_url.is_some());
                            }
                            Err(error) => {
                                let _ = app_debug_log(
                                    log_app.clone(),
                                    format!(
                                        "[fenbi-image-transparency] rust-process-error {} {}",
                                        request.url, error
                                    ),
                                );
                                eval_fenbi_image_result(&webview, &request.id, None, false);
                            }
                        },
                        Err(error) => {
                            let _ = app_debug_log(
                                log_app.clone(),
                                format!("[fenbi-image-transparency] invalid-image-request {}", error),
                            );
                        }
                    }
                }
            })
            .on_page_load({
                let url = target_url.to_string();
                move |webview, _payload| {
                    let _ = webview.eval(content_init_script(&url, transparent, fenbi_personalized));
                }
            });
    } else {
        builder = builder
            .transparent(false)
            .background_color(parse_color("#ffffff").unwrap_or(Color(0xff, 0xff, 0xff, 0xff)))
            .initialization_script(&content_init_script(target_url, transparent, fenbi_personalized))
            .on_page_load({
                let url = target_url.to_string();
                move |webview, _payload| {
                    let _ = webview.eval(content_init_script(&url, transparent, fenbi_personalized));
                }
            });
    }

    Ok(builder)
}

fn is_fenbi_url(target_url: &str) -> bool {
    Url::parse(target_url)
        .ok()
        .and_then(|url| url.host_str().map(|host| host == "www.fenbi.com" || host.ends_with(".fenbi.com")))
        .unwrap_or(false)
}

fn content_init_script(target_url: &str, transparent: bool, fenbi_personalized: bool) -> String {
    let is_fenbi = is_fenbi_url(target_url);
    let site_specific_css = if transparent && is_fenbi {
        if fenbi_personalized {
            format!("{FENBI_TRANSPARENCY_CSS}\n{FENBI_PERSONALIZED_CSS}")
        } else {
            FENBI_TRANSPARENCY_CSS.to_string()
        }
    } else {
        String::new()
    };
    let image_script = if transparent && is_fenbi && fenbi_personalized {
        FENBI_IMAGE_TRANSPARENCY_SCRIPT
    } else {
        ""
    };
    let css = if transparent { GENERIC_TRANSPARENCY_CSS } else { "" };
    format!(
        r#"
(() => {{
  const STYLE_ID = '__moyu_transparent_guard__';
  const css = {css:?} + {site_css:?};
  function install() {{
    let style = document.getElementById(STYLE_ID);
    if (!style) {{
      style = document.createElement('style');
      style.id = STYLE_ID;
      document.documentElement.appendChild(style);
    }}
    if (style.textContent !== css) {{
      style.textContent = css;
    }}
    document.documentElement.style.setProperty('background', 'transparent', 'important');
    document.documentElement.style.setProperty('background-color', 'transparent', 'important');
    if (document.body) {{
      document.body.style.setProperty('background', 'transparent', 'important');
      document.body.style.setProperty('background-color', 'transparent', 'important');
    }}
  }}
  install();
  if (!window.__moyuTransparentGuardInstalled) {{
    window.__moyuTransparentGuardInstalled = true;
    const observer = new MutationObserver(() => window.requestAnimationFrame(install));
    observer.observe(document.documentElement, {{
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['class', 'style']
    }});
    window.setInterval(install, 500);
  }}
}})();
{same_window_nav}
{image_script}
"#,
        css = css,
        site_css = site_specific_css,
        same_window_nav = UNIVERSAL_SAME_WINDOW_NAVIGATION_SCRIPT,
        image_script = image_script
    )
}

fn build_shell_window(
    app: &AppHandle,
    label: &str,
    route: &str,
    title: &str,
    state_key: &str,
    min_width: f64,
    min_height: f64,
    transparent: bool,
    opaque_color: Option<&str>,
) -> Result<WebviewWindow, String> {
    let use_native_chrome = cfg!(target_os = "windows");
    if let Some(window) = app.get_webview_window(label) {
        if window.menu().is_none() {
            if let Ok(menu) = build_native_window_menu(app) {
                let _ = window.set_menu(menu);
            }
        }
        let _ = window.show();
        let _ = window.set_focus();
        #[cfg(target_os = "windows")]
        if !use_native_chrome {
            if let Ok(hwnd) = window.hwnd() {
                install_native_resize_hit_test(hwnd.0 as isize);
            }
        }
        return Ok(window);
    }

    let state = load_window_state(app, state_key);
    let mut builder = WebviewWindowBuilder::new(
        app,
        label,
        WebviewUrl::App(format!("index.html#{route}").into()),
    )
    .title(title)
    .inner_size(state.width, state.height)
    .min_inner_size(min_width, min_height)
    .decorations(use_native_chrome)
    .transparent(transparent)
    .visible(true);

    if transparent {
        builder = builder
            .background_color(Color(0x01, 0x00, 0x00, 0x00))
            .transparent(true);
    } else if let Some(color) = opaque_color.and_then(parse_color) {
        builder = builder.background_color(color);
    }

    let window = builder.build().map_err(|error| error.to_string())?;
    if let Ok(menu) = build_native_window_menu(app) {
        let _ = window.set_menu(menu);
    }
    #[cfg(target_os = "windows")]
    if !use_native_chrome {
        if let Ok(hwnd) = window.hwnd() {
            install_native_resize_hit_test(hwnd.0 as isize);
        }
    }
    Ok(window)
}

fn build_native_window_menu(app: &AppHandle) -> tauri::Result<tauri::menu::Menu<Wry>> {
    let file = SubmenuBuilder::new(app, "&文件")
        .text(MENU_FILE_SHOW_LAUNCHER, "返回&首页")
        .text(MENU_FILE_OPEN_PDF, "打开 &PDF...")
        .separator()
        .text(MENU_FILE_CLOSE_WINDOW, "关闭当前窗口")
        .build()?;
    let settings = SubmenuBuilder::new(app, "&设置")
        .text(MENU_SETTINGS_OPEN, "&偏好")
        .separator()
        .text(MENU_SETTINGS_ZOOM_IN, "放大 (Zoom In)")
        .text(MENU_SETTINGS_ZOOM_OUT, "缩小 (Zoom Out)")
        .build()?;
    let help = SubmenuBuilder::new(app, "&帮助")
        .text(MENU_HELP_CONTACT, "&联系")
        .build()?;

    MenuBuilder::new(app)
        .item(&file)
        .item(&settings)
        .item(&help)
        .build()
}

fn focused_window_label(app: &AppHandle) -> String {
    for label in ["vocabulary", "pdf_reader", "mental_math", "launcher"] {
        if app
            .get_webview_window(label)
            .and_then(|window| window.is_focused().ok())
            .unwrap_or(false)
        {
            return label.to_string();
        }
    }
    if app
        .get_window("content")
        .and_then(|window| window.is_focused().ok())
        .unwrap_or(false)
    {
        return "content".to_string();
    }
    "launcher".to_string()
}

async fn open_pdf_with_picker(app: AppHandle, transparent: Option<bool>) -> Result<Value, String> {
    let picked = rfd::AsyncFileDialog::new()
        .set_title("打开 PDF 文件")
        .add_filter("PDF 文件", &["pdf"])
        .pick_file()
        .await;

    let Some(file) = picked else {
        return Ok(json!({ "ok": false }));
    };

    let path = file.path().to_string_lossy().to_string();
    let handle = app.clone();
    let state = handle.state::<RuntimeState>();
    let settings = load_settings(&app);
    let transparent = transparent.unwrap_or(settings.module_settings.pdf_reader.transparent);
    pdf_open_from_path(app, state, path, transparent).await
}

fn emit_native_menu_action(app: &AppHandle, label: &str, action: &str) {
    // Dispatch inside the target webview as well as through Tauri events. The
    // native menu is a Rust-owned surface, while the business action belongs
    // to the active page; direct dispatch keeps it working when event ACLs
    // reject frontend event delivery.
    if let Some(window) = app.get_webview_window(label) {
        if let Ok(action_json) = serde_json::to_string(action) {
            let script = format!(
                "window.dispatchEvent(new CustomEvent('local:native-menu-action', {{ detail: {{ action: {} }} }}));",
                action_json
            );
            let _ = window.eval(&script);
        }
    }
    let _ = app.emit_to(label, "native-menu-action", json!({ "action": action }));
}

fn now_stamp() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn emit_settings_changed(app: &AppHandle, settings: &AppSettings) {
    let _ = app.emit("app:settings-changed", settings.clone());
}

async fn apply_content_transparency_change(
    app: &AppHandle,
    enabled: bool,
) {
    let state = app.state::<RuntimeState>();
    if app.get_window("content").is_some() {
        let url = state
            .content_url
            .lock()
            .ok()
            .and_then(|slot| slot.clone())
            .unwrap_or_else(|| DEFAULT_URL.to_string());
        reopen_content_window_later(app.clone(), url, enabled);
    }
}

fn refresh_content_transparency_script(app: &AppHandle) {
    let state = app.state::<RuntimeState>();
    let settings = load_settings(app);
    if !settings.module_settings.content.transparent {
        return;
    }
    let url = state
        .content_url
        .lock()
        .ok()
        .and_then(|slot| slot.clone())
        .unwrap_or_else(|| DEFAULT_URL.to_string());
    {
        let slot = state.content_webview.lock();
        if let Ok(slot) = slot {
            if let Some(webview) = slot.as_ref() {
                let _ = webview.eval(content_init_script(
                    &url,
                    settings.module_settings.content.transparent,
                    settings.plugin_settings.content.fenbi_personalized,
                ));
            }
        }
    }
}

async fn apply_mental_math_transparency_change(app: &AppHandle, enabled: bool) {
    if let Some(window) = app.get_webview_window("mental_math") {
        let _ = save_window_state(app, "mentalMath", Some(&window), None);
        let _ = window.close();
        let _ = mental_math_open(app.clone(), Some(enabled)).await;
    }
}

async fn apply_pdf_transparency_change(
    app: &AppHandle,
    enabled: bool,
) {
    if let Some(window) = app.get_webview_window("pdf_reader") {
        let _ = save_window_state(app, "pdfReader", Some(&window), None);
        let color = if enabled {
            Color(0x01, 0x00, 0x00, 0x00)
        } else {
            parse_color("#ffffff").unwrap_or(Color(0xff, 0xff, 0xff, 0xff))
        };
        let _ = window.set_background_color(Some(color));
    }
}

#[tauri::command]
fn app_get_settings(app: AppHandle) -> AppSettings {
    load_settings(&app)
}

#[tauri::command]
fn app_set_theme(app: AppHandle, theme: String) -> Result<AppSettings, String> {
    if !["mist", "paper", "ink", "pine", "night"].contains(&theme.as_str()) {
        return Err("invalid theme".to_string());
    }
    let mut settings = load_settings(&app);
    if settings.theme != theme {
        settings.theme = theme;
        save_settings(&app, &settings)?;
    }
    emit_settings_changed(&app, &settings);
    Ok(settings)
}

#[tauri::command]
fn app_set_expanded_resize_zone(app: AppHandle, enabled: bool) -> Result<AppSettings, String> {
    let mut settings = load_settings(&app);
    if settings.expanded_resize_zone != enabled {
        settings.expanded_resize_zone = enabled;
        save_settings(&app, &settings)?;
    }
    #[cfg(target_os = "windows")]
    sync_resize_zone_setting(settings.expanded_resize_zone);
    emit_settings_changed(&app, &settings);
    Ok(settings)
}

#[tauri::command]
fn app_set_titlebar_transparent_follow(
    app: AppHandle,
    enabled: bool,
) -> Result<AppSettings, String> {
    let mut settings = load_settings(&app);
    if settings.titlebar_transparent_follow != enabled {
        settings.titlebar_transparent_follow = enabled;
        save_settings(&app, &settings)?;
    }
    emit_settings_changed(&app, &settings);
    Ok(settings)
}

#[tauri::command]
fn app_debug_log(app: AppHandle, message: String) -> Result<Value, String> {
    let path = app_data_file(&app, "debug.log")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    const MAX_DEBUG_LOG_BYTES: u64 = 2 * 1024 * 1024;
    if fs::metadata(&path)
        .map(|metadata| metadata.len() >= MAX_DEBUG_LOG_BYTES)
        .unwrap_or(false)
    {
        fs::write(&path, b"").map_err(|error| error.to_string())?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    let stamp = now_stamp();
    writeln!(file, "[{stamp}] {message}").map_err(|error| error.to_string())?;
    Ok(json!({ "ok": true }))
}

#[tauri::command]
async fn app_apply_transparency(
    app: AppHandle,
    state: tauri::State<'_, RuntimeState>,
    label: String,
    enabled: bool,
) -> Result<AppSettings, String> {
    let mut settings = load_settings(&app);
    if settings.default_transparent != enabled {
        settings.default_transparent = enabled;
        save_settings(&app, &settings)?;
    }
    emit_settings_changed(&app, &settings);

    match label.as_str() {
        "launcher" => {
            if let Some(window) = app.get_webview_window("launcher") {
                let color = if enabled {
                    Color(0x01, 0x00, 0x00, 0x00)
                } else {
                    parse_color("#f2f4f8").unwrap_or(Color(0xf2, 0xf4, 0xf8, 0xff))
                };
                let _ = window.set_background_color(Some(color));
            }
        }
        "content" => {
            if app.get_window("content").is_some() {
                let url = state
                    .content_url
                    .lock()
                    .map_err(|error| error.to_string())?
                    .clone()
                    .unwrap_or_else(|| DEFAULT_URL.to_string());
                reopen_content_window_later(app.clone(), url, enabled);
            }
        }
        "mental_math" => {}
        "pdf_reader" => {}
        _ => {}
    }

    Ok(settings)
}

#[tauri::command]
async fn app_set_module_setting(
    app: AppHandle,
    module_key: String,
    setting_key: String,
    enabled: bool,
) -> Result<AppSettings, String> {
    let mut settings = load_settings(&app);
    let mut changed = false;

    match (module_key.as_str(), setting_key.as_str()) {
        ("content", "transparent") => {
            if settings.module_settings.content.transparent != enabled {
                settings.module_settings.content.transparent = enabled;
                changed = true;
            }
        }
        ("content", "universalTransparency") => {
            if settings.module_settings.content.universal_transparency != enabled {
                settings.module_settings.content.universal_transparency = enabled;
                changed = true;
            }
        }
        ("content", "imageTransparency") => {
            if settings.module_settings.content.image_transparency != enabled {
                settings.module_settings.content.image_transparency = enabled;
                settings.plugin_settings.content.fenbi_personalized = enabled;
                changed = true;
            }
        }
        ("mentalMath", "transparent") => {
            if settings.module_settings.mental_math.transparent != enabled {
                settings.module_settings.mental_math.transparent = enabled;
                changed = true;
            }
        }
        ("mentalMath", "weaknessFocus") => {
            if settings.module_settings.mental_math.weakness_focus != enabled {
                settings.module_settings.mental_math.weakness_focus = enabled;
                changed = true;
            }
        }
        ("mentalMath", "strongText") => {
            if settings.module_settings.mental_math.strong_text != enabled {
                settings.module_settings.mental_math.strong_text = enabled;
                changed = true;
            }
        }
        ("pdfReader", "transparent") => {
            if settings.module_settings.pdf_reader.transparent != enabled {
                settings.module_settings.pdf_reader.transparent = enabled;
                changed = true;
            }
        }
        ("pdfReader", "monochrome") => {
            if settings.module_settings.pdf_reader.monochrome != enabled {
                settings.module_settings.pdf_reader.monochrome = enabled;
                changed = true;
            }
        }
        ("pdfReader", "hideImages") => {
            if settings.module_settings.pdf_reader.hide_images != enabled {
                settings.module_settings.pdf_reader.hide_images = enabled;
                changed = true;
            }
        }
        ("pdfReader", "nightMode") => {
            if settings.module_settings.pdf_reader.night_mode != enabled {
                settings.module_settings.pdf_reader.night_mode = enabled;
                changed = true;
            }
        }
        ("pdfReader", "trimMargins") => {
            if settings.module_settings.pdf_reader.trim_margins != enabled {
                settings.module_settings.pdf_reader.trim_margins = enabled;
                changed = true;
            }
        }
        _ => return Err("unsupported module setting".to_string()),
    }

    if changed {
        save_settings(&app, &settings)?;
    }
    emit_settings_changed(&app, &settings);

    match (module_key.as_str(), setting_key.as_str()) {
        ("content", "transparent") => apply_content_transparency_change(&app, enabled).await,
        ("content", "imageTransparency") => refresh_content_transparency_script(&app),
        ("mentalMath", "transparent") => apply_mental_math_transparency_change(&app, enabled).await,
        ("pdfReader", "transparent") => apply_pdf_transparency_change(&app, enabled).await,
        _ => {}
    }

    Ok(settings)
}

#[tauri::command]
async fn app_set_plugin_setting(
    app: AppHandle,
    plugin_key: String,
    enabled: bool,
) -> Result<AppSettings, String> {
    let mut settings = load_settings(&app);
    let mut changed = false;

    match plugin_key.as_str() {
        "content.fenbiPersonalized" => {
            if settings.plugin_settings.content.fenbi_personalized != enabled {
                settings.plugin_settings.content.fenbi_personalized = enabled;
                settings.module_settings.content.image_transparency = enabled;
                changed = true;
            }
        }
        _ => return Err("unsupported plugin setting".to_string()),
    }

    if changed {
        save_settings(&app, &settings)?;
    }
    emit_settings_changed(&app, &settings);

    if plugin_key == "content.fenbiPersonalized" {
        apply_content_transparency_change(&app, settings.module_settings.content.transparent).await;
    }

    Ok(settings)
}

#[tauri::command]
fn window_set_transparency_mode(
    app: AppHandle,
    label: String,
    enabled: bool,
) -> Result<Value, String> {
    if label == "content" {
        tauri::async_runtime::spawn(async move {
            apply_content_transparency_change(&app, enabled).await;
        });
        return Ok(json!({ "ok": true }));
    }
    if label == "mental_math" {
        tauri::async_runtime::spawn(async move {
            apply_mental_math_transparency_change(&app, enabled).await;
        });
        return Ok(json!({ "ok": true }));
    }
    if label == "pdf_reader" {
        tauri::async_runtime::spawn(async move {
            apply_pdf_transparency_change(&app, enabled).await;
        });
        return Ok(json!({ "ok": true }));
    }
    if let Some(window) = app.get_webview_window(&label) {
        let color = if enabled {
            Color(0x01, 0x00, 0x00, 0x00)
        } else {
            parse_color("#f6f8fb").unwrap_or(Color(0xf6, 0xf8, 0xfb, 0xff))
        };
        let _ = window.set_background_color(Some(color));
    }
    Ok(json!({ "ok": true }))
}

#[tauri::command]
fn content_get_window_state(app: AppHandle) -> WindowState {
    load_window_state(&app, "transparentContent")
}

#[tauri::command]
fn content_set_zoom_factor(app: AppHandle, zoom_factor: f64) -> Result<Value, String> {
    let next = zoom_factor.clamp(0.5, 2.0);
    let state = app.state::<RuntimeState>();
    if let Ok(slot) = state.content_webview.lock() {
        if let Some(webview) = slot.as_ref() {
            webview.set_zoom(next).map_err(|error| error.to_string())?;
        }
    }
    let _ = save_window_state_for_window(
        &app,
        "transparentContent",
        app.get_window("content").as_ref(),
        Some(next),
    )?;
    Ok(json!({ "ok": true, "zoomFactor": next }))
}

#[tauri::command]
async fn viewer_open(
    app: AppHandle,
    state: tauri::State<'_, RuntimeState>,
    target_url: Option<String>,
    transparent: Option<bool>,
) -> Result<Value, String> {
    let url = normalize_url(target_url);
    let previous_url = state.content_url.lock().ok().and_then(|slot| slot.clone());
    if previous_url.as_deref() != Some(url.as_str()) {
        clear_fenbi_image_cache(&app);
    }
    {
        let mut slot = state.content_url.lock().map_err(|error| error.to_string())?;
        *slot = Some(url.clone());
    }

    let settings = load_settings(&app);
    let transparent = transparent.unwrap_or(settings.module_settings.content.transparent);
    let fenbi_personalized = settings.plugin_settings.content.fenbi_personalized;
    let should_rebuild = state
        .content_transparent
        .lock()
        .map(|slot| slot.map(|current| current != transparent).unwrap_or(false))
        .unwrap_or(false);
    if should_rebuild {
        destroy_content_window(&app, &state);
    }

    let window = if let Some(window) = app.get_window("content") {
        if window.menu().is_none() {
            if let Ok(menu) = build_native_window_menu(&app) {
                let _ = window.set_menu(menu);
            }
        }
        if let Ok(slot) = state.content_webview.lock() {
            if let Some(webview) = slot.as_ref() {
                let _ = webview.set_background_color(Some(if transparent {
                    Color(0x01, 0x00, 0x00, 0x00)
                } else {
                    parse_color("#ffffff").unwrap_or(Color(0xff, 0xff, 0xff, 0xff))
                }));
                webview
                    .navigate(Url::parse(&url).map_err(|error| error.to_string())?)
                    .map_err(|error| error.to_string())?;
                if transparent {
                    let _ = webview.eval(content_init_script(&url, transparent, fenbi_personalized));
                }
            }
        }
        window
    } else {
        let saved = load_window_state(&app, "transparentContent");
        let mut builder = WindowBuilder::new(&app, "content")
            .title("玻璃网页")
            .inner_size(saved.width, saved.height)
            .min_inner_size(120.0, 120.0)
            .decorations(true)
            .visible(false)
            .transparent(transparent);
        if transparent {
            builder = builder.background_color(Color(0x01, 0x00, 0x00, 0x00));
        } else if let Some(color) = parse_color("#ffffff") {
            builder = builder.background_color(color);
        }
        let window = builder.build().map_err(|error| error.to_string())?;
        if let Ok(menu) = build_native_window_menu(&app) {
            let _ = window.set_menu(menu);
        }
        let (position, size) = content_webview_bounds(&window)
            .unwrap_or((LogicalPosition::new(0.0, 0.0), LogicalSize::new(800.0, 600.0)));
        let webview = window
            .add_child(
                content_page_builder(app.clone(), &url, transparent, fenbi_personalized)?,
                position,
                size,
            )
            .map_err(|error| error.to_string())?;
        {
            let mut slot = state.content_webview.lock().map_err(|error| error.to_string())?;
            *slot = Some(webview);
        }
        {
            let mut slot = state
                .content_transparent
                .lock()
                .map_err(|error| error.to_string())?;
            *slot = Some(transparent);
        }
        window
    };
    let window_state = load_window_state(&app, "transparentContent");
    if let Ok(slot) = state.content_webview.lock() {
        if let Some(webview) = slot.as_ref() {
            let _ = webview.set_zoom(window_state.zoom_factor.unwrap_or(1.0));
        }
    }
    resize_content_webview(&app, &state);
    if let Some(launcher) = app.get_webview_window("launcher") {
        let _ = launcher.hide();
    }
    let _ = window.show();
    let _ = window.set_focus();
    Ok(json!({ "ok": true }))
}

#[tauri::command]
async fn mental_math_open(app: AppHandle, transparent: Option<bool>) -> Result<Value, String> {
    let settings = load_settings(&app);
    let transparent = transparent.unwrap_or(settings.module_settings.mental_math.transparent);
    let window = build_shell_window(
        &app,
        "mental_math",
        "/mental-math",
        "速算训练",
        "mentalMath",
        320.0,
        240.0,
        transparent,
        Some("#f6f8fb"),
    )?;
    if let Some(launcher) = app.get_webview_window("launcher") {
        let _ = launcher.hide();
    }
    let _ = window.show();
    Ok(json!({ "ok": true }))
}

#[tauri::command]
async fn vocabulary_pick_txt() -> Result<Value, String> {
    let picked = rfd::AsyncFileDialog::new()
        .set_title("选择言语积累 TXT")
        .add_filter("TXT 文本", &["txt"])
        .pick_file()
        .await;

    let Some(file) = picked else {
        return Ok(json!({ "ok": false }));
    };
    let path = file.path();
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.len() > VOCABULARY_TXT_MAX_BYTES {
        return Err("TXT 文件不能超过 8 MB".to_string());
    }
    let text = decode_txt(fs::read(path).map_err(|error| error.to_string())?)?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("未命名.txt")
        .to_string();
    Ok(json!({
        "ok": true,
        "path": path.to_string_lossy(),
        "fileName": file_name,
        "text": text
    }))
}

#[tauri::command]
fn vocabulary_save_source(app: AppHandle, payload: VocabularySourcePayload) -> Result<Value, String> {
    if payload.path.trim().is_empty() || payload.file_name.trim().is_empty() {
        return Err("来源文件信息不完整".to_string());
    }
    let id = payload.path.to_lowercase();
    let mut store = load_vocabulary_store(&app);
    let blacklist = vocabulary_blacklist_set(&store);
    let source = VocabularySource {
        id: id.clone(),
        path: payload.path,
        file_name: payload.file_name,
        imported_at: now_stamp(),
        char_count: payload.char_count,
        sentence_count: payload.sentence_count,
        entries: filter_vocabulary_entries(payload.entries, &blacklist),
    };
    if let Some(existing) = store.sources.iter_mut().find(|item| item.id == id) {
        *existing = source;
    } else {
        store.sources.push(source);
    }
    if store.sources.len() > VOCABULARY_SOURCE_LIMIT {
        let remove_count = store.sources.len() - VOCABULARY_SOURCE_LIMIT;
        store.sources.drain(0..remove_count);
    }
    save_vocabulary_store(&app, &store)?;
    Ok(json!({
        "ok": true,
        "sources": store.sources,
        "blacklist": store.blacklist
    }))
}

#[tauri::command]
async fn vocabulary_list(app: AppHandle) -> Value {
    let _ = app_debug_log(app.clone(), "[vocabulary] list:start".to_string());
    let store = load_vocabulary_store(&app);
    let entry_count: usize = store.sources.iter().map(|source| source.entries.len()).sum();
    let _ = app_debug_log(
        app.clone(),
        format!(
            "[vocabulary] list:complete sources={} entries={entry_count}",
            store.sources.len()
        ),
    );
    json!({
        "ok": true,
        "sources": store.sources,
        "blacklist": store.blacklist
    })
}

#[tauri::command]
fn vocabulary_remove_source(app: AppHandle, source_id: String) -> Result<Value, String> {
    let mut store = load_vocabulary_store(&app);
    store.sources.retain(|item| item.id != source_id);
    save_vocabulary_store(&app, &store)?;
    Ok(json!({
        "ok": true,
        "sources": store.sources,
        "blacklist": store.blacklist
    }))
}

#[tauri::command]
fn vocabulary_blacklist_term(app: AppHandle, term: String) -> Result<Value, String> {
    let normalized = normalize_vocabulary_term(&term);
    if normalized.is_empty() {
        return Err("词语不能为空".to_string());
    }
    let mut store = load_vocabulary_store(&app);
    if !store.blacklist.iter().any(|item| normalize_vocabulary_term(item) == normalized) {
        store.blacklist.push(normalized.clone());
    }
    remove_blacklisted_entries_from_sources(&mut store);
    save_vocabulary_store(&app, &store)?;
    Ok(json!({
        "ok": true,
        "sources": store.sources,
        "blacklist": store.blacklist
    }))
}

#[tauri::command]
fn vocabulary_restore_term(app: AppHandle, term: String) -> Result<Value, String> {
    let normalized = normalize_vocabulary_term(&term);
    if normalized.is_empty() {
        return Err("词语不能为空".to_string());
    }
    let mut store = load_vocabulary_store(&app);
    let original_len = store.blacklist.len();
    store
        .blacklist
        .retain(|item| normalize_vocabulary_term(item) != normalized);
    if store.blacklist.len() == original_len {
        return Err("词语不在黑名单中".to_string());
    }
    save_vocabulary_store(&app, &store)?;
    Ok(json!({
        "ok": true,
        "sources": store.sources,
        "blacklist": store.blacklist
    }))
}

#[tauri::command]
async fn vocabulary_open(app: AppHandle) -> Result<Value, String> {
    let _ = app_debug_log(app.clone(), "[vocabulary] open:start".to_string());
    let launcher = app.get_webview_window("launcher");
    let window = match build_shell_window(
        &app,
        "vocabulary",
        "/vocabulary",
        "言语词语积累",
        "vocabulary",
        720.0,
        520.0,
        false,
        Some("#f6f8fb"),
    ) {
        Ok(window) => {
            let _ = app_debug_log(app.clone(), "[vocabulary] open:window-ready".to_string());
            window
        }
        Err(error) => {
            let _ = app_debug_log(
                app.clone(),
                format!("[vocabulary] open:window-error {error}"),
            );
            if let Some(window) = launcher {
                let _ = window.show();
                let _ = window.set_focus();
            }
            return Err(error);
        }
    };
    if let Some(window) = launcher.as_ref() {
        let _ = window.hide();
    }
    let _ = window.show();
    let _ = window.set_focus();
    let _ = app_debug_log(app, "[vocabulary] open:complete".to_string());
    Ok(json!({ "ok": true }))
}

async fn pdf_open_from_path(
    app: AppHandle,
    state: tauri::State<'_, RuntimeState>,
    path: String,
    transparent: bool,
) -> Result<Value, String> {
    {
        let mut slot = state.pdf_path.lock().map_err(|error| error.to_string())?;
        *slot = Some(path.clone());
    }

    let window = build_shell_window(
        &app,
        "pdf_reader",
        "/pdf-reader",
        "PDF 阅读",
        "pdfReader",
        400.0,
        300.0,
        transparent,
        Some("#ffffff"),
    )?;
    let _ = window.emit("pdf:path-changed", path.clone());
    if let Some(launcher) = app.get_webview_window("launcher") {
        let _ = launcher.hide();
    }
    let _ = window.show();
    Ok(json!({ "ok": true, "path": path }))
}

#[tauri::command]
async fn pdf_open(
    app: AppHandle,
    state: tauri::State<'_, RuntimeState>,
    transparent: Option<bool>,
) -> Result<Value, String> {
    let _ = state;
    open_pdf_with_picker(app, transparent).await
}

#[tauri::command]
fn pdf_get_path(state: tauri::State<'_, RuntimeState>) -> Result<Option<String>, String> {
    Ok(state
        .pdf_path
        .lock()
        .map_err(|error| error.to_string())?
        .clone())
}

#[tauri::command]
fn pdf_save_progress(app: AppHandle, payload: PdfSavePayload) -> Result<Value, String> {
    let mut progress = load_pdf_progress(&app);
    progress.bookmarks.insert(
        payload.file_path,
        PdfBookmark {
            page: payload.page,
            total_pages: payload.total_pages,
            last_read: now_stamp(),
        },
    );
    save_pdf_progress(&app, &progress)?;
    Ok(json!({ "ok": true }))
}

#[tauri::command]
fn pdf_load_progress(app: AppHandle, file_path: String) -> Value {
    let progress = load_pdf_progress(&app);
    json!({
        "visualMode": progress.visual_mode,
        "bookmark": progress.bookmarks.get(&file_path)
    })
}

#[tauri::command]
fn pdf_save_visual_mode(app: AppHandle, mode: String) -> Result<Value, String> {
    if !["default", "dark", "transparent"].contains(&mode.as_str()) {
        return Ok(json!({ "ok": false }));
    }
    let mut progress = load_pdf_progress(&app);
    progress.visual_mode = mode;
    save_pdf_progress(&app, &progress)?;
    Ok(json!({ "ok": true }))
}

#[tauri::command]
fn read_pdf_file(file_path: String) -> Result<PdfReadResult, String> {
    let data = fs::read(&file_path).map_err(|error| error.to_string())?;
    Ok(PdfReadResult { ok: true, data })
}

#[tauri::command]
fn mental_math_history_list(app: AppHandle) -> Value {
    let history = load_mental_math_history(&app);
    json!({
        "ok": true,
        "items": history.items
    })
}

#[tauri::command]
fn mental_math_history_append(
    app: AppHandle,
    payload: MentalMathHistorySavePayload,
) -> Result<Value, String> {
    let mut history = load_mental_math_history(&app);
    history.items.push(MentalMathHistoryItem {
        id: format!("{}-{}", now_stamp(), history.items.len() + 1),
        timestamp: now_stamp(),
        mode_key: payload.mode_key,
        mode_label: payload.mode_label,
        question_text: payload.question_text,
        correct_answer: payload.correct_answer,
        user_answer: payload.user_answer,
        is_correct: payload.is_correct,
        elapsed_ms: payload.elapsed_ms,
        weakness_focus_enabled: payload.weakness_focus_enabled,
        from_weakness_focus: payload.from_weakness_focus,
        features: payload.features,
    });
    if history.items.len() > MENTAL_MATH_HISTORY_LIMIT {
        let keep_from = history.items.len() - MENTAL_MATH_HISTORY_LIMIT;
        history.items = history.items.split_off(keep_from);
    }
    save_mental_math_history(&app, &history)?;
    Ok(json!({
        "ok": true,
        "count": history.items.len()
    }))
}

#[tauri::command]
fn window_set_inner_size(
    app: AppHandle,
    label: String,
    width: f64,
    height: f64,
) -> Result<Value, String> {
    if let Some(window) = app.get_webview_window(&label) {
        window
            .set_size(LogicalSize::new(width, height))
            .map_err(|error| error.to_string())?;
    }
    Ok(json!({ "ok": true }))
}

#[tauri::command]
fn window_close(app: AppHandle, label: String) -> Result<Value, String> {
    if label == "content" {
        let state = app.state::<RuntimeState>();
        destroy_content_window(&app, &state);
        show_launcher_window(&app);
        return Ok(json!({ "ok": true }));
    }

    if let Some(window) = app.get_webview_window(&label) {
        let _ = match label.as_str() {
            "mental_math" => save_window_state(&app, "mentalMath", Some(&window), None),
            "pdf_reader" => save_window_state(&app, "pdfReader", Some(&window), None),
            "vocabulary" => save_window_state(&app, "vocabulary", Some(&window), None),
            _ => Ok(default_window_state("unknown")),
        };
        let _ = window.close();
    }
    Ok(json!({ "ok": true }))
}

pub fn run() {
    tauri::Builder::default()
        .manage(RuntimeState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            #[cfg(target_os = "windows")]
            sync_resize_zone_setting(true);
            if let Some(window) = app.get_webview_window("launcher") {
                if let Ok(menu) = build_native_window_menu(&handle) {
                    let _ = window.set_menu(menu);
                }
                let state = load_window_state(&handle, "launcher");
                let width = state.width.max(820.0);
                let height = state.height.max(560.0);
                let _ = window.set_min_size(Some(LogicalSize::new(820.0, 560.0)));
                let _ = window.set_size(LogicalSize::new(width, height));
            }
            register_global_hotkeys(handle);
            Ok(())
        })
        .on_menu_event(|app, event| {
            let action = event.id().as_ref();
            let label = focused_window_label(app);
            match action {
                MENU_FILE_SHOW_LAUNCHER => show_default_window(app),
                MENU_FILE_OPEN_PDF => {
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = open_pdf_with_picker(app_handle, None).await;
                    });
                }
                MENU_FILE_CLOSE_WINDOW => {
                    let _ = window_close(app.clone(), label);
                }
                MENU_SETTINGS_OPEN => emit_native_menu_action(app, &label, "settings-open"),
                MENU_HELP_CONTACT => emit_native_menu_action(app, &label, "contact-open"),
                // zoom 只作用于 pdf_reader 窗口：直接定向投递（与已验证的偏好/联系同机制），
                // 避免 focused_window_label 在 Windows 菜单交互时误判焦点导致事件发错窗口。
                MENU_SETTINGS_ZOOM_IN => emit_native_menu_action(app, "pdf_reader", "pdf-zoom-in"),
                MENU_SETTINGS_ZOOM_OUT => emit_native_menu_action(app, "pdf_reader", "pdf-zoom-out"),
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            app_get_settings,
            app_set_theme,
            app_set_expanded_resize_zone,
            app_set_titlebar_transparent_follow,
            app_debug_log,
            app_apply_transparency,
            app_set_module_setting,
            app_set_plugin_setting,
            window_set_transparency_mode,
            viewer_open,
            content_get_window_state,
            content_set_zoom_factor,
            mental_math_open,
            pdf_open,
            pdf_get_path,
            pdf_save_progress,
            pdf_load_progress,
            pdf_save_visual_mode,
            read_pdf_file,
            mental_math_history_list,
            mental_math_history_append,
            vocabulary_pick_txt,
            vocabulary_save_source,
            vocabulary_list,
            vocabulary_remove_source,
            vocabulary_blacklist_term,
            vocabulary_restore_term,
            vocabulary_open,
            window_set_inner_size,
            window_close
        ])
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::Resized(_) => {
                let app = window.app_handle();
                match window.label() {
                    "launcher" => {
                        let webview_window = app.get_webview_window("launcher");
                        let _ = save_window_state(&app, "launcher", webview_window.as_ref(), None);
                    }
                    "content" => {
                        let content_window = app.get_window("content");
                        let _ = save_window_state_for_window(&app, "transparentContent", content_window.as_ref(), None);
                        let state = app.state::<RuntimeState>();
                        resize_content_webview(&app, &state);
                    }
                    "mental_math" => {
                        let webview_window = app.get_webview_window("mental_math");
                        let _ = save_window_state(&app, "mentalMath", webview_window.as_ref(), None);
                    }
                    "pdf_reader" => {
                        let webview_window = app.get_webview_window("pdf_reader");
                        let _ = save_window_state(&app, "pdfReader", webview_window.as_ref(), None);
                    }
                    "vocabulary" => {
                        let webview_window = app.get_webview_window("vocabulary");
                        let _ = save_window_state(&app, "vocabulary", webview_window.as_ref(), None);
                    }
                    _ => {}
                }
            }
            tauri::WindowEvent::CloseRequested { .. } => {
                let app = window.app_handle();
                match window.label() {
                    "launcher" => {
                        let webview_window = app.get_webview_window("launcher");
                        let _ = save_window_state(&app, "launcher", webview_window.as_ref(), None);
                    }
                    "content" => {
                        let content_window = app.get_window("content");
                        let _ = save_window_state_for_window(&app, "transparentContent", content_window.as_ref(), None);
                        let state = app.state::<RuntimeState>();
                        if let Ok(mut slot) = state.content_webview.lock() {
                            if let Some(webview) = slot.take() {
                                let _ = webview.close();
                            }
                        }
                        if let Ok(mut slot) = state.content_transparent.lock() {
                            *slot = None;
                        }
                        show_launcher_window(&app);
                    }
                    "mental_math" => {
                        let webview_window = app.get_webview_window("mental_math");
                        let _ = save_window_state(&app, "mentalMath", webview_window.as_ref(), None);
                        show_launcher_window(&app);
                    }
                    "pdf_reader" => {
                        let webview_window = app.get_webview_window("pdf_reader");
                        let _ = save_window_state(&app, "pdfReader", webview_window.as_ref(), None);
                        if app.get_webview_window("mental_math").is_none() {
                            show_launcher_window(&app);
                        }
                    }
                    "vocabulary" => {
                        let webview_window = app.get_webview_window("vocabulary");
                        let _ = save_window_state(&app, "vocabulary", webview_window.as_ref(), None);
                        show_launcher_window(&app);
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

