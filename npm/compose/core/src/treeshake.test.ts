import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import { build } from "vite-plus";

const packageRoot = fileURLToPath(new URL("..", import.meta.url));
const sourceDirectory = fileURLToPath(new URL(".", import.meta.url));
const entries = {
  index: fileURLToPath(new URL("index.ts", import.meta.url)),
  temporal: fileURLToPath(new URL("temporal.ts", import.meta.url)),
} as const;

/**
 * Fingerprints that identify each source module inside an unminified
 * production bundle. Comments are stripped before matching, so identifiers
 * and string literals are reliable module markers: none of them appears in
 * another module or in the retained external `vue` import specifiers.
 */
const sentinels = {
  "abort-signal": ["anyAbortSignal"],
  scope: ["tryOnScopeDispose"],
  "capability-available": ['status: "available"'],
  "capability-unavailable": ['status: "unavailable"'],
  "capability-is-available": ["isCapabilityAvailable"],
  "capability-is-unavailable": ["isCapabilityUnavailable"],
  "disposal-scope": ["createDisposalScope", "VIZE_COMPOSE_DISPOSAL_FAILED"],
  "event-listener": ["useEventListener", "isListening"],
  "media-query": ["useMediaQuery", "matchMedia"],
  locale: ["useLocale", "getTextInfo"],
  "async-resource": ["useAsyncResource", "A newer execution started."],
  temporal: ["useTemporalNow", "VIZE_COMPOSE_TEMPORAL_INVALID_INTERVAL"],
  "use-previous": ["usePrevious"],
  "use-history": ["useHistory", "VIZE_COMPOSE_HISTORY_INVALID_CAPACITY"],
  "use-debounced": ["useDebounced", "VIZE_COMPOSE_DEBOUNCE_INVALID_WAIT"],
  "use-throttled": ["useThrottled", "VIZE_COMPOSE_THROTTLE_INVALID_WAIT"],
  "use-toggle": ["useToggle"],
  "use-counter": ["useCounter", "VIZE_COMPOSE_COUNTER_INVALID_RANGE"],
} as const;

type ModuleName = keyof typeof sentinels;

interface UtilityCase {
  readonly binding: string;
  readonly entry: keyof typeof entries;
  readonly module: ModuleName;
  /** Documented shared infrastructure allowed alongside the module itself. */
  readonly shared: readonly ModuleName[];
}

const utilities: readonly UtilityCase[] = [
  { binding: "anyAbortSignal", entry: "index", module: "abort-signal", shared: [] },
  { binding: "tryOnScopeDispose", entry: "index", module: "scope", shared: [] },
  {
    binding: "availableCapability",
    entry: "index",
    module: "capability-available",
    shared: [],
  },
  {
    binding: "unavailableCapability",
    entry: "index",
    module: "capability-unavailable",
    shared: [],
  },
  {
    binding: "isCapabilityAvailable",
    entry: "index",
    module: "capability-is-available",
    shared: [],
  },
  {
    binding: "isCapabilityUnavailable",
    entry: "index",
    module: "capability-is-unavailable",
    shared: [],
  },
  {
    binding: "createDisposalScope",
    entry: "index",
    module: "disposal-scope",
    shared: ["scope"],
  },
  { binding: "useEventListener", entry: "index", module: "event-listener", shared: ["scope"] },
  { binding: "useMediaQuery", entry: "index", module: "media-query", shared: [] },
  { binding: "useReducedMotion", entry: "index", module: "media-query", shared: [] },
  { binding: "useLocale", entry: "index", module: "locale", shared: [] },
  { binding: "useAsyncResource", entry: "index", module: "async-resource", shared: ["scope"] },
  { binding: "useTemporalNow", entry: "temporal", module: "temporal", shared: [] },
  { binding: "useTemporalZonedDateTime", entry: "temporal", module: "temporal", shared: [] },
  { binding: "usePrevious", entry: "index", module: "use-previous", shared: [] },
  { binding: "useHistory", entry: "index", module: "use-history", shared: ["scope"] },
  { binding: "useDebounced", entry: "index", module: "use-debounced", shared: ["scope"] },
  { binding: "useThrottled", entry: "index", module: "use-throttled", shared: ["scope"] },
  { binding: "useToggle", entry: "index", module: "use-toggle", shared: [] },
  { binding: "useCounter", entry: "index", module: "use-counter", shared: [] },
];

const VIRTUAL_ID = "virtual:compose-treeshake-entry";
const RESOLVED_ID = `\0${VIRTUAL_ID}`;

/**
 * Bundle a scratch entry in production mode, exactly like a consumer build.
 *
 * By default the package's own `"sideEffects": false` manifest hint is
 * neutralized: every `src` module is resolved with `moduleSideEffects: true`,
 * so statements survive on honest per-statement analysis alone. Without this,
 * the manifest lets the bundler drop a genuinely side-effectful module
 * wholesale and the emptiness assertions below would be vacuous.
 * `trustManifest` restores the consumer-visible semantics.
 */
async function bundleEntry(entryCode: string, trustManifest = false): Promise<string> {
  const result = await build({
    configFile: false,
    logLevel: "error",
    root: packageRoot,
    plugins: [
      {
        name: "compose-treeshake-entry",
        enforce: "pre",
        resolveId(id: string, importer: string | undefined) {
          if (id === VIRTUAL_ID) return RESOLVED_ID;
          if (trustManifest) return undefined;
          const importedFrom = importer === RESOLVED_ID ? packageRoot : dirname(importer ?? "/");
          const target = id.startsWith("./") ? resolve(importedFrom, id) : id;
          if (target.startsWith(sourceDirectory) && target.endsWith(".ts")) {
            return { id: target, moduleSideEffects: true };
          }
          return undefined;
        },
        load(id: string) {
          return id === RESOLVED_ID ? entryCode : undefined;
        },
      },
    ],
    build: {
      write: false,
      minify: false,
      target: "es2022",
      reportCompressedSize: false,
      rollupOptions: {
        input: VIRTUAL_ID,
        preserveEntrySignatures: "strict",
        // The peer and the runtime dependency stay external, matching how
        // `vp pack` builds the published entries.
        external: ["vue", "temporal-polyfill-lite"],
        // Both externals declare side-effect freedom in their own manifests;
        // unresolved externals would otherwise be conservatively retained.
        treeshake: { moduleSideEffects: "no-external" },
        output: { format: "es" },
      },
    },
  });
  assert.ok(!Array.isArray(result), "expected a single-environment build result");
  assert.ok("output" in result, "expected an in-memory rollup output");
  return result.output.map((item) => (item.type === "chunk" ? item.code : "")).join("\n");
}

/** Remove block comments and comment-only lines before sentinel matching. */
function stripComments(code: string): string {
  return code.replaceAll(/\/\*[^]*?\*\//g, "").replaceAll(/^[\t ]*\/\/.*$/gm, "");
}

void test("the package manifest declares side-effect freedom", () => {
  const manifest = JSON.parse(
    readFileSync(new URL("../package.json", import.meta.url), "utf8"),
  ) as { sideEffects?: unknown };

  assert.equal(manifest.sideEffects, false);
});

void test("module-level code performs no work: a bare import bundles to nothing", async () => {
  for (const [name, entry] of Object.entries(entries)) {
    const code = stripComments(await bundleEntry(`import ${JSON.stringify(entry)};`));
    assert.equal(
      code.trim(),
      "",
      `entry "${name}" retained module-level side effects:\n${code.trim()}`,
    );
  }
});

void test("consumers honoring the manifest can drop the unused package wholesale", async () => {
  for (const [name, entry] of Object.entries(entries)) {
    const code = stripComments(await bundleEntry(`import ${JSON.stringify(entry)};`, true));
    assert.equal(code.trim(), "", `entry "${name}" must be fully removable when unused`);
  }
});

for (const { binding, entry, module, shared } of utilities) {
  void test(`bundling only ${binding} excludes every unrelated utility`, async () => {
    const code = stripComments(
      await bundleEntry(`export { ${binding} } from ${JSON.stringify(entries[entry])};`),
    );

    for (const sentinel of sentinels[module]) {
      assert.ok(code.includes(sentinel), `${binding} bundle lost its own marker "${sentinel}"`);
    }

    const allowed = new Set<ModuleName>([module, ...shared]);
    for (const other of Object.keys(sentinels) as ModuleName[]) {
      if (allowed.has(other)) continue;
      for (const sentinel of sentinels[other]) {
        assert.ok(
          !code.includes(sentinel),
          `${binding} bundle must not pull "${sentinel}" from ${other}`,
        );
      }
    }
  });
}

void test("unused sibling exports of the same module are eliminated", async () => {
  const media = stripComments(
    await bundleEntry(`export { useMediaQuery } from ${JSON.stringify(entries.index)};`),
  );
  assert.ok(!media.includes("useReducedMotion"), "useReducedMotion should be dropped");
  assert.ok(!media.includes("prefers-reduced-motion"), "the motion query should be dropped");

  const temporal = stripComments(
    await bundleEntry(`export { useTemporalNow } from ${JSON.stringify(entries.temporal)};`),
  );
  assert.ok(
    !temporal.includes("useTemporalZonedDateTime"),
    "useTemporalZonedDateTime should be dropped",
  );
});

void test("importing an entry executes no module-level browser-global access", () => {
  for (const [name, entry] of Object.entries(entries)) {
    // The peer dependency is imported before the traps are installed, so the
    // probe measures this package (plus its bundled dependencies) only. The
    // throwing getters fail even a `typeof window` guard hoisted to module
    // scope, keeping capability detection lazy by construction. The probe
    // runs `src/*.ts` through the same type-stripping runtime as this suite.
    const probe = [
      'import "vue";',
      'for (const name of ["window", "document", "navigator"]) {',
      "  Object.defineProperty(globalThis, name, {",
      "    configurable: true,",
      "    get() {",
      "      throw new Error(`[import-side-effect] module-level access to globalThis.${name}`);",
      "    },",
      "  });",
      "}",
      `await import(${JSON.stringify(pathToFileURL(entry).href)});`,
    ].join("\n");

    const result = spawnSync(process.execPath, ["--input-type=module", "--eval", probe], {
      cwd: packageRoot,
      encoding: "utf8",
    });
    assert.equal(
      result.status,
      0,
      `entry "${name}" ran a module-level side effect:\n${result.stderr}`,
    );
  }
});
