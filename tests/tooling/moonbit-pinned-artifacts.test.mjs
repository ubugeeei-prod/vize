import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";
import { test } from "node:test";
import {
  adaptInstaller,
  coreSha256,
  pinnedVersion,
  selectArchive,
  verifyBytes,
} from "../../.github/actions/setup-moonbit/pinned-artifacts.mjs";

const action = new URL("../../.github/actions/setup-moonbit/", import.meta.url);
const fixtures = new URL("../_fixtures/tooling/moonbit-pinned-transport/", import.meta.url);
const unix = fs.readFileSync(new URL("unix.sh.txt", fixtures), "utf8");
const windows = fs.readFileSync(new URL("windows.ps1.txt", fixtures), "utf8");
const digest = (body) => createHash("sha256").update(body).digest("hex");

void test("fixed artifacts retain all existing Unix and core Nix pins, exact installer bodies and Windows checksum", () => {
  const nix = fs.readFileSync(new URL("../../tools/nix/moonbit.nix", import.meta.url), "utf8");
  for (const [platform, arch] of [
    ["linux", "x64"],
    ["linux", "arm64"],
    ["darwin", "arm64"],
  ]) {
    const archive = selectArchive(pinnedVersion, platform, arch);
    assert.ok(nix.includes(`sha256-${Buffer.from(archive.sha256, "hex").toString("base64")}`));
    assert.ok(archive.url.includes("v0.10.7%2Bbc794d341"));
  }
  assert.ok(nix.includes(`sha256-${Buffer.from(coreSha256, "hex").toString("base64")}`));
  verifyBytes(
    fs.readFileSync(new URL("assets/core-0.10.7+bc794d341.tar.gz", action)),
    coreSha256,
    1449345,
  );
  assert.equal(digest(unix), "46495f8cdc0050f79b6cb195d66478d101cb3601d68506568fbe377fcdf2a9fe");
  assert.equal(digest(windows), "a5101e91ffa9905fb25cd009b9a4aa942971a294bd055c89836e3af89b710c64");
  const checksum = fs.readFileSync(new URL("windows.zip.sha256.txt", fixtures), "utf8");
  assert.equal(selectArchive(pinnedVersion, "win32", "x64").sha256, checksum.split(/\s/)[0]);
  assert.deepEqual(
    selectArchive(pinnedVersion, "win32", "arm64"),
    selectArchive(pinnedVersion, "win32", "x64"),
  );
  for (const version of ["latest", "nightly", "0.10.7", "0.10.7+unknown", ""]) {
    assert.throws(() => selectArchive(version, "linux", "x64"), /Unreviewed/);
  }
  for (const [platform, arch] of [
    ["darwin", "x64"],
    ["win32", "ia32"],
    ["linux", "riscv64"],
  ]) {
    assert.throws(() => selectArchive(pinnedVersion, platform, arch), /Unsupported/);
  }
});

void test("transport changes exactly two Unix expressions and the two complete Windows download/extract sites", () => {
  const archive = path.resolve(os.tmpdir(), "moon 日本 ' archive.tar.gz");
  const core = path.resolve(os.tmpdir(), "core 日本 ' .tar.gz");
  const adapted = adaptInstaller(unix, "linux", archive, core);
  const reversed = adapted
    .replace(
      `moonbit_uri='${pathToFileURL(archive).href.replaceAll("'", "%27")}'`,
      'moonbit_uri="$CLI_MOONBIT/binaries/$version/moonbit-$target.tar.gz"',
    )
    .replace(
      `core_uri='${pathToFileURL(core).href.replaceAll("'", "%27")}'`,
      'core_uri="$CLI_MOONBIT/cores/core-$version.tar.gz"',
    );
  assert.equal(reversed, unix);
  const win = adaptInstaller(
    windows,
    "win32",
    "C:\\runner ' 日本\\compiler.zip",
    "C:\\runner ' 日本\\core.tar.gz",
  );
  assert.ok(win.includes("Copy-Item -LiteralPath 'C:\\runner '' 日本\\compiler.zip'"));
  assert.ok(win.includes("tar -xf 'C:\\runner '' 日本\\core.tar.gz' --directory $MoonLib"));
  assert.ok(
    win.includes(
      'if ($LASTEXITCODE -ne 0) { throw "Failed to extract authenticated MoonBit core" }',
    ),
  );
  const expected = windows
    .replace(
      '  Invoke-WebRequest -Uri $MoonbitUri -OutFile "${HOME}\\moonbit.zip"',
      "  Copy-Item -LiteralPath 'C:\\runner '' 日本\\compiler.zip' -Destination \"${HOME}\\moonbit.zip\"",
    )
    .replace(
      "  Invoke-WebRequest -Uri $CoreUri -OutFile $MoonHome\\core.zip\n  Expand-Archive $MoonHome\\core.zip -DestinationPath $MoonLib -Force\n  Remove-Item -Force $MoonHome\\core.zip",
      "  New-Item -Path $MoonLib -ItemType Directory -Force | Out-Null\n  tar -xf 'C:\\runner '' 日本\\core.tar.gz' --directory $MoonLib\n  if ($LASTEXITCODE -ne 0) { throw \"Failed to extract authenticated MoonBit core\" }",
    );
  assert.equal(win, expected);
  for (const changed of [unix.replace("moonbit_uri=", "changed="), unix + unix]) {
    assert.throws(() => adaptInstaller(changed, "linux", archive, core), /boundary changed/);
  }
  assert.throws(
    () =>
      adaptInstaller(
        windows.replace("Expand-Archive $MoonHome\\core.zip", "changed"),
        "win32",
        archive,
        core,
      ),
    /boundary changed/,
  );
});

void test("real staging refuses network/hash/size/core/script/version failures before modifying the installer", async () => {
  const compiler = Buffer.from("controlled complete compiler transport\n");
  const selected = selectArchive(pinnedVersion, process.platform, process.arch);
  const production = fs.readFileSync(new URL("pinned-artifacts.mjs", action), "utf8");
  const before = new RegExp(`${selected.bytes},\\s*"${selected.sha256}"`, "gu");
  assert.equal([...production.matchAll(before)].length, 1);
  // The only test substitution admits a tiny compiler payload in a private
  // copy. All real staging, core/script authentication and refusal code runs.
  const controlled = production.replace(before, `${compiler.length}, "${digest(compiler)}"`);
  const originalFetch = globalThis.fetch;
  const scenarios = [
    "complete",
    "http403",
    "tamper",
    "truncated",
    "oversize",
    "badcore",
    "missingcore",
    "badscript",
    "unknown",
    "dev",
  ];
  try {
    for (const scenario of scenarios) {
      const directory = fs.mkdtempSync(path.join(os.tmpdir(), "moon transport 日本 ' "));
      try {
        fs.mkdirSync(path.join(directory, "assets"));
        const coreFile = path.join(directory, "assets", "core-0.10.7+bc794d341.tar.gz");
        if (scenario !== "missingcore")
          fs.copyFileSync(new URL("assets/core-0.10.7+bc794d341.tar.gz", action), coreFile);
        if (scenario === "badcore") fs.appendFileSync(coreFile, "changed");
        const helper = path.join(directory, "pinned-artifacts.mjs");
        fs.writeFileSync(helper, controlled);
        const installer = path.join(directory, "installer.sh");
        const body = process.platform === "win32" ? windows : unix;
        fs.writeFileSync(installer, scenario === "badscript" ? body + "changed\n" : body);
        const initial = fs.readFileSync(installer);
        const requests = [];
        globalThis.fetch = async (url) => {
          requests.push(url);
          let payload = compiler;
          if (scenario === "tamper") payload = Buffer.alloc(compiler.length, 1);
          if (scenario === "truncated") payload = compiler.subarray(1);
          if (scenario === "oversize") payload = Buffer.concat([compiler, Buffer.from("x")]);
          return new Response(scenario === "http403" ? "AccessDenied" : payload, {
            status: scenario === "http403" ? 403 : 200,
          });
        };
        const { preparePinnedInstaller } = await import(pathToFileURL(helper).href);
        const previousDev = process.env.MOONBIT_INSTALL_DEV;
        if (scenario === "dev") process.env.MOONBIT_INSTALL_DEV = "1";
        try {
          const promise = preparePinnedInstaller(
            installer,
            scenario === "unknown" ? "latest" : pinnedVersion,
            directory,
          );
          if (scenario === "complete") {
            await promise;
            assert.equal(requests.length, 1);
            assert.equal(requests[0], selected.url);
            assert.notDeepEqual(fs.readFileSync(installer), initial);
          } else {
            await assert.rejects(promise);
            assert.deepEqual(fs.readFileSync(installer), initial);
            assert.equal(
              requests.length,
              ["badcore", "missingcore", "unknown", "dev"].includes(scenario) ? 0 : 1,
            );
          }
        } finally {
          if (previousDev === undefined) delete process.env.MOONBIT_INSTALL_DEV;
          else process.env.MOONBIT_INSTALL_DEV = previousDev;
        }
      } finally {
        fs.rmSync(directory, { recursive: true, force: true });
      }
    }
  } finally {
    globalThis.fetch = originalFetch;
  }
});
