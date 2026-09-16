import {
  environmentSection,
  readmeSection,
  typecheckSection,
  viteSection,
} from "./published-snapshot-render.mjs";

const LEGACY_HEADINGS = {
  en: [
    "Benchmark Environment",
    "Why Rust?",
    "Benchmark: Type Checker — canon vs vue-tsc",
    "Type checker profile",
    "Benchmark: Vite Plugin — @vizejs/vite-plugin vs @vitejs/plugin-vue",
  ],
  ja: [
    "ベンチマーク環境",
    "なぜ Rust なのか?",
    "ベンチマーク: 型チェッカー — canon 対 vue-tsc",
    "タイプチェッカープロファイル",
    "ベンチマーク: Vite プラグイン — @vizejs/vite-plugin 対 @vitejs/plugin-vue",
  ],
  "zh-CN": [
    "基准环境",
    "为什么是 Rust？",
    "基准测试：类型检查器 — 正史与vue-tsc的对比",
    "类型检查员配置文件",
    "基准测试：Vite 插件 — @vizejs/vite-plugin 与 @vitejs/plugin-vue 的比较",
  ],
  fr: [
    "Environnement de référence",
    "Pourquoi Rust ?",
    "Benchmark : Type Checker — canon vs vue-tsc",
    "Profil de vérification de type",
    "Benchmark : Vite Plugin — @vizejs/vite-plugin vs @vitejs/plugin-vue",
  ],
  "pt-BR": [
    "Ambiente de Benchmark",
    "Por que Rust?",
    "Benchmark: Type Checker — cânone vs vue-tsc",
    "Perfil do verificador de tipos",
    "Benchmark: Vite Plugin — @vizejs/vite-plugin vs @vitejs/plugin-vue",
  ],
};

function uniqueOffset(source, needle) {
  const offset = source.indexOf(needle);
  if (offset < 0 || source.indexOf(needle, offset + needle.length) >= 0) {
    throw new Error(`benchmark publication: missing or ambiguous section ${needle.trim()}`);
  }
  return offset;
}

function replaceSection(source, id, content, legacyStart, legacyEnd) {
  const begin = `<!-- benchmark:${id}:start -->`;
  const end = `<!-- benchmark:${id}:end -->`;
  const marked = source.includes(begin) || source.includes(end);
  const start = uniqueOffset(source, marked ? begin : legacyStart);
  const finish = marked ? uniqueOffset(source, end) + end.length : uniqueOffset(source, legacyEnd);
  if (finish <= start) throw new Error(`benchmark publication: reversed section ${id}`);
  return `${source.slice(0, start)}${begin}\n\n${content}\n\n${end}\n\n${source.slice(finish).replace(/^\n+/u, "")}`;
}

export function updateReadme(source, data) {
  return replaceSection(source, "readme", readmeSection(data), "## Benchmarks\n", "## Credits\n");
}

export function updatePerformance(source, data, locale) {
  const [environment, rust, typecheck, profile, vite] = LEGACY_HEADINGS[locale];
  let output = replaceSection(
    source,
    "environment",
    environmentSection(data, locale),
    `## ${environment}\n`,
    `## ${rust}\n`,
  );
  output = replaceSection(
    output,
    "typecheck",
    typecheckSection(data, locale),
    `## ${typecheck}\n`,
    `### ${profile}\n`,
  );
  // Preserve the historical cache-bias retraction after the generated Vite table.
  const retraction = output.split("\n").find((line) => line.includes("`957ms` / `479ms` / `2.0x`"));
  if (!retraction) throw new Error("benchmark publication: missing historical Vite retraction");
  return replaceSection(output, "vite", viteSection(data, locale), `## ${vite}\n`, retraction);
}
