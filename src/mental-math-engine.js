const DIVISION_THRESHOLDS = [60, 75, 90];
const FAST_MENTAL_THRESHOLDS = [24, 30, 38];
const ESTIMATION_THRESHOLDS = [32, 40, 48];

function randomInt(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function canonicalFact(a, b) {
  const [left, right] = [Number(a), Number(b)].sort((x, y) => x - y);
  return `${left}x${right}`;
}

function bandLabel(value, size) {
  const lower = Math.floor(Number(value) / size) * size;
  return `${lower}-${lower + size - 1}`;
}

function feature(name, value) {
  return `${name}=${value}`;
}

function additionCarries(a, b) {
  let carries = 0;
  let carry = 0;
  let left = a;
  let right = b;
  for (let i = 0; i < 3; i += 1) {
    const total = (left % 10) + (right % 10) + carry;
    if (total >= 10) {
      carries += 1;
      carry = 1;
    } else {
      carry = 0;
    }
    left = Math.floor(left / 10);
    right = Math.floor(right / 10);
  }
  return carries;
}

function subtractionBorrows(a, b) {
  let borrows = 0;
  let borrow = 0;
  let left = a;
  let right = b;
  for (let i = 0; i < 3; i += 1) {
    const leftDigit = (left % 10) - borrow;
    const rightDigit = right % 10;
    if (leftDigit < rightDigit) {
      borrows += 1;
      borrow = 1;
    } else {
      borrow = 0;
    }
    left = Math.floor(left / 10);
    right = Math.floor(right / 10);
  }
  return borrows;
}

function remainderKind(a, b) {
  const remainder = a % b;
  if (remainder === 0) return "exact";
  const edge = Math.min(remainder, b - remainder) / b;
  if (edge <= 0.05) return "near_integer";
  const ratio = remainder / b;
  if (ratio >= 0.33 && ratio <= 0.66) return "middle_remainder";
  return "edge_remainder";
}

function question(qText, answer, features) {
  return {
    qText,
    answer,
    metadata: { features }
  };
}

function makeMode1Question() {
  const a = randomInt(10000, 99999);
  const b = randomInt(100, 999);
  const answer = a / b;
  return question(`${a} / ${b}`, answer, [
    feature("dividend_10k", bandLabel(a, 10000)),
    feature("dividend_5k", bandLabel(a, 5000)),
    feature("divisor_100", bandLabel(b, 100)),
    feature("divisor_50", bandLabel(b, 50)),
    feature("divisor_last", b % 10),
    feature("quotient_50", bandLabel(Math.floor(answer), 50)),
    feature("remainder", remainderKind(a, b))
  ]);
}

function makeMode2Question() {
  let a = randomInt(100, 999);
  let b = randomInt(100, 999);
  if (Math.random() < 0.5) {
    const answer = a + b;
    const carries = additionCarries(a, b);
    const transition = carries === 0 ? "no_carry" : carries === 1 ? "single_carry" : "multi_carry";
    return question(`${a} + ${b}`, answer, [
      feature("operation", "add"),
      feature("left_100", bandLabel(a, 100)),
      feature("right_100", bandLabel(b, 100)),
      feature("result_100", bandLabel(answer, 100)),
      feature("carries", carries),
      feature("transition", transition)
    ]);
  }
  if (a < b) [a, b] = [b, a];
  const answer = a - b;
  const borrows = subtractionBorrows(a, b);
  const transition = borrows === 0 ? "no_borrow" : borrows === 1 ? "single_borrow" : "multi_borrow";
  return question(`${a} - ${b}`, answer, [
    feature("operation", "subtract"),
    feature("left_100", bandLabel(a, 100)),
    feature("right_100", bandLabel(b, 100)),
    feature("result_100", bandLabel(answer, 100)),
    feature("borrows", borrows),
    feature("transition", transition)
  ]);
}

function makeMode3Question() {
  const a = randomInt(10, 99);
  const b = randomInt(2, 9);
  const answer = a * b;
  const onesProduct = (a % 10) * b;
  const tensProduct = Math.floor(a / 10) * b;
  const unitCarryAmount = Math.floor(onesProduct / 10);
  return question(`${a} * ${b}`, answer, [
    feature("multiplicand_10", bandLabel(a, 10)),
    feature("multiplicand_tens", Math.floor(a / 10)),
    feature("multiplicand_ones", a % 10),
    feature("multiplier", b),
    feature("ones_fact", canonicalFact(a % 10, b)),
    feature("tens_fact", canonicalFact(Math.floor(a / 10), b)),
    feature("full_fact", `${a}x${b}`),
    feature("unit_carry", unitCarryAmount > 0 ? "yes" : "no"),
    feature("unit_carry_amount", unitCarryAmount),
    feature("tens_product_10", bandLabel(tensProduct, 10)),
    feature("cross_100", answer >= 100 ? "yes" : "no"),
    feature("result_50", bandLabel(answer, 50))
  ]);
}

function makeMode4Question() {
  const a = randomInt(100, 999);
  const b = randomInt(2, 9);
  const answer = a / b;
  return question(`${a} / ${b}`, answer, [
    feature("dividend_100", bandLabel(a, 100)),
    feature("dividend_50", bandLabel(a, 50)),
    feature("divisor", b),
    feature("quotient_25", bandLabel(Math.floor(answer), 25)),
    feature("remainder", remainderKind(a, b))
  ]);
}

function makeMode5QuestionWithFixedTeen(fixedTeen = null) {
  const a = fixedTeen == null ? randomInt(11, 19) : Number(fixedTeen);
  const b = randomInt(2, 9);
  const answer = a * b;
  const ones = a % 10;
  const onesProduct = ones * b;
  const unitCarryAmount = Math.floor(onesProduct / 10);
  return question(`${a} * ${b}`, answer, [
    feature("teen_value", a),
    feature("teen_ones", ones),
    feature("multiplier", b),
    feature("ones_fact", canonicalFact(ones, b)),
    feature("teen_fact", `${a}x${b}`),
    feature("unit_carry", unitCarryAmount > 0 ? "yes" : "no"),
    feature("unit_carry_amount", unitCarryAmount),
    feature("teen_offset", a - 10),
    feature("distance_to_20", 20 - a),
    feature("result_25", bandLabel(answer, 25))
  ]);
}

function estimateBand(value) {
  if (value < 20) return "10-19";
  if (value < 40) return "20-39";
  if (value < 60) return "40-59";
  if (value < 80) return "60-79";
  return "80-99";
}

function makeMode6Question() {
  const a = randomInt(10, 99);
  const b = randomInt(10, 99);
  const answer = a * b;
  const roundedA = Math.round(a / 10) * 10;
  const roundedB = Math.round(b / 10) * 10;
  return question(`${a} * ${b}`, answer, [
    feature("left_band", estimateBand(a)),
    feature("right_band", estimateBand(b)),
    feature("left_round_10", roundedA),
    feature("right_round_10", roundedB),
    feature("cross_1000", answer >= 1000 ? "yes" : "no"),
    feature("result_500", bandLabel(answer, 500))
  ]);
}

const MODE_SPECS = {
  mode1: { maker: makeMode1Question },
  mode2: { maker: makeMode2Question },
  mode3: { maker: makeMode3Question },
  mode4: { maker: makeMode4Question },
  mode5_mix: { maker: () => makeMode5QuestionWithFixedTeen() },
  mode5_16: { maker: () => makeMode5QuestionWithFixedTeen(16) },
  mode5_17: { maker: () => makeMode5QuestionWithFixedTeen(17) },
  mode5_18: { maker: () => makeMode5QuestionWithFixedTeen(18) },
  mode5_19: { maker: () => makeMode5QuestionWithFixedTeen(19) },
  mode6: { maker: makeMode6Question }
};

export const MODES = [
  { key: "mode1", label: "五位数除三位数", tolerance: 0.03, thresholds: DIVISION_THRESHOLDS, grading: true },
  { key: "mode2", label: "三位数加减", tolerance: 0, thresholds: null, grading: false },
  { key: "mode3", label: "两位数乘一位数", tolerance: 0, thresholds: FAST_MENTAL_THRESHOLDS, grading: true },
  { key: "mode4", label: "三位数除一位数", tolerance: 0.03, thresholds: FAST_MENTAL_THRESHOLDS, grading: true },
  { key: "mode5_mix", label: "11-19 乘一位数", tolerance: 0, thresholds: FAST_MENTAL_THRESHOLDS, grading: true },
  { key: "mode5_16", label: "16 乘一位数", tolerance: 0, thresholds: FAST_MENTAL_THRESHOLDS, grading: true },
  { key: "mode5_17", label: "17 乘一位数", tolerance: 0, thresholds: FAST_MENTAL_THRESHOLDS, grading: true },
  { key: "mode5_18", label: "18 乘一位数", tolerance: 0, thresholds: FAST_MENTAL_THRESHOLDS, grading: true },
  { key: "mode5_19", label: "19 乘一位数", tolerance: 0, thresholds: FAST_MENTAL_THRESHOLDS, grading: true },
  { key: "mode6", label: "2*2估算", tolerance: 0.05, thresholds: ESTIMATION_THRESHOLDS, grading: true }
];

export function emptyStats() {
  return { version: 1, modes: {} };
}

function getModeStats(stats, modeKey) {
  const modes = stats.modes || (stats.modes = {});
  const modeStats = modes[modeKey] || (modes[modeKey] = {});
  modeStats.attempts = Number(modeStats.attempts || 0);
  modeStats.correct = Number(modeStats.correct || 0);
  modeStats.total_time = Number(modeStats.total_time || 0);
  modeStats.features = modeStats.features && typeof modeStats.features === "object" ? modeStats.features : {};
  modeStats.recent_questions = Array.isArray(modeStats.recent_questions) ? modeStats.recent_questions : [];
  modeStats.last_sessions = Array.isArray(modeStats.last_sessions) ? modeStats.last_sessions : [];
  return modeStats;
}

function weightedChoice(items) {
  const total = items.reduce((sum, [, weight]) => sum + weight, 0);
  if (total <= 0) return null;
  let pick = Math.random() * total;
  for (const [value, weight] of items) {
    pick -= weight;
    if (pick <= 0) return value;
  }
  return items[items.length - 1][0];
}

function featureFocusBonus(modeKey, featureName) {
  if (modeKey === "mode3") {
    if (featureName.startsWith("ones_fact=") || featureName.startsWith("tens_fact=")) return 1.8;
    if (featureName.startsWith("unit_carry=") || featureName.startsWith("unit_carry_amount=")) return 1.35;
    if (featureName.startsWith("multiplier=") || featureName.startsWith("multiplicand_tens=")) return 1.15;
  }
  if (modeKey.startsWith("mode5")) {
    if (featureName.startsWith("teen_fact=") || featureName.startsWith("ones_fact=")) return 1.9;
    if (featureName.startsWith("unit_carry=") || featureName.startsWith("unit_carry_amount=")) return 1.4;
    if (featureName.startsWith("teen_value=") || featureName.startsWith("multiplier=")) return 1.2;
  }
  return 1;
}

function weakTargetsForMode(modeKey, modeStats) {
  const targets = [];
  for (const [name, data] of Object.entries(modeStats.features || {})) {
    const weakness = Number(data.weakness || 0);
    const attempts = Number(data.attempts || 0);
    if (attempts > 0 && weakness >= 0.75) {
      targets.push([name, weakness * featureFocusBonus(modeKey, name)]);
    }
  }
  targets.sort((a, b) => b[1] - a[1]);
  return targets.slice(0, 20);
}

function generateQuestionForTarget(modeKey, target, recentQuestions, sessionSeen) {
  const maker = MODE_SPECS[modeKey].maker;
  for (const allowRecent of [false, true]) {
    for (const allowSessionRepeat of [false, true]) {
      for (let i = 0; i < 300; i += 1) {
        const next = maker();
        if (!allowSessionRepeat && sessionSeen.has(next.qText)) continue;
        if (!allowRecent && recentQuestions.has(next.qText)) continue;
        if (target == null || next.metadata.features.includes(target)) return next;
      }
    }
  }
  return maker();
}

export function generateAdaptiveQuestions(modeKey, count, stats) {
  const modeStats = getModeStats(stats, modeKey);
  const weakTargets = weakTargetsForMode(modeKey, modeStats);
  const recentQuestions = new Set(modeStats.recent_questions || []);
  const sessionSeen = new Set();
  const questions = [];
  for (let index = 0; index < count; index += 1) {
    let target = null;
    if (weakTargets.length && Math.random() < 0.7) {
      target = weightedChoice(weakTargets);
    }
    const next = generateQuestionForTarget(modeKey, target, recentQuestions, sessionSeen);
    sessionSeen.add(next.qText);
    questions.push(next);
  }
  return questions;
}

function median(values) {
  if (!values.length) return 0;
  const ordered = [...values].sort((a, b) => a - b);
  const middle = Math.floor(ordered.length / 2);
  if (ordered.length % 2) return ordered[middle];
  return (ordered[middle - 1] + ordered[middle]) / 2;
}

export function evaluateAnswer(answer, userAnswer, tolerance) {
  const parsed = Number(userAnswer);
  if (!Number.isFinite(parsed)) return { userAnswer: null, isCorrect: false };
  if (tolerance > 0) {
    const lower = answer * (1 - tolerance);
    const upper = answer * (1 + tolerance);
    return { userAnswer: parsed, isCorrect: parsed >= lower && parsed <= upper };
  }
  return { userAnswer: parsed, isCorrect: Math.abs(parsed - answer) < 1e-9 };
}

export function updateStats(modeKey, responses, stats, totalElapsed) {
  const modeStats = getModeStats(stats, modeKey);
  const elapsedValues = responses.map((item) => item.elapsed);
  const referenceTime = Math.max(0.2, median(elapsedValues));
  modeStats.attempts += responses.length;
  modeStats.correct += responses.filter((item) => item.isCorrect).length;
  modeStats.total_time += totalElapsed;
  const featureStats = modeStats.features || (modeStats.features = {});
  const recentQuestions = modeStats.recent_questions || (modeStats.recent_questions = []);
  for (const response of responses) {
    const ratio = response.elapsed / referenceTime;
    let eventScore = 0;
    if (!response.isCorrect) eventScore += 3;
    if (ratio > 1.2) eventScore += Math.min(3, (ratio - 1) * 1.5);
    else if (response.isCorrect && ratio < 0.9) eventScore -= 0.35;
    for (const featureName of response.metadata?.features || []) {
      const data = featureStats[featureName] || (featureStats[featureName] = {
        attempts: 0,
        wrong: 0,
        slow: 0,
        total_time: 0,
        weakness: 0
      });
      data.attempts += 1;
      data.total_time += response.elapsed;
      if (!response.isCorrect) data.wrong += 1;
      if (ratio > 1.2) data.slow += 1;
      data.weakness = Math.max(0, Number(data.weakness || 0) * 0.88 + eventScore);
    }
    recentQuestions.push(response.qText);
  }
  if (recentQuestions.length > 500) recentQuestions.splice(0, recentQuestions.length - 500);
}

export function scaledThresholds(totalQuestions, thresholds) {
  const base = totalQuestions / 10;
  return thresholds.map((threshold) => Math.trunc(threshold * base));
}

export function getGrade(elapsed, totalQuestions, thresholds = DIVISION_THRESHOLDS) {
  const [excellent, good, passing] = scaledThresholds(totalQuestions, thresholds);
  if (elapsed <= excellent) return "Excellent";
  if (elapsed <= good) return "Good";
  if (elapsed <= passing) return "Pass";
  return "Fail";
}

export function formatTime(seconds) {
  const minutes = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return minutes ? `${minutes}m ${secs.toFixed(2)}s` : `${secs.toFixed(2)}s`;
}

function describeMode3Feature(name) {
  const [key, value] = name.split("=");
  const mapping = {
    ones_fact: `个位乘法 ${value}`,
    tens_fact: `十位乘法 ${value}`,
    full_fact: `完整算式 ${value}`,
    multiplier: `乘数 ${value}`,
    multiplicand_tens: `十位数字 ${value}`,
    multiplicand_ones: `个位数字 ${value}`,
    multiplicand_10: `两位数区间 ${value}`,
    unit_carry: `个位进位 ${value === "yes" ? "有" : "无"}`,
    unit_carry_amount: `进位量 ${value}`,
    tens_product_10: `十位乘积区间 ${value}`,
    cross_100: `跨百 ${value === "yes" ? "是" : "否"}`,
    result_50: `结果区间 ${value}`
  };
  return mapping[key] || name;
}

function describeMode5Feature(name) {
  const [key, value] = name.split("=");
  const mapping = {
    teen_value: `十几数 ${value}`,
    teen_ones: `个位数字 ${value}`,
    multiplier: `乘数 ${value}`,
    ones_fact: `基础乘法 ${value}`,
    teen_fact: `完整算式 ${value}`,
    unit_carry: `个位进位 ${value === "yes" ? "有" : "无"}`,
    unit_carry_amount: `进位量 ${value}`,
    teen_offset: `距 10 偏移 ${value}`,
    distance_to_20: `距 20 距离 ${value}`,
    result_25: `结果区间 ${value}`
  };
  return mapping[key] || name;
}

function describeMode6Feature(name) {
  const [key, value] = name.split("=");
  const mapping = {
    left_band: `左因数区间 ${value}`,
    right_band: `右因数区间 ${value}`,
    left_round_10: `左因数十位估算 ${value}`,
    right_round_10: `右因数十位估算 ${value}`,
    cross_1000: `是否过千 ${value === "yes" ? "是" : "否"}`,
    result_500: `结果区间 ${value}`
  };
  return mapping[key] || name;
}

export function weaknessSummary(modeKey, stats) {
  const modeStats = getModeStats(stats, modeKey);
  const weakTargets = weakTargetsForMode(modeKey, modeStats).slice(0, 5);
  if (!weakTargets.length) return ["弱项样本还不够，下一轮会保持随机覆盖。"];
  return weakTargets.map(([name, score]) => {
    const label = modeKey === "mode3"
      ? describeMode3Feature(name)
      : modeKey.startsWith("mode5")
        ? describeMode5Feature(name)
        : modeKey === "mode6"
          ? describeMode6Feature(name)
          : name;
    return `${label}，弱项 ${score.toFixed(2)}`;
  });
}
