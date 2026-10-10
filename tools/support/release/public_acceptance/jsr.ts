import { createHash } from "node:crypto";
import type { PublicationTarget } from "./plan.ts";
import type { publicReader } from "./http.ts";

export const jsrAuthorityPaths = [
  ".github/workflows/jsr-package.yml",
  ".github/workflows/release-jsr.yml",
  "jsr/vize/jsr.json",
  "jsr/vize/README.md",
  "tools/support/release/jsr/prepare.mjs",
  "tools/support/release/jsr/consumer.mjs",
  "jsr/vize/channel.json",
  "tools/support/release/jsr/channel.mjs",
] as const;

const exports = {
  ".": "./mod.ts",
  "./config": "./config.ts",
  "./native": "./native.ts",
  "./vite": "./vite.ts",
};
const entrypoints = [
  ["mod.ts", "npm/cli", "vize", "", false],
  ["config.ts", "npm/cli", "vize", "/config", false],
  ["native.ts", "npm/native", "@vizejs/native", "", false],
  ["vite.ts", "npm/builder/vite", "@vizejs/vite-plugin", "", true],
] as const;

export const jsrJobNames = [
  "Release and verify JSR distribution / Publish JSR package",
  "Release and verify JSR distribution / JSR consumer (ubuntu-24.04, Node 22)",
  "Release and verify JSR distribution / JSR consumer (ubuntu-24.04, Node 24)",
  "Release and verify JSR distribution / JSR consumer (macos-15, Node 24)",
  "Release and verify JSR distribution / JSR consumer (windows-2025, Node 24)",
] as const;

export interface JsrPublicationPlan {
  enabled: boolean;
  name: "@vizejs/vize";
  version: string;
  exports: Record<string, string>;
  sources: { path: string; sha256: string; bytes: number }[];
}

export interface JsrPublicationReceipt {
  required: boolean;
  name: string;
  version: string;
  exports?: Record<string, string>;
  metadataUrl?: string;
  versionMetadataUrl?: string;
  sources?: (JsrPublicationPlan["sources"][number] & { url: string })[];
  jobs?: Record<string, unknown>[];
}

function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function object(value: unknown): Record<string, unknown> {
  requireValue(value && typeof value === "object" && !Array.isArray(value), "JSR object required");
  return value as Record<string, unknown>;
}
function sameEntries(value: unknown, expected: Record<string, unknown>): boolean {
  const actual = object(value);
  return (
    Object.keys(actual).length === Object.keys(expected).length &&
    Object.entries(expected).every(([key, item]) => actual[key] === item)
  );
}

/** Optional only for source histories predating the complete JSR lane. */
export function deriveJsrPublication(
  version: string,
  npm: PublicationTarget[],
  read: (path: string) => Buffer,
  exists: (path: string) => boolean,
): JsrPublicationPlan | undefined {
  if (!jsrAuthorityPaths.some(exists)) return undefined;
  // Capture the whole publication/control lane; partial source is not a disabled policy.
  const captured = new Map(jsrAuthorityPaths.map((path) => [path, read(path)]));
  const policy = object(JSON.parse(captured.get("jsr/vize/channel.json")!.toString("utf8")));
  requireValue(
    Object.keys(policy).sort().join(",") === "enabled,schema" &&
      policy.schema === "vize-jsr-channel-v1" &&
      typeof policy.enabled === "boolean",
    "exact JSR channel policy required",
  );
  const template = object(JSON.parse(captured.get("jsr/vize/jsr.json")!.toString("utf8")));
  requireValue(
    Object.keys(template).sort().join(",") === "exports,name,publish" &&
      template.name === "@vizejs/vize" &&
      sameEntries(template.exports, exports),
    "exact supported JSR template required",
  );
  const publish = object(template.publish);
  requireValue(
    Object.keys(publish).join(",") === "include" &&
      JSON.stringify(publish.include) === JSON.stringify(["*.ts", "README.md", "LICENSE"]),
    "exact JSR publication include required",
  );
  const contents = new Map<string, Buffer>();
  for (const [filename, directory, name, subpath, hasDefault] of entrypoints) {
    const manifest = object(JSON.parse(read(`${directory}/package.json`).toString("utf8")));
    requireValue(
      manifest.name === name &&
        manifest.version === version &&
        manifest.private !== true &&
        npm.some((target) => target.name === name && target.version === version),
      `exact published JSR npm dependency required: ${directory}`,
    );
    const specifier = JSON.stringify(`npm:${name}@${version}${subpath}`);
    contents.set(
      `/${filename}`,
      Buffer.from(
        `/** Node.js 22+ entry point for ${name}. */\nexport * from ${specifier};\n` +
          (hasDefault ? `export { default } from ${specifier};\n` : ""),
      ),
    );
  }
  contents.set("/README.md", captured.get("jsr/vize/README.md")!);
  contents.set("/LICENSE", read("LICENSE"));
  // Deno 2.9.5 force-includes its configuration even with this explicit include list.
  contents.set(
    "/jsr.json",
    Buffer.from(`${JSON.stringify({ ...template, version, minimumDependencyAge: 0 }, null, 2)}\n`),
  );
  return {
    enabled: policy.enabled,
    name: "@vizejs/vize",
    version,
    exports: { ...exports },
    sources: [...contents]
      .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
      .map(([path, bytes]) => ({
        path,
        bytes: bytes.length,
        sha256: createHash("sha256").update(bytes).digest("hex"),
      })),
  };
}

/** Bind public source bytes and real consumer jobs to the already authenticated official R/H. */
export async function verifyJsrPublication(
  plan: JsrPublicationPlan,
  version: string,
  jobs: Record<string, unknown>[],
  reader: ReturnType<typeof publicReader>,
): Promise<JsrPublicationReceipt> {
  const paths = [
    "/LICENSE",
    "/README.md",
    "/config.ts",
    "/jsr.json",
    "/mod.ts",
    "/native.ts",
    "/vite.ts",
  ];
  requireValue(
    typeof plan.enabled === "boolean" &&
      plan.name === "@vizejs/vize" &&
      plan.version === version &&
      sameEntries(plan.exports, exports) &&
      Array.isArray(plan.sources) &&
      plan.sources.length === paths.length &&
      new Set(plan.sources.map((item) => item.path)).size === paths.length &&
      plan.sources.every(
        (item) =>
          paths.includes(item.path) &&
          /^[0-9a-f]{64}$/.test(item.sha256) &&
          Number.isSafeInteger(item.bytes) &&
          item.bytes > 0,
      ),
    "exact source-frozen JSR plan required",
  );
  const identity = { required: plan.enabled, name: plan.name, version: plan.version };
  if (!plan.enabled) return identity;
  const requiredJobs = jsrJobNames.map((name) => {
    const matching = jobs.filter((job) => job.name === name);
    requireValue(
      matching.length === 1 && matching[0].conclusion === "success",
      `successful official Release JSR job required: ${name}`,
    );
    return matching[0];
  });
  const metadataUrl = "https://jsr.io/@vizejs/vize/meta.json";
  const metadata = await reader.json(metadataUrl);
  const repository = object(metadata.githubRepository);
  const versions = object(metadata.versions);
  const published = object(versions[version]);
  requireValue(
    metadata.scope === "vizejs" &&
      metadata.name === "vize" &&
      repository.owner === "ubugeeei-prod" &&
      repository.name === "vize" &&
      (published.yanked === undefined || published.yanked === false),
    "JSR exact package/version/non-yanked repository identity mismatch",
  );
  const versionMetadataUrl = `https://jsr.io/@vizejs/vize/${version}_meta.json`;
  const versionMetadata = await reader.json(versionMetadataUrl);
  requireValue(sameEntries(versionMetadata.exports, plan.exports), "JSR exact exports mismatch");
  const manifest = object(versionMetadata.manifest);
  requireValue(
    Object.keys(manifest).length === paths.length &&
      paths.every((path) => Object.hasOwn(manifest, path)),
    "JSR exact published source inventory mismatch",
  );
  const sources = [];
  for (const source of plan.sources) {
    const entry = object(manifest[source.path]);
    requireValue(
      entry.size === source.bytes && entry.checksum === `sha256-${source.sha256}`,
      `JSR frozen source checksum mismatch: ${source.path}`,
    );
    const url = `https://jsr.io/@vizejs/vize/${version}${source.path}`;
    const actual = await reader.download(url);
    requireValue(
      actual.sha256 === source.sha256 && actual.bytes === source.bytes,
      `JSR downloaded source checksum mismatch: ${source.path}`,
    );
    sources.push({ ...source, url });
  }
  return {
    ...identity,
    exports: plan.exports,
    metadataUrl,
    versionMetadataUrl,
    sources,
    jobs: requiredJobs,
  };
}
