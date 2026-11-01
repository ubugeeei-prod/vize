import assert from "node:assert/strict";
import test from "node:test";
import { deriveHumanBins } from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-inputs.ts";
import { humanDependencyBinPartition } from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-partition.ts";
import type { HumanBinFacts } from "../../../tools/support/release/human_reports_11252_11253_public/dependency-bin-types.ts";
import {
  derivedNodePath,
  pnpmSource,
  renderPnpm12161Unix,
} from "../../../tools/support/release/human_reports_11252_11253_public/pnpm-shim-render.ts";
import {
  root,
  parentDir,
  ownerDir,
  image,
  fixture,
  pkg,
  mutatePlan,
  expectedNode,
} from "./human-reports-11252-11253-bin.fixture.ts";
test("both synthetic role graphs derive independent complete bin rosters and37payloads", () => {
  for (const role of ["runtime-prop-null-11252", "imports-vue-ts-11253"] as const) {
    const facts = fixture(role),
      proof = humanDependencyBinPartition(facts);
    assert.equal(proof.publicPayloadCount, 37);
    assert.equal(proof.sourceConformantBinCount, role === "runtime-prop-null-11252" ? 1 : 2);
    assert.equal(proof.allInstalledBytesEqualArchive, false);
    assert.equal(proof.everyActualEntryAccountedFor, true);
    assert.equal(proof.accepted, false);
    assert.equal(proof.productQualified, false);
    assert.equal(proof.afterQualified, false);
    assert.equal(proof.issueQualified, false);
    assert(
      proof.files
        .filter((file) => file.bucket === "SOURCE_CONFORMANT_BIN")
        .every(
          (file) =>
            !file.publicSriCredit && !file.nativeCredit && !file.sdkCredit && file.rawBase64,
        ),
    );
  }
});
test("renderer equals independent fixed primary Node vector wholebytes", () => {
  for (const bin of ["tsc", "tsserver"])
    assert(
      renderPnpm12161Unix({
        target: ownerDir + "/bin/" + bin,
        shimPath: parentDir + "/node_modules/.bin/" + bin,
        runtime: "node",
        nodePath: [],
        relocatableRoot: null,
      }).equals(expectedNode(bin)),
    );
});
test("nointerpreter, orderedNODEPATH andrelocatable branches match literal source laws", () => {
  const shim = "/r/node_modules/.bin/tsc",
    target = "/r/typescript/bin/tsc";
  const expected =
    pnpmSource.header +
    pnpmSource.physicalPrelude +
    pnpmSource.platform +
    'if [ -z "$NODE_PATH" ]; then\n  export NODE_PATH="$basedir_abs/../../x:$basedir_abs/../../y"\nelse\n  export NODE_PATH="$basedir_abs/../../x:$basedir_abs/../../y:$NODE_PATH"\nfi\nexec "$basedir_abs/../../typescript/bin/tsc"  "$@"\nexit $?\n# cmd-shim-target=../../typescript/bin/tsc\n';
  assert.equal(
    renderPnpm12161Unix({
      target,
      shimPath: shim,
      runtime: null,
      nodePath: ["/r/x", "/r/y"],
      relocatableRoot: "/r",
    }).toString(),
    expected,
  );
  assert.deepEqual(
    derivedNodePath("/r/node_modules/.pnpm/typescript@5.9.3/node_modules/typescript", [
      "/r/node_modules/.pnpm/node_modules",
    ]),
    [
      "/r/node_modules/.pnpm/typescript@5.9.3/node_modules/typescript/node_modules",
      "/r/node_modules/.pnpm/typescript@5.9.3/node_modules",
      "/r/node_modules/.pnpm/node_modules",
    ],
  );
});
test("missing/extra/duplicate files and unknownemptydir refuse; originals remain", () => {
  for (const mutate of [
    (f: HumanBinFacts) => f.inventory.pop(),
    (f: HumanBinFacts) =>
      f.inventory.push(image(parentDir + "/node_modules/.bin/unknown", Buffer.from("anything"))),
    (f: HumanBinFacts) => f.inventory.push(f.inventory[0]!),
    (f: HumanBinFacts) =>
      f.inventory.push({
        path: parentDir + "/unknown",
        realPath: parentDir + "/unknown",
        kind: "directory",
        body: null,
      }),
  ]) {
    const facts = fixture();
    mutate(facts);
    assert.throws(() => humanDependencyBinPartition(facts));
  }
});
test("payload modification and symlink/special/redirected vectors refuse", () => {
  for (const mutate of [
    (f: HumanBinFacts) => {
      f.inventory[0]!.body = Buffer.from("changed");
    },
    (f: HumanBinFacts) => {
      f.inventory[0]!.kind = "symlink";
    },
    (f: HumanBinFacts) => {
      f.inventory[0]!.kind = "other";
    },
    (f: HumanBinFacts) => {
      f.inventory[0]!.realPath = "/elsewhere/file";
    },
  ]) {
    const facts = fixture();
    mutate(facts);
    assert.throws(() => humanDependencyBinPartition(facts));
  }
});
test("whole shim mutations cannotpass header/marker recognition orhashonlyrule", () => {
  const changes = [
    "# Resolve",
    "command -p",
    "node.exe",
    "typescript/bin/tsc",
    "cmd-shim-target",
    '"$@"',
  ];
  for (const token of changes) {
    const facts = fixture(),
      shim = facts.inventory.find((file) => file.path.endsWith("/.bin/tsc"))!;
    shim.body = Buffer.from(shim.body!.toString().replace(token, token + "changed"));
    assert.throws(() => humanDependencyBinPartition(facts));
  }
  for (const transform of [
    (raw: Buffer) => Buffer.concat([raw, Buffer.from("echo bad\n")]),
    (raw: Buffer) => raw.subarray(0, -1),
  ]) {
    const facts = fixture(),
      shim = facts.inventory.find((file) => file.path.endsWith("/.bin/tsc"))!;
    shim.body = transform(shim.body!);
    assert.throws(() => humanDependencyBinPartition(facts));
  }
});
test("wrong/ambiguous owner, declaredtarget, publicbody orlock cannot choose oracle", () => {
  for (const mutate of [
    (f: HumanBinFacts) => {
      f.owner.version = "0.0.0";
    },
    (f: HumanBinFacts) => f.graph.push({ ...f.graph[0]! }),
    (f: HumanBinFacts) => {
      f.dependencyOwners.pop();
    },
    (f: HumanBinFacts) => {
      f.dependencyOwners.push(f.owner);
    },
    (f: HumanBinFacts) => {
      f.graph[0]!.directory = "/wrongowner";
    },
    (f: HumanBinFacts) => {
      f.targets[0]!.path += "wrong";
    },
    (f: HumanBinFacts) => {
      f.targets[0]!.body = Buffer.from("wrongtarget");
    },
    (f: HumanBinFacts) => {
      f.targets[0]!.kind = "symlink";
    },
    (f: HumanBinFacts) => {
      f.lock = Buffer.from("unknownlock");
    },
    (f: HumanBinFacts) => {
      f.owner.integrity = "sha512-YWJj";
    },
  ]) {
    const facts = fixture();
    mutate(facts);
    assert.throws(() => deriveHumanBins(facts));
  }
});
test("all five no-bin owners are authenticated and another declared executable cannot hide behind TypeScript", () => {
  for (const name of ["tsc", "unregistered"]) {
    const facts = fixture(),
      prior = facts.dependencyOwners[1]!;
    const replacement = pkg(
      prior.name,
      prior.version,
      prior.directory,
      { name: prior.name, version: prior.version, bin: { [name]: "./bin/extra" } },
      { "bin/extra": Buffer.from("#!/usr/bin/env node\n// inert other owner\n") },
    );
    facts.dependencyOwners[1] = replacement;
    facts.lock = Buffer.from(facts.lock.toString().replace(prior.integrity, replacement.integrity));
    assert.throws(() => deriveHumanBins(facts));
  }
  const facts = fixture();
  facts.dependencyOwners[1]!.archive = Buffer.from("not the actual public package");
  assert.throws(() => deriveHumanBins(facts));
});
test("missing sourceauthority, output-bearing plan andchanged options refuse", () => {
  for (const mutate of [
    (f: HumanBinFacts) => {
      f.expectedInputPolicyGitBlob = null;
    },
    (f: HumanBinFacts) => {
      f.inputPolicyRaw = null;
    },
    (f: HumanBinFacts) =>
      mutatePlan(
        f,
        (plan) => {
          plan.stdout = "observed";
        },
        true,
      ),
    (f: HumanBinFacts) =>
      mutatePlan(f, (plan) => {
        (plan.options as Record<string, unknown>).relocatableRoot = root;
      }),
    (f: HumanBinFacts) =>
      mutatePlan(
        f,
        (plan) => {
          (plan.options as Record<string, unknown>).preferSymlinkedExecutables = true;
        },
        true,
      ),
    (f: HumanBinFacts) =>
      mutatePlan(
        f,
        (plan) => {
          (plan.options as Record<string, unknown>).extraNodePaths = ["/outside"];
        },
        true,
      ),
    (f: HumanBinFacts) => {
      f.settings.userConfig = Buffer.from("unowned=true");
    },
  ]) {
    const facts = fixture();
    mutate(facts);
    assert.throws(() => humanDependencyBinPartition(facts));
  }
});
test("manager observations stay exact without claiming a writer source/image join", () => {
  for (const mutate of [
    (f: HumanBinFacts) => {
      f.manager = null;
    },
    (f: HumanBinFacts) => {
      f.manager!.currentImage.body = Buffer.alloc(1109);
    },
    (f: HumanBinFacts) => {
      f.manager!.reportedVersion = "12.1.0";
    },
    (f: HumanBinFacts) => {
      f.manager!.versionStatus = 1;
    },
    (f: HumanBinFacts) => {
      f.manager!.originalArgv = ["install", "--override"];
    },
  ]) {
    const facts = fixture();
    mutate(facts);
    assert.throws(() => humanDependencyBinPartition(facts));
  }
});
test("wronghost and sourceunsafe paths refuse instead ofgeneralized platformfallback", () => {
  const facts = fixture();
  facts.host = { platform: "darwin", arch: "arm64" };
  assert.throws(() => deriveHumanBins(facts));
  assert.throws(() =>
    renderPnpm12161Unix({
      target: '/r/"unsafe',
      shimPath: "/r/bin/tsc",
      runtime: "node",
      nodePath: [],
      relocatableRoot: null,
    }),
  );
});
