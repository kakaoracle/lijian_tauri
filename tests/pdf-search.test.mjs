import assert from "node:assert/strict";
import fs from "node:fs";

const appSource = fs.readFileSync(new URL("../dist/app.js", import.meta.url), "utf8");
const styles = fs.readFileSync(new URL("../dist/styles.css", import.meta.url), "utf8");

const routeSource = appSource.slice(
  appSource.indexOf("async function renderPdfReader"),
  appSource.indexOf("const ROUTES")
);

assert.match(routeSource, /id="pdf-search"/, "PDF reader must expose an inline search entry");
assert.match(routeSource, /id="pdf-search-field"/, "Search must provide a query input");
assert.match(routeSource, /id="pdf-search-status"/, "Search must report match progress");
assert.match(routeSource, /id="pdf-search-prev"/, "Search must allow stepping backwards");
assert.match(routeSource, /id="pdf-search-next"/, "Search must allow stepping forwards");
assert.match(routeSource, /id="pdf-search-close"/, "Search must be dismissible");

assert.match(appSource, /async function getPageSearchData\s*\(/, "Search must extract text per page");
assert.match(appSource, /getTextContent\(\)/, "Search must use the PDF text layer");
assert.match(appSource, /searchTextCache\.set\(pageNumber/, "Extracted page text must be cached");
assert.match(appSource, /getDocument\(\{ data: new Uint8Array\(file\.data\) \}\)/, "Document loading must stay intact");
assert.match(appSource, /function ensurePageLoaded\s*\(/, "Page loading must remain incremental");

const searchBody = appSource.slice(
  appSource.indexOf("async function performSearch"),
  appSource.indexOf("function scheduleSearch")
);
assert.match(searchBody, /toLowerCase\(\)/, "Matching must be case-insensitive");
assert.match(searchBody, /TEXT\.pdf\.searchIndexing/, "Long indexing runs must expose visible stage feedback");
assert.match(searchBody, /setTimeout\(resolve, 0\)/, "Indexing must yield so the UI thread stays responsive");
assert.match(searchBody, /searchSessionToken/, "Stale async searches must be invalidated");
assert.match(searchBody, />= 2000/, "Result count must be bounded");

assert.match(appSource, /pdf-highlight-layer/, "Matches must render into a dedicated highlight layer");
assert.match(appSource, /function renderShellHighlights\s*\(/, "Highlights must be redrawn per page shell");
assert.match(appSource, /function refreshHighlights\s*\(/, "Visible highlights must refresh after layout changes");
assert.match(appSource, /function focusMatch\s*\(/, "Search must navigate to the focused match");
assert.match(appSource, /debugPdf\("search:focus"/, "Match navigation must leave diagnostic evidence");
assert.match(appSource, /clearSearchResults\(\{ keepCache: false \}\)/, "Loading another document must drop cached text");
assert.match(appSource, /if \(searchMatches\.length\) refreshHighlights\(\)/, "Scroll-driven renders must realign highlights");
assert.match(appSource, /if \(searchMatches\.length\) renderShellHighlights\(state\)/, "Margin trimming must realign that page's highlights");

assert.match(appSource, /openSearchBar\s*\(/, "Search must be openable");
assert.match(appSource, /closeSearchBar\s*\(/, "Search must be closable");
assert.match(appSource, /event\.key === "f" \|\| event\.key === "F"/, "Ctrl+F must open search");
assert.match(appSource, /stepMatch\(event\.shiftKey \? -1 : 1\)/, "Enter and Shift+Enter must step through matches");
assert.match(appSource, /event\.key === "F3"/, "F3 must repeat the last search step");

assert.match(styles, /\.pdf-search\s*\{/, "Search bar styling must exist");
assert.match(styles, /\.pdf-search\[hidden\]\s*\{[^}]*display:\s*none/, "Hidden search bar must not intercept pointer events");
assert.match(styles, /\.pdf-mark\s*\{/, "Match marks must be styled");
assert.match(styles, /\.pdf-mark-current\s*\{/, "The active match must be visually distinct");
assert.match(styles, /\.pdf-page-shell\s*\{[^}]*position:\s*relative/, "Page shells must anchor highlight overlays");
const searchStyles = styles.slice(styles.indexOf(".pdf-highlight-layer"));
assert.doesNotMatch(searchStyles.slice(0, searchStyles.indexOf("body.pdf-reader.mode-dark .pdf-search-btn:hover") + 200), /#ff0000|#00b4ff/, "Search visuals must stay restrained");

console.log("pdf-search tests passed");
