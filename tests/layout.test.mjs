import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const css = readFileSync(new URL("../dist/styles.css", import.meta.url), "utf8");
const app = readFileSync(new URL("../dist/app.js", import.meta.url), "utf8");
const worker = readFileSync(new URL("../dist/vocabulary-worker.js", import.meta.url), "utf8");
const tauriConfig = readFileSync(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8");
const rust = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

function rule(selector) {
  const escaped = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return css.match(new RegExp(`${escaped}\\s*\\{([^}]+)\\}`))?.[1] || "";
}

const launcherBody = rule("body.launcher");
const launcherApp = rule("body.launcher #app");
const launcherMain = rule("body.launcher .launcher-main");
const vocabularyBody = rule("body.vocabulary");
const vocabularyShell = rule(".vocabulary-shell");
const settingsOverlay = rule(".settings-overlay");

assert.match(launcherBody, /display:\s*block/);
assert.doesNotMatch(launcherBody, /grid-template-rows/);
assert.match(launcherApp, /height:\s*100%/);
assert.doesNotMatch(launcherApp, /grid-template-rows/);
assert.match(launcherMain, /overflow:\s*auto/);
assert.match(vocabularyBody, /overflow:\s*auto/);
assert.match(vocabularyShell, /overflow:\s*auto/);
assert.match(css, /body\.mental-math #app,\s*body\.vocabulary #app,\s*body\.pdf-reader #app\s*\{[\s\S]*?overflow:\s*hidden/);
assert.match(settingsOverlay, /inset:\s*0/);
assert.match(app, /class="launcher-dashboard"/);
assert.equal(app.match(/<section class="launcher-module(?:\s|")/g)?.length, 4);
assert.doesNotMatch(app, /备考工作台|公务员考试|<h2>网页入口<\/h2>/);
assert.doesNotMatch(app, /launcher-quick-actions|launcher-workspace/);
assert.doesNotMatch(app, /renderTitlebar|activateTitlebar|content_set_titlebar_overlay|\/content-shell/);
assert.doesNotMatch(app, /launcher-settings|launcher-contact|mental-back|mental-settings|mental-contact|pdf-settings|pdf-contact|pdf-back|close-vocabulary-page|vocabulary-settings|vocabulary-contact/);
assert.match(app, /native-menu-action/);
assert.match(app, /settings-open/);
assert.match(app, /contact-open/);
assert.match(app, /id="vocabulary-progress"/);
assert.match(app, /id="cancel-vocabulary-extraction"/);
assert.match(app, /new Worker\(new URL\("\.\/vocabulary-worker\.js"/);
assert.match(worker, /type:\s*"progress"/);
assert.match(app, /const pageSize = 100/);
assert.match(app, /id="vocabulary-page-prev"/);
assert.match(app, /viewVocabulary:\s*"打开词库"/);
assert.match(app, /type: "aggregate"/);
assert.match(app, /import \{ aggregateVocabularySources, VOCABULARY_CATEGORIES \}/);
assert.match(app, /id="vocabulary-blacklist-panel"/);
assert.match(app, /invoke\("vocabulary_blacklist_term"/);
assert.match(app, /invoke\("vocabulary_restore_term"/);
assert.match(app, /data-delete-term=/);
assert.match(app, /item\.synonymNote \|\| ""/);
assert.match(css, /\.term-delete-btn\s*\{/);
assert.match(css, /\.blacklist-chip\s*\{/);
assert.match(rust, /MENU_SETTINGS_OPEN/);
assert.match(rust, /MENU_HELP_CONTACT/);
assert.match(rust, /build_native_window_menu/);
assert.match(rust, /window\.set_menu\(menu\)/);
assert.match(tauriConfig, /"label": "launcher"[\s\S]*?"decorations": true/);
assert.match(tauriConfig, /"label": "launcher"[\s\S]*?"width": 820[\s\S]*?"height": 620/);
assert.match(rust, /launcher_compact_migrated:\s*bool/);
assert.match(rust, /if !settings\.launcher_compact_migrated/);
assert.match(rust, /insert\("launcher"\.to_string\(\), default_window_state\("launcher"\)\)/);

const extractionHandler = app.match(/extractVocabularyBtn\.addEventListener\("click",[\s\S]+?openVocabularyBtn\.addEventListener/)?.[0] || "";
assert.doesNotMatch(extractionHandler, /invoke\("vocabulary_open"/);

console.log("layout tests passed");
