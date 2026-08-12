//! The single source of truth for the `test:check:fixtures` lane (#4126).
//!
//! The lane used to be one shell string in `tests/package.json` that handed 35
//! files to a single `node --test` runner. That shape gave the runner no way to
//! attribute a failure to a phase: when the `vue-parity` job died with
//! `Failed to spawn process: Resource temporarily unavailable (os error 11)`,
//! the log truncated mid-way through one fixture's `--show-virtual-ts` dump and
//! nothing recorded which fixture owned the processes that were alive.
//!
//! Every entry here is a *phase*: one supervised child that the supervisor
//! samples before, at peak, and after, and whose descendants must be gone when
//! it returns. The order is the order the shell string used, and the runner
//! options are the ones it passed, because both are load-bearing: `zz-`
//! fixtures repair state the earlier ones plant, and `--test-concurrency=1`
//! keeps the fixtures from competing for the same materialized project.

export type CheckFixturePhase = {
  /** Stable phase id used in telemetry, artifacts, and `--only`. */
  readonly id: string;
  /** Test file path, relative to `tests/`. */
  readonly file: string;
};

/**
 * Node options every phase runs with. `--test-concurrency=1` is preserved from
 * the original script: a phase is a single test file, so the flag now pins the
 * per-phase runner to one in-flight file rather than merely serializing the
 * lane.
 */
export const CHECK_FIXTURE_NODE_ARGS: readonly string[] = ["--test", "--test-concurrency=1"];

/**
 * Environment every phase inherits. `VIZE_TEST_REQUIRE_TSGO=1` makes the
 * typecheck oracles fail closed when the Corsa runtime is missing instead of
 * quietly skipping.
 */
export const CHECK_FIXTURE_ENV: Readonly<Record<string, string>> = {
  VIZE_TEST_REQUIRE_TSGO: "1",
};

function phase(file: string): CheckFixturePhase {
  const id = file.replace(/^.*\//, "").replace(/\.(test\.)?ts$/, "");
  return { file, id };
}

export const checkFixturePhases: readonly CheckFixturePhase[] = [
  "snapshots/check/typecheck-errors.ts",
  "snapshots/check/typecheck-vue-imports.ts",
  "snapshots/check/compiler-macros.ts",
  "snapshots/check/style-preprocessors.ts",
  "snapshots/check/ecosystem-products.ts",
  "snapshots/check/typescript-go-module-resolution-determinism.ts",
  "snapshots/check/generic-build.ts",
  "snapshots/check/nuxt-parity.ts",
  "snapshots/check/toolchain-parity.ts",
  "snapshots/check/options-api.ts",
  "snapshots/check/class-component.ts",
  "snapshots/check/class-component-lsp-oracle.ts",
  "snapshots/check/create-vue-patch-oracle.ts",
  "snapshots/check/create-vue-generated-template-oracle.ts",
  "snapshots/check/create-vue-editor-range-oracle.ts",
  "snapshots/check/vue-benchmarks-correctness-plants.ts",
  "snapshots/check/javascript-sfc-checkjs-oracle.ts",
  "snapshots/check/vue-benchmarks-lsp-ref-unwrap-oracle.ts",
  "snapshots/check/template-ref-unwrap-oracle.ts",
  "snapshots/check/vue-benchmarks-scaled-corpus-plants.ts",
  "snapshots/check/vue-router-patch-oracle.ts",
  "snapshots/check/vue-router-formatter-oracle.ts",
  "snapshots/check/pinia-generic-store-oracle.ts",
  "snapshots/check/typescript-project-references-oracle.ts",
  "snapshots/check/vue-router-dmts-oracle.ts",
  "snapshots/check/element-plus-slot-oracle.ts",
  "snapshots/check/nuxt-ui-ambient-oracle.ts",
  "snapshots/check/nuxt-no-tsconfig-oracle.ts",
  "snapshots/check/nuxt-template-globals.ts",
  "snapshots/check/vitepress-theme-oracle.ts",
  "snapshots/check/vue-element-admin-legacy-oracle.ts",
  "snapshots/check/vue-element-admin-legacy-lsp-oracle.ts",
  "snapshots/check/vue-element-admin-unmapped-diagnostic-oracle.ts",
  "snapshots/check/vue2-elm.ts",
  "snapshots/check/vue2-class-component-oracle.ts",
  "snapshots/check/zz-intentional-errors-fixtures.ts",
  "tooling/compat-ratchet.test.ts",
].map(phase);
