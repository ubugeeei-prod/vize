import assert from "node:assert/strict";
import { test } from "node:test";

import {
  SURFACES,
  scanConsumerMigrationSurfaces,
  surfaceNameKind,
} from "../../tools/davinci/lib/consumer-migration-scan.mjs";
import { renderConsumerMigrationSurfaceRows } from "../../tools/davinci/lib/consumer-migration-render.mjs";

function sumValues(record) {
  return Object.values(record).reduce((sum, value) => sum + value, 0);
}

void test("consumer migration scan classifies stage physical names separately from code names", () => {
  const s0 = SURFACES.find((surface) => surface.id === "s0");
  assert.ok(s0);
  assert.equal(surfaceNameKind(s0, "vize_s0"), "preferred");
  assert.equal(surfaceNameKind(s0, "vize_carton"), "compat");

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
  const tsv = renderConsumerMigrationSurfaceRows(scan);
  const [header, ...rows] = tsv.trimEnd().split("\n");

  assert.deepEqual(header.split("\t"), [
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
  ]);
  assert.ok(rows.every((row) => row.split("\t").length === 11));
  assert.ok(rows.some((row) => row.includes("\tvize_s0\tpreferred\t")));
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
