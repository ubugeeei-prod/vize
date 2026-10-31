import assert from "node:assert/strict";
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
  assertListhenPackage,
  assertNoDirectForge,
  patchHash,
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

test("the retired backport stays immutable while the lock binds upstream registry identity", () => {
  verifyPatchFile(root);
  assertForgeLock(workspace, lock);
});

test("changed registry, version, graph, aliases or direct importer reject recognition", () => {
  for (const mutate of [
    (next: typeof lock) => {
      next.packages["node-forge@1.4.1"].resolution.integrity += "x";
    },
    (next: typeof lock) => {
      next.overrides["node-forge"] = "1.4.0";
    },
    (next: typeof lock) => {
      next.snapshots["unreviewed@1.0.0"] = { dependencies: { "node-forge": "1.4.1" } };
    },
    (next: typeof lock) => {
      next.snapshots["unreviewed@1.0.0"] = { dependencies: { renamed: "node-forge@1.4.1" } };
    },
    (next: typeof lock) => {
      next.snapshots["unreviewed@1.0.0"] = { dependencies: { renamed: "listhen@1.10.1" } };
    },
    (next: typeof lock) => {
      next.importers["."].dependencies = { "node-forge": { specifier: "1.4.1", version: "1.4.1" } };
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
        renamed: { specifier: "npm:node-forge", version: "node-forge@1.4.1" },
      };
    },
    (next: typeof lock) => {
      next.importers["."].dependencies = {
        renamed: { specifier: "*", version: "node-forge@1.4.1" },
      };
    },
    (next: typeof lock) => {
      next.packages["node-forge@1.4.0"] = next.packages["node-forge@1.4.1"];
    },
    (next: typeof lock) => {
      next.snapshots["node-forge@1.4.1(patch_hash=" + patchHash + ")"] = {};
    },
  ]) {
    const next = structuredClone(lock);
    mutate(next);
    assert.throws(() => assertForgeLock(workspace, next));
  }
  assert.throws(() => assertForgeLock({ ...workspace, audit: { ignore: [advisoryId] } }, lock));
  const changedWorkspace = structuredClone(workspace);
  changedWorkspace.overrides["node-forge"] = "1.4.0";
  assert.throws(() => assertForgeLock(changedWorkspace, lock));
});

test("no old or new Forge patch registration can inherit the upstream proof", () => {
  for (const version of ["1.4.0", "1.4.1", "1.4.2"]) {
    for (const patch of [patchHash, { hash: patchHash, path: patchPath }]) {
      const next = structuredClone(lock);
      next.patchedDependencies["node-forge@" + version] = patch;
      assert.throws(() => assertForgeLock(workspace, next));
    }
    const next = structuredClone(workspace);
    next.patchedDependencies["node-forge@" + version] = patchPath;
    assert.throws(() => assertForgeLock(next, lock));
  }
});

test("missing or changed historical patch cannot attest its preserved bytes", () => {
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

test("the former advisory exception no longer permits severe audit findings", () => {
  assert.throws(() => assertCleanReport(report()));
  assertCleanReport({
    advisories: {},
    metadata: { vulnerabilities: { info: 0, low: 2, moderate: 0, high: 0, critical: 0 } },
  });
  const low = { severity: "low", github_advisory_id: "synthetic-low-control" };
  assertCleanReport({
    advisories: { low },
    metadata: { vulnerabilities: { info: 0, low: 1, moderate: 0, high: 0, critical: 0 } },
  });
  assert.throws(() =>
    assertCleanReport({
      advisories: { low },
      metadata: { vulnerabilities: { info: 0, low: 0, moderate: 0, high: 0, critical: 0 } },
    }),
  );
  assert.throws(() => assertCleanReport({ error: "registry failed" }));
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
  const entry = fs.readdirSync(store).find((name) => name.startsWith("node-forge@1.4.1"));
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

test("only four observed pnpm launchers vary while all listhen source files remain exact", () => {
  const store = path.join(root, "node_modules/.pnpm");
  const entry = fs.readdirSync(store).find((name) => name.startsWith("listhen@1.10.1_"));
  assert.ok(entry);
  const packageRoot = path.join(scratch, "listhen");
  fs.cpSync(path.join(store, entry, "node_modules/listhen"), packageRoot, { recursive: true });
  assertListhenPackage(packageRoot);
  for (const name of ["jiti", "listen", "listhen", "untun"])
    fs.writeFileSync(path.join(packageRoot, "node_modules/.bin", name), "generated Linux path\n");
  assertListhenPackage(packageRoot);
  const extra = path.join(packageRoot, "node_modules/.bin/extra");
  fs.writeFileSync(extra, "extra launcher");
  assert.throws(() => assertListhenPackage(packageRoot));
  fs.rmSync(extra);
  const launcher = path.join(packageRoot, "node_modules/.bin/jiti");
  fs.rmSync(launcher);
  assert.throws(() => assertListhenPackage(packageRoot));
  fs.mkdirSync(launcher);
  assert.throws(() => assertListhenPackage(packageRoot));
  fs.rmSync(launcher, { recursive: true });
  fs.symlinkSync(path.join(packageRoot, "bin/listhen.mjs"), launcher);
  assert.throws(() => assertListhenPackage(packageRoot));
  fs.rmSync(launcher);
  fs.writeFileSync(launcher, "generated Linux path\n");
  fs.writeFileSync(path.join(packageRoot, "node_modules/extra.js"), "extra source");
  assert.throws(() => assertListhenPackage(packageRoot));
  fs.rmSync(path.join(packageRoot, "node_modules/extra.js"));
  fs.appendFileSync(path.join(packageRoot, "dist/index.mjs"), "\n// changed source\n");
  assert.throws(() => assertListhenPackage(packageRoot));
});

test("upstream canonical DER rejects original and partial repairs while retaining RSA, X509 and PKCS12", () => {
  const candidateHash = "6632e195295ce76f6bb9131ee770dcd6cc1ee4ffac7906302208b00faf95a87f";
  const packageRoot = path.join(scratch, "node-forge");
  fs.cpSync(installedForge(), packageRoot, { recursive: true });
  const rsaFile = path.join(packageRoot, "lib/rsa.js");
  const candidate = fs.readFileSync(rsaFile, "utf8");
  assert.equal(createHash("sha256").update(candidate).digest("hex"), candidateHash);
  assertForgePackage(packageRoot);
  verifyForgeSecurity(loadFresh(packageRoot));
  const canonicalStart = candidate.indexOf("          // require a canonical DER DigestInfo");
  const canonicalEnd = candidate.indexOf("          // compare the given digest", canonicalStart);
  assert.ok(canonicalStart >= 0 && canonicalEnd > canonicalStart);
  assert.equal(
    candidate.indexOf("          // require a canonical DER DigestInfo", canonicalStart + 1),
    -1,
  );
  const withoutCanonical = candidate.slice(0, canonicalStart) + candidate.slice(canonicalEnd);
  const originalCondition = "            obj.value.length !== 2) {";
  assert.equal(withoutCanonical.split(originalCondition).length, 2);
  const nestedOnly = withoutCanonical.replace(
    originalCondition,
    "            obj.value.length !== 2 ||\n            obj.value[0].value.length !==\n" +
      "              (('parameters' in capture) ? 2 : 1) ||\n" +
      "            ('parameters' in capture && capture.parameters !== '')) {",
  );
  const oidOriginal = "          var oid = asn1.derToOid(capture.algorithmIdentifier);";
  assert.equal(withoutCanonical.split(oidOriginal).length, 2);
  const oidOnly = withoutCanonical.replace(
    oidOriginal,
    oidOriginal +
      "\n          if(asn1.oidToDer(oid).getBytes() !== capture.algorithmIdentifier) {\n" +
      "            throw new Error('DigestInfo value');\n          }",
  );
  for (const partial of [withoutCanonical, nestedOnly, oidOnly, candidate + "\n"]) {
    fs.writeFileSync(rsaFile, partial);
    assert.throws(() => assertForgePackage(packageRoot), /unreviewed or modified/);
    if (partial !== candidate + "\n")
      assert.throws(
        () => verifyForgeSecurity(loadFresh(packageRoot)),
        /Missing expected exception/,
      );
  }
  fs.writeFileSync(rsaFile, candidate);
  assertForgePackage(packageRoot);
});
