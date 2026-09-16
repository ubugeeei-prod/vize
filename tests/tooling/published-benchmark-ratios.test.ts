import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { ENGINE_CLASSES_BY_SURFACE } from "../../tools/benchmarks/scripts/compare-tools-report.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const data = JSON.parse(
  fs.readFileSync(path.join(root, "tools/benchmarks/results/tool-benchmark-latest.json"), "utf8"),
);
const labels = {
  en: ["Type check", "Large SFC type check"],
  ja: ["型検査", "巨大 SFC 型検査"],
  "zh-CN": ["类型检查", "大型 SFC 类型检查"],
  fr: ["Vérification des types", "Vérification SFC volumineux"],
  "pt-BR": ["Verificação de tipos", "Tipos em SFC grande"],
};

function cells(line: string): string[] {
  return line
    .split("|")
    .slice(1, -1)
    .map((cell) => cell.trim());
}

test("all public type-check rows name and time the same native-engine comparator", () => {
  for (const [locale, rowLabels] of Object.entries(labels)) {
    const directory = `docs/content/${locale === "en" ? "" : `${locale}/`}architecture`;
    for (const page of ["performance.md", "performance-blacksmith.md"]) {
      const text = fs.readFileSync(path.join(root, directory, page), "utf8");
      for (const [index, id] of ["check", "large-check"].entries()) {
        const surface = data.surfaces.find((s: { id: string }) => s.id === id);
        const row = text
          .split("\n")
          .filter((line) => line.startsWith("|"))
          .map(cells)
          .find((row) => row[0] === rowLabels[index]);
        assert.ok(row, `${locale}/${page}: missing ${id}`);
        assert.equal(row[2], "verter-tsc");
        assert.equal(row[1], String(surface.files));
        const baseline = surface.variants.find((v: { id: string }) => v.id === "verter-tsc");
        const max = surface.variants.find((v: { id: string }) => v.id === surface.vizeMaxId);
        assert.equal(
          row.at(-1)?.replaceAll("*", ""),
          `${(baseline.medianMs / max.medianMs).toFixed(1)}x`,
        );
        assert.ok(text.includes(data.commit.runUrl));
        assert.ok(text.includes(data.commit.sha.slice(0, 12)));
      }
    }
  }
});

test("the committed snapshot never rates one engine class against another", () => {
  const classified = data.surfaces.filter(
    (s: { engineClasses?: object }) => s.engineClasses != null,
  );
  assert.deepEqual(
    classified.map((s: { id: string }) => s.id).sort(),
    Object.keys(ENGINE_CLASSES_BY_SURFACE).sort(),
  );
  for (const surface of classified) {
    assert.deepEqual(surface.engineClasses, ENGINE_CLASSES_BY_SURFACE[surface.id]);
    assert.equal(surface.speedupStatus, "in-class");
    assert.equal(surface.speedupBaselineId, "verter-tsc");
    assert.equal(
      surface.engineClasses[surface.speedupBaselineId],
      surface.engineClasses[surface.vizeMaxId],
    );
  }
});

test("other comparisons remain ranked while Musea without a comparator has no ratio", () => {
  for (const surface of data.surfaces) {
    if (surface.engineClasses) continue;
    if (surface.id === "musea") {
      assert.equal(surface.primarySpeedup, null);
      assert.equal(surface.speedupBaselineId, null);
      assert.equal(surface.speedupStatus, "unavailable");
    } else {
      assert.equal(surface.speedupStatus, "ranked");
      assert.ok(Number.isFinite(surface.primarySpeedup));
      assert.ok(surface.primarySpeedup > 0);
    }
  }
});
