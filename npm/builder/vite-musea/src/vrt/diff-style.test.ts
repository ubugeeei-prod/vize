import assert from "node:assert/strict";
import { copyFile, mkdtemp, readFile, rm, stat } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { PNG } from "pngjs";
import type { ComparisonConfig, DiffStyle } from "../types/index.ts";
import { readPng, writePng } from "./comparison.ts";
import { resolveDiffStyle, sideBySidePng, UnsupportedDiffStyleError } from "./diff-style.ts";
import { compareImages } from "./runner-comparison.ts";
import { MuseaVrtRunner } from "./runner.ts";
import { UnsupportedDiffStyleError as PublicUnsupportedDiffStyleError } from "../vrt.ts";

const styles: DiffStyle[] = ["overlay", "diffOnly", "sideBySide"];
const baseline = [1, 2, 3, 0, 10, 20, 30, 255, 50, 60, 70, 128, 100, 110, 120, 255];
const current = [222, 8, 10, 0, 200, 20, 30, 255, 50, 60, 70, 128, 100, 110, 120, 255];

function png(width: number, height: number, rgba: number[]) {
  const image = new PNG({ width, height });
  image.data.set(rgba);
  return image;
}

async function fixture(t: { after: (callback: () => Promise<void>) => void }) {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-diff-style-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  return {
    root,
    compare: async (a: PNG, b: PNG, comparison: ComparisonConfig = {}) => {
      const aPath = path.join(root, "baseline.png");
      const bPath = path.join(root, "current.png");
      const output = path.join(root, `${comparison.diffStyle ?? "default"}.png`);
      // PNG encoders are streams and each instance can only be packed once.
      await writePng(png(a.width, a.height, [...a.data]), aPath);
      await writePng(png(b.width, b.height, [...b.data]), bPath);
      const result = await compareImages(aPath, bPath, output, {
        colorThreshold: 0,
        antiAliasing: false,
        diffColor: { r: 7, g: 8, b: 9 },
        ...comparison,
      });
      return { result, output };
    },
  };
}

void test("styles preserve complete statistics while selecting exact full PNG geometry and RGBA", async (t) => {
  const { compare } = await fixture(t);
  const expected: Record<DiffStyle, { width: number; height: number; data: number[] }> = {
    overlay: {
      width: 2,
      height: 2,
      data: [80, 80, 80, 128, 7, 8, 9, 255, 60, 60, 60, 128, 110, 110, 110, 128],
    },
    diffOnly: { width: 2, height: 2, data: [0, 0, 0, 0, 7, 8, 9, 255, 0, 0, 0, 0, 0, 0, 0, 0] },
    sideBySide: {
      width: 4,
      height: 2,
      data: [
        ...baseline.slice(0, 8),
        ...current.slice(0, 8),
        ...baseline.slice(8),
        ...current.slice(8),
      ],
    },
  };
  for (const diffStyle of styles) {
    const { result, output } = await compare(png(2, 2, baseline), png(2, 2, current), {
      diffStyle,
    });
    assert.deepEqual(result, { diffPixels: 1, totalPixels: 4, diffPercentage: 25 });
    const actual = await readPng(output);
    assert.deepEqual(
      { width: actual.width, height: actual.height, data: [...actual.data] },
      expected[diffStyle],
    );
  }
  const implicit = await compare(png(2, 2, baseline), png(2, 2, current));
  const explicit = await compare(png(2, 2, baseline), png(2, 2, current), { diffStyle: "overlay" });
  assert.deepEqual(await readFile(implicit.output), await readFile(explicit.output));
  // Retained from the pre-style comparator, including the complete PNG stream.
  assert.equal(
    (await readFile(implicit.output)).toString("base64"),
    "iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAAklEQVR4AewaftIAAAAYSURBVGMICAhoYOfg/M8sIiLi4OvjcxAAJ5YEraJKK+sAAAAASUVORK5CYII=",
  );
});

void test("side-by-side padding preserves both unequal shapes and every alpha/RGB byte", () => {
  const a = png(1, 2, [1, 2, 3, 0, 4, 5, 6, 128]);
  const b = png(2, 1, [7, 8, 9, 255, 10, 11, 12, 255]);
  const actual = sideBySidePng(a, b);
  assert.deepEqual(
    { width: actual.width, height: actual.height, data: [...actual.data] },
    {
      width: 3,
      height: 2,
      data: [1, 2, 3, 0, 7, 8, 9, 255, 10, 11, 12, 255, 4, 5, 6, 128, 0, 0, 0, 0, 0, 0, 0, 0],
    },
  );
  assert.deepEqual([...a.data], [1, 2, 3, 0, 4, 5, 6, 128]);
  assert.deepEqual([...b.data], [7, 8, 9, 255, 10, 11, 12, 255]);
});

void test("dimension mismatches retain 100-percent statistics in every presentation", async (t) => {
  const { compare } = await fixture(t);
  for (const diffStyle of styles) {
    const a = png(1, 2, [1, 2, 3, 0, 4, 5, 6, 128]);
    const b = png(2, 1, [7, 8, 9, 255, 10, 11, 12, 255]);
    const { result, output } = await compare(a, b, { diffStyle });
    assert.deepEqual(result, { diffPixels: 4, totalPixels: 4, diffPercentage: 100 });
    const actual = await readPng(output);
    const expected =
      diffStyle === "sideBySide"
        ? sideBySidePng(a, b)
        : png(2, 2, [255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255]);
    assert.deepEqual(
      { width: actual.width, height: actual.height, data: actual.data },
      {
        width: expected.width,
        height: expected.height,
        data: expected.data,
      },
    );
  }
});

void test("unchanged, tolerant, and alpha-ignored comparisons create no presentation artifact", async (t) => {
  for (const diffStyle of styles) {
    for (const [a, b, options] of [
      [[10, 20, 30, 255], [10, 20, 30, 255], {}],
      [[238, 238, 238, 255], [237, 237, 237, 255], { colorThreshold: 0.1 }],
      [[10, 20, 30, 0], [10, 20, 30, 255], { alpha: false }],
    ] as const) {
      const { compare } = await fixture(t);
      const { result, output } = await compare(png(1, 1, [...a]), png(1, 1, [...b]), {
        diffStyle,
        ...options,
      });
      assert.deepEqual(result, { diffPixels: 0, totalPixels: 1, diffPercentage: 0 });
      await assert.rejects(stat(output), { code: "ENOENT" });
    }
  }
});

void test("anti-aliasing highlights and counts survive transparent diff-only output", async (t) => {
  const { compare } = await fixture(t);
  const a = png(3, 3, Array(9).fill([255, 255, 255, 255]).flat());
  const b = png(3, 3, [...a.data]);
  b.data.set([0, 0, 0, 255], 4);
  b.data.set([0, 0, 0, 255], 16);
  for (const diffStyle of ["overlay", "diffOnly"] as const) {
    const { result, output } = await compare(a, b, { diffStyle, antiAliasing: true });
    assert.deepEqual(result, { diffPixels: 2, totalPixels: 9, diffPercentage: (2 / 9) * 100 });
    const expected = Array(9)
      .fill(diffStyle === "overlay" ? [255, 255, 255, 128] : [0, 0, 0, 0])
      .flat();
    expected.splice(4, 4, 255, 200, 0, 128);
    expected.splice(16, 4, 255, 200, 0, 128);
    assert.deepEqual([...(await readPng(output)).data], expected);
  }
});

for (const style of ["animated", "bogus", "", null, 42]) {
  void test(`unsupported style ${String(style)} rejects before file access or a first baseline`, async (t) => {
    const { root } = await fixture(t);
    const comparison = { diffStyle: style } as unknown as ComparisonConfig;
    const validate = (error: unknown) => {
      assert.ok(error instanceof UnsupportedDiffStyleError);
      assert.ok(error instanceof PublicUnsupportedDiffStyleError);
      assert.equal(error.code, "ERR_MUSEA_UNSUPPORTED_DIFF_STYLE");
      assert.equal(error.style, style);
      return true;
    };
    assert.throws(
      () => new MuseaVrtRunner({ snapshotDir: path.join(root, "not-created"), comparison }),
      validate,
    );
    await assert.rejects(
      compareImages(
        path.join(root, "absent-a.png"),
        path.join(root, "absent-b.png"),
        path.join(root, "diff.png"),
        comparison,
      ),
      validate,
    );
    assert.throws(() => resolveDiffStyle(style), validate);
    await assert.rejects(stat(path.join(root, "not-created")), { code: "ENOENT" });
    await assert.rejects(stat(path.join(root, "diff.png")), { code: "ENOENT" });
  });
}

for (const existing of [false, true]) {
  void test(`an invalid style mutated after construction cannot ${existing ? "replace an existing" : "create a first"} baseline`, async (t) => {
    const { root } = await fixture(t);
    const source = path.join(root, "input.png");
    const baselinePath = path.join(root, "baseline.png");
    await writePng(png(1, 1, [10, 20, 30, 255]), source);
    if (existing) await copyFile(source, baselinePath);
    const bytes = existing ? await readFile(baselinePath) : undefined;
    const comparison: ComparisonConfig = { diffStyle: "overlay" };
    const runner = new MuseaVrtRunner({ projectRoot: root, snapshotDir: root, comparison });
    comparison.diffStyle = "animated" as DiffStyle;
    let contexts = 0;
    const page = {
      async goto() {},
      async waitForSelector() {},
      async waitForTimeout() {},
      async screenshot({ path: file }: { path: string }) {
        await copyFile(source, file);
      },
      async close() {},
    };
    runner.getBrowser = () =>
      ({
        async newContext() {
          contexts++;
          return {
            async newPage() {
              return page;
            },
            async close() {},
          };
        },
      }) as unknown as ReturnType<MuseaVrtRunner["getBrowser"]>;
    runner.getSnapshotName = async () => "baseline.png";
    const viewport = { width: 1, height: 1 };
    const result = await runner.captureAndCompare(
      { path: "Pixels.art.vue" } as Parameters<MuseaVrtRunner["captureAndCompare"]>[0],
      "Default",
      viewport,
      "http://localhost",
    );
    assert.deepEqual(result, {
      artPath: "Pixels.art.vue",
      variantName: "Default",
      viewport,
      passed: false,
      snapshotPath: baselinePath,
      error: "Unsupported VRT diff style: animated. Use overlay, sideBySide, or diffOnly.",
    });
    assert.equal(contexts, 0);
    await assert.rejects(stat(path.join(root, "current", "baseline.png")), { code: "ENOENT" });
    if (existing) assert.deepEqual(await readFile(baselinePath), bytes);
    else await assert.rejects(stat(baselinePath), { code: "ENOENT" });
  });
}

void test("async hook configuration changes apply to the next capture without changing the current PNG or metadata", async (t) => {
  const { root } = await fixture(t);
  const source = path.join(root, "input.png");
  const baselinePath = path.join(root, "baseline.png");
  await writePng(png(1, 1, [200, 20, 30, 255]), source);
  await writePng(png(1, 1, [10, 20, 30, 255]), baselinePath);
  const baselineBytes = await readFile(baselinePath);
  const comparison: ComparisonConfig = {
    diffStyle: "diffOnly",
    colorThreshold: 0,
    antiAliasing: false,
    diffColor: { r: 7, g: 8, b: 9 },
  };
  const runner = new MuseaVrtRunner({
    projectRoot: root,
    snapshotDir: root,
    threshold: 0,
    comparison,
    capture: {
      async beforeCapture() {
        comparison.diffStyle = "sideBySide";
        comparison.diffColor!.r = 99;
        await Promise.resolve();
      },
    },
  });
  let closed = 0;
  const page = {
    async goto() {},
    async waitForSelector() {},
    async waitForTimeout() {},
    async screenshot({ path: file }: { path: string }) {
      await copyFile(source, file);
    },
    async close() {
      closed++;
    },
  };
  runner.getBrowser = () =>
    ({
      async newContext() {
        return {
          async newPage() {
            return page;
          },
          async close() {
            closed++;
          },
        };
      },
    }) as unknown as ReturnType<MuseaVrtRunner["getBrowser"]>;
  runner.getSnapshotName = async () => "baseline.png";
  const art = { path: "Pixels.art.vue" } as Parameters<MuseaVrtRunner["captureAndCompare"]>[0];
  const viewport = { width: 1, height: 1 };
  const expected = {
    artPath: art.path,
    variantName: "Default",
    viewport,
    passed: false,
    snapshotPath: baselinePath,
    currentPath: path.join(root, "current", "baseline.png"),
    diffPath: path.join(root, "diff", "baseline.png"),
    diffPixels: 1,
    totalPixels: 1,
    diffPercentage: 100,
  };
  const first = await runner.captureAndCompare(art, "Default", viewport, "http://localhost");
  assert.deepEqual(first, { ...expected, diffStyle: "diffOnly" });
  const firstPng = await readPng(first.diffPath!);
  assert.deepEqual(
    { width: firstPng.width, height: firstPng.height, data: [...firstPng.data] },
    {
      width: 1,
      height: 1,
      data: [7, 8, 9, 255],
    },
  );
  const second = await runner.captureAndCompare(art, "Default", viewport, "http://localhost");
  assert.deepEqual(second, { ...expected, diffStyle: "sideBySide" });
  const secondPng = await readPng(second.diffPath!);
  assert.deepEqual(
    { width: secondPng.width, height: secondPng.height, data: [...secondPng.data] },
    {
      width: 2,
      height: 1,
      data: [10, 20, 30, 255, 200, 20, 30, 255],
    },
  );
  assert.deepEqual(await readFile(baselinePath), baselineBytes);
  assert.equal(closed, 4);
});
