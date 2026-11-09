import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  certifyComponentConstructorTransition,
  certifyGenericComponentContextTransition,
  componentConstructorTransitions,
  currentGenericComponentContext,
  previousGenericComponentContext,
} from "../../tools/support/compat/content-mapper/component-constructor-transition.ts";
import { certifyComponentExportTransition } from "../../tools/support/compat/content-mapper/component-export-transition.ts";
import { certifyJsxWitnessChange } from "../../tools/support/compat/content-mapper/constructor-jsx-witness-transition.ts";
import { beforeInstanceInputKeys } from "../../tools/support/compat/content-mapper/instance-input-keys-transition.ts";
import {
  beforeDiscriminatedJsxUnion,
  discriminatedJsxUnionBeforeSha256,
} from "../../tools/support/compat/content-mapper/discriminated-jsx-union-transition.ts";
import {
  beforeCanonLeftovers,
  beforeStrictExtraKeys,
  canonLeftoversSha256,
  strictExtraKeysAfterSha256,
} from "../../tools/support/compat/content-mapper/strict-extra-keys-transition.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));

test("component constructor transitions retain the complete reviewed variants", () => {
  for (const { previous, current } of componentConstructorTransitions) {
    const before = Buffer.from(previous);
    const after = Buffer.from(current);
    const certificate = certifyComponentConstructorTransition(before, after);
    assert.ok(certificate);
    assert.deepEqual(
      Buffer.concat([
        before.subarray(0, certificate.start),
        certificate.replacement,
        before.subarray(certificate.end),
      ]),
      after,
    );
    assert.equal(certifyComponentConstructorTransition(before, before), undefined);
    assert.equal(certifyComponentConstructorTransition(after, after), undefined);
    assert.throws(
      () =>
        certifyComponentConstructorTransition(
          before,
          Buffer.from(current.replace("$JsxProps", "$Changed")),
        ),
      assert.AssertionError,
    );
  }
});

test("generic component context transition retains the complete reviewed signature", () => {
  const before = Buffer.from(previousGenericComponentContext);
  const after = Buffer.from(currentGenericComponentContext);
  const certificate = certifyGenericComponentContextTransition(before, after);
  assert.ok(certificate);
  assert.deepEqual(
    Buffer.concat([
      before.subarray(0, certificate.start),
      certificate.replacement,
      before.subarray(certificate.end),
    ]),
    after,
  );
  assert.equal(certifyGenericComponentContextTransition(before, before), undefined);
  assert.equal(certifyGenericComponentContextTransition(after, after), undefined);
  for (const mutated of [
    Buffer.from(currentGenericComponentContext.replace("attrs: any", "attrs: unknown")),
    Buffer.from(currentGenericComponentContext.replace("'slots'", "'changed'")),
  ])
    assert.throws(
      () => certifyGenericComponentContextTransition(before, mutated),
      assert.AssertionError,
    );
});

test("component export naming retains the complete generated body", () => {
  const body = "{ readonly label: '日本'; } & Constructor";
  const before = Buffer.from(
    `declare const __vize_component__: ${body}\ndeclare const Constructor: typeof __vize_component__;\nexport default Constructor;`,
  );
  const after = Buffer.from(
    `type ConstructorComponent = ${body}\ndeclare const __vize_component__: ConstructorComponent;\ndeclare const ConstructorVueComponent: typeof __vize_component__;\nexport default ConstructorVueComponent;`,
  );
  const certificate = certifyComponentExportTransition(before, after);
  assert.ok(certificate);
  let cursor = 0;
  const fragments = [];
  for (const edit of certificate.edits) {
    fragments.push(before.subarray(cursor, edit.start), edit.replacement);
    cursor = edit.end;
  }
  fragments.push(before.subarray(cursor));
  assert.deepEqual(Buffer.concat(fragments), after);
  assert.deepEqual(certificate.addedMapping, [
    after.indexOf(
      "ConstructorVueComponent",
      after.indexOf("declare const ConstructorVueComponent"),
    ),
    23,
    0,
    0,
    1,
    248,
  ]);
  const typed = Buffer.from(
    `type ConstructorComponent = ${body}\ndeclare const __vize_component__: ConstructorComponent;\ndeclare const Constructor: typeof __vize_component__;\nexport default Constructor;`,
  );
  const typedCertificate = certifyComponentExportTransition(before, typed);
  assert.ok(typedCertificate);
  assert.deepEqual(typedCertificate.addedMapping, [
    typed.indexOf("Constructor", typed.indexOf("declare const Constructor:")),
    11,
    0,
    0,
    1,
    248,
  ]);
  cursor = 0;
  const typedFragments = [];
  for (const edit of typedCertificate.edits) {
    typedFragments.push(before.subarray(cursor, edit.start), edit.replacement);
    cursor = edit.end;
  }
  typedFragments.push(before.subarray(cursor));
  assert.deepEqual(Buffer.concat(typedFragments), typed);
  assert.equal(
    certifyComponentExportTransition(
      Buffer.from("declare const __vize_component__: Existing;"),
      Buffer.from("declare const __vize_component__: Existing;"),
    ),
    undefined,
  );
  assert.equal(certifyComponentExportTransition(before, before), undefined);
  assert.throws(
    () =>
      certifyComponentExportTransition(
        before,
        Buffer.from(after.toString().replace("日本", "changed")),
      ),
    assert.AssertionError,
  );
});

test("only the complete reviewed JSX witness source transition can change canonical helpers", () => {
  const file = "crates/vize_canon/src/virtual_ts/component_prop_helpers.txt";
  const image = JSON.parse(
    readFileSync(
      path.join(
        root,
        "tests/_fixtures/differential/typechecker/tsx-single-required-exact-keys/integrated-source-transition-11964.json",
      ),
      "utf8",
    ),
  );
  const record = image.file;
  assert.equal(record.path, file);
  const after = Buffer.from(
    beforeInstanceInputKeys(file, readFileSync(path.join(root, file), "utf8")),
  );
  const relaxedAfter = beforeStrictExtraKeys(after.toString());
  const integratedAfter = beforeDiscriminatedJsxUnion(relaxedAfter);
  assert.equal(record.after.sha256, discriminatedJsxUnionBeforeSha256);
  let previous = integratedAfter;
  for (const edit of [...record.edits].reverse())
    previous = previous.replace(edit.after, edit.before);
  const before = Buffer.from(previous);
  const edit = record.edits.at(-1);
  assert.ok(edit);
  const proof = certifyJsxWitnessChange(file, before, after);
  assert.equal(proof.beforeSha256, record.before.sha256);
  assert.equal(
    proof.afterSha256,
    "78d6af92022c9779819704cfb63b91955101396c4929bde5e6dddc308e546b2a",
  );
  assert.equal(proof.successorRecordSha256s?.length, 3);
  assert.equal(proof.successorRecordSha256, canonLeftoversSha256);
  assert.equal(proof.successorRecordSha256s?.at(-1), canonLeftoversSha256);
  assert.equal(proof.nativeAcceptanceCredit, false);
  const directBefore = Buffer.from(beforeCanonLeftovers(file, after.toString()));
  const directProof = certifyJsxWitnessChange(file, directBefore, after);
  assert.equal(directProof.beforeSha256, strictExtraKeysAfterSha256);
  assert.equal(directProof.afterSha256, proof.afterSha256);
  assert.equal(directProof.recordSha256, canonLeftoversSha256);
  assert.equal(directProof.successorRecordSha256, undefined);
  assert.equal(directProof.nativeAcceptanceCredit, false);
  for (const mutated of [
    after.subarray(0, after.length - 1),
    Buffer.concat([after, Buffer.from(edit.after)]),
    before,
    Buffer.from(after.toString().replace("type ", "xxxx ")),
    Buffer.concat([after, Buffer.from("\n")]),
  ])
    assert.throws(() => certifyJsxWitnessChange(file, before, mutated), assert.AssertionError);
  assert.throws(() => certifyJsxWitnessChange(file, after, before), assert.AssertionError);
  assert.throws(
    () => certifyJsxWitnessChange("unowned/source.txt", before, after),
    assert.AssertionError,
  );
});
