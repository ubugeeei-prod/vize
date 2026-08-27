import assert from "node:assert/strict";
import { test } from "node:test";

import {
  SURFACES,
  scanConsumerMigrationSurfaces,
  surfaceNameKind,
} from "../../tools/davinci/lib/consumer-migration-scan.mjs";
import { renderConsumerMigrationSurfaceRows } from "../../tools/davinci/lib/consumer-migration-render.mjs";

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

function expectCompilerS0PreferredRow(compiler, relPath, mode, sites) {
  const row = compiler.fileRows.find(
    (candidate) => candidate.relPath === relPath && candidate.mode === mode,
  );
  assert.ok(row, `${relPath} (${mode})`);
  assert.equal(row.surfaceCounts.s0, sites, relPath);
  assert.equal(row.surfaceNameCounts.s0.vize_s0, sites, relPath);
  assert.equal(row.surfaceNameCounts.s0.vize_carton ?? 0, 0, relPath);
}

void test("consumer migration scan classifies stage physical names separately from code names", () => {
  const s0 = SURFACES.find((surface) => surface.id === "s0");
  assert.ok(s0);
  assert.equal(s0.label, "S0");
  assert.equal(surfaceNameKind(s0, "vize_s0"), "preferred");
  assert.equal(surfaceNameKind(s0, "vize_carton"), "compat");

  const stageSurfaces = SURFACES.filter((surface) => surface.group === "stage");
  assert.deepEqual(
    stageSurfaces.map((surface) => surface.label),
    ["Davinci", "S0", "S1", "S2", "S1->S2"],
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
        row.surface_id === "s0" && row.matched_name === "vize_s0" && row.name_kind === "preferred",
    ),
  );

  const physicalLabels = new Map([
    ["s0", "S0"],
    ["s1", "S1"],
    ["s2", "S2"],
    ["s1_to_s2", "S1->S2"],
  ]);
  for (const row of rows.filter((row) => physicalLabels.has(row.surface_id))) {
    assert.equal(row.surface_group, "stage");
    assert.equal(row.surface, physicalLabels.get(row.surface_id));
  }
  for (const legacyLabel of ["S0/carton", "S1/sinopia", "S2/disegno", "S1->S2/ricalco"]) {
    assert.ok(!rows.some((row) => row.surface === legacyLabel), legacyLabel);
  }
});

void test("consumer migration TSV serializes every stage label and name class", () => {
  const stages = [
    { id: "s0", label: "S0", preferred: "vize_s0", compat: "vize_carton" },
    { id: "s1", label: "S1", preferred: "vize_s1", compat: "vize_sinopia" },
    { id: "s2", label: "S2", preferred: "vize_s2", compat: "vize_disegno" },
    { id: "s1_to_s2", label: "S1->S2", preferred: "vize_s1_to_s2", compat: "vize_ricalco" },
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

void test("Atelier core emit codegen imports S0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  expectCompilerS0PreferredRow(
    compiler,
    "crates/vize_atelier_core/src/codegen/emit.rs",
    "source",
    2,
  );
});

void test("Atelier core expression codegen imports S0 through the preferred physical name", () => {
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
    expectCompilerS0PreferredRow(compiler, relPath, mode, sites);
  }
});

void test("Atelier core generate codegen imports S0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  const expectedRows = [
    ["crates/vize_atelier_core/src/codegen/generate.rs", "source", 6],
    ["crates/vize_atelier_core/src/codegen/generate/collect_helpers.rs", "source", 1],
    ["crates/vize_atelier_core/src/codegen/generate/static_vnode.rs", "source", 4],
  ];
  for (const [relPath, mode, sites] of expectedRows) {
    expectCompilerS0PreferredRow(compiler, relPath, mode, sites);
  }
});

void test("Atelier core node codegen imports S0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  expectCompilerS0PreferredRow(
    compiler,
    "crates/vize_atelier_core/src/codegen/node.rs",
    "source",
    1,
  );
});

void test("Atelier core patch flag codegen imports S0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  const expectedRows = [
    ["crates/vize_atelier_core/src/codegen/patch_flag.rs", "source", 1],
    ["crates/vize_atelier_core/src/codegen/patch_flag/static_literal.rs", "source", 1],
  ];
  for (const [relPath, mode, sites] of expectedRows) {
    expectCompilerS0PreferredRow(compiler, relPath, mode, sites);
  }
});

void test("Atelier core root codegen imports S0 through the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const compiler = scan.consumers.find((consumer) => consumer.id === "compiler");
  assert.ok(compiler);

  expectCompilerS0PreferredRow(
    compiler,
    "crates/vize_atelier_core/src/codegen/root.rs",
    "source",
    1,
  );
});

void test("consumer migration scan keeps every rollout consumer and surface class visible", () => {
  const scan = scanConsumerMigrationSurfaces();
  const consumers = new Map(scan.consumers.map((consumer) => [consumer.id, consumer]));
  const expected = new Map([
    ["compiler", { stage: true, old: true, raw: true }],
    ["linter", { stage: true, old: true, raw: true }],
    ["typechecker", { stage: true, old: true, raw: true }],
    ["typechecker-content-mapper", { stage: true, old: true, raw: false }],
    ["formatter", { stage: true, old: false, raw: true }],
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

void test("content-mapper S0 surface stays on the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const contentMapper = scan.consumers.find(
    (consumer) => consumer.id === "typechecker-content-mapper",
  );
  assert.ok(contentMapper);
  assert.equal(contentMapper.surfaceCounts.s0, 8);
  assert.equal(contentMapper.nameKindCounts.compat, 0);
  assert.equal(contentMapper.nameKindCounts.preferred, 8);
  assert.ok(
    contentMapper.sites
      .filter((site) => site.surfaceId === "s0")
      .every((site) => site.matchedName === "vize_s0" && site.nameKind === "preferred"),
  );
});

void test("LSP S0 surface stays on the preferred physical name", () => {
  const scan = scanConsumerMigrationSurfaces();
  const lsp = scan.consumers.find((consumer) => consumer.id === "lsp");
  assert.ok(lsp);
  assert.equal(lsp.nameKindCounts.compat, 0);
  assert.ok(lsp.nameKindCounts.preferred > 0, "LSP should keep importing S0 explicitly");
  assert.ok(
    lsp.sites
      .filter((site) => site.surfaceId === "s0")
      .every((site) => site.matchedName === "vize_s0" && site.nameKind === "preferred"),
  );
});
