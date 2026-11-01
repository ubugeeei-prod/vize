import assert from "node:assert/strict";
import { asRecord, asString } from "../../json-value.ts";
import type { Context } from "./bootstrap.ts";
import {
  applyReturnedEdits,
  corpus,
  digest,
  expected,
  referenceFor,
  rule,
  sourceFor,
  type AttributeCase,
} from "./contract.ts";

export function errorPacket(error: unknown): unknown {
  if (!(error instanceof Error)) return { kind: "non-error-thrown", value: error };
  return {
    kind: "threw",
    error: {
      name: error.name,
      message: error.message,
      code: "code" in error ? error.code : null,
      stack: error.stack,
      own: Object.fromEntries(
        Object.getOwnPropertyNames(error).map((name) => [name, Reflect.get(error, name)]),
      ),
    },
  };
}
export function runNative(context: Context): void {
  const packets: unknown[] = [];
  const invoke = (
    id: string,
    row: AttributeCase,
    source: string,
    extra: Record<string, unknown>,
  ) => {
    const options = {
      filename: row.filename,
      preset: "opinionated",
      enabledRules: [rule],
      includeFixes: true,
      helpLevel: "full",
      typeAware: false,
      ...extra,
    };
    const before = JSON.stringify(options);
    const ownBefore = Object.getOwnPropertyNames(options);
    const undefinedBefore = ownBefore.filter((name) => Reflect.get(options, name) === undefined);
    context.record.json("native-pending.json", {
      id,
      source,
      sourceSha256: digest(source),
      options,
      optionsBefore: before,
      ownProperties: ownBefore,
      undefinedProperties: undefinedBefore,
    });
    let outcome: unknown;
    try {
      outcome = { kind: "returned", value: context.native.lintPatinaSfc(source, options) };
    } catch (error) {
      outcome = errorPacket(error);
    }
    const packet = {
      id,
      source,
      sourceSha256: digest(source),
      options,
      optionsBefore: before,
      optionsAfter: JSON.stringify(options),
      ownPropertiesBefore: ownBefore,
      ownPropertiesAfter: Object.getOwnPropertyNames(options),
      undefinedPropertiesBefore: undefinedBefore,
      undefinedPropertiesAfter: Object.getOwnPropertyNames(options).filter(
        (name) => Reflect.get(options, name) === undefined,
      ),
      outcome,
    };
    packets.push(packet);
    context.record.json("native-packets.json", { complete: false, packets });
    assert.equal(packet.optionsAfter, before);
    assert.deepEqual(packet.ownPropertiesAfter, ownBefore);
    assert.deepEqual(packet.undefinedPropertiesAfter, undefinedBefore);
    return asRecord(outcome);
  };
  for (const row of corpus.valid) {
    const source = sourceFor(row),
      reference = referenceFor(row);
    const style = typeof row.policy === "string" ? row.policy : (row.policy.style ?? "always");
    const ignore = typeof row.policy === "string" ? [] : (row.policy.ignore ?? []);
    for (const [form, extra] of [
      ["canonical", { ruleOptionsJson: JSON.stringify({ [rule]: row.policy }) }],
      ["flat", { attributeHyphenation: style, attributeHyphenationIgnore: ignore }],
    ] as const) {
      const id = row.id + "/" + form;
      const verified = invoke(id + "/verify", row, source, extra);
      assert.equal(verified.kind, "returned");
      assert.deepEqual(verified.value, expected(row), id);
      const fixed = applyReturnedEdits(source, verified.value);
      context.record.json("native-" + id.replaceAll("/", "-") + "-fix.json", { source, fixed });
      assert.equal(fixed, reference.fixedSource, id);
      const clean = { filename: row.filename, errorCount: 0, warningCount: 0, diagnostics: [] };
      for (const pass of ["requery", "second-requery"]) {
        const queried = invoke(id + "/" + pass, row, fixed, extra);
        assert.deepEqual(queried, { kind: "returned", value: clean }, id);
      }
    }
  }
  const first = corpus.valid[0]!,
    original = sourceFor(first);
  for (const row of corpus.invalid) {
    const refused = invoke(row.id, first, original, {
      ruleOptionsJson: JSON.stringify({ [rule]: { ignore: row.ignore } }),
    });
    assert.equal(refused.kind, "threw", row.id);
    const error = asRecord(refused.error);
    assert.equal(error.name, "Error");
    assert.equal(error.code, "InvalidArg");
    assert.match(asString(error.message), /^Invalid Patina rule options: .+/su);
  }
  for (const [name, extra] of [
    ["canonical-unselected", { ruleOptionsJson: JSON.stringify({ [rule]: {} }) }],
    ["legacy-unselected", { ruleOptionsJson: JSON.stringify({ [rule]: "always" }) }],
    ["flat-unselected", { attributeHyphenation: "always", attributeHyphenationIgnore: ["active"] }],
  ] as const) {
    const result = invoke(name, first, original, {
      ...extra,
      preset: "incremental",
      enabledRules: undefined,
    });
    assert.deepEqual(result, {
      kind: "returned",
      value: { filename: first.filename, errorCount: 0, warningCount: 0, diagnostics: [] },
    });
  }
  assert.equal(packets.length, 63);
  context.record.json("native-packets.json", {
    complete: true,
    inputs: 9,
    forms: 2,
    refusals: 6,
    unselected: 3,
    packets,
    limits:
      "Original finite Native DTO/edit/fixed-point contracts. Provider-specific malformed-JSON/null-array wording and other configured rules are outside this public role.",
  });
}
