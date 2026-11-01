import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";
import {
  verifyCandidateSourceInputs,
  historicalSourceFix,
  componentChecksPath,
  candidateProducerPaths,
  type BodyPin,
  type CandidateSources,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-source.ts";

const pin = (body: Buffer): BodyPin => ({
  bytes: body.length,
  sha256: createHash("sha256").update(body).digest("hex"),
  gitBlob: createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
});
function fixture() {
  const left = [Buffer.from("borrowed runner"), Buffer.from("borrowed prepare")];
  const right = [Buffer.from("owned runner"), Buffer.from("owned prepare")];
  const cargo = [Buffer.from("frozen whole manifest"), Buffer.from("frozen whole lock")];
  const protocol = [Buffer.from("whole source printer"), Buffer.from("whole source materializer")];
  const frozen: CandidateSources = {
    kind: "route-move",
    base: { sha: "1".repeat(40), tree: "2".repeat(40) },
    head: { sha: "3".repeat(40), tree: "4".repeat(40) },
    containing: { optionsFix: "5".repeat(40), historicalSourceFix },
    changed: [
      "crates/vize/src/commands/check/runner.rs",
      "crates/vize/src/commands/check/runner/execution.rs",
    ].map((path, index) => ({ path, base: pin(left[index]), head: pin(right[index]) })),
    cargo: [
      { file: "Cargo.toml", pin: pin(cargo[0]) },
      { file: "Cargo.lock", pin: pin(cargo[1]) },
    ],
    protocol: candidateProducerPaths.map((file, index) => ({ file, pin: pin(protocol[index]) })),
  };
  const actual = {
    base: structuredClone(frozen.base),
    head: structuredClone(frozen.head),
    changedPaths: frozen.changed.map((row) => row.path),
    bodies: { base: left, head: right },
    cargo: { base: cargo, head: cargo.map((body) => Buffer.from(body)) },
    protocol: { base: protocol, head: protocol.map((body) => Buffer.from(body)) },
    ancestry: { base: [true, true], head: [true, true] },
  };
  return { frozen, actual };
}

test("candidate sources bind both whole source trees, exact paths and both containing fixes", () => {
  const { frozen, actual } = fixture();
  const saved = structuredClone(frozen);
  const result = verifyCandidateSourceInputs(frozen, actual);
  assert.deepEqual(result.changedPaths, actual.changedPaths);
  assert.deepEqual(frozen, saved);
  assert.equal(result.sources.base.sha, "1".repeat(40));
});

test("candidate sources refuse stale ancestry, mixed effects, extra/omitted/reordered paths and source tamper", () => {
  for (const mutate of [
    (x: ReturnType<typeof fixture>) => {
      x.actual.ancestry.base[0] = false;
    },
    (x: ReturnType<typeof fixture>) => {
      x.actual.ancestry.head.pop();
    },
    (x: ReturnType<typeof fixture>) => {
      x.actual.head.tree = "7".repeat(40);
    },
    (x: ReturnType<typeof fixture>) => {
      x.actual.bodies.head[0] = Buffer.from("different whole source");
    },
    (x: ReturnType<typeof fixture>) => {
      x.actual.changedPaths.reverse();
    },
    (x: ReturnType<typeof fixture>) => {
      x.actual.changedPaths.pop();
    },
    (x: ReturnType<typeof fixture>) => {
      x.actual.changedPaths.push("crates/vize/src/unrelated.rs");
    },
    (x: ReturnType<typeof fixture>) => {
      x.frozen.changed[0].path = "../escape.rs";
    },
    (x: ReturnType<typeof fixture>) => {
      x.frozen.changed[1].path = x.frozen.changed[0].path;
    },
    (x: ReturnType<typeof fixture>) => {
      x.frozen.changed[0].head!.gitBlob = "a".repeat(40);
    },
    (x: ReturnType<typeof fixture>) => {
      x.frozen.kind = "type-work";
    },
  ]) {
    const x = fixture();
    mutate(x);
    assert.throws(() => verifyCandidateSourceInputs(x.frozen, x.actual));
  }
});

test("candidate sources require complete unchanged Cargo bodies rather than a version substring", () => {
  for (const mutate of [
    (x: ReturnType<typeof fixture>) => {
      x.actual.cargo.head[1] = Buffer.from("same SDK label, another graph");
    },
    (x: ReturnType<typeof fixture>) => {
      x.actual.cargo.base.pop();
    },
    (x: ReturnType<typeof fixture>) => {
      x.frozen.cargo.reverse();
    },
    (x: ReturnType<typeof fixture>) => {
      x.frozen.cargo[0].pin.bytes += 1;
    },
    (x: ReturnType<typeof fixture>) => {
      x.frozen.changed[0].base = null;
    },
    (x: ReturnType<typeof fixture>) => {
      x.actual.protocol.head[0] = Buffer.from("changed printing wrapper");
    },
    (x: ReturnType<typeof fixture>) => {
      x.frozen.protocol.reverse();
    },
  ]) {
    const x = fixture();
    mutate(x);
    assert.throws(() => verifyCandidateSourceInputs(x.frozen, x.actual));
  }
});

test("type-work source roster includes its exact source-custody tests without another product effect", () => {
  const x = fixture();
  const names = [
    "crates/vize_canon/src/batch/virtual_project/snapshots/vize_canon__batch__virtual_project__tests__pruned_helpers.snap",
    componentChecksPath,
    "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update.rs",
    "crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/instance_props.rs",
    "tests/_fixtures/differential/typechecker/empty-vue-star-exports/malformed-membership/instance-prop-metadata-update.json",
    "tests/tooling/canon-instance-prop-ambient.test.ts",
    "tests/tooling/canon-instance-prop-metadata.test.ts",
    "tests/tooling/fixtures/canon-instance-prop-metadata-before.json",
  ];
  x.frozen.kind = "type-work";
  x.actual.bodies = {
    base: names.map((name) => Buffer.from("old whole " + name)),
    head: names.map((name) => Buffer.from("new whole " + name)),
  };
  x.frozen.changed = names.map((path, index) => ({
    path,
    base: pin(x.actual.bodies.base[index]),
    head: pin(x.actual.bodies.head[index]),
  }));
  x.actual.changedPaths = names;
  assert.doesNotThrow(() => verifyCandidateSourceInputs(x.frozen, x.actual));
  x.actual.changedPaths = [...names, "crates/vize/src/commands/check/runner.rs"];
  assert.throws(() => verifyCandidateSourceInputs(x.frozen, x.actual));
});
