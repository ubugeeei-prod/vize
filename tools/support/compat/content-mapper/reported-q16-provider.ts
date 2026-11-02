/** Canonical published launcher/manifest role; no theorem about its hidden backend. */
import assert from "node:assert/strict";
import path from "node:path";
import { createHash } from "node:crypto";
const hash = (b: Buffer) => createHash("sha256").update(b).digest("hex");

export const q16Package = {
  private: true,
  type: "module",
  dependencies: {
    tsrs: "0.20.1-ts7.1.0-beta",
    typescript: "6.0.3",
    vue: "3.5.44",
    "vue-tsc": "3.3.11",
  },
};
/** One case-set selects one immutable report profile; no caller-supplied version override. */
export function q16Profile(caseSet: unknown) {
  assert(
    typeof caseSet === "string" &&
      ["all", "A", "A-with-declaration", "B", "C", "D", "11373"].includes(caseSet),
  );
  if (caseSet === "11373")
    return {
      id: "11373",
      package: {
        ...q16Package,
        dependencies: {
          ...q16Package.dependencies,
          typescript: "5.9.3",
          vue: "3.5.43",
          "vue-tsc": "3.2.6",
        },
      },
      reported: { vize: "0.640.0", node: "26.12.1", platform: "macOS", lock: "not supplied" },
    };
  return {
    id: "q16",
    package: q16Package,
    reported: { vize: "0.638.0", node: "26.13.0", platform: "Windows 11", lock: "not supplied" },
  };
}
export function verifyQ16ProviderNode(version: string, caseSet: unknown) {
  assert.equal(version, "v" + q16Profile(caseSet).reported.node);
}
export function declaredBin(
  manifest: { name: string; bin?: string | Record<string, string> },
  key: string,
) {
  const bin =
    typeof manifest.bin === "string" ? manifest.name === key && manifest.bin : manifest.bin?.[key];
  assert.equal(typeof bin, "string", `published package must declare bin.${key}`);
  assert(bin && !path.isAbsolute(bin) && !bin.split(/[\\/]/u).includes(".."));
  return bin;
}
export const q16PublishedLauncherPin = {
  bin: "./bin/tsrs",
  launcherBytes: 332,
  launcherSha256: "9d7f26b2c80d4e45576f6e783675c5f624b055a15f94cb4089b9e9cc529722f3",
  manifestBytes: 2612,
  manifestSha256: "d1808e75483abc0329211cd3fe29972718decae378a65ed4f1ce8785aa3320ea",
  integrity:
    "sha512-9BiTYTorhgJTEjlJmU0yy14wx2kJBbPIE5aSQjZ3G1cDKZFWRRThAASNlwt870EN1oiuiibxdjxxoIW3jBRDBg==",
};
export function verifyQ16PublishedLauncher(
  bytes: Buffer,
  manifest: Buffer,
  bin: string,
  integrity: string,
) {
  assert.equal(bin, q16PublishedLauncherPin.bin);
  assert.equal(integrity, q16PublishedLauncherPin.integrity);
  assert.equal(bytes.length, q16PublishedLauncherPin.launcherBytes);
  assert.equal(hash(bytes), q16PublishedLauncherPin.launcherSha256);
  assert.equal(manifest.length, q16PublishedLauncherPin.manifestBytes);
  assert.equal(hash(manifest), q16PublishedLauncherPin.manifestSha256);
  return {
    binRole: "source-closed standard published JavaScript launcher",
    backendIdentity: "unclaimed; launcher version PID is not a backend image/PID attestation",
  };
}
export function verifyQ16Lock(
  lock: {
    lockfileVersion: number;
    packages: Record<
      string,
      { dependencies?: unknown; link?: boolean; integrity?: string; resolved?: string }
    >;
  },
  expected = q16Package.dependencies,
) {
  assert.equal(lock.lockfileVersion, 3);
  assert.deepEqual(lock.packages[""].dependencies, expected);
  for (const [relative, p] of Object.entries(lock.packages))
    if (relative && !p.link) {
      assert.match(p.integrity || "", /^sha(?:256|384|512)-[A-Za-z0-9+/=]+$/u);
      assert(p.resolved?.startsWith("https://registry.npmjs.org/"));
    }
}
