import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

export const vrtHarness = [
  "vrt_observer.ts",
  "vrt_fixtures.ts",
  "vrt_artifacts.ts",
  "vrt_api.ts",
  "vrt_cli.ts",
  "vrt_host.ts",
  "vrt_hosted.ts",
  "vrt_audit.ts",
];
const prefix = "tests/tooling/fixtures/musea/snapshot-collision/";
export const vrtSources = [
  `${prefix}left/Button.art.vue`,
  `${prefix}right/Button.art.vue`,
  "tests/_fixtures/differential/musea/gallery-vrt-options.json",
  "tests/_fixtures/differential/musea/hosted-audits.json",
  "tests/_fixtures/differential/musea/vrt-report-ownership.json",
];
export const sha256 = (bytes: string | Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");
export const galleryPath = "/gallery/vrt/";
export const sides = ["left", "right"] as const;
export type Side = (typeof sides)[number];

/** The official prepare entry supplies only exact, normal raw H blobs. */
export function materializeVrtInputs(consumer: string, read: (file: string) => Buffer) {
  return vrtSources.map((sourceFile, index) => {
    const file =
      index < 2
        ? `vrt-inputs/${sides[index]}/Button.art.vue`
        : `vrt-inputs/${path.basename(sourceFile)}`;
    const bytes = read(sourceFile);
    const target = path.join(consumer, file);
    mkdirSync(path.dirname(target), { recursive: true });
    writeFileSync(target, bytes, { flag: "wx" });
    return { file, sourceFile, sha256: sha256(bytes) };
  });
}

export interface VrtSettings {
  viewports: Array<{ name: string; width: number; height: number }>;
  snapshotDir: string;
  threshold: number;
  workers: number;
  capture: { settleTime: number; waitForNetwork: boolean };
  comparison: { antiAliasing: boolean };
}
export interface VrtFixture {
  root: string;
  config: string;
  settings: VrtSettings;
  authored: Record<Side, Buffer>;
  arts: Record<Side, string>;
  delay: number;
}

export function createVrtFixture(consumer: string): VrtFixture {
  const inputs = path.join(consumer, "vrt-inputs");
  const history = JSON.parse(readFileSync(path.join(inputs, "vrt-report-ownership.json"), "utf8"));
  const settings = JSON.parse(
    readFileSync(path.join(inputs, "gallery-vrt-options.json"), "utf8"),
  ) as VrtSettings;
  assert.deepEqual(settings, {
    viewports: [{ name: "authored-compact", width: 320, height: 180 }],
    snapshotDir: "reviewed-baselines",
    threshold: 100,
    workers: 2,
    capture: { settleTime: 0, waitForNetwork: false },
    comparison: { antiAliasing: false },
  });
  const delay = JSON.parse(
    readFileSync(path.join(inputs, "hosted-audits.json"), "utf8"),
  ).setupDelayMs;
  assert.equal(delay, 300);
  const root = path.join(consumer, "vrt-workspace");
  mkdirSync(root);
  const authored = {} as Record<Side, Buffer>;
  const arts = {} as Record<Side, string>;
  for (const side of sides) {
    const bytes = readFileSync(path.join(inputs, side, "Button.art.vue"));
    const source = history.sources.find(
      (item: { path: string }) => item.path === `${prefix}${side}/Button.art.vue`,
    );
    assert.ok(source, side);
    assert.equal(sha256(bytes), source.sha256, `exact delivered ${side} Art`);
    assert.equal(
      sha256(bytes),
      side === "left"
        ? "ecbe89ffd4276aff872c00a258622f12cba608315f35f464392bfd33b40c088b"
        : "5b5a1bfe6f0eea822548d2e4671fbff615842287b9ab52197c6080774d7bdd5a",
    );
    authored[side] = bytes;
    arts[side] = path.join(root, "src", side, "Button.art.vue");
    mkdirSync(path.dirname(arts[side]), { recursive: true });
    copyFileSync(path.join(inputs, side, "Button.art.vue"), arts[side]);
  }
  const fixture = {
    root,
    config: path.join(root, "vite.config.mjs"),
    settings,
    authored,
    arts,
    delay,
  };
  writeFileSync(path.join(root, "index.html"), "<!doctype html><html><body></body></html>\n", {
    flag: "wx",
  });
  writeVrtConfig(fixture, false);
  return fixture;
}

export function writeVrtConfig(f: VrtFixture, hosted: boolean) {
  const settings = hosted
    ? { ...f.settings, snapshotDir: "cli-baselines", threshold: 0 }
    : f.settings;
  const options = {
    include: ["src/**/*.art.vue"],
    basePath: galleryPath,
    ...(hosted ? { previewSetup: "vrt-delayed.setup.ts" } : {}),
    vrt: settings,
  };
  writeFileSync(
    f.config,
    `import vize from '@vizejs/vite-plugin';
import { musea } from '@vizejs/vite-plugin-musea';
import { fileURLToPath } from 'node:url';
export default {
  root: ${JSON.stringify(f.root)}, base: ${JSON.stringify(hosted ? "/built/" : "/")},
  resolve: { alias: { vue: fileURLToPath(import.meta.resolve('vue/dist/vue.runtime.esm-bundler.js')) } },
  plugins: [vize(), musea(${JSON.stringify(options)})],
};\n`,
  );
}
