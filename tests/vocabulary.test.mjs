import assert from "node:assert/strict";
import fs from "node:fs";
import { aggregateVocabularySources, extractVocabulary } from "../src/vocabulary-engine.js";

const text = [
  "完善基层治理机制，能够提升治理效能。",
  "不仅要完善制度，而且要夯实治理基础。",
  "文化传承并非一蹴而就，而是在潜移默化中形成共识。",
  "推进基层治理，需要因地制宜，持续提升治理效能。",
  "新闻报道里的改革口号很多，但这类泛词本身不应作为词库重点。"
].join("\n");

const progress = [];
const result = extractVocabulary(text, {
  onProgress(value) {
    progress.push(value);
  }
});

assert.equal(progress.at(-1).percent, 100);
assert.equal(progress.at(-1).stage, "提取完成");
assert.ok(progress.every((value, index) => index === 0 || value.percent >= progress[index - 1].percent));

const byTerm = new Map(result.entries.map((item) => [item.term, item]));
assert.equal(byTerm.get("治理").count, 5);
assert.equal(byTerm.get("不仅").category, "虚词·关联词");
assert.equal(byTerm.get("一蹴而就").category, "成语");
assert.equal(byTerm.get("推进").synonymNote.includes("近义"), true);
assert.deepEqual(byTerm.get("推进").examples.length, 1);
assert.equal(byTerm.get("完善").examples[0].includes("完善"), true);
assert.equal(byTerm.has("改革"), false);
assert.equal(byTerm.has("习近"), false);
assert.deepEqual(byTerm.get("治理").collocations, []);

const aggregate = aggregateVocabularySources([
  { fileName: "甲.txt", entries: result.entries },
  {
    fileName: "乙.txt",
    entries: [
      {
        ...byTerm.get("治理"),
        count: 2,
        collocations: [{ text: "治理效能", count: 1 }],
        examples: ["治理效能仍需持续提升。"]
      }
    ]
  }
]);

const governance = aggregate.find((item) => item.term === "治理");
assert.equal(governance.totalCount, 7);
assert.equal(governance.sourceCount, 2);
assert.equal(governance.weight, 9);
assert.equal(governance.examples.length, 1);
assert.equal(governance.synonymNote, "");
assert.ok(governance.collocations.some((item) => item.text === "治理效能"));

const noisyResult = extractVocabulary("改革发展改革发展习近平全面深化改革".repeat(100));
const noisyTerms = new Set(noisyResult.entries.map((item) => item.term));
assert.equal(noisyTerms.has("改革"), false);
assert.equal(noisyTerms.has("习近平"), false);
assert.equal(noisyTerms.has("全面深化改革"), false);

const launcherScript = fs.readFileSync(new URL("../dist/app.js", import.meta.url), "utf8");
const rustSource = fs.readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
assert.match(launcherScript, /invokeWithTimeout\("vocabulary_list"/);
assert.match(launcherScript, /词库整理超过 15 秒/);
assert.match(launcherScript, /aggregateWorker = null/);
assert.match(launcherScript, /debugLog\("vocabulary-ui"/);
assert.match(rustSource, /DwmDefWindowProc/);
assert.match(rustSource, /\.decorations\(true\)/);
assert.doesNotMatch(rustSource, /content_shell_webview/);
assert.doesNotMatch(rustSource, /content_set_titlebar_overlay/);
assert.match(rustSource, /async fn vocabulary_open\(/);
assert.match(rustSource, /\[vocabulary\] open:window-ready/);
assert.doesNotMatch(rustSource, /\nfn vocabulary_open\(/);
assert.match(rustSource, /blacklist:\s*Vec<String>/);
assert.match(rustSource, /fn vocabulary_blacklist_term\(/);
assert.match(rustSource, /fn vocabulary_restore_term\(/);
assert.match(rustSource, /filter_vocabulary_entries\(payload\.entries,\s*&blacklist\)/);
assert.match(rustSource, /window_states\.insert\("vocabulary"\.to_string\(\), default_window_state\("vocabulary"\)\)/);

console.log("vocabulary tests passed");
