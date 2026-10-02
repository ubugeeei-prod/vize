import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { after, test } from "node:test";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

import { verifyForgeSecurity } from "../../tools/support/security/node-forge-laws.ts";
import {
  advisoryId,
  assertCleanReport,
  assertForgeLock,
  assertForgePackage,
  assertNoDirectForge,
  assertRemediatedReport,
  patchPath,
  verifyPatchFile,
} from "../../tools/support/security/node-forge-proof.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const workspace = YAML.parse(fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf8"));
const lock = YAML.parseAllDocuments(
  fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"),
)[1].toJS();
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), "vize-node-forge-laws-"));
after(() => fs.rmSync(scratch, { recursive: true, force: true }));

test("patch is bound to its actual bytes and both lock documents retain registry identity", () => {
  verifyPatchFile(root);
  assertForgeLock(workspace, lock);
});

test("changed registry, patch, version, graph, aliases or direct importer reject recognition", () => {
  for (const mutate of [
    (next: typeof lock) => {
      next.packages["node-forge@1.4.0"].resolution.integrity += "x";
    },
    (next: typeof lock) => {
      next.patchedDependencies["node-forge@1.4.0"].hash = "wrong";
    },
    (next: typeof lock) => {
      next.snapshots["unreviewed@1.0.0"] = { dependencies: { "node-forge": "1.4.0" } };
    },
    (next: typeof lock) => {
      next.snapshots["unreviewed@1.0.0"] = { dependencies: { renamed: "node-forge@1.4.0" } };
    },
    (next: typeof lock) => {
      next.snapshots["unreviewed@1.0.0"] = { dependencies: { renamed: "listhen@1.10.1" } };
    },
    (next: typeof lock) => {
      next.importers["."].dependencies = { "node-forge": { specifier: "1.4.0", version: "1.4.0" } };
    },
    (next: typeof lock) => {
      next.importers["."].dependencies = { listhen: { specifier: "1.10.1", version: "1.10.1" } };
    },
    (next: typeof lock) => {
      next.importers["."].dependencies = {
        renamed: { specifier: "npm:listhen@1.10.1", version: "listhen@1.10.1" },
      };
    },
    (next: typeof lock) => {
      next.importers["."].dependencies = {
        renamed: { specifier: "npm:listhen", version: "listhen@1.10.1" },
      };
    },
    (next: typeof lock) => {
      next.importers["."].dependencies = {
        renamed: { specifier: "npm:node-forge", version: "node-forge@1.4.0" },
      };
    },
    (next: typeof lock) => {
      next.importers["."].dependencies = {
        renamed: { specifier: "*", version: "node-forge@1.4.0" },
      };
    },
    (next: typeof lock) => {
      next.packages["node-forge@1.4.1"] = next.packages["node-forge@1.4.0"];
    },
  ]) {
    const next = structuredClone(lock);
    mutate(next);
    assert.throws(() => assertForgeLock(workspace, next));
  }
  assert.throws(() => assertForgeLock({ ...workspace, audit: { ignore: [advisoryId] } }, lock));
});

test("missing or changed patch cannot attest remediation", () => {
  assert.throws(() => verifyPatchFile(scratch), /ENOENT/);
  fs.mkdirSync(path.join(scratch, "patches"));
  fs.writeFileSync(
    path.join(scratch, patchPath),
    fs.readFileSync(path.join(root, patchPath)) + "\n",
  );
  assert.throws(() => verifyPatchFile(scratch));
});

function report() {
  return {
    advisories: {
      "1130000": {
        github_advisory_id: advisoryId,
        module_name: "node-forge",
        severity: "high",
        vulnerable_versions: "<=1.4.0",
        patched_versions: null,
        url: "https://github.com/advisories/" + advisoryId,
        findings: [
          {
            version: "1.4.0",
            paths: [".>nuxt>@nuxt/cli>listhen>node-forge"],
            dev: false,
            optional: false,
            bundled: false,
          },
        ],
      },
    },
    metadata: { vulnerabilities: { info: 0, low: 0, moderate: 0, high: 1, critical: 0 } },
  };
}

test("only the one exact version-based advisory permits source attestation", () => {
  assertRemediatedReport(report());
  for (const mutate of [
    (value: ReturnType<typeof report>) => {
      value.advisories["1130000"].github_advisory_id = "GHSA-new";
    },
    (value: ReturnType<typeof report>) => {
      value.advisories["1130000"].module_name = "other";
    },
    (value: ReturnType<typeof report>) => {
      value.advisories["1130000"].findings[0].version = "1.4.1";
    },
    (value: ReturnType<typeof report>) => {
      value.advisories["1130000"].findings[0].paths = [".>unknown>node-forge"];
    },
    (value: ReturnType<typeof report>) => {
      value.metadata.vulnerabilities.moderate = 1;
    },
    (value: ReturnType<typeof report>) => {
      value.metadata.vulnerabilities.critical = 1;
    },
    (value: ReturnType<typeof report>) => {
      value.advisories["another"] = value.advisories["1130000"];
    },
  ]) {
    const value = report();
    mutate(value);
    assert.throws(() => assertRemediatedReport(value));
  }
  assert.throws(() => assertRemediatedReport({ error: "registry failed" }));
  assert.throws(() => assertCleanReport(report()));
  assertCleanReport({
    advisories: {},
    metadata: { vulnerabilities: { info: 0, low: 2, moderate: 0, high: 0, critical: 0 } },
  });
  const withLow = report();
  const low = {
    ...withLow.advisories["1130000"],
    severity: "low",
    github_advisory_id: "synthetic-low-control",
  };
  withLow.advisories["synthetic-low-control"] = low;
  withLow.metadata.vulnerabilities.low = 1;
  assertRemediatedReport(withLow);
  assertCleanReport({
    advisories: { "synthetic-low-control": low },
    metadata: { vulnerabilities: { info: 0, low: 1, moderate: 0, high: 0, critical: 0 } },
  });
  withLow.metadata.vulnerabilities.low = 0;
  assert.throws(() => assertRemediatedReport(withLow));
});

test("a direct or browser-distribution consumer does not inherit the Node listhen proof", () => {
  const name = "node-" + "forge";
  const listenerName = "lis" + "then";
  const encodedSlash = "&" + "#x2f;";
  for (const source of [
    `import forge from "${name}";`,
    `const forge = require("${name}/dist/forge.min.js");`,
    `const forge = await import("${name}/lib/rsa.js");`,
    `<script src="${name}/dist/forge.min.js"></script>`,
    `<script src="https://cdn.jsdelivr.net/npm/${name}@1.4.0/dist/forge.min.js"></script>`,
    `import "https://cdn.example/${name}/lib/index.js";`,
    `import "https://esm.sh/${name}@1.4.0";`,
    `<script type="module" src="//esm.sh/${name}@1.4.0"></script>`,
    `<template>Vue source</template><script type="module" src="HTTPS://esm.sh/${name}@1.4.0"></script>`,
    `<script type="module" src=//esm.sh/${name}@1.4.0></script>`,
    `<script type="module" src=" //esm.sh/${name}@1.4.0 "></script>`,
    `<script type="module" src=" ${encodedSlash}${encodedSlash}esm.sh/${name}@1.4.0 "></script>`,
    `import forge from/*x*/"${name}";`,
    `const forge = require(/*x*/"${name}");`,
    `const forge = require/*x*/(/*x*/"${name}");`,
    `const forge = require("node\\u002dforge");`,
    `const forge = require("\\u006eode-forge");`,
    `const forge = await import(\`${name}\`);`,
    `import { listen } from "${listenerName}";`,
    `import { listen } from//comment\n"${listenerName}";`,
    `const listen = require(/*x*/"${listenerName}");`,
    `import "https://esm.sh/${listenerName}@1.10.1";`,
  ])
    assert.throws(() => assertNoDirectForge(source));
  assertNoDirectForge('import crypto from "node:crypto";');
});

function installedForge(): string {
  const store = path.join(root, "node_modules/.pnpm");
  const entry = fs.readdirSync(store).find((name) => name.startsWith("node-forge@1.4.0"));
  assert.ok(entry, "real installed Forge required");
  return fs.realpathSync(path.join(store, entry, "node_modules/node-forge"));
}
function loadFresh(packageRoot: string) {
  packageRoot = fs.realpathSync(packageRoot);
  const require = createRequire(path.join(packageRoot, "package.json"));
  for (const filename of Object.keys(require.cache)) {
    if (filename.startsWith(packageRoot + path.sep)) delete require.cache[filename];
  }
  return require(packageRoot);
}

test("real source patch rejects original and partial repairs while retaining RSA, X509 and PKCS12", () => {
  const originalHash = "fd4740238145ec26470eb3f06a627c72039538ce1307dbdce40521f94dfd0a50";
  const candidateHash = "bb2c61cef273c89aaedb9c865173f5330d76917024ff1c2c1d39ffd5be01cc2d";
  const originalCondition = "            obj.value.length !== 2) {";
  const completeCondition =
    "            obj.value.length !== 2 ||\n            obj.value[0].value.length !==\n              (('parameters' in capture) ? 2 : 1) ||\n            ('parameters' in capture && capture.parameters !== '')) {";
  const oidOriginal = "          var oid = asn1.derToOid(capture.algorithmIdentifier);";
  const oidComplete =
    oidOriginal +
    "\n          if(asn1.oidToDer(oid).getBytes() !== capture.algorithmIdentifier) {\n            throw new Error(\n              'ASN.1 object does not contain a valid RSASSA-PKCS1-v1_5 ' +\n              'DigestInfo value.');\n          }";
  const packageRoot = path.join(scratch, "node-forge");
  fs.cpSync(installedForge(), packageRoot, { recursive: true });
  const rsaFile = path.join(packageRoot, "lib/rsa.js");
  let original = fs.readFileSync(rsaFile, "utf8");
  if (createHash("sha256").update(original).digest("hex") === candidateHash) {
    original = original
      .replace(completeCondition, originalCondition)
      .replace(oidComplete, oidOriginal);
    fs.writeFileSync(rsaFile, original);
  }
  assert.equal(createHash("sha256").update(original).digest("hex"), originalHash);
  assert.throws(() => assertForgePackage(packageRoot), /unpatched or modified/);
  assert.throws(() => verifyForgeSecurity(loadFresh(packageRoot)), /Missing expected exception/);
  const applied = spawnSync("git", ["apply", path.join(root, patchPath)], {
    cwd: packageRoot,
    encoding: "utf8",
  });
  assert.equal(applied.status, 0, applied.stderr);
  const candidate = fs.readFileSync(rsaFile, "utf8");
  assert.equal(createHash("sha256").update(candidate).digest("hex"), candidateHash);
  assertForgePackage(packageRoot);
  verifyForgeSecurity(loadFresh(packageRoot));
  for (const partial of [
    candidate.replace(oidComplete, oidOriginal),
    candidate
      .replace(
        completeCondition,
        completeCondition.replace(
          " ||\n            ('parameters' in capture && capture.parameters !== '')",
          "",
        ),
      )
      .replace(oidComplete, oidOriginal),
    candidate + "\n",
  ]) {
    fs.writeFileSync(rsaFile, partial);
    assert.throws(() => assertForgePackage(packageRoot), /unpatched or modified/);
    if (!partial.endsWith("\n\n")) {
      assert.throws(
        () => verifyForgeSecurity(loadFresh(packageRoot)),
        /Missing expected exception/,
      );
    }
  }
  fs.writeFileSync(rsaFile, candidate);
  assertForgePackage(packageRoot);
});
