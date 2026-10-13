import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { authenticateCssProducerContext } from "../../tools/support/dependencies/css-engine-producer-context-custody.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const sha = (body) => createHash("sha256").update(body).digest("hex");
const data = JSON.parse(
  fs.readFileSync(
    path.join(root, "tools/support/dependencies/css-engine-producer-context-source.json"),
  ),
);
const rows = [...data.context, ...data.kernel];
function privateRoot(action) {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-context-source-control-"));
  try {
    for (const row of rows) {
      const body = fs.readFileSync(path.join(root, row.file));
      assert.equal(sha(body), row.sha256, row.file);
      const target = path.join(fixture, row.file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, body, { flag: "wx", mode: 0o644 });
    }
    return action(fixture);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
}

void test("exact current four-body context and kernel expose only the whole original helper authority", () => {
  privateRoot((fixture) => {
    const current = authenticateCssProducerContext(fixture);
    assert.equal(
      current.originalSha256,
      "31a86011e3d8cda9b3817b5cceb210f2b8d2099a8af033202755e09c73db7f1b",
    );
    assert.equal(
      current.actualSha256,
      "5bf5cc249c0ea4b46a7f2d011ce51f2f8d0943627af23bf254376183509d26d3",
    );
    assert.equal(current.contextBodies, 4);
    assert.equal(current.kernelBodies, 4);
    assert.equal(current.runtimeCredit, false);
    const original = Buffer.from(data.original.rawBase64, "base64");
    fs.writeFileSync(path.join(fixture, data.original.file), original);
    assert.deepEqual(authenticateCssProducerContext(fixture), {
      originalSha256: sha(original),
      actualSha256: sha(original),
      contextBodies: 0,
      runtimeCredit: false,
    });
  });
});

void test("every new context or kernel body, mode, missing input and symlink refuses before projection", () => {
  let refusals = 0;
  for (const row of rows)
    for (const kind of ["body", "mode", "missing", "symlink"])
      privateRoot((fixture) => {
        const target = path.join(fixture, row.file);
        if (kind === "body") fs.appendFileSync(target, "\nforeign source\n");
        if (kind === "mode") fs.chmodSync(target, 0o755);
        if (kind === "missing") fs.unlinkSync(target);
        if (kind === "symlink") {
          fs.unlinkSync(target);
          fs.symlinkSync(path.join(root, row.file), target);
        }
        assert.throws(
          () => authenticateCssProducerContext(fixture),
          (error) => {
            assert(error instanceof Error);
            assert(error.message.includes(row.file), error.message);
            return true;
          },
        );
        refusals++;
      });
  assert.equal(refusals, 32);
});

void test("a mixed helper quartet and unknown whole helper cannot borrow the literal original checksum", () => {
  privateRoot((fixture) => {
    const before = Buffer.from(data.original.rawBase64, "base64");
    fs.writeFileSync(path.join(fixture, data.context[1].file), before);
    assert.throws(() => authenticateCssProducerContext(fixture));
  });
  privateRoot((fixture) => {
    const helper = path.join(fixture, data.original.file);
    const original = Buffer.from(data.original.rawBase64, "base64");
    fs.writeFileSync(
      helper,
      Buffer.concat([original, Buffer.from("\n// unknown helper continuation\n")]),
    );
    assert.throws(() => authenticateCssProducerContext(fixture));
  });
});

void test("only one original checksum-list branch and a whole import inverse change the complete checker", () => {
  const file = path.join(root, data.consumerBefore.file);
  let current = fs.readFileSync(file, "utf8");
  const guardFile = path.join(
    root,
    "tools/support/dependencies/css-engine-producer-context-custody.mjs",
  );
  const guardSHA = sha(fs.readFileSync(guardFile));
  assert.equal(guardSHA, "81fdd0a6f99bd24d013722d2e16c79ef7e572c5d408804dd9f8b7c4ed2d5832b");
  const imported = 'import { pathToFileURL } from "node:url";\n';
  assert.equal(current.split(imported).length, 2);
  current = current.replace(imported, "");
  const start = current.indexOf(
    '  if (filename === "tools/support/dependencies/css-engine-producer-manifests.ts") {',
  );
  const tail =
    "  } else assert.equal(sha256(fs.readFileSync(path.join(root, filename))), checksum, filename);";
  const end = current.indexOf(tail, start) + tail.length;
  assert(start > 0 && end > start);
  const branch = current.slice(start, end);
  assert(branch.includes(`"${guardSHA}"`), "whole fixed verifier hash before import");
  assert(branch.indexOf(guardSHA) < branch.indexOf("await import("));
  current =
    current.slice(0, start) +
    "  assert.equal(sha256(fs.readFileSync(path.join(root, filename))), checksum, filename);" +
    current.slice(end);
  const original = Buffer.from(data.consumerBefore.rawBase64, "base64");
  assert.equal(sha(original), data.consumerBefore.sha256);
  assert.deepEqual(
    Buffer.from(current),
    original,
    "all other original checksum rows and whole tail exact",
  );
});
