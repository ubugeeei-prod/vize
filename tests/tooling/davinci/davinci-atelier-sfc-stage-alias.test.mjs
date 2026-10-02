import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse as parseToml } from "@iarna/toml";

import { scanConsumerMigrationSurfaces } from "../../../tools/support/compat/davinci/lib/consumer-migration-scan.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const sfcPreferredL0Rows = [
  ["crates/vize_atelier_sfc/src/bundler/asset_rewrite.rs", "source", 1],
  ["crates/vize_atelier_sfc/src/bundler/asset_rewrite/replacements.rs", "source", 1],
  ["crates/vize_atelier_sfc/src/bundler/assets.rs", "source", 1],
  ["crates/vize_atelier_sfc/src/bundler/blocks.rs", "source", 2],
  ["crates/vize_atelier_sfc/src/bundler/css.rs", "source", 1],
  ["crates/vize_atelier_sfc/src/bundler/scope.rs", "source", 1],
  ["crates/vize_atelier_sfc/src/compile/bindings.rs", "source", 1],
  ["crates/vize_atelier_sfc/src/compile/diagnostics.rs", "source", 1],
  ["crates/vize_atelier_sfc/src/compile/empty_component.rs", "source", 1],
  ["crates/vize_atelier_sfc/src/compile_template.rs", "source", 2],
  ["crates/vize_atelier_sfc/src/compile_template.rs", "test", 1],
  ["crates/vize_atelier_sfc/src/compile_template/extraction.rs", "source", 1],
  ["crates/vize_atelier_sfc/src/compile_template/section_offsets_tests.rs", "test", 1],
  ["crates/vize_atelier_sfc/src/compile_template/vapor.rs", "source", 7],
  ["crates/vize_atelier_sfc/tests/allocation_budget.rs", "test", 1],
  ["crates/vize_atelier_sfc/tests/component_spread_props.rs", "test", 1],
  ["crates/vize_atelier_sfc/tests/css_engine_panic_boundary.rs", "test", 5],
  ["crates/vize_atelier_sfc/tests/css_nesting_guard.rs", "test", 6],
  ["crates/vize_atelier_sfc/tests/custom_directive_patch_flag.rs", "test", 1],
  ["crates/vize_atelier_sfc/tests/davinci_arena_pool.rs", "test", 1],
  ["crates/vize_atelier_sfc/tests/emitter_javascript_output.rs", "test", 3],
  ["crates/vize_atelier_sfc/tests/imported_pick_intersection_props.rs", "test", 1],
  ["crates/vize_atelier_sfc/tests/imported_type_cache.rs", "test", 2],
  ["crates/vize_atelier_sfc/tests/workspace_prop_types.rs", "test", 2],
];

function assertPreferredDependencies(cargoToml) {
  const manifest = parseToml(cargoToml);
  assert.deepEqual(manifest.dependencies.vize_l0, { workspace: true });
  assert.deepEqual(manifest["dev-dependencies"].vize_carton, { workspace: true });
  for (const scope of [manifest, ...Object.values(manifest.target ?? {})])
    for (const kind of ["dependencies", "build-dependencies"])
      for (const [key, value] of Object.entries(scope[kind] ?? {}))
        assert.ok(key !== "vize_carton" && value?.package !== "vize_carton");
}

function compiler() {
  const scan = scanConsumerMigrationSurfaces();
  const consumer = scan.consumers.find((candidate) => candidate.id === "compiler");
  assert.ok(consumer);
  return consumer;
}

function assertPreferredSource(source, relPath) {
  const selected =
    relPath === "crates/vize_atelier_sfc/tests/allocation_budget.rs"
      ? source.replace(
          /^#\[global_allocator\]\nstatic GLOBAL: ProfilingAllocator<System> = vize_carton::profile_allocator::system_allocator\(\);$/mu,
          "",
        )
      : source;
  assert.doesNotMatch(selected, /\bvize_carton\b/u, relPath);
}

void test("Atelier SFC declares the L0 dependency through the preferred name", () => {
  const cargoToml = fs.readFileSync(
    path.join(repoRoot, "crates", "vize_atelier_sfc", "Cargo.toml"),
    "utf8",
  );

  assertPreferredDependencies(cargoToml);
});

void test("Atelier SFC permits the profiler example dev edge while rejecting normal and build Carton edges", () => {
  const cargoToml = fs.readFileSync(
    path.join(repoRoot, "crates", "vize_atelier_sfc", "Cargo.toml"),
    "utf8",
  );
  for (const declaration of [
    "[dependencies]\nvize_carton.workspace = true\n",
    '[build-dependencies]\nprofile_host = { package = "vize_carton", version = "*" }\n',
    '[target.\'cfg(unix)\'.dependencies]\n"vize_carton"."workspace" = true\n',
    '[target.\'cfg(unix)\'.build-dependencies]\nprofile_host = { package = "vize_carton", version = "*", optional = true }\n',
  ]) {
    const section = declaration.slice(0, declaration.indexOf("\n") + 1);
    const changed = cargoToml.includes(section)
      ? cargoToml.replace(section, declaration)
      : cargoToml + "\n" + declaration;
    assert.throws(() => assertPreferredDependencies(changed), { code: "ERR_ASSERTION" });
  }
});

void test("Atelier SFC selected compiler and integration test slices import L0 through the preferred name", () => {
  const rows = compiler().fileRows;

  for (const [relPath, mode, sites] of sfcPreferredL0Rows) {
    const row = rows.find((candidate) => candidate.relPath === relPath && candidate.mode === mode);
    assert.ok(row, `${relPath} (${mode})`);
    assert.equal(row.surfaceCounts.l0, sites, relPath);
    assert.equal(row.surfaceNameCounts.l0.vize_l0, sites, relPath);
    assert.equal(row.surfaceNameCounts.l0.vize_carton ?? 0, 0, relPath);

    const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
    assertPreferredSource(source, relPath);
  }
});

void test("Atelier SFC admits only its exact host allocator selection and rejects Carton storage", () => {
  const relPath = "crates/vize_atelier_sfc/tests/allocation_budget.rs";
  const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
  assertPreferredSource(source, relPath);
  assert.throws(() =>
    assertPreferredSource(source, "crates/vize_atelier_sfc/src/compile_template.rs"),
  );
  for (const changed of [
    source.replace("#[global_allocator]\n", ""),
    source.replace("ProfilingAllocator<System>", "ProfilingAllocator<WrongAllocator>"),
    source.replace("system_allocator()", "system_allocator(WrongAllocator)"),
    source.replace("profile_allocator::system_allocator", "profile_export::system_allocator"),
    source + "\nuse vize_carton::String;\n",
    source + "\nuse vize_carton::profiler::ProfilingAllocator;\n",
    source +
      "\nstatic OTHER: ProfilingAllocator<System> = vize_carton::profile_allocator::system_allocator();\n",
    source +
      "\nstatic GLOBAL: ProfilingAllocator<System> = vize_carton::profile_allocator::system_allocator();\n",
  ]) {
    assert.throws(() => assertPreferredSource(changed, relPath), { code: "ERR_ASSERTION" });
  }
});
