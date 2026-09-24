/**
 * Generate the UI and composable reference pages under `docs/content/guide/`.
 *
 *   node scripts/generate-reference-docs.ts          # write
 *   node scripts/generate-reference-docs.ts --check  # fail when out of date
 *
 * Pages are derived from `uiFamilyCatalog`, `COMPOSABLE_CATALOG`, the SFC
 * `defineProps` / `defineEmits` / `defineSlots` / `defineExpose` contracts
 * and their TSDoc, and each family's behavior contract. Every page is
 * English; the two index pages exist in every docs locale.
 */
import { mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { COMPOSABLE_CATALOG } from "../../compose/core/src/catalog.ts";
import { uiFamilyCatalog } from "../src/catalog/family-catalog.ts";
import {
  COMPOSABLE_REFERENCE_EXCLUDED,
  composableIndexRows,
  composableName,
  renderComposablePage,
} from "./reference-docs/composables.ts";
import { GENERATED_NOTICE, blocks, frontmatter } from "./reference-docs/markdown.ts";
import { renderUiFamilyPage, uiIndexRows } from "./reference-docs/ui.ts";

const uiRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const composableRoot = path.resolve(uiRoot, "../compose/core");
/** `docs/content` of the repository. */
export const docsContentRoot = path.resolve(uiRoot, "../../docs/content");

/** Locales whose index pages are generated (English is the unprefixed default). */
const indexCopy = {
  en: {
    ui: [
      "UI Reference",
      "Every `@vizejs/ui` family: props, events, slots, exposed members, and the normative behavior contract, generated from the source.",
    ],
    composables: [
      "Composables Reference",
      "Every `@vizejs/composable` entry: runtime contract (SSR, hydration, cleanup, targets) and API, generated from the source.",
    ],
  },
  ja: {
    ui: [
      "UI リファレンス",
      "`@vizejs/ui` の全ファミリー: props、イベント、スロット、公開メンバー、規範的な振る舞い仕様をソースから生成しています (英語)。",
    ],
    composables: [
      "コンポーザブル リファレンス",
      "`@vizejs/composable` の全エントリー: ランタイム契約 (SSR、ハイドレーション、クリーンアップ、ターゲット) と API をソースから生成しています (英語)。",
    ],
  },
  "zh-CN": {
    ui: [
      "UI 参考",
      "`@vizejs/ui` 的所有系列：props、事件、插槽、暴露成员以及规范的行为契约，均由源码生成（英文）。",
    ],
    composables: [
      "组合式函数参考",
      "`@vizejs/composable` 的所有入口：运行时契约（SSR、水合、清理、目标）与 API，均由源码生成（英文）。",
    ],
  },
  "pt-BR": {
    ui: [
      "Referência de UI",
      "Todas as famílias de `@vizejs/ui`: props, eventos, slots, membros expostos e o contrato de comportamento, gerados a partir do código-fonte (em inglês).",
    ],
    composables: [
      "Referência de composables",
      "Todas as entradas de `@vizejs/composable`: contrato de runtime (SSR, hidratação, limpeza, alvos) e API, gerados a partir do código-fonte (em inglês).",
    ],
  },
  fr: {
    ui: [
      "Référence UI",
      "Toutes les familles de `@vizejs/ui` : props, événements, slots, membres exposés et contrat de comportement, générés depuis les sources (en anglais).",
    ],
    composables: [
      "Référence des composables",
      "Toutes les entrées de `@vizejs/composable` : contrat d'exécution (SSR, hydratation, nettoyage, cibles) et API, générés depuis les sources (en anglais).",
    ],
  },
} as const;

/** Composable entries that get a reference page. */
export function referencedComposables() {
  return COMPOSABLE_CATALOG.entries.filter(
    (entry) => !COMPOSABLE_REFERENCE_EXCLUDED.has(entry.subpath),
  );
}

/** Every generated page, keyed by path relative to `docs/content`. */
export function renderReferenceDocs(): Map<string, string> {
  const pages = new Map<string, string>();
  for (const entry of uiFamilyCatalog) {
    pages.set(`guide/ui/${entry.canonicalName}.md`, renderUiFamilyPage(uiRoot, entry));
  }
  const composables = referencedComposables();
  for (const entry of composables) {
    pages.set(
      `guide/composables/${composableName(entry)}.md`,
      renderComposablePage(composableRoot, entry, COMPOSABLE_CATALOG.utilities),
    );
  }
  for (const [locale, copy] of Object.entries(indexCopy)) {
    const prefix = locale === "en" ? "" : `${locale}/`;
    // Locale index pages link to the English pages.
    const linkPrefix = (section: string) => (locale === "en" ? "./" : `../../../guide/${section}/`);
    const [uiTitle, uiIntro] = copy.ui;
    pages.set(
      `${prefix}guide/ui/index.md`,
      blocks(
        frontmatter(uiTitle, uiIntro),
        GENERATED_NOTICE,
        `# ${uiTitle}`,
        uiIntro,
        uiIndexRows(uiRoot, uiFamilyCatalog, linkPrefix("ui")),
      ),
    );
    const [composableTitle, composableIntro] = copy.composables;
    pages.set(
      `${prefix}guide/composables/index.md`,
      blocks(
        frontmatter(composableTitle, composableIntro),
        GENERATED_NOTICE,
        `# ${composableTitle}`,
        composableIntro,
        composableIndexRows(
          composableRoot,
          composables,
          COMPOSABLE_CATALOG.utilities,
          linkPrefix("composables"),
        ),
      ),
    );
  }
  return pages;
}

/** Generated files currently on disk (for orphan detection). */
export function existingReferenceDocs(): string[] {
  const found: string[] = [];
  for (const locale of Object.keys(indexCopy)) {
    const prefix = locale === "en" ? "" : `${locale}/`;
    for (const section of ["ui", "composables"]) {
      const directory = path.join(docsContentRoot, `${prefix}guide/${section}`);
      let names: string[] = [];
      try {
        names = readdirSync(directory);
      } catch {
        continue;
      }
      found.push(
        ...names
          .filter((name) => name.endsWith(".md"))
          .map((name) => `${prefix}guide/${section}/${name}`),
      );
    }
  }
  return found.sort();
}

function run(check: boolean): number {
  const pages = renderReferenceDocs();
  const stale: string[] = [];
  for (const [relative, content] of pages) {
    const target = path.join(docsContentRoot, relative);
    let current: string | null = null;
    try {
      current = readFileSync(target, "utf8");
    } catch {
      current = null;
    }
    if (current === content) continue;
    stale.push(relative);
    if (!check) {
      mkdirSync(path.dirname(target), { recursive: true });
      writeFileSync(target, content);
    }
  }
  const orphans = existingReferenceDocs().filter((relative) => !pages.has(relative));
  if (!check) for (const relative of orphans) rmSync(path.join(docsContentRoot, relative));
  if (check && (stale.length > 0 || orphans.length > 0)) {
    process.stderr.write(
      `reference docs are out of date (run: node npm/ui/scripts/generate-reference-docs.ts)\n${[...stale, ...orphans].map((file) => `  ${file}`).join("\n")}\n`,
    );
    return 1;
  }
  process.stdout.write(
    `${check ? "checked" : "wrote"} ${pages.size} reference pages (${stale.length} changed, ${orphans.length} removed)\n`,
  );
  return 0;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  process.exitCode = run(process.argv.includes("--check"));
}
