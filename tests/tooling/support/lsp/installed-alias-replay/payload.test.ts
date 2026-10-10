import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import type { VizePublicRegistryInstallAuthority } from "./authority-schema.ts";
import { digest } from "./custody.ts";
import { recheckPublicPayload } from "./payload.ts";

test("public payload recheck rejects patched loader JS, extra files and unowned links", () => {
  // Plain independent file controls; no package/provider is loaded or executed.
  const installRoot = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "registry-payload-control-")),
  );
  try {
    const source = { C: "1".repeat(40), H: "2".repeat(40), tag: "v0.1.0", R: "1" };
    const packageName = "@typescript/control-platform";
    const records = [
      {
        name: "vize",
        version: "0.1.0",
        resolved: "https://registry.npmjs.org/control.tgz",
        integrity: "sha512-control",
      },
    ];
    const corsa = {
      packageName,
      version: "7.0.2",
      resolved: "https://registry.npmjs.org/control-corsa.tgz",
      integrity: "sha512-control-corsa",
    };
    const files = [
      { name: "vize", file: "native-targets.js", text: "original routing bytes" },
      { name: packageName, file: "lib/tsc", text: "plain runtime control bytes" },
    ];
    const packages = files.map((control) => {
      const file = path.join(installRoot, "node_modules", control.name, control.file);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, control.text);
      const record = control.name === "vize" ? records[0] : { ...corsa, name: packageName };
      return {
        ...record,
        tarballSHA512: "0".repeat(128),
        allInstalledFileBytesEqualPublicArchive: true,
        source:
          control.name === "vize" ? { H: source.H, R: source.R } : { thirdPartyRegistry: true },
        files: [
          { path: file, sha256: digest(control.text), bytes: Buffer.byteLength(control.text) },
        ],
      };
    });
    const payloadManifestPath = path.join(installRoot, "payload.json");
    fs.writeFileSync(
      payloadManifestPath,
      JSON.stringify({
        schema: "vize-public-registry-payload-files-v1",
        installRoot,
        source,
        packages,
      }),
    );
    const authority = {
      installRoot,
      source,
      registry: records,
      bundledCorsa: corsa,
      payloadManifestPath,
      payloadManifestSha256: digest(fs.readFileSync(payloadManifestPath)),
    } as VizePublicRegistryInstallAuthority;
    assert.doesNotThrow(() => recheckPublicPayload(authority));
    const loader = packages[0].files[0].path;
    fs.writeFileSync(loader, "patched source loader bytes");
    assert.throws(() => recheckPublicPayload(authority));
    fs.writeFileSync(loader, files[0].text);
    const extra = path.join(path.dirname(loader), "unreviewed.js");
    fs.writeFileSync(extra, "extra routing bytes");
    assert.throws(() => recheckPublicPayload(authority), /whole installed public file set/);
    fs.unlinkSync(extra);
    fs.symlinkSync(process.execPath, extra);
    assert.throws(() => recheckPublicPayload(authority), /must not redirect/);
  } finally {
    fs.rmSync(installRoot, { recursive: true });
  }
});
