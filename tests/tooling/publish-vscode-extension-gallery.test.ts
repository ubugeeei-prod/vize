import assert from "node:assert/strict";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";

import {
  marketplaceQuery,
  marketplaceQueryAccept,
  marketplaceQueryUrl,
} from "../../tools/support/release/marketplace_query.ts";
import { galleryListsVersion } from "../../tools/support/release/publish_vscode_extension/main.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { galleryWith, recordedGallery } from "./support/marketplace-gallery.ts";
import { runReleaseCommand } from "./support/release-command.ts";

const platforms = [
  "win32-x64",
  "alpine-x64",
  "alpine-arm64",
  "linux-x64",
  "darwin-arm64",
  "darwin-x64",
  "win32-arm64",
  "linux-arm64",
];
const lists = (body: unknown, version: string, target: string) =>
  galleryListsVersion("ubugeeei.vize", JSON.stringify(body), version, target);

test("the recorded gallery lists 0.584.0 for every platform and universal", () => {
  for (const target of [...platforms, "universal"]) {
    assert.equal(lists(recordedGallery(), "0.584.0", target), true, target);
    assert.equal(lists(recordedGallery(), "0.585.0", target), false, target);
  }
});

test("a gallery row matches only its own target", () => {
  const rows = recordedGallery().results[0].extensions[0].versions;
  const withoutLinux = rows.filter(
    (row) => !(row.version === "0.584.0" && row.targetPlatform === "linux-x64"),
  );
  assert.equal(lists(galleryWith(withoutLinux), "0.584.0", "linux-x64"), false);
  assert.equal(lists(galleryWith(withoutLinux), "0.583.0", "linux-x64"), true);
  const universalOnly = rows.filter((row) => row.targetPlatform === undefined);
  assert.equal(lists(galleryWith(universalOnly), "0.584.0", "universal"), true);
  assert.equal(lists(galleryWith(universalOnly), "0.584.0", "linux-x64"), false);
  const platformsOnly = rows.filter((row) => row.targetPlatform !== undefined);
  assert.equal(lists(galleryWith(platformsOnly), "0.584.0", "universal"), false);
});

test("a gallery row must finish Marketplace validation before it is visible", () => {
  for (const target of ["linux-x64", "universal"]) {
    const row = {
      version: "0.584.0",
      ...(target === "universal" ? {} : { targetPlatform: target }),
    };
    for (const flags of [undefined, "", "none", "unvalidated", "prerelease"]) {
      assert.equal(lists(galleryWith([{ ...row, flags }]), "0.584.0", target), false);
    }
    assert.equal(lists(galleryWith([{ ...row, flags: "validated" }]), "0.584.0", target), true);
    assert.equal(
      lists(galleryWith([{ ...row, flags: "validated, prerelease" }]), "0.584.0", target),
      true,
    );
  }
});

test("a partial or foreign gallery inventory is not visible", () => {
  const paged = recordedGallery() as unknown as { results: { pagingToken: string }[] };
  paged.results[0].pagingToken = "next";
  assert.equal(lists(paged, "0.584.0", "linux-x64"), false);
  const foreign = JSON.parse(JSON.stringify(recordedGallery()).replace('"vize"', '"other"'));
  assert.equal(lists(foreign, "0.584.0", "linux-x64"), false);
  assert.equal(galleryListsVersion("ubugeeei.vize", "", "0.584.0", "linux-x64"), false);
  assert.equal(galleryListsVersion("ubugeeei.vize", "{", "0.584.0", "linux-x64"), false);
});

function rerun(t: TestContext, gallery: unknown) {
  const root = fs.mkdtempSync(path.join(tmpdir(), "vscode-gallery-")),
    bin = path.join(root, "bin"),
    calls = path.join(root, "calls.json"),
    manifest = path.join(root, "package.json");
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(bin);
  fs.writeFileSync(
    manifest,
    JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.584.0" }),
  );
  const record = (tool: string) =>
    [
      "const fs=require('node:fs');",
      "const calls=fs.existsSync(process.env.CALLS)?JSON.parse(fs.readFileSync(process.env.CALLS)):[];",
      `calls.push([${JSON.stringify(tool)},...process.argv.slice(2)]);`,
      "fs.writeFileSync(process.env.CALLS,JSON.stringify(calls));",
    ].join("\n");
  writeFakeCommand(
    bin,
    "curl",
    `${record("curl")}\nprocess.stdout.write(${JSON.stringify(JSON.stringify(gallery))});`,
  );
  // vsce would refuse a second upload of the same version and target.
  writeFakeCommand(bin, "vp", `${record("vp")}\nprocess.exit(1);`);
  const result = runReleaseCommand(
    "publish_vscode_extension",
    [path.join(root, "x.vsix"), manifest],
    {
      env: {
        PATH: [bin, path.dirname(process.execPath), process.env.PATH].join(path.delimiter),
        VSCE_DLX_BIN: "vp",
        CALLS: calls,
        VIZE_EXTENSION_TARGET: "linux-x64",
        PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
        PUBLISH_RESOLUTION_RETRY_DELAY: "0",
      },
    },
  );
  return { result, calls: JSON.parse(fs.readFileSync(calls, "utf8")) as string[][] };
}

test("a rerun skips a platform version the gallery already lists", (t) => {
  const { result, calls } = rerun(t, recordedGallery());
  assert.equal(result.status, 0, result.stderr);
  assert.equal(
    result.stdout,
    "ubugeeei.vize 0.584.0 is already published, skipping VS Code Marketplace publish.\n",
  );
  assert.deepEqual(calls, [
    [
      "curl",
      "--fail",
      "--silent",
      "--show-error",
      "--max-time",
      "30",
      "--header",
      `Accept: ${marketplaceQueryAccept}`,
      "--header",
      "Content-Type: application/json",
      "--header",
      "Cache-Control: no-cache",
      "--data",
      JSON.stringify(marketplaceQuery("ubugeeei.vize")),
      marketplaceQueryUrl,
    ],
  ]);
});

test("a rerun uploads a platform version the gallery lists only for other targets", (t) => {
  const rows = recordedGallery().results[0].extensions[0].versions;
  const { result, calls } = rerun(
    t,
    galleryWith(rows.filter((row) => row.targetPlatform !== "linux-x64")),
  );
  assert.equal(result.status, 1);
  assert.deepEqual(
    calls.map((call) => call[0] + (call[0] === "vp" ? ` ${call[5]}` : "")),
    ["curl", "vp publish", "curl"],
  );
});

test("an unvalidated upload cannot turn a failed publisher into success", (t) => {
  const rows = recordedGallery().results[0].extensions[0].versions;
  const { result, calls } = rerun(t, galleryWith(rows.map((row) => ({ ...row, flags: "none" }))));
  assert.equal(result.status, 1);
  assert.deepEqual(
    calls.map((call) => call[0] + (call[0] === "vp" ? ` ${call[5]}` : "")),
    ["curl", "vp publish", "curl"],
  );
});
