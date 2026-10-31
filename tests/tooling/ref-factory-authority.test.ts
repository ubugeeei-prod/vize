import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { test } from "node:test";
import {
  readRefFactoryAuthority,
  refFactoryCurrentPreamble,
  refFactoryPrevious,
  refFactoryRecord,
} from "../../tools/benchmarks/scripts/ref-factory-authority.ts";

test("the actual observed insertion alone reconstructs the frozen complete C3 helper", () => {
  const record = readRefFactoryAuthority();
  assert.equal(record.preamble.insertionBytes, 441);
  assert.equal(record.preamble.afterBytes - record.preamble.beforeBytes, 441);
  assert.equal(
    refFactoryPrevious(record.preamble.afterWholeUtf8, record.preamble),
    record.preamble.beforeWholeUtf8,
  );
  assert.equal(
    refFactoryCurrentPreamble(record.preamble.beforeWholeUtf8),
    record.preamble.afterWholeUtf8,
  );
  assert.throws(() => refFactoryCurrentPreamble(record.preamble.beforeWholeUtf8 + "\n"));
  assert.throws(() => refFactoryPrevious(record.preamble.beforeWholeUtf8, record.preamble));
});

test("missing, duplicate, changed or unrelated helper bytes cannot be normalized away", () => {
  const { preamble } = readRefFactoryAuthority();
  const after = preamble.afterWholeUtf8,
    block = preamble.insertionWholeUtf8;
  for (const changed of [
    after.replace(block, ""),
    after + block,
    after.replace("__vizeResolveRef", "__foreignResolveRef"),
    after.replace("(1 & _C)", "(0 & _C)"),
    after + "\n// foreign\n",
    "\n" + after,
  ]) {
    assert.throws(() => refFactoryPrevious(changed, preamble));
  }
});

test("only the exact current source, declaration asset and immutable record can authenticate the postimage", (context) => {
  const record = readRefFactoryAuthority();
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "factory-authority-"));
  context.after(() => fs.rmSync(temporary, { recursive: true }));
  const files = [
    refFactoryRecord,
    record.source.path,
    record.declaration.path,
    record.frozenC3.path,
  ];
  for (const file of files) {
    fs.mkdirSync(path.dirname(path.join(temporary, file)), { recursive: true });
    fs.copyFileSync(file, path.join(temporary, file));
  }
  assert.deepEqual(readRefFactoryAuthority(temporary), record);
  for (const file of files) {
    const destination = path.join(temporary, file),
      original = fs.readFileSync(destination);
    fs.appendFileSync(destination, "\n");
    assert.throws(() => readRefFactoryAuthority(temporary));
    fs.writeFileSync(destination, original);
  }
});
