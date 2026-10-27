import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { test } from "node:test";

import {
  SURFACES,
  scanConsumerMigrationSurfaces,
  surfaceNameKind,
} from "../../tools/support/compat/davinci/lib/consumer-migration-scan.ts";
import { lexicalView } from "../../tools/support/levels/move-host-runtime.ts";
import { renderConsumerMigrationSurfaceRows } from "../../tools/support/compat/davinci/lib/consumer-migration-render.ts";

const TSV_HEADERS = [
  "consumer_id",
  "consumer",
  "class",
  "file",
  "first_line",
  "surface_id",
  "surface",
  "surface_group",
  "matched_name",
  "name_kind",
  "sites",
];

function sumValues(record) {
  return Object.values(record).reduce((sum, value) => sum + value, 0);
}

function parseSurfaceRows(tsv) {
  const [header, ...lines] = tsv.trimEnd().split("\n");
  assert.deepEqual(header.split("\t"), TSV_HEADERS);
  return lines.map((line) => {
    const fields = line.split("\t");
    assert.equal(fields.length, TSV_HEADERS.length, line);
    return Object.fromEntries(TSV_HEADERS.map((name, index) => [name, fields[index]]));
  });
}

function expectCompilerL0PreferredRow(compiler, relPath, mode, sites) {
  const row = compiler.fileRows.find(
    (candidate) => candidate.relPath === relPath && candidate.mode === mode,
  );
  assert.ok(row, `${relPath} (${mode})`);
  assert.equal(row.surfaceCounts.l0, sites, relPath);
  assert.equal(row.surfaceNameCounts.l0.vize_l0, sites, relPath);
  assert.equal(row.surfaceNameCounts.l0.vize_carton ?? 0, 0, relPath);
}

void test("consumer migration scan classifies stage physical names separately from code names", () => {
  const s0 = SURFACES.find((surface) => surface.id === "l0");
  assert.ok(s0);
  assert.equal(s0.label, "L0");
  assert.equal(surfaceNameKind(s0, "vize_l0"), "preferred");
  assert.ok(!s0.names.includes("vize_carton"));
  const stageSurfaces = SURFACES.filter((surface) => surface.group === "stage");
  assert.deepEqual(
    stageSurfaces.map((surface) => surface.label),
    ["L0", "L1", "L2", "L1->L2"],
  );
  assert.ok(stageSurfaces.every((surface) => !surface.label.includes("/")));
  const rawOxc = SURFACES.find((surface) => surface.id === "raw_oxc");
  assert.ok(rawOxc);
  assert.equal(surfaceNameKind(rawOxc, "oxc_ast"), "raw");
});

void test("consumer migration rows preserve surface totals when split by matched name", () => {
  const scan = scanConsumerMigrationSurfaces();
  for (const consumer of scan.consumers) {
    assert.equal(
      consumer.groupCounts.stage,
      consumer.nameKindCounts.preferred + consumer.nameKindCounts.compat,
      `${consumer.id} stage totals must equal preferred plus compat names`,
    );
    for (const row of consumer.fileRows) {
      for (const surface of SURFACES) {
        assert.equal(
          row.surfaceCounts[surface.id],
          sumValues(row.surfaceNameCounts[surface.id]),
          `${consumer.id} ${row.relPath} ${surface.id} name totals drifted`,
        );
      }
    }
  }
});

void test("consumer migration TSV exposes matched names and name kind", () => {
  const scan = scanConsumerMigrationSurfaces();
  const rows = parseSurfaceRows(renderConsumerMigrationSurfaceRows(scan));

  assert.ok(
    rows.some(
      (row) =>
        row.surface_id === "l0" && row.matched_name === "vize_l0" && row.name_kind === "preferred",
    ),
  );

  const physicalLabels = new Map([
    ["l0", "L0"],
    ["l1", "L1"],
    ["l2", "L2"],
    ["l1_to_l2", "L1->L2"],
  ]);
  for (const row of rows.filter((row) => physicalLabels.has(row.surface_id))) {
    assert.equal(row.surface_group, "stage");
    assert.equal(row.surface, physicalLabels.get(row.surface_id));
  }
  for (const legacyLabel of ["L0/carton", "L1/sinopia", "L2/disegno", "L1->L2/ricalco"]) {
    assert.ok(!rows.some((row) => row.surface === legacyLabel), legacyLabel);
  }
});

void test("consumer migration TSV serializes retained compatibility stage names", () => {
  const stages = [
    { id: "l1", label: "L1", preferred: "vize_l1", compat: "vize_sinopia" },
    { id: "l2", label: "L2", preferred: "vize_l2", compat: "vize_disegno" },
    { id: "l1_to_l2", label: "L1->L2", preferred: "vize_l1_to_l2", compat: "vize_ricalco" },
  ];
  const surfaceNameCounts = Object.fromEntries(SURFACES.map((surface) => [surface.id, {}]));
  for (const stage of stages) {
    surfaceNameCounts[stage.id] = {
      [stage.preferred]: 1,
      [stage.compat]: 2,
    };
  }

  const rows = parseSurfaceRows(
    renderConsumerMigrationSurfaceRows({
      consumers: [
        {
          id: "fixture",
          label: "Fixture",
          fileRows: [
            {
              relPath: "fixture.rs",
              mode: "source",
              firstLine: 1,
              surfaceNameCounts,
            },
          ],
        },
      ],
    }),
  );

  for (const stage of stages) {
    const preferred = rows.find(
      (row) => row.surface_id === stage.id && row.matched_name === stage.preferred,
    );
    assert.ok(preferred, stage.preferred);
    assert.equal(preferred.surface, stage.label);
    assert.equal(preferred.surface_group, "stage");
    assert.equal(preferred.name_kind, "preferred");
    assert.equal(preferred.sites, "1");

    const compat = rows.find(
      (row) => row.surface_id === stage.id && row.matched_name === stage.compat,
    );
    assert.ok(compat, stage.compat);
    assert.equal(compat.surface, stage.label);
    assert.equal(compat.surface_group, "stage");
    assert.equal(compat.name_kind, "compat");
    assert.equal(compat.sites, "2");
  }
});

void test("Atelier core emit codegen imports L0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  expectCompilerL0PreferredRow(
    compiler,
    "crates/vize_atelier_core/src/codegen/emit.rs",
    "source",
    2,
  );
});

void test("Atelier core expression codegen imports L0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);
  const expectedRows = [
    ["crates/vize_atelier_core/src/codegen/expression.rs", "source", 2],
    ["crates/vize_atelier_core/src/codegen/expression/comment_rewrite.rs", "source", 1],
    ["crates/vize_atelier_core/src/codegen/expression/generate.rs", "source", 1],
    ["crates/vize_atelier_core/src/codegen/expression/generate.rs", "test", 2],
    ["crates/vize_atelier_core/src/codegen/expression/helpers.rs", "source", 1],
    ["crates/vize_atelier_core/src/codegen/expression/prefix_context.rs", "source", 3],
    ["crates/vize_atelier_core/src/codegen/expression/prefix_visitor.rs", "source", 3],
    ["crates/vize_atelier_core/src/codegen/expression/scope_prefix.rs", "source", 1],
  ];
  for (const [relPath, mode, sites] of expectedRows) {
    expectCompilerL0PreferredRow(compiler, relPath, mode, sites);
  }
});

void test("Atelier core generate codegen imports L0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  const expectedRows = [
    ["crates/vize_atelier_core/src/codegen/generate.rs", "source", 6],
    ["crates/vize_atelier_core/src/codegen/generate/collect_helpers.rs", "source", 1],
    ["crates/vize_atelier_core/src/codegen/generate/static_vnode.rs", "source", 4],
  ];
  for (const [relPath, mode, sites] of expectedRows) {
    expectCompilerL0PreferredRow(compiler, relPath, mode, sites);
  }
});

void test("Atelier core node codegen imports L0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  expectCompilerL0PreferredRow(
    compiler,
    "crates/vize_atelier_core/src/codegen/node.rs",
    "source",
    1,
  );
});

void test("Atelier core helper codegen imports L0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  const expectedRows = [
    ["crates/vize_atelier_core/src/codegen/helpers.rs", "source", 2],
    ["crates/vize_atelier_core/src/codegen/helpers/constant_expression.rs", "source", 3],
  ];
  for (const [relPath, mode, sites] of expectedRows) {
    expectCompilerL0PreferredRow(compiler, relPath, mode, sites);
  }
});

void test("Atelier core patch flag codegen imports L0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  const expectedRows = [
    ["crates/vize_atelier_core/src/codegen/patch_flag.rs", "source", 1],
    ["crates/vize_atelier_core/src/codegen/patch_flag/static_literal.rs", "source", 1],
  ];
  for (const [relPath, mode, sites] of expectedRows) {
    expectCompilerL0PreferredRow(compiler, relPath, mode, sites);
  }
});

void test("Atelier core singleton and element step slices import L0 through the preferred name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  const expectedRows = [
    ["crates/vize_atelier_core/src/codegen/root.rs", "source", 1],
    ["crates/vize_atelier_core/src/runtime_helpers.rs", "source", 1],
    ["crates/vize_atelier_core/src/lane/options.rs", "source", 1],
    ["crates/vize_atelier_core/src/lane/structural_keys.rs", "source", 1],
    ["crates/vize_atelier_core/src/lane/context.rs", "source", 3],
    ["crates/vize_atelier_core/src/lane/context/for_bindings.rs", "source", 1],
    ["crates/vize_atelier_core/src/steps/element.rs", "source", 3],
    ["crates/vize_atelier_core/src/steps/element.rs", "test", 1],
  ];
  for (const [relPath, mode, sites] of expectedRows) {
    expectCompilerL0PreferredRow(compiler, relPath, mode, sites);
  }
});

void test("Atelier core lane element imports L0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  const expectedRows = [
    ["crates/vize_atelier_core/src/lane/element.rs", "source", 2],
    ["crates/vize_atelier_core/src/lane/element/directive_expressions.rs", "source", 1],
    ["crates/vize_atelier_core/src/lane/element/tests.rs", "test", 1],
    ["crates/vize_atelier_core/src/lane/extensions.rs", "source", 1],
    ["crates/vize_atelier_core/src/lane.rs", "source", 1],
    ["crates/vize_atelier_core/src/lane/patterned_template.rs", "source", 1],
    ["crates/vize_atelier_core/src/lane/structural.rs", "source", 1],
    ["crates/vize_atelier_core/src/lane/traverse.rs", "source", 3],
  ];
  for (const [relPath, mode, sites] of expectedRows) {
    expectCompilerL0PreferredRow(compiler, relPath, mode, sites);
  }
});

void test("consumer migration scan keeps every rollout consumer and surface class visible", () => {
  const scan = scanConsumerMigrationSurfaces();
  const consumers = new Map(scan.consumers.map((consumer) => [consumer.id, consumer]));
  const expected = new Map([
    ["compiler", { stage: true, old: true, raw: true }],
    ["linter", { stage: true, old: true, raw: true }],
    ["typechecker", { stage: true, old: true, raw: true }],
    // Project-root normalization is an OS host adapter. It is recorded as
    // Carton host presence; the focused guard below still rejects old AST/OXC.
    ["typechecker-content-mapper", { stage: true, old: true, raw: false }],
    ["formatter", { stage: true, old: true, raw: true }],
    ["lsp", { stage: true, old: true, raw: true }],
  ]);

  assert.deepEqual([...consumers.keys()], [...expected.keys()]);
  for (const [id, groups] of expected) {
    const consumer = consumers.get(id);
    assert.ok(consumer, id);
    assert.ok(consumer.fileCount > 0, `${id} must scan at least one file`);
    assert.ok(consumer.surfaceFileCount > 0, `${id} must expose surface rows`);
    assert.ok(
      consumer.modeCounts.source + consumer.modeCounts.manifest > 0,
      `${id} must scan source or manifest files`,
    );
    for (const [group, present] of Object.entries(groups)) {
      assert.equal(consumer.groupCounts[group] > 0, present, `${id} ${group} presence drifted`);
    }
  }
});

void test("content-mapper L0 surface stays on the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const contentMapper = scan.consumers.find(
    (consumer) => consumer.id === "typechecker-content-mapper",
  );
  assert.ok(contentMapper);
  assertPreferredContentMapperL0(contentMapper);
});

function assertPreferredContentMapperL0(contentMapper) {
  const sites = contentMapper.sites.filter((site) => site.surfaceId === "l0");
  // This scanner counts lexical references, including test helpers, not public
  // exports. Their cardinality can grow while the physical-name contract holds.
  assert.ok(sites.length > 0, "content mapper must have at least one L0 site");
  assert.equal(contentMapper.nameKindCounts.compat, 0);
  assert.ok(
    sites.every((site) => site.matchedName === "vize_l0" && site.nameKind === "preferred"),
    "content mapper must use the preferred physical L0 name",
  );
}

void test("content-mapper physical-name contract refuses missing L0 and legacy aliases", () => {
  const contentMapper = scanConsumerMigrationSurfaces().consumers.find(
    (consumer) => consumer.id === "typechecker-content-mapper",
  );
  assert.ok(contentMapper);
  const missing = {
    ...contentMapper,
    sites: contentMapper.sites.filter((site) => site.surfaceId !== "l0"),
  };
  assert.throws(() => assertPreferredContentMapperL0(missing), {
    name: "AssertionError",
    message: "content mapper must have at least one L0 site",
  });
  const legacy = { ...contentMapper, sites: structuredClone(contentMapper.sites) };
  legacy.sites.find((site) => site.surfaceId === "l0").matchedName = "vize_carton";
  assert.throws(() => assertPreferredContentMapperL0(legacy), {
    name: "AssertionError",
    message: "content mapper must use the preferred physical L0 name",
  });
});

void test("LSP L0 surface stays on the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const lsp = scan.consumers.find((consumer) => consumer.id === "lsp");
  assert.ok(lsp);
  assert.equal(lsp.nameKindCounts.compat, 0);
  assert.ok(lsp.nameKindCounts.preferred > 0, "LSP should keep importing L0 explicitly");
  assert.ok(
    lsp.sites
      .filter((site) => site.surfaceId === "l0")
      .every((site) => site.matchedName === "vize_l0" && site.nameKind === "preferred"),
  );
});

const readSurfaceSource = (relPath) =>
  readFileSync(new URL(`../../${relPath}`, import.meta.url), "utf8");

function assertContentMapperHostPath(
  contentMapper,
  read = readSurfaceSource,
  foundation = read("davinci/vize_l0/src/lib.rs"),
  host = read("crates/vize_carton/src/path.rs"),
  lowerPathExists = existsSync(new URL("../../davinci/vize_l0/src/path.rs", import.meta.url)),
) {
  const nonStage = contentMapper.sites.filter((site) => site.group !== "stage");
  assert.ok(nonStage.length > 0, "the host adapter must remain visible in the inventory");
  assert.deepEqual([...new Set(nonStage.map((site) => site.surfaceId))], ["carton"]);
  for (const site of nonStage) {
    assert.equal(site.matchedName, "vize_carton");
    const line = lexicalView(read(site.relPath)).split("\n")[site.line - 1];
    assert.match(
      line,
      /\bvize_carton\s*::\s*path\s*::\s*canonicalize_non_verbatim\b/u,
      `${site.relPath}:${site.line} must be the existing filesystem normalization function`,
    );
  }
  assert.match(lexicalView(host), /\bpub\s+fn\s+canonicalize_non_verbatim\s*\(/u);
  assert.doesNotMatch(lexicalView(foundation), /\bpub\s+mod\s+path\s*;/u);
  assert.equal(lowerPathExists, false, "filesystem path implementation remains in the host layer");
}

void test("content mapper records the exact host path role without admitting legacy AST or raw OXC", () => {
  const mapper = scanConsumerMigrationSurfaces().consumers.find(
    (consumer) => consumer.id === "typechecker-content-mapper",
  );
  assert.ok(mapper);
  assertContentMapperHostPath(mapper);
  for (const [surfaceId, matchedName, group] of [
    ["old_ast", "vize_relief", "old"],
    ["croquis", "vize_croquis", "old"],
    ["raw_oxc", "oxc_parser", "raw"],
  ]) {
    const introduced = { ...mapper, sites: structuredClone(mapper.sites) };
    const host = introduced.sites.find((site) => site.surfaceId === "carton");
    introduced.sites.push({ ...host, surfaceId, matchedName, group });
    assert.throws(() => assertContentMapperHostPath(introduced), { name: "AssertionError" });
  }
  assert.throws(
    () =>
      assertContentMapperHostPath(mapper, (relPath) =>
        readSurfaceSource(relPath).replaceAll(
          "vize_carton::path::canonicalize_non_verbatim",
          "vize_carton::AstNode",
        ),
      ),
    { name: "AssertionError" },
  );
  assert.throws(
    () =>
      assertContentMapperHostPath({
        ...mapper,
        sites: mapper.sites.filter((site) => site.surfaceId !== "carton"),
      }),
    { name: "AssertionError" },
  );
  const foundation = readSurfaceSource("davinci/vize_l0/src/lib.rs");
  assert.throws(() =>
    assertContentMapperHostPath(mapper, readSurfaceSource, `${foundation}\npub mod path;\n`),
  );
  assert.throws(() =>
    assertContentMapperHostPath(
      mapper,
      readSurfaceSource,
      foundation,
      readSurfaceSource("crates/vize_carton/src/path.rs"),
      true,
    ),
  );
});
