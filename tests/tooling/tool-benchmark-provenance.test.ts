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
} from "../../bench/benchmark-provenance.mjs";

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

test("a missing tsgo makes the backend not ready with an explicit reason", () => {
  withTempDir((dir) => {
    const absent = path.join(dir, "absent-tsgo");
    assert.deepEqual(resolveBackend([absent]), {
      engine: "tsgo-native",
      corsaPath: null,
      corsaVersion: null,
      ready: false,
      reason: `no tsgo binary at: ${absent}`,
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
      reason: `tsgo at ${tsgo} failed --version`,
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

test("a ready artifact renders every version and names the measured backend", () => {
  assert.deepEqual(
    renderProvenanceLines({
      versions: READY_VERSIONS,
      binaries: READY_BINARIES,
      backend: {
        engine: "tsgo-native",
        corsaPath: "/repo/node_modules/.bin/tsgo",
        corsaVersion: "7.0.0-dev.20260602.1",
        ready: true,
        reason: null,
      },
    }),
    [
      "Versions: vize `vize 0.303.0` · tsgo `7.0.0-dev.20260602.1` · vue-tsc `3.2.0` (typescript `5.9.0`) · verter-tsc `verter-tsc 0.0.1-beta.3` · Golar `golar 0.1.10` · vue `3.6.0` · eslint `9.0.0` · prettier `3.4.0` · node `v24.0.0`",
      `Binaries (sha256): vize \`${READY_BINARIES.vize}\` tsgo \`${READY_BINARIES.tsgo}\` vueTsc \`${READY_BINARIES.vueTsc}\` verterTsc \`${READY_BINARIES.verterTsc}\` golar \`${READY_BINARIES.golar}\` eslint n/a prettier n/a`,
      "Backend: native TypeScript engine ready at `/repo/node_modules/.bin/tsgo`. Planted-diagnostic gating for the type-check rows lives in bench/check-gate.mjs (.github/workflows/check-bench.yml).",
    ],
  );
});

test("an unready backend is stated outright and forbids a published timing", () => {
  assert.deepEqual(
    renderProvenanceLines({
      versions: { ...READY_VERSIONS, tsgo: null, vueTsc: null, typescript: null },
      binaries: { ...READY_BINARIES, tsgo: null, vueTsc: null },
      backend: {
        engine: "tsgo-native",
        corsaPath: null,
        corsaVersion: null,
        ready: false,
        reason: "no tsgo binary at: /repo/node_modules/.bin/tsgo",
      },
    }),
    [
      "Versions: vize `vize 0.303.0` · tsgo n/a · vue-tsc n/a (typescript n/a) · verter-tsc `verter-tsc 0.0.1-beta.3` · Golar `golar 0.1.10` · vue `3.6.0` · eslint `9.0.0` · prettier `3.4.0` · node `v24.0.0`",
      `Binaries (sha256): vize \`${READY_BINARIES.vize}\` tsgo n/a vueTsc n/a verterTsc \`${READY_BINARIES.verterTsc}\` golar \`${READY_BINARIES.golar}\` eslint n/a prettier n/a`,
      "Backend: native TypeScript engine NOT ready (no tsgo binary at: /repo/node_modules/.bin/tsgo); no type-check timing may be published from this artifact.",
    ],
  );
});

test("an artifact without provenance says so instead of rendering a blank line", () => {
  assert.deepEqual(renderProvenanceLines({ versions: null, backend: null }), [
    UNRECORDED_PROVENANCE_LINE,
  ]);
  assert.deepEqual(renderProvenanceLines({}), [UNRECORDED_PROVENANCE_LINE]);
});
