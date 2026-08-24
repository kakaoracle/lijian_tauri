import { aggregateVocabularySources, extractVocabulary } from "./vocabulary-engine.js";

self.addEventListener("message", (event) => {
  try {
    if (event.data?.type === "aggregate") {
      const result = aggregateVocabularySources(event.data.sources);
      self.postMessage({ type: "aggregate-complete", result });
      return;
    }
    if (event.data?.type !== "extract") return;
    const result = extractVocabulary(event.data.text, {
      onProgress(progress) {
        self.postMessage({ type: "progress", ...progress });
      }
    });
    self.postMessage({ type: "complete", result });
  } catch (error) {
    self.postMessage({
      type: "error",
      message: error instanceof Error ? error.message : String(error)
    });
  }
});
