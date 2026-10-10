import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import {
  preparePublication,
  publishSnapshot,
  REPO_ROOT,
  performancePath,
} from "../../tools/benchmarks/scripts/publish-snapshot.mjs";
import { RESULT_PATH } from "../../tools/benchmarks/scripts/published-snapshot-render.mjs";
import { SNAPSHOT_LOCALES } from "../../tools/benchmarks/scripts/published-snapshot-locales.mjs";
import { diagnosticText } from "../../tools/benchmarks/scripts/published-snapshot-diagnostics.mjs";
import { PROVENANCE_LABELS } from "../../tools/benchmarks/scripts/benchmark-provenance.mjs";
import {
  updateReadme,
  updatePerformance,
} from "../../tools/benchmarks/scripts/published-snapshot-sections.mjs";

const read = (file: string) => readFileSync(join(REPO_ROOT, file), "utf8");
const artifact = JSON.parse(read(RESULT_PATH));

test("one publisher reproduces the README, JSON and all ten public pages exactly", async () => {
  const outputs = await preparePublication(artifact, read);
  assert.equal(outputs.size, 12);
  for (const [file, content] of outputs) {
    assert.equal(content, read(file), file);
    if (file !== RESULT_PATH)
      assert.equal(content.match(/<details class="benchmark-provenance">/g)?.length, 1, file);
  }
  assert.deepEqual(await publishSnapshot({ check: true }), []);
});

test("re-rendering is idempotent and retains every rejected comparator and raw sample", async () => {
  const outputs = await preparePublication(artifact, read);
  const again = await preparePublication(artifact, (file: string) => outputs.get(file));
  assert.deepEqual(again, outputs);
  for (const locale of Object.keys(SNAPSHOT_LOCALES)) {
    const page = outputs.get(
      performancePath(locale).replace(/performance\.md$/u, "performance-blacksmith.md"),
    );
    for (const surface of artifact.surfaces) {
      for (const rejected of surface.rejectedVariants ?? []) {
        assert.ok(page.includes(rejected.label));
        assert.ok(page.includes(rejected.reason.replaceAll("```", "'''")));
      }
    }
    assert.ok(page.includes(artifact.commit.runUrl));
    const t = diagnosticText(locale);
    assert.ok(page.includes(t.validation));
    const rows = page
      .split("\n")
      .filter((line: string) => line.startsWith("|"))
      .map((line: string) =>
        line
          .split("|")
          .slice(1, -1)
          .map((cell) => cell.trim()),
      );
    for (const header of [
      t.columns,
      t.proofColumns,
      ...(locale === "en" ? [] : [t.sampleColumns]),
    ]) {
      assert.deepEqual(
        rows.find((row: string[]) => row[0] === header[0] && row.length === header.length),
        header,
      );
    }
    if (locale !== "en") {
      assert.ok(!page.includes("engine classes ranked separately"));
      assert.ok(!page.includes("diagnostics rechecked on every run"));
    }
    for (const surface of artifact.surfaces) {
      for (const variant of surface.variants) {
        if (variant.correctness)
          assert.ok(
            page.includes(String(variant.correctness.diagnosticCount)),
            "publish actual diagnostic counts",
          );
      }
    }
  }
});

test("missing or duplicate markers fail closed before any files can be published", async () => {
  for (const corrupt of [
    (source: string) => source.replace("<!-- benchmark:readme:end -->", ""),
    (source: string) => `${source}\n<!-- benchmark:readme:start -->`,
  ]) {
    await assert.rejects(
      preparePublication(artifact, (file: string) =>
        file === "README.md" ? corrupt(read(file)) : read(file),
      ),
      /missing or ambiguous/,
    );
  }
});

test("every locale puts measured results before labeled, full binary provenance", async () => {
  const outputs = await preparePublication(artifact, read);
  const toolNames = {
    vize: "Vize",
    tsgo: "tsgo",
    vueTsc: "vue-tsc",
    verterTsc: "verter-tsc",
    golar: "Golar",
    eslint: "ESLint",
    prettier: "Prettier",
  };
  for (const locale of Object.keys(SNAPSHOT_LOCALES)) {
    const source = outputs.get(
      performancePath(locale).replace(/performance\.md$/u, "performance-blacksmith.md"),
    );
    const start = source.indexOf('<details class="benchmark-provenance">');
    const end = source.indexOf("</details>", start);
    assert.ok(start > source.indexOf("| Vize"), `${locale}: result table before metadata`);
    assert.ok(
      start > source.indexOf(SNAPSHOT_LOCALES[locale].note),
      `${locale}: explanation first`,
    );
    const details = source.slice(start, end);
    assert.ok(details.includes(`<summary>${PROVENANCE_LABELS[locale][0]}</summary>`));
    const rows = details.split("\n").filter((line: string) => line.startsWith("|"));
    for (const [key, label] of Object.entries(toolNames)) {
      const row = rows.find((line: string) => line.split("|")[1].trim() === label);
      assert.ok(row, `${locale}: ${label} has its own row`);
      assert.deepEqual(
        row
          .split("|")
          .slice(1, -1)
          .map((cell: string) => cell.trim()),
        [label, `<code>${artifact.versions[key]}</code>`, `<code>${artifact.binaries[key]}</code>`],
        `${locale}: complete version and SHA-256 stay associated`,
      );
      assert.equal(artifact.binaries[key].length, 64);
    }
    assert.ok(!source.includes("Binaries (sha256):"));
  }
});

test("legacy sections migrate once without changing the surrounding narrative", () => {
  const intro = "# Performance\n\nIntroduction.\n\n";
  const architecture = "## Why Rust?\n\nKeep architecture and historical profiles.\n\n";
  const profile = "### Type checker profile\n\nKeep the local profile.\n\n";
  const retraction = "Historical `957ms` / `479ms` / `2.0x` was retracted.\n";
  const original = `${intro}## Benchmark Environment\n\nOld environment.\n\n## Benchmark: 15,000 SFC Files\n\nOld table.\n\n${architecture}## Benchmark: Type Checker — canon vs vue-tsc\n\nOld type check.\n\n${profile}## Benchmark: Vite Plugin — @vizejs/vite-plugin vs @vitejs/plugin-vue\n\nOld Vite.\n\n${retraction}`;
  const updated = updatePerformance(original, artifact, "en");
  assert.ok(updated.startsWith(intro));
  assert.ok(updated.includes(architecture));
  assert.ok(updated.includes(profile));
  assert.ok(updated.endsWith(retraction));
  assert.doesNotMatch(updated, /Old environment|Old table|Old type check|Old Vite/);
  assert.equal(updatePerformance(updated, artifact, "en"), updated);
  const readme = updateReadme(
    "# Vize\n\n## Benchmarks\n\nOld table.\n\n## Credits\n\nKeep credits.\n",
    artifact,
  );
  assert.ok(readme.startsWith("# Vize\n\n"));
  assert.ok(readme.endsWith("## Credits\n\nKeep credits.\n"));
  assert.equal(updateReadme(readme, artifact), readme);
});

test("check detects locale drift and failed preparation writes nothing", async () => {
  const root = mkdtempSync(join(tmpdir(), "vize-publication-"));
  try {
    const outputs = await preparePublication(artifact, read);
    for (const [file, source] of outputs) {
      mkdirSync(dirname(join(root, file)), { recursive: true });
      writeFileSync(join(root, file), source);
    }
    const page = performancePath("ja");
    writeFileSync(join(root, page), read(page).replace(/\*\*\d+\.\d+x\*\*/u, "**999.0x**"));
    await assert.rejects(publishSnapshot({ root, check: true }), /stale generated files.*ja/);
    writeFileSync(join(root, page), "missing sections\n");
    const before = readFileSync(join(root, "README.md"), "utf8");
    await assert.rejects(publishSnapshot({ root }), /missing or ambiguous/);
    assert.equal(readFileSync(join(root, "README.md"), "utf8"), before);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
