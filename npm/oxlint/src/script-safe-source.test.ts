import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Context } from "@oxlint/plugins";
import { it } from "vite-plus/test";
import { clearFileStateCache, getFileState } from "./file-state.ts";
import { extractSfcBlocks } from "./sfc-blocks.ts";
import { prepareWorkaroundSource, resolveWorkaroundSource } from "./workaround.ts";

const corpus = fileURLToPath(
  new URL(
    "../../../tests/_fixtures/differential/lint/oxlint-script-safe-carrier-7903/",
    import.meta.url,
  ),
);
const manifest = JSON.parse(fs.readFileSync(path.join(corpus, "source.json"), "utf8")) as {
  cases: { file: string; originalFilename: string; sha256: string }[];
};

for (const entry of manifest.cases)
  it(`keeps the complete ${entry.originalFilename} source behind one safe host script`, () => {
    const source = fs.readFileSync(path.join(corpus, entry.file), "utf8");
    assert.equal(createHash("sha256").update(source).digest("hex"), entry.sha256);
    const filename = `/original/${entry.originalFilename}`;
    const carrier = prepareWorkaroundSource(source, filename);
    assert.deepEqual(
      extractSfcBlocks(carrier.source).map((block) => block.kind),
      ["script-setup"],
    );
    assert.deepEqual(
      resolveWorkaroundSource(carrier.source, "/carrier.vue", (name) => {
        assert.equal(name, filename);
        return source;
      }),
      { filename, source, usesOriginalLocations: true },
    );
  });

it("rejects modified base64, source checksum, filename and whitespace before native input", () => {
  const source = '<template>\n<div v-html="html"/>\n</template>\n';
  const filename = "/original/Static.vue";
  const carrier = prepareWorkaroundSource(source, filename);
  const changedDigest = carrier.source.replace(
    /(<!--vize-original-source:)([a-f0-9])/u,
    (_, prefix: string, value: string) => prefix + (value === "0" ? "1" : "0"),
  );
  const changedBase64 = carrier.source.replace(/:([A-Za-z0-9_-]*)-->$/u, ":%-->");
  const changedFilename = carrier.source.replace(
    Buffer.from(filename).toString("base64url"),
    Buffer.from("/original/Other.vue").toString("base64url"),
  );
  const changedMirror =
    carrier.source.slice(0, carrier.locations.scriptStart) +
    "\t" +
    carrier.source.slice(carrier.locations.scriptStart + 1);
  for (const changed of [changedDigest, changedBase64, changedFilename, changedMirror])
    assert.throws(
      () => resolveWorkaroundSource(changed, "/carrier.vue", () => source),
      /Invalid Vize|carrier bytes/u,
    );

  const forged = prepareWorkaroundSource(source.replaceAll("html", "evil"), filename);
  assert.throws(
    () => resolveWorkaroundSource(forged.source, "/carrier.vue", () => source),
    /original file bytes/u,
    "a self-consistent forged payload cannot override original authority",
  );
});

it("revalidates original authority when physical carrier, original or settings change", () => {
  clearFileStateCache();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-carrier-authority-"));
  try {
    const original = path.join(root, "Original.vue"),
      physical = path.join(root, "Carrier.vue");
    const source = '<template>\n<div v-html="html"/>\n</template>\n';
    fs.writeFileSync(original, source);
    fs.writeFileSync(physical, prepareWorkaroundSource(source, original).source);
    const context = {
      filename: physical,
      physicalFilename: physical,
      settings: {},
      sourceCode: { text: "/*vize-location-bridge*/" },
    } as unknown as Context;
    const state = getFileState(context);
    assert.equal(state.filename, original);
    assert.equal(state.source, source);
    assert.equal(getFileState(context), state);
    fs.writeFileSync(original, source.replaceAll("html", "evil"));
    assert.throws(() => getFileState(context), /original file bytes/u);
    assert.throws(
      () => getFileState({ ...context, settings: { vize: { preset: "recommended" } } }),
      /original file bytes/u,
    );
    fs.writeFileSync(original, source);
    fs.writeFileSync(
      physical,
      prepareWorkaroundSource(source.replaceAll("html", "evil"), original).source,
    );
    assert.throws(() => getFileState(context), /original file bytes/u);
    assert.equal(fs.readFileSync(original, "utf8"), source);
    fs.writeFileSync(physical, prepareWorkaroundSource(source, original).source);
    fs.writeFileSync(original, Buffer.from([0xff]));
    assert.throws(() => getFileState(context), /non-UTF8 original file bytes/u);
  } finally {
    clearFileStateCache();
    fs.rmSync(root, { recursive: true, force: true });
  }
});
