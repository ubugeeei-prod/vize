import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import test from "node:test";
import YAML from "yaml";
import {
  assertBracesLock,
  bracesPatchHash,
  bracesPatchPath,
} from "../../tools/support/security/braces-lock.ts";
import { assertNoDirectBraces } from "../../tools/support/security/braces-consumers.ts";
import {
  assertBracesPackage,
  installedBracesCensus,
  verifyBracesPatch,
  verifyInstalledBraces,
} from "../../tools/support/security/braces-proof.ts";
import { verifyBracesDepth, type Braces } from "../../tools/support/security/braces-laws.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const workspace = () => YAML.parse(fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf8"));
const lock = () =>
  YAML.parseAllDocuments(fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"))[1].toJS();

test("real installed Braces source, all visible instances and linked consumers are bounded", () => {
  const proof = verifyInstalledBraces(root);
  assert.ok(proof.instances > 0);
  assert.equal(proof.patch, bracesPatchHash);
  assert.equal(proof.negativeChecks, proof.instances * 59);
  assert.equal(proof.positiveChecks, proof.instances * 18);
});

test("the actual lock rejects altered source, aliases, direct importers and new consumers", () => {
  assertBracesLock(workspace(), lock());
  const mutations = [
    (w: any, _l: any) => {
      delete w.patchedDependencies["braces@3.0.3"];
    },
    (_w: any, l: any) => {
      l.patchedDependencies["braces@3.0.3"] = "0".repeat(64);
    },
    (_w: any, l: any) => {
      l.packages["braces@3.0.3"].resolution.integrity = "bad";
    },
    (_w: any, l: any) => {
      l.packages["braces@3.0.2"] = l.packages["braces@3.0.3"];
    },
    (_w: any, l: any) => {
      l.snapshots["chokidar@3.6.0"].dependencies.braces = "3.0.3";
    },
    (_w: any, l: any) => {
      l.snapshots["alien@1.0.0"] = {
        dependencies: { braces: "3.0.3(patch_hash=" + bracesPatchHash + ")" },
      };
    },
    (_w: any, l: any) => {
      l.snapshots["chokidar@3.6.0"].dependencies.alias =
        "braces@3.0.3(patch_hash=" + bracesPatchHash + ")";
    },
    (_w: any, l: any) => {
      l.snapshots["fill-range@7.1.1"].dependencies["to-regex-range"] = "5.0.0";
    },
    (w: any, _l: any) => {
      w.audit = { ignoreGhsas: ["GHSA-vfj7-8cjw-p6xm"] };
    },
    (w: any, _l: any) => {
      w.overrides.braces = "3.0.3";
    },
  ];
  for (const field of ["dependencies", "devDependencies", "optionalDependencies"])
    for (const specifier of ["npm:braces", "npm:braces@3.0.3"]) {
      mutations.push((_w: any, l: any) => {
        l.importers["."][field] ??= {};
        l.importers["."][field].alias = {
          specifier,
          version: "braces@3.0.3(patch_hash=" + bracesPatchHash + ")",
        };
      });
    }
  mutations.push((_w: any, l: any) => {
    l.importers["."].dependencies ??= {};
    l.importers["."].dependencies.braces = {
      specifier: "3.0.3",
      version: "3.0.3(patch_hash=" + bracesPatchHash + ")",
    };
  });
  for (const mutate of mutations) {
    const w = workspace(),
      l = lock();
    mutate(w, l);
    assert.throws(() => assertBracesLock(w, l));
  }
});

test("literal source admission rejects unreviewed module, escape and browser routes", () => {
  const name = "bra" + "ces";
  const module = (value: string) => "re" + "quire(/* gap */" + JSON.stringify(value) + ")";
  for (const value of [
    name,
    "npm:" + name,
    name + "/lib/parse",
    `https://esm.sh/${name}@3.0.3`,
    `//esm.sh/${name}@3.0.3`,
  ])
    assert.throws(() => assertNoDirectBraces(module(value)));
  for (const suffix of [String.raw`\u0063es`, String.raw`\x63es`, String.raw`\u{63}es`]) {
    const escaped = "bra" + suffix;
    assert.throws(() => assertNoDirectBraces("im" + 'port/* gap */"' + escaped + '"'));
  }
  for (const value of [
    `//esm.sh/${name}@3.0.3`,
    `HTTPS://esm.sh/${name}@3.0.3`,
    ` //esm.sh/${name}@3.0.3 `,
    ` &#x2f;&#x2f;esm.sh/${name}@3.0.3 `,
  ])
    assert.throws(() =>
      assertNoDirectBraces("<script src=" + JSON.stringify(value) + "></script>"),
    );
  assert.throws(() =>
    assertNoDirectBraces("<script src=" + `//esm.sh/${name}@3.0.3` + "></script>"),
  );
  assertNoDirectBraces(
    "im" + 'port path from "node:path"; const label=' + JSON.stringify(name) + ";",
  );
});

test("whole-package proof rejects original unpatched, changed, extra and symlink bytes", () => {
  const { instances } = installedBracesCensus(root),
    original = [...instances][0];
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "braces-source-laws-"));
  try {
    const packageRoot = path.join(temporary, "package");
    fs.cpSync(original, packageRoot, { recursive: true });
    assertBracesPackage(packageRoot);
    const patch = fs.readFileSync(path.join(root, bracesPatchPath));
    const result = spawnSync("git", ["apply", "--reverse", "--unsafe-paths", "-"], {
      cwd: packageRoot,
      input: patch,
      encoding: "utf8",
    });
    assert.equal(result.status, 0, result.stderr);
    assert.throws(() => assertBracesPackage(packageRoot));
    fs.mkdirSync(path.join(temporary, "node_modules"));
    const actualRequire = createRequire(path.join(original, "package.json"));
    fs.symlinkSync(
      path.dirname(actualRequire.resolve("fill-range/package.json")),
      path.join(temporary, "node_modules/fill-range"),
    );
    const unpatchedRequire = createRequire(path.join(packageRoot, "package.json"));
    assert.throws(() => verifyBracesDepth(unpatchedRequire(packageRoot) as Braces));
    fs.rmSync(packageRoot, { recursive: true });
    fs.cpSync(original, packageRoot, { recursive: true });
    const file = path.join(packageRoot, "lib/parse.js"),
      bytes = fs.readFileSync(file);
    fs.writeFileSync(file, Buffer.concat([bytes, Buffer.from("\n")]));
    assert.throws(() => assertBracesPackage(packageRoot));
    fs.writeFileSync(file, bytes);
    fs.writeFileSync(path.join(packageRoot, "extra.js"), "module.exports={};");
    assert.throws(() => assertBracesPackage(packageRoot));
    fs.unlinkSync(path.join(packageRoot, "extra.js"));
    fs.symlinkSync(file, path.join(packageRoot, "extra.js"));
    assert.throws(() => assertBracesPackage(packageRoot));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("public runtime laws pass the actual installed repair", () => {
  const { instances } = installedBracesCensus(root),
    packageRoot = [...instances][0],
    require = createRequire(path.join(packageRoot, "package.json"));
  assert.deepEqual(verifyBracesDepth(require(packageRoot) as Braces), {
    negativeChecks: 59,
    positiveChecks: 18,
  });
});

test("patch attestation rejects missing and changed bytes", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "braces-patch-laws-"));
  try {
    assert.throws(() => verifyBracesPatch(temporary));
    fs.mkdirSync(path.join(temporary, "patches"));
    const file = path.join(temporary, bracesPatchPath),
      bytes = fs.readFileSync(path.join(root, bracesPatchPath));
    fs.writeFileSync(file, bytes);
    verifyBracesPatch(temporary);
    fs.appendFileSync(file, "\n");
    assert.throws(() => verifyBracesPatch(temporary));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
