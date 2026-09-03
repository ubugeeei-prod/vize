import fs from "node:fs";
import path from "node:path";

import type { PlannedFile } from "./config.js";

const VITE_CONFIG_FILES = [
  "vite.config.ts",
  "vite.config.mts",
  "vite.config.js",
  "vite.config.mjs",
] as const;

const VITE_PLUS_LINT_IMPORT = 'import { createVizeLintConfig } from "oxlint-plugin-vize";\n';

const VITE_PLUS_LINT_BLOCK = `  lint: createVizeLintConfig({
    preset: "happy-path",
    settings: {
      helpLevel: "short",
    },
  }),
`;

export interface ViteMigrationPlan {
  readonly file: PlannedFile | null;
  readonly preserved: string | null;
  readonly removesOfficialPlugin: boolean;
  readonly enablesVitePlusLint: boolean;
  readonly hasVitePlusLint: boolean;
  readonly usesVitePlus: boolean;
}

export function planViteMigration(
  root: string,
  mayConfigureVitePlusLint: boolean,
): ViteMigrationPlan {
  const existing = VITE_CONFIG_FILES.filter((candidate) =>
    fs.existsSync(path.join(root, candidate)),
  );
  if (existing.length === 0) {
    return {
      file: null,
      preserved: null,
      removesOfficialPlugin: false,
      enablesVitePlusLint: false,
      hasVitePlusLint: false,
      usesVitePlus: false,
    };
  }
  if (existing.length > 1) {
    return {
      file: null,
      preserved: `Vite configs (${existing.join(", ")})`,
      removesOfficialPlugin: false,
      enablesVitePlusLint: false,
      hasVitePlusLint: false,
      usesVitePlus: false,
    };
  }

  const relativeFilename = existing[0]!;
  const filename = path.join(root, relativeFilename);
  const source = fs.readFileSync(filename, "utf8");
  const usesVitePlus = /from\s+["']vite-plus["']/u.test(source);
  let migratedSource = source;
  let removesOfficialPlugin = false;
  let preserved: string | null = null;

  if (!source.includes("@vizejs/vite-plugin")) {
    const importPattern =
      /^([^\S\r\n]*import[^\S\r\n]+)([$A-Z_a-z][$\w]*)([^\S\r\n]+from[^\S\r\n]+)(["'])@vitejs\/plugin-vue\4([^\S\r\n]*;?[^\S\r\n]*)$/gmu;
    const imports = [...source.matchAll(importPattern)];
    if (imports.length === 1) {
      const localName = imports[0]![2]!;
      const zeroArgumentCallPattern = new RegExp(
        `\\b${escapeRegExp(localName)}\\s*\\(\\s*\\)`,
        "gu",
      );
      const calls = [...source.matchAll(zeroArgumentCallPattern)];
      const importIndex = imports[0]!.index!;
      const importSource = imports[0]![0];
      const sourceWithoutExpectedUse =
        source.slice(0, importIndex) +
        source.slice(importIndex + importSource.length).replace(zeroArgumentCallPattern, "");
      const hasOtherUses = new RegExp(`\\b${escapeRegExp(localName)}\\b`, "u").test(
        sourceWithoutExpectedUse,
      );
      if (calls.length === 1 && !hasOtherUses) {
        migratedSource = source.replace(importPattern, `$1$2$3$4@vizejs/vite-plugin$4$5`);
        removesOfficialPlugin = true;
      } else {
        preserved = relativeFilename;
      }
    } else if (source.includes("@vitejs/plugin-vue")) {
      preserved = relativeFilename;
    }
  }

  const hadVitePlusLint = usesVitePlus && source.includes("oxlint-plugin-vize");
  let enablesVitePlusLint = false;
  if (mayConfigureVitePlusLint && !hadVitePlusLint && canInjectVitePlusLint(migratedSource)) {
    const injectedSource = injectVitePlusLint(migratedSource);
    if (injectedSource !== null) {
      migratedSource = injectedSource;
      enablesVitePlusLint = true;
    }
  }

  return {
    file: migratedSource === source ? null : { filename, source: migratedSource },
    preserved:
      preserved ??
      (migratedSource === source && source.includes("@vizejs/vite-plugin")
        ? relativeFilename
        : null),
    removesOfficialPlugin,
    enablesVitePlusLint,
    hasVitePlusLint: hadVitePlusLint || enablesVitePlusLint,
    usesVitePlus,
  };
}

function canInjectVitePlusLint(source: string): boolean {
  if (
    !/from\s+["']vite-plus["']/u.test(source) ||
    /^\s*lint\s*:/mu.test(source) ||
    /\bcreateVizeLintConfig\b/u.test(source)
  ) {
    return false;
  }
  return [...source.matchAll(/\bdefineConfig\s*\(\s*\{/gu)].length === 1;
}

function injectVitePlusLint(source: string): string | null {
  const importLines = [
    ...source.matchAll(
      /^import[^\r\n]*(?:from\s+["'][^"']+["']|["'][^"']+["'])\s*;?[^\S\r\n]*(?:\r?\n|$)/gmu,
    ),
  ];
  const lastImport = importLines.at(-1);
  if (!lastImport || lastImport.index === undefined) {
    return null;
  }
  const importEnd = lastImport.index + lastImport[0].length;
  const withImport = source.slice(0, importEnd) + VITE_PLUS_LINT_IMPORT + source.slice(importEnd);
  return withImport.replace(
    /\bdefineConfig\s*\(\s*\{/u,
    (opening) => `${opening}\n${VITE_PLUS_LINT_BLOCK}`,
  );
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
