import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

type SourceContract = {
  anchors: readonly string[];
};

export const NPMX_HEAD_SOURCE_CONTRACTS = {
  "app/app.vue": {
    anchors: ["useHead({", "titleTemplate:", "name: 'color-scheme'"],
  },
  "app/pages/about.vue": {
    anchors: ["useSeoMeta({", "ogTitle:", "twitterDescription:"],
  },
  "app/pages/accessibility.vue": {
    anchors: ["definePageMeta({", "name: 'accessibility'", "useSeoMeta({"],
  },
  "app/pages/package-docs/[...path].vue": {
    anchors: [
      "definePageMeta({",
      "name: 'docs'",
      "alias: ['/package/docs/:path+', '/docs/:path+']",
      "scrollMargin: 180",
      "useSeoMeta({",
    ],
  },
  "app/pages/package/[[org]]/[name].vue": {
    anchors: ["useHead({", "rel: 'canonical'", "useSeoMeta({"],
  },
} as const satisfies Record<string, SourceContract>;

export type NpmxHeadSourceEvidence = Record<string, string>;

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

export function assertNpmxHeadMacroAnchors(sources: Record<string, string>): void {
  for (const [relativePath, contract] of Object.entries(NPMX_HEAD_SOURCE_CONTRACTS)) {
    const source = sources[relativePath];
    if (source === undefined) {
      throw new Error(`missing pinned npmx head source: ${relativePath}`);
    }
    for (const anchor of contract.anchors) {
      if (!source.includes(anchor)) {
        throw new Error(`missing npmx head macro anchor in ${relativePath}: ${anchor}`);
      }
    }
  }
}

export function readNpmxHeadSourceEvidence(fixtureRoot: string): NpmxHeadSourceEvidence {
  const sources: Record<string, string> = {};
  const evidence: NpmxHeadSourceEvidence = {};

  for (const relativePath of Object.keys(NPMX_HEAD_SOURCE_CONTRACTS)) {
    const source = fs.readFileSync(path.join(fixtureRoot, relativePath), "utf8");
    sources[relativePath] = source;
    evidence[relativePath] = sha256(source);
  }

  assertNpmxHeadMacroAnchors(sources);
  return evidence;
}
