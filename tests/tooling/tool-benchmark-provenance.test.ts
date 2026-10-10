import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  probeVersion,
  renderProvenanceLines,
  resolveBackend,
  resolveFirstExisting,
  UNRECORDED_PROVENANCE_LINE,
} from "../../tools/benchmarks/scripts/benchmark-provenance.mjs";

function withTempDir<T>(run: (dir: string) => T): T {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-provenance-"));
  try {
    return run(dir);
  } finally {
    fs.rmSync(dir, { force: true, recursive: true });
  }
}

function writeFakeBinary(dir: string, name: string, body: string): string {
  const file = path.join(dir, name);
  fs.writeFileSync(file, body);
  fs.chmodSync(file, 0o755);
  return file;
}

const READY_VERSIONS = {
  vize: "vize 0.303.0",
  tsgo: "7.0.0-dev.20260602.1",
  vueTsc: "3.2.0",
  verterTsc: "verter-tsc 0.0.1-beta.3",
  golar: "golar 0.1.10",
  typescript: "5.9.0",
  vue: "3.6.0",
  eslint: "9.0.0",
  prettier: "3.4.0",
  node: "v24.0.0",
};

test("probeVersion reports the first --version line, or null", () => {
  withTempDir((dir) => {
    const good = writeFakeBinary(dir, "good.sh", '#!/bin/sh\necho "tool 1.2.3"\necho "extra"\n');
    const failing = writeFakeBinary(dir, "bad.sh", "#!/bin/sh\nexit 3\n");
    const silent = writeFakeBinary(dir, "silent.sh", "#!/bin/sh\nexit 0\n");

    assert.equal(probeVersion(good), "tool 1.2.3");
    assert.equal(probeVersion(failing), null);
    assert.equal(probeVersion(silent), null);
    assert.equal(probeVersion(path.join(dir, "missing")), null);
    assert.equal(probeVersion(null), null);
  });
});

test("resolveFirstExisting skips empty candidates and missing paths", () => {
  withTempDir((dir) => {
    const present = writeFakeBinary(dir, "present.sh", "#!/bin/sh\nexit 0\n");
    assert.equal(resolveFirstExisting([undefined, "", path.join(dir, "absent"), present]), present);
    assert.equal(resolveFirstExisting([path.join(dir, "absent")]), null);
  });
});

test("a resolvable tsgo makes the backend ready and records its exact version", () => {
  withTempDir((dir) => {
    const tsgo = writeFakeBinary(dir, "tsgo", '#!/bin/sh\necho "7.0.0-dev.20260602.1"\n');
    assert.deepEqual(resolveBackend([tsgo]), {
      engine: "tsgo-native",
      corsaPath: tsgo,
      corsaVersion: "7.0.0-dev.20260602.1",
      ready: true,
      reason: null,
    });
  });
});

test("a missing TypeScript 7/Corsa runtime makes the backend not ready with an explicit reason", () => {
  withTempDir((dir) => {
    const absent = path.join(dir, "absent-tsgo");
    assert.deepEqual(resolveBackend([absent]), {
      engine: "tsgo-native",
      corsaPath: null,
      corsaVersion: null,
      ready: false,
      reason: `no TypeScript 7/Corsa runtime at: ${absent}`,
    });
  });
});

test("a tsgo that cannot answer --version makes the backend not ready", () => {
  withTempDir((dir) => {
    const tsgo = writeFakeBinary(dir, "tsgo", "#!/bin/sh\nexit 9\n");
    assert.deepEqual(resolveBackend([tsgo]), {
      engine: "tsgo-native",
      corsaPath: tsgo,
      corsaVersion: null,
      ready: false,
      reason: `TypeScript 7/Corsa runtime at ${tsgo} failed --version`,
    });
  });
});

const READY_BINARIES = {
  vize: "a".repeat(64),
  tsgo: "b".repeat(64),
  vueTsc: "c".repeat(64),
  verterTsc: "d".repeat(64),
  golar: "e".repeat(64),
  eslint: null,
  prettier: null,
};

test("provenance labels each complete checksum beside the correct tool and version", () => {
  const lines = renderProvenanceLines({
    versions: READY_VERSIONS,
    binaries: READY_BINARIES,
    backend: { corsaPath: "/repo/node_modules/.bin/tsgo", ready: true },
  });
  assert.equal(lines[0], '<details class="benchmark-provenance">');
  assert.ok(lines.includes("| Tool | Version | Binary SHA-256 |"));
  for (const [key, label] of [
    ["vize", "Vize"],
    ["tsgo", "tsgo"],
    ["vueTsc", "vue-tsc"],
    ["verterTsc", "verter-tsc"],
    ["golar", "Golar"],
    ["eslint", "ESLint"],
    ["prettier", "Prettier"],
  ] as const) {
    const hash = READY_BINARIES[key];
    assert.ok(
      lines.includes(
        `| ${label} | <code>${READY_VERSIONS[key]}</code> | ${hash == null ? "n/a" : `<code>${hash}</code>`} |`,
      ),
      `${label}: preserve the full version/checksum association`,
    );
  }
  assert.ok(lines.includes("| TypeScript (vue-tsc) | <code>5.9.0</code> | n/a |"));
  assert.ok(lines.includes("| Vue | <code>3.6.0</code> | n/a |"));
  assert.ok(lines.includes("| Node.js | <code>v24.0.0</code> | n/a |"));
  assert.ok(
    lines.some((line) => line.includes("ready at <code>/repo/node&#95;modules/.bin/tsgo</code>")),
  );
  assert.equal(lines.at(-1), "</details>");
});

test("an unready backend keeps the explicit refusal and missing values", () => {
  const text = renderProvenanceLines({
    versions: { ...READY_VERSIONS, tsgo: null, vueTsc: null, typescript: null },
    binaries: { ...READY_BINARIES, tsgo: null, vueTsc: null },
    backend: {
      ready: false,
      reason: "no TypeScript 7/Corsa runtime at: /repo/node_modules/.bin/tsgo",
    },
  }).join("\n");
  assert.ok(text.includes("| tsgo | n/a | n/a |"));
  assert.ok(text.includes("| vue-tsc | n/a | n/a |"));
  assert.ok(text.includes("| TypeScript (vue-tsc) | n/a | n/a |"));
  assert.ok(
    text.includes(
      "engine NOT ready (<code>no TypeScript 7/Corsa runtime at: /repo/node&#95;modules/.bin/tsgo</code>); no type-check timing may be published",
    ),
  );
});

test("provenance escapes recorded values without breaking rows or losing text", () => {
  const text = renderProvenanceLines({
    versions: { ...READY_VERSIONS, vize: "tool <&>|`dev`\nnext" },
    binaries: { ...READY_BINARIES, extra: "h".repeat(64) },
    backend: { ready: false, reason: "<missing & unsafe>" },
  }).join("\n");
  assert.ok(text.includes("<code>tool &lt;&amp;&gt;&#124;&#96;dev&#96;&#10;next</code>"));
  assert.ok(text.includes(`| <code>extra</code> | n/a | <code>${"h".repeat(64)}</code> |`));
  assert.ok(text.includes("<code>&lt;missing &amp; unsafe&gt;</code>"));
  assert.ok(!text.includes("<missing"));
});

test("an artifact without provenance says so instead of rendering a blank line", () => {
  assert.deepEqual(renderProvenanceLines({ versions: null, backend: null }), [
    UNRECORDED_PROVENANCE_LINE,
  ]);
  assert.deepEqual(renderProvenanceLines({}), [UNRECORDED_PROVENANCE_LINE]);
});
