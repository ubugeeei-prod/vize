/** Source-bound whole packet verification, including the imported observer. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { identify, save, hash } from "./canon-instance-directives-io.ts";
import { readRefFactoryAuthority } from "./ref-factory-authority.ts";
import { compareFactoryProjection } from "./ref-factory-parity.ts";

export function refFactorySourcePair(base: string, head: string) {
  const authority = readRefFactoryAuthority(head);
  const preamble = authority.source.path;
  assert.equal(
    fs.readFileSync(path.join(base, preamble), "utf8"),
    authority.source.beforeWholeUtf8,
  );
  const scope = "crates/vize_canon/src/virtual_ts/scope/ref_instances.rs";
  const previous = fs.readFileSync(path.join(base, scope), "utf8"),
    current = fs.readFileSync(path.join(head, scope), "utf8");
  const string = '"(?:[^"\\\\]|\\\\.)*"';
  const old = [
    ...previous.matchAll(
      new RegExp(`pub\\(crate\\) const REF_INSTANCE_HELPERS: &str = (${string});`, "gu"),
    ),
  ];
  const instance = [
    ...current.matchAll(
      new RegExp(
        `macro_rules! template_ref_instance_helper \\{\\s*\\(\\) => \\{\\s*(${string})`,
        "gu",
      ),
    ),
  ];
  assert.equal(old.length, 1);
  assert.equal(instance.length, 1);
  assert.equal(
    JSON.parse(old[0][1]),
    "  " + authority.declaration.wholeUtf8 + (JSON.parse(instance[0][1]) as string),
  );
  for (const file of ["Cargo.toml", "Cargo.lock", "crates/vize_canon/Cargo.toml"])
    assert.equal(identify(path.join(base, file)).sha256, identify(path.join(head, file)).sha256);
  return {
    baselinePreamble: identify(path.join(base, preamble)),
    headPreamble: identify(path.join(head, preamble)),
    baselineScope: identify(path.join(base, scope)),
    headScope: identify(path.join(head, scope)),
    factory: identify(path.join(head, authority.declaration.path)),
    embeddedHelperWholeEqual: true,
  };
}

function verify(directory: string, base: string, head: string) {
  const sources = refFactorySourcePair(base, head);
  const beforeBytes = fs.readFileSync(path.join(directory, "base-projection.json")),
    afterBytes = fs.readFileSync(path.join(directory, "head-projection.json"));
  const before = JSON.parse(beforeBytes.toString("utf8")) as Parameters<
      typeof compareFactoryProjection
    >[0],
    after = JSON.parse(afterBytes.toString("utf8")) as typeof before;
  const parity = compareFactoryProjection(before, after);
  const pinnedInputs = [
    ["Child.vue.txt", "src/Child.vue"],
    ["Probe.vue.txt", "src/Probe.vue"],
    ["tsconfig.json.txt", "tsconfig.json"],
  ].map(([source, file]) => ({
    file,
    wholeUtf8: fs.readFileSync(
      path.join(head, "tests/_fixtures/differential/typechecker/shared-ref-factory", source),
      "utf8",
    ),
  }));
  assert.deepEqual(before.inputs, pinnedInputs);
  assert.deepEqual(after.inputs, pinnedInputs);
  assert.deepEqual(
    before.files.map((file) => ({ file: file.file, wholeUtf8: file.source })),
    pinnedInputs.slice(0, 2),
  );
  const observer = "crates/vize_canon/tests/ref_factory_projection.rs";
  const observerIdentity = identify(path.join(head, observer));
  assert.equal(identify(path.join(base, observer)).sha256, observerIdentity.sha256);
  const status = fs.readFileSync(path.join(directory, "base-after-observer-copy.status"), "utf8");
  assert.equal(
    status,
    "?? " + observer + "\n",
    "baseline test-only import must be the sole dirty path",
  );
  const git = (root: string, ...args: string[]) =>
    execFileSync("git", ["-C", root, ...args], { encoding: "utf8" }).trim();
  assert.equal(git(base, "diff", "--name-only", "--", "crates/vize_canon/src"), "");
  const images = (["base", "head"] as const).map((role) => {
    const target = path.join(
      process.env.RUNNER_TEMP!,
      role === "base" ? "ref-factory-base-target" : "slot-head-target",
      "ci-opt",
      "deps",
    );
    const selected = fs
      .readdirSync(target)
      .filter((file) => /^ref_factory_projection-[a-f0-9]+$/u.test(file));
    assert.equal(selected.length, 1);
    return {
      role,
      source: git(role === "base" ? base : head, "rev-parse", "HEAD"),
      tree: git(role === "base" ? base : head, "rev-parse", "HEAD^{tree}"),
      executable: identify(path.join(target, selected[0])),
    };
  });
  const baseBuild = JSON.parse(
    fs.readFileSync(path.join(directory, "base-build.json"), "utf8"),
  ) as {
    source: string;
    tree: string;
    cleanBeforeObserverCopy: boolean;
    observerImportedYet: boolean;
    image: { path: string; sha256: string };
  };
  assert.equal(baseBuild.source, images[0].source);
  assert.equal(baseBuild.tree, images[0].tree);
  assert.equal(baseBuild.cleanBeforeObserverCopy, true);
  assert.equal(baseBuild.observerImportedYet, false);
  assert.equal(identify(baseBuild.image.path).sha256, baseBuild.image.sha256);
  const headBuild = JSON.parse(
    fs.readFileSync(path.join(directory, "../head-build.json"), "utf8"),
  ) as { source: string; sha256: string };
  assert.equal(headBuild.source, images[1].source);
  assert.equal(identify(path.join(process.env.RUNNER_TEMP!, "slot-head")).sha256, headBuild.sha256);
  save(path.join(directory, "projection-verification.json"), {
    sources,
    parity,
    observer: observerIdentity,
    importedObserver: {
      soleBaseDirtyPath: observer,
      sameObserverBytes: true,
      productionOverlay: false,
      baselineWasCleanAtImageBuild: true,
      baselineIsTestModifiedNow: true,
    },
    images,
    baselineImageReceipt: identify(path.join(directory, "base-build.json")),
    headImageReceipt: identify(path.join(directory, "../head-build.json")),
    beforePacket: { bytes: beforeBytes.length, sha256: hash(beforeBytes) },
    headPacket: { bytes: afterBytes.length, sha256: hash(afterBytes) },
    scope:
      "full generated code/maps/roles/links/import lookup parity only; native/type-engine proof is separate",
    timingComparison: false,
  });
}

if (process.argv[2] === "--source-only") {
  assert.equal(process.argv.length, 5);
  refFactorySourcePair(process.argv[3], process.argv[4]);
} else {
  assert.equal(process.argv.length, 5);
  verify(...(process.argv.slice(2) as [string, string, string]));
}
