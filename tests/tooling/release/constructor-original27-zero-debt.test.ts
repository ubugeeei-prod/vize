import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { writeTarGz } from "../../../tools/support/archive/tar.ts";
import { asArray } from "../../../tools/support/json-value.ts";
import {
  afterReference,
  assertOriginalVectors,
  expectedMapper,
  reference,
} from "../../../tools/support/release/constructor_original27_public/oracle.ts";
import {
  afterRelease,
  requiredMerge,
  sdkFiles,
  afterSdkFiles,
} from "../../../tools/support/release/constructor_original27_public/authority.ts";
import { shippedImages } from "../../../tools/support/release/constructor_original27_public/native-prerequisite.ts";

import { Recorder } from "../../../tools/support/config/windows_consumer/common.ts";
import {
  capture,
  completed,
} from "../../../tools/support/release/constructor_original27_public/capture.ts";

import { originalChecker } from "../../../tools/support/release/constructor_original27_public/install.ts";

const version = "0.650.0";
test("independent pre-native intent removes exactly two whole synthetic records and preserves both authored orders", () => {
  const current = afterReference(),
    historical = reference();
  assert.equal(current.candidate, "9f0324a09c86b5bf0aeed8b4ac10d711fbfa92b4");
  assert.deepEqual(current.cases, historical.caseIds);
  assert.deepEqual(current.authoredCases, historical.authoredCases);
  const rows = structuredClone(asArray(historical.mapper));
  const removed = asArray(current.removedExpectedSyntheticRows);
  assert.equal(removed.length, 2);
  for (const row of removed) {
    const matches = rows.flatMap((value, index) =>
      JSON.stringify(value) === JSON.stringify(row) ? [index] : [],
    );
    assert.equal(matches.length, 1);
    rows.splice(matches[0], 1);
  }
  assert.deepEqual(current.expectedMapper, rows);
  assert.deepEqual(current.expectedCli, historical.cli);
  assert.equal(rows.length, 14);
  assert.equal(asArray(current.cases).length - asArray(current.authoredCases).length, 13);
});
test("every complete authored negative remains required and either historical generated row is refused", () => {
  const current = afterReference(),
    mapper = expectedMapper(version),
    cli = asArray(current.expectedCli);
  assertOriginalVectors(version, mapper, cli);
  for (let index = 0; index < 14; index++) {
    assert.throws(() =>
      assertOriginalVectors(
        version,
        mapper.filter((_row, i) => i !== index),
        cli,
      ),
    );
    assert.throws(() =>
      assertOriginalVectors(
        version,
        mapper,
        cli.filter((_row, i) => i !== index),
      ),
    );
  }
  for (const debt of asArray(current.removedExpectedSyntheticRows))
    assert.throws(() => assertOriginalVectors(version, [...mapper, debt], cli));
  const changed = structuredClone(mapper);
  changed[0].continuation = ["  unknown continuation"];
  assert.throws(() => assertOriginalVectors(version, changed, cli));
});
test("AFTER role binds containing merge, producer source and run while the complete SDK closure has ten members", () => {
  assert.equal(requiredMerge, "1d1bb26a7f7dcbcb9adf75075bd7ad301c0bb957");
  assert.deepEqual(afterRelease, {
    version,
    run: "41507479394",
    source: "85ab7600097035afa060ca03f8a77108cdc5e337",
    candidate: "3ec8f999df04a50b37aea7f12a5fa85161d43d8d",
  });
  assert.equal(sdkFiles.length, 9);
  assert(!sdkFiles.some((file: string) => file === "virtual-ts-slots.d.ts"));
  assert.equal(new Set(afterSdkFiles).size, 10);
  assert(afterSdkFiles.includes("virtual-ts-slots.d.ts"));
});
test("actual shipped-byte authentication rejects changed payload, installed mutation and symlink before any loader runs", () => {
  const root = mkdtempSync(join(tmpdir(), "original27-shipped-"));
  try {
    const body = Buffer.from("inert public fixture; never executed\n");
    const payload = writeTarGz([
      { name: "package/package.json", data: Buffer.from("{}\n") },
      { name: "package/index.js", data: body },
    ]);
    const integrity = "sha512-" + createHash("sha512").update(payload).digest("base64");
    writeFileSync(join(root, "package.json"), "{}\n");
    writeFileSync(join(root, "index.js"), body);
    assert.equal(shippedImages(root, payload, integrity).images.length, 2);
    const changed = Buffer.from(payload);
    changed[changed.length - 1] ^= 1;
    assert.throws(() => shippedImages(root, changed, integrity));
    writeFileSync(join(root, "index.js"), "mutated");
    assert.throws(() => shippedImages(root, payload, integrity));
    rmSync(join(root, "index.js"));
    symlinkSync(join(root, "package.json"), join(root, "index.js"));
    assert.throws(() => shippedImages(root, payload, integrity));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("actual capture strips credential names while preserving safe journal values and never persists the canary", () => {
  const root = mkdtempSync(join(tmpdir(), "original27-credential-"));
  const canary = "inert-original27-credential-canary-650";
  const env = {
    PATH: process.env.PATH,
    GITHUB_TOKEN: canary,
    NPM_CONFIG__AUTH: canary,
    example_secret: canary,
    VIZE_PUBLIC_NATIVE_CUSTODY: "inert-journal-retained",
  };
  try {
    const code =
      "process.stdout.write(JSON.stringify({ token: !!process.env.GITHUB_TOKEN, auth: !!process.env.NPM_CONFIG__AUTH, secret: !!process.env.example_secret, journal: process.env.VIZE_PUBLIC_NATIVE_CUSTODY }))";
    const row = capture(new Recorder(root), "canary", [process.execPath, "-e", code], root, env);
    completed(row, 0);
    assert.deepEqual(JSON.parse(row.stdout.toString("utf8")), {
      token: false,
      auth: false,
      secret: false,
      journal: "inert-journal-retained",
    });
    assert.equal(env.GITHUB_TOKEN, canary);
    for (const name of readdirSync(root)) assert(!readFileSync(join(root, name)).includes(canary));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("explicit historical checker is selected without resolver execution and refuses altered declarations or image", () => {
  const root = mkdtempSync(join(tmpdir(), "original27-checker-"));
  const target = "@typescript/typescript-linux-x64";
  const imagePath = join(root, "node_modules", target, "lib/tsc");
  const aliasPath = join(root, "node_modules/typescript-native/package.json");
  const body = Buffer.from("inert historical checker bytes; never executed");
  const digest = (data: Buffer) => createHash("sha256").update(data).digest("hex");
  const alias = Buffer.from(
    JSON.stringify({
      name: "typescript",
      version: "7.1.0-beta",
      optionalDependencies: { [target]: "7.1.0-beta" },
    }),
  );
  try {
    mkdirSync(join(imagePath, ".."), { recursive: true });
    mkdirSync(join(aliasPath, ".."), { recursive: true });
    writeFileSync(aliasPath, alias);
    writeFileSync(
      join(root, "node_modules", target, "package.json"),
      JSON.stringify({ name: target, version: "7.1.0-beta" }),
    );
    writeFileSync(imagePath, body);
    const historical = {
      identities: [
        {
          file: "node_modules/typescript-native/package.json",
          bytes: alias.length,
          sha256: digest(alias),
        },
      ],
      checker: {
        path: "/historical/node_modules/" + target + "/lib/tsc",
        bytes: body.length,
        sha256: digest(body),
      },
      stock: { bytes: 1, sha256: "unused" },
      vueTsc: { bytes: 1, sha256: "unused" },
    };
    const proof = originalChecker(root, historical);
    assert.equal(proof.image.path, realpathSync(imagePath));
    assert.equal(proof.resolverExecuted, false);
    assert.equal(proof.defaultResolverSelectionObserved, false);
    assert.throws(() =>
      originalChecker(root, {
        ...historical,
        checker: { ...historical.checker, path: "/unknown" },
      }),
    );
    writeFileSync(imagePath, "changed");
    assert.throws(() => originalChecker(root, historical));
    writeFileSync(imagePath, body);
    writeFileSync(aliasPath, Buffer.concat([alias, Buffer.from(" ")]));
    assert.throws(() => originalChecker(root, historical));
    const wrong = Buffer.from(
      JSON.stringify({
        name: "typescript",
        version: "7.1.0-beta",
        optionalDependencies: { [target]: "wrong" },
      }),
    );
    writeFileSync(aliasPath, wrong);
    assert.throws(() =>
      originalChecker(root, {
        ...historical,
        identities: [{ ...historical.identities[0], bytes: wrong.length, sha256: digest(wrong) }],
      }),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
