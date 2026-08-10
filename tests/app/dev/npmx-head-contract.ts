import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

type SourceContract = {
  anchors: readonly string[];
  sha256: string;
};

export const NPMX_HEAD_SOURCE_CONTRACTS = {
  "app/app.vue": {
    sha256: "eb9448db4b25d7ee5baff8fa6b9d6a91f01863a0d8f3db4f5f97366f614165cf",
    anchors: ["useHead({", "titleTemplate:", "name: 'color-scheme'"],
  },
  "app/pages/about.vue": {
    sha256: "7637a3b8fede9d671a4bb5d748f32079c8631451e58e3090a72121e86ebf06d4",
    anchors: ["useSeoMeta({", "ogTitle:", "twitterDescription:"],
  },
  "app/pages/accessibility.vue": {
    sha256: "4cd0c9728434c3e6ed527c626252fd2335b53a77eca5f55a7400b2dbcdfe98b0",
    anchors: ["definePageMeta({", "name: 'accessibility'", "useSeoMeta({"],
  },
  "app/pages/package-docs/[...path].vue": {
    sha256: "4edb381b31dbd711f5b782bfb33ea1d4cb70fa2827afde64576f1b16904c2446",
    anchors: [
      "definePageMeta({",
      "name: 'docs'",
      "alias: ['/package/docs/:path+', '/docs/:path+']",
      "scrollMargin: 180",
      "useSeoMeta({",
    ],
  },
  "app/pages/package/[[org]]/[name].vue": {
    sha256: "7b69ca1c32f8766ff0dc994c1049e6b76ce32fa4dbd66d1f75bf07a8869653c6",
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

  for (const [relativePath, contract] of Object.entries(NPMX_HEAD_SOURCE_CONTRACTS)) {
    const source = fs.readFileSync(path.join(fixtureRoot, relativePath), "utf8");
    const digest = sha256(source);
    if (digest !== contract.sha256) {
      throw new Error(
        `pinned npmx head source changed: ${relativePath} expected ${contract.sha256}, got ${digest}`,
      );
    }
    sources[relativePath] = source;
    evidence[relativePath] = digest;
  }

  assertNpmxHeadMacroAnchors(sources);
  return evidence;
}
