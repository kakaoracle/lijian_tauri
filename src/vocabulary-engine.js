const EXAM_TERMS = {
  "实词·动词": [
    "推动", "推进", "促进", "提升", "提高", "增强", "增进", "改善", "改进", "完善", "健全",
    "建立", "构建", "形成", "实现", "保障", "维护", "保护", "治理", "规范", "遏制", "抑制",
    "制约", "约束", "引导", "激发", "释放", "培育", "塑造", "彰显", "凸显", "体现", "反映",
    "揭示", "阐释", "诠释", "契合", "符合", "顺应", "适应", "回应", "应对", "消除", "化解",
    "缓解", "解决", "填补", "弥补", "拓展", "拓宽", "扩大", "延伸", "延续", "传承", "弘扬",
    "汲取", "吸收", "借鉴", "融合", "整合", "统筹", "协调", "兼顾", "夯实", "筑牢", "巩固"
  ],
  "实词·形容词": [
    "深刻", "深入", "显著", "突出", "鲜明", "严峻", "紧迫", "迫切", "艰巨", "复杂", "多元",
    "丰富", "广泛", "普遍", "稀缺", "脆弱", "薄弱", "稳定", "持续", "长远", "根本", "核心",
    "关键", "重要", "必要", "充分", "合理", "有效", "精准", "审慎", "稳妥", "规范", "有序"
  ],
  "实词·名词": [
    "基础", "根基", "保障", "支撑", "动力", "活力", "潜力", "合力", "共识", "格局", "路径",
    "渠道", "载体", "平台", "机制", "制度", "体系", "效能", "效率", "成效", "成果", "困境",
    "瓶颈", "短板", "障碍", "壁垒", "风险", "隐患", "契机", "机遇", "趋势", "规律", "边界"
  ],
  "虚词·关联词": [
    "不仅", "不但", "而且", "并且", "况且", "甚至", "虽然", "尽管", "但是", "然而", "却",
    "可是", "因为", "由于", "因此", "因而", "所以", "既然", "如果", "假如", "倘若", "那么",
    "只要", "只有", "除非", "无论", "不论", "与其", "不如", "宁可", "也不", "既要", "又要"
  ],
  "虚词·副介词": [
    "尤其", "特别", "更为", "愈发", "日益", "逐渐", "不断", "始终", "仍然", "依然", "尚且",
    "仅仅", "几乎", "大抵", "未免", "不免", "反而", "进而", "从而", "继而", "基于", "鉴于",
    "关于", "对于", "由于", "通过", "随着", "沿着", "按照", "依据", "围绕", "针对"
  ],
  "成语": [
    "相辅相成", "相得益彰", "息息相关", "休戚与共", "一脉相承", "薪火相传", "源远流长", "根深蒂固",
    "潜移默化", "耳濡目染", "润物无声", "约定俗成", "蔚然成风", "方兴未艾", "如火如荼", "层出不穷",
    "屡见不鲜", "司空见惯", "习以为常", "不足为奇", "见怪不怪", "大相径庭", "背道而驰", "南辕北辙",
    "截然不同", "泾渭分明", "殊途同归", "异曲同工", "不谋而合", "不约而同", "一蹴而就", "一劳永逸",
    "立竿见影", "行之有效", "卓有成效", "事半功倍", "事倍功半", "缘木求鱼", "饮鸩止渴", "扬汤止沸",
    "釜底抽薪", "未雨绸缪", "居安思危", "防微杜渐", "亡羊补牢", "曲突徙薪", "因地制宜", "因势利导",
    "循序渐进", "循规蹈矩", "按部就班", "墨守成规", "固步自封", "抱残守缺", "推陈出新", "革故鼎新",
    "标新立异", "独树一帜", "别出心裁", "匠心独运", "因循守旧", "无所适从", "莫衷一是", "众说纷纭",
    "不置可否", "模棱两可", "含糊其辞", "语焉不详", "鞭辟入里", "入木三分", "高屋建瓴", "提纲挈领",
    "纲举目张", "一针见血", "切中要害", "直击要害", "避重就轻", "舍本逐末", "本末倒置", "喧宾夺主"
  ]
};

const TERM_DETAILS = new Map(Object.entries({
  "推动": "近义：推进。推动强调施加力量使事物发展，常搭配发展、改革、合作；推进强调按步骤向前，常搭配工作、项目、进程。",
  "推进": "近义：推动。推进侧重有计划、有步骤地向前开展；推动侧重外力促成变化。",
  "促进": "近义：推动、增进。促进多用于使发展加快或关系改善；增进多搭配了解、友谊、福祉。",
  "提升": "近义：提高。提升常搭配能力、品质、层次、效能，也可指地位上升；提高侧重数量、程度或水平由低到高。",
  "提高": "近义：提升。提高常搭配水平、效率、认识、标准；提升更强调层级、品质和整体状态。",
  "增强": "近义：增进。增强常搭配能力、意识、信心、实力；增进常搭配了解、友谊、感情、福祉。",
  "改善": "近义：改进。改善多搭配环境、条件、关系、生活，使原有状态变好；改进多搭配方法、技术、作风、工作。",
  "改进": "近义：改善。改进强调针对方法、技术或工作中的缺点作修改；改善强调状态和条件变好。",
  "完善": "近义：健全。完善强调在已有基础上补充，使制度、体系更完整；健全强调建立或使组织、机制完备。",
  "健全": "近义：完善。健全常搭配机制、制度、体系、法治；完善更强调对既有事物补缺优化。",
  "遏制": "近义：抑制。遏制语气较强，强调阻止不良趋势继续发展；抑制强调压制程度、冲动或增长。",
  "抑制": "近义：遏制。抑制可用于情绪、需求、增长和生理反应；遏制多用于蔓延、上涨等不良势头。",
  "培育": "近义：培养。培育多用于产业、市场、品牌、文化和新动能；培养多用于人才、能力、习惯。",
  "彰显": "近义：凸显、体现。彰显强调鲜明地显示价值或精神；凸显强调从背景中突出；体现强调通过具体事物表现出来。",
  "凸显": "近义：彰显、突出。凸显强调某种特点或问题变得明显，常含对比或环境作用。",
  "体现": "近义：反映、彰显。体现强调抽象特征通过具体事物表现；反映强调把客观情况呈现出来。",
  "揭示": "近义：揭露、阐释。揭示强调使规律、本质或内在联系显现；阐释强调解释说明。",
  "阐释": "近义：诠释。阐释偏重说明道理、理论和含义；诠释既可解释含义，也可通过表现赋予理解。",
  "契合": "近义：符合。契合强调彼此吻合、内在呼应；符合强调数量、标准、事实或要求相一致。",
  "顺应": "近义：适应。顺应强调主动依循趋势、规律或民意；适应强调调整自身以符合环境。",
  "回应": "近义：应对。回应强调对诉求、关切或质疑作出反应；应对强调采取措施处理局面或问题。",
  "消除": "近义：化解、缓解。消除强调使问题或影响不复存在；化解多搭配矛盾、风险、危机；缓解只表示程度减轻。",
  "化解": "近义：消除、缓解。化解强调通过措施使矛盾、风险、隔阂得到解决，常带过程性。",
  "填补": "近义：弥补。填补多搭配空白、缺口、空缺；弥补多搭配不足、损失、缺陷、遗憾。",
  "拓展": "近义：拓宽。拓展常搭配领域、空间、市场、功能；拓宽常搭配渠道、道路、视野、思路。",
  "汲取": "近义：吸取。汲取偏书面，常搭配智慧、营养、力量；吸取常搭配经验、教训，也可指吸收液体。",
  "整合": "近义：融合。整合强调把分散要素重新组合形成整体；融合强调不同事物相互渗透、融为一体。",
  "统筹": "近义：协调、兼顾。统筹强调从全局统一筹划；协调强调使关系配合适当；兼顾强调同时照顾多方。",
  "夯实": "近义：巩固。夯实强调把基础打牢；巩固强调使已有成果、地位或基础保持稳固。",
  "尤其": "近义：特别。尤其用于在整体中突出某一项；特别既可表示非常，也可表示与一般不同。",
  "未免": "近义：不免。未免多含委婉否定，表示实在不能不说过分；不免表示客观上难以避免。",
  "进而": "近义：从而。进而表示在已有行动基础上更进一步；从而表示前因导致后果。",
  "相辅相成": "近义：相得益彰。相辅相成强调双方互相辅助、缺一不可；相得益彰强调互相配合后优点更加显著。",
  "潜移默化": "近义：耳濡目染。潜移默化强调在不知不觉中受到影响；耳濡目染强调因经常听到看到而受到影响。",
  "一蹴而就": "近义：一劳永逸。一蹴而就强调事情轻易、一下子完成，多用于否定；一劳永逸强调一次处理后长期省事。",
  "因地制宜": "近义：因势利导。因地制宜强调根据不同地区实际采取办法；因势利导强调顺着事物发展趋势加以引导。",
  "循序渐进": "近义：按部就班。循序渐进强调遵循次序逐步深入，偏褒义；按部就班强调依照程序，也可含缺乏创新。",
  "鞭辟入里": "近义：入木三分。鞭辟入里多形容分析透彻；入木三分既可形容书法有力，也可形容见解深刻。"
}));

const EXAM_COLLOCATIONS = [
  "推动发展", "推动改革", "推动合作", "推进改革", "推进工作", "推进项目", "推进进程",
  "促进发展", "促进交流", "促进公平", "提升能力", "提升品质", "提升效能", "提高水平",
  "提高效率", "提高认识", "增强能力", "增强意识", "增强信心", "增进了解", "增进友谊",
  "改善环境", "改善条件", "改善民生", "改进方法", "改进技术", "改进作风", "完善制度",
  "完善体系", "完善机制", "健全机制", "健全制度", "健全体系", "遏制蔓延", "遏制上涨",
  "抑制冲动", "抑制需求", "抑制增长", "培育产业", "培育市场", "培育动能", "培养人才",
  "彰显价值", "彰显精神", "凸显优势", "凸显问题", "体现价值", "体现特点", "揭示规律",
  "揭示本质", "阐释理论", "阐释内涵", "契合需求", "契合趋势", "符合标准", "符合要求",
  "顺应趋势", "顺应规律", "适应环境", "回应关切", "回应诉求", "应对挑战", "应对风险",
  "消除障碍", "消除影响", "化解矛盾", "化解风险", "缓解压力", "缓解矛盾", "填补空白",
  "填补缺口", "弥补不足", "弥补损失", "拓展领域", "拓展空间", "拓宽渠道", "拓宽视野",
  "汲取智慧", "汲取力量", "吸取经验", "吸取教训", "整合资源", "融合发展", "统筹规划",
  "统筹协调", "兼顾各方", "夯实基础", "筑牢防线", "巩固成果", "巩固基础"
];

const EXACT_STOP_WORDS = new Set([
  "我们", "你们", "他们", "自己", "一个", "一些", "一种", "这个", "那个", "这些", "那些", "以及",
  "进行", "可以", "能够", "需要", "已经", "没有", "不是", "就是", "为了", "其中", "目前", "方面",
  "问题", "工作", "时候", "之后", "之前", "有关", "相关", "表示", "认为", "指出", "记者", "报道",
  "今天", "昨天", "今年", "去年", "同时", "此外", "等等", "这样的", "这种", "一般", "主要"
]);
const EDGE_CHARS = new Set("的了着过和与及或而也都就把被让将从于在是一其该这那各某很更最并则仍还又可会能".split(""));
const SENTENCE_SPLIT = /(?<=[。！？!?；;])|[\r\n]+/u;
const HAN_RUN = /[\u3400-\u9fff]{2,}/gu;

const TERM_CATEGORY = new Map();
for (const [category, terms] of Object.entries(EXAM_TERMS)) {
  for (const term of terms) {
    if (!TERM_CATEGORY.has(term)) TERM_CATEGORY.set(term, category);
  }
}
const CURATED_TERMS_BY_FIRST = new Map();
for (const term of TERM_CATEGORY.keys()) {
  const first = term[0];
  if (!CURATED_TERMS_BY_FIRST.has(first)) CURATED_TERMS_BY_FIRST.set(first, []);
  CURATED_TERMS_BY_FIRST.get(first).push(term);
}
for (const terms of CURATED_TERMS_BY_FIRST.values()) {
  terms.sort((a, b) => b.length - a.length);
}
const COLLOCATIONS_BY_TERM = new Map();
for (const term of TERM_CATEGORY.keys()) {
  const patterns = EXAM_COLLOCATIONS.filter((pattern) => pattern.includes(term));
  if (patterns.length) COLLOCATIONS_BY_TERM.set(term, patterns);
}

function cleanSentence(value) {
  return String(value || "")
    .replace(/https?:\/\/\S+|www\.\S+/giu, " ")
    .replace(/\s+/gu, " ")
    .trim()
    .slice(0, 220);
}

function candidateAllowed(term) {
  if (term.length < 2 || term.length > 4 || EXACT_STOP_WORDS.has(term)) return false;
  if (EDGE_CHARS.has(term[0]) || EDGE_CHARS.has(term.at(-1))) return false;
  if (/(.)\1{1,}/u.test(term)) return false;
  return !/^[零一二三四五六七八九十百千万亿]+$/u.test(term);
}

function collectContexts(sentences, selected, reportProgress) {
  const termsByFirstCharacter = new Map();
  const contexts = new Map();
  for (const item of selected.values()) {
    const first = item.term[0];
    if (!termsByFirstCharacter.has(first)) termsByFirstCharacter.set(first, []);
    termsByFirstCharacter.get(first).push(item.term);
    contexts.set(item.term, { collocations: new Map(), examples: [], exampleSet: new Set() });
  }
  for (const terms of termsByFirstCharacter.values()) {
    terms.sort((a, b) => b.length - a.length);
  }

  const progressInterval = Math.max(1, Math.floor(sentences.length / 35));
  for (let sentenceIndex = 0; sentenceIndex < sentences.length; sentenceIndex += 1) {
    const sentence = sentences[sentenceIndex];
    for (const [term, patterns] of COLLOCATIONS_BY_TERM) {
      const context = contexts.get(term);
      if (!context) continue;
      for (const pattern of patterns) {
        let offset = 0;
        while ((offset = sentence.indexOf(pattern, offset)) >= 0) {
          context.collocations.set(pattern, (context.collocations.get(pattern) || 0) + 1);
          offset += pattern.length;
        }
      }
    }
    for (let offset = 0; offset < sentence.length; offset += 1) {
      const terms = termsByFirstCharacter.get(sentence[offset]);
      if (!terms) continue;
      for (const term of terms) {
        if (!sentence.startsWith(term, offset)) continue;
        const context = contexts.get(term);
        if (context.examples.length < 1 && sentence.length >= term.length + 2 && !context.exampleSet.has(sentence)) {
          context.exampleSet.add(sentence);
          context.examples.push(sentence);
        }
      }
    }
    if (sentenceIndex % progressInterval === 0) {
      reportProgress(60 + Math.round(((sentenceIndex + 1) / Math.max(1, sentences.length)) * 35), "正在整理搭配和例句");
    }
  }
  return contexts;
}

export function extractVocabulary(text, options = {}) {
  const onProgress = typeof options.onProgress === "function" ? options.onProgress : () => {};
  let lastPercent = -1;
  function reportProgress(percent, stage) {
    const normalizedPercent = Math.max(0, Math.min(100, Math.round(percent)));
    if (normalizedPercent === lastPercent && normalizedPercent !== 100) return;
    lastPercent = normalizedPercent;
    onProgress({ percent: normalizedPercent, stage });
  }

  reportProgress(2, "正在读取文本");
  const normalized = String(text || "").normalize("NFKC").replace(/\u0000/gu, "");
  const sentences = normalized.split(SENTENCE_SPLIT).map(cleanSentence).filter((item) => item.length >= 4);
  const selected = new Map();

  const curatedCounts = new Map();
  const curatedProgressInterval = Math.max(1, Math.floor(sentences.length / 20));
  for (let sentenceIndex = 0; sentenceIndex < sentences.length; sentenceIndex += 1) {
    const sentence = sentences[sentenceIndex];
    for (let offset = 0; offset < sentence.length; offset += 1) {
      const terms = CURATED_TERMS_BY_FIRST.get(sentence[offset]);
      if (!terms) continue;
      for (const term of terms) {
        if (sentence.startsWith(term, offset)) {
          curatedCounts.set(term, (curatedCounts.get(term) || 0) + 1);
        }
      }
    }
    if (sentenceIndex % curatedProgressInterval === 0) {
      reportProgress(8 + Math.round(((sentenceIndex + 1) / Math.max(1, sentences.length)) * 17), "正在识别常考词语");
    }
  }
  for (const [term, count] of curatedCounts) {
    selected.set(term, { term, category: TERM_CATEGORY.get(term), count, curated: true });
  }

  reportProgress(55, "正在整理考试型搭配和例句");
  const contexts = collectContexts(sentences, selected, reportProgress);
  const entries = [...selected.values()]
    .map((item) => {
      const context = contexts.get(item.term);
      return {
        ...item,
        synonymNote: TERM_DETAILS.get(item.term) || "",
        collocations: [...context.collocations.entries()]
          .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0], "zh-CN"))
          .slice(0, 3)
          .map(([collocationText, count]) => ({ text: collocationText, count })),
        examples: context.examples
      };
    })
    .sort((a, b) => b.count - a.count || Number(b.curated) - Number(a.curated) || a.term.localeCompare(b.term, "zh-CN"));

  reportProgress(100, "提取完成");
  return {
    charCount: normalized.length,
    sentenceCount: sentences.length,
    entries
  };
}

export function aggregateVocabularySources(sources = []) {
  const aggregate = new Map();
  for (const source of sources) {
    const sourceName = source.fileName || "未知来源";
    for (const entry of source.entries || []) {
      if (!aggregate.has(entry.term)) {
        aggregate.set(entry.term, {
          term: entry.term,
          category: entry.category,
          curated: Boolean(entry.curated),
          totalCount: 0,
          sourceCount: 0,
          synonymNote: entry.synonymNote || TERM_DETAILS.get(entry.term) || "",
          collocations: new Map(),
          examples: [],
          exampleKeys: new Set()
        });
      }
      const item = aggregate.get(entry.term);
      item.totalCount += Number(entry.count) || 0;
      item.sourceCount += 1;
      item.curated ||= Boolean(entry.curated);
      if (!item.synonymNote && entry.synonymNote) item.synonymNote = entry.synonymNote;
      for (const collocation of entry.collocations || []) {
        item.collocations.set(collocation.text, (item.collocations.get(collocation.text) || 0) + (Number(collocation.count) || 0));
      }
      for (const example of entry.examples || []) {
        const key = `${sourceName}\u0000${example}`;
        if (item.examples.length < 5 && !item.exampleKeys.has(key)) {
          item.exampleKeys.add(key);
          item.examples.push({ key, sourceName, text: example });
        }
      }
    }
  }

  return [...aggregate.values()].map((item) => ({
    term: item.term,
    category: item.category,
    curated: item.curated,
    totalCount: item.totalCount,
    sourceCount: item.sourceCount,
    synonymNote: item.synonymNote,
    weight: item.totalCount + Math.max(0, item.sourceCount - 1) * 2,
    collocations: [...item.collocations.entries()]
      .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0], "zh-CN"))
      .slice(0, 5)
      .map(([text, count]) => ({ text, count })),
    examples: item.examples.slice(0, 1).map(({ sourceName, text }) => ({ sourceName, text }))
  }));
}

export const VOCABULARY_CATEGORIES = Object.freeze([...Object.keys(EXAM_TERMS)]);
