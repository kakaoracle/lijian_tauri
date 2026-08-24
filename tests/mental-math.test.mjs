import assert from "node:assert/strict";
import { MODES, emptyStats, evaluateAnswer, generateAdaptiveQuestions, getGrade, scaledThresholds, updateStats, weaknessSummary } from "../src/mental-math-engine.js";

const stats = emptyStats();
const mode = MODES.find((item) => item.key === "mode3");
const questions = generateAdaptiveQuestions("mode3", 20, stats);
assert.equal(questions.length, 20);
assert.ok(questions.every((item) => typeof item.qText === "string" && Array.isArray(item.metadata.features)));

const result = evaluateAnswer(12, "12", mode.tolerance);
assert.equal(result.isCorrect, true);
assert.equal(result.userAnswer, 12);

const wrong = evaluateAnswer(12, "13", mode.tolerance);
assert.equal(wrong.isCorrect, false);

updateStats("mode3", [
  { qText: "12 * 3", answer: 36, userAnswer: 36, isCorrect: true, elapsed: 1.2, metadata: questions[0].metadata },
  { qText: "13 * 4", answer: 52, userAnswer: 50, isCorrect: false, elapsed: 2.5, metadata: questions[1].metadata }
], stats, 3.7);

const weak = weaknessSummary("mode3", stats);
assert.ok(Array.isArray(weak) && weak.length >= 1);

const thresholds = scaledThresholds(20, mode.thresholds);
assert.deepEqual(thresholds, [48, 60, 76]);
assert.equal(getGrade(40, 20, mode.thresholds), "Excellent");
assert.equal(getGrade(55, 20, mode.thresholds), "Good");
assert.equal(getGrade(70, 20, mode.thresholds), "Pass");
assert.equal(getGrade(90, 20, mode.thresholds), "Fail");

const estimateMode = MODES.find((item) => item.key === "mode6");
assert.ok(estimateMode);
const estimateQuestions = generateAdaptiveQuestions("mode6", 10, emptyStats());
assert.equal(estimateQuestions.length, 10);
assert.equal(evaluateAnswer(1000, "1049", estimateMode.tolerance).isCorrect, true);
assert.equal(evaluateAnswer(1000, "1051", estimateMode.tolerance).isCorrect, false);
assert.deepEqual(scaledThresholds(30, estimateMode.thresholds), [96, 120, 144]);
assert.equal(getGrade(95, 30, estimateMode.thresholds), "Excellent");
assert.equal(getGrade(110, 30, estimateMode.thresholds), "Good");
assert.equal(getGrade(140, 30, estimateMode.thresholds), "Pass");
assert.equal(getGrade(150, 30, estimateMode.thresholds), "Fail");

console.log("mental-math.test.mjs passed");
