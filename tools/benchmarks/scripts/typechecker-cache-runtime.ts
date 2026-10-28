/** Task-owned runtime namespaces; no system home variable is changed. */
import assert from "node:assert/strict";
import { createHash, randomBytes } from "node:crypto";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, resolve } from "node:path";

export function createColdScaleRuntime(
  out: string,
  base = process.env.VIZE_BENCH_RUNTIME_CACHE_BASE,
) {
  let owned: string | null = null;
  let ownedPhysical: string | null = null;
  let marker: string | null = null;
  let markerBytes: string | null = null;
  let cacheBasePhysical: string | null = null;
  if (base) {
    assert.ok(isAbsolute(base), "runtime cache base must be absolute");
    mkdirSync(base, { recursive: true });
    cacheBasePhysical = realpathSync(base);
    owned = mkdtempSync(join(cacheBasePhysical, "invocation-"));
    ownedPhysical = realpathSync(owned);
    assert.deepEqual(readdirSync(owned), [], "new runtime namespace must start empty");
    marker = join(owned, ".cold-scale-owner.json");
    markerBytes = JSON.stringify({
      owner: "vize-cold-scale",
      nonce: randomBytes(16).toString("hex"),
    });
    writeFileSync(marker, markerBytes, { flag: "wx", mode: 0o600 });
  }
  const env = {
    XDG_CACHE_HOME: owned ? join(owned, "xdg-cache") : join(out, "xdg-cache"),
    TMPDIR: owned ? join(owned, "tmp") : join(out, "tmp"),
  };
  mkdirSync(env.TMPDIR, { recursive: true });
  const evidence = {
    mode: owned
      ? "fresh task UUID; native placement audited separately"
      : "invocation paths; native namespace unverified",
    ownedRoot: ownedPhysical,
    createdEmpty: owned !== null,
    markerSha256: markerBytes ? createHash("sha256").update(markerBytes).digest("hex") : null,
    requestedCachePath: env.XDG_CACHE_HOME,
    requestedTemporaryPath: env.TMPDIR,
    context: {
      platform: process.platform,
      arch: process.arch,
      cacheBasePhysical,
      outputParentPhysical: realpathSync(dirname(resolve(out))),
    },
  };
  return {
    env,
    evidence,
    cleanup() {
      if (!owned || !marker || !markerBytes) return;
      assert.ok(
        lstatSync(owned).isDirectory(),
        "refuse cleanup after runtime directory replacement",
      );
      assert.equal(realpathSync(owned), ownedPhysical, "runtime physical identity changed");
      assert.equal(
        readFileSync(marker, "utf8"),
        markerBytes,
        "refuse cleanup after ownership marker replacement",
      );
      rmSync(owned, { recursive: true });
      assert.equal(existsSync(owned), false);
      writeFileSync(
        join(out, "runtime-cleanup.json"),
        JSON.stringify(
          {
            ownedRoot: ownedPhysical,
            markerSha256: evidence.markerSha256,
            removed: true,
          },
          null,
          2,
        ) + "\n",
      );
    },
  };
}

export function qualifyColdScaleCache(options: {
  receiptPath?: string;
  provider: string;
  runtime: ReturnType<typeof createColdScaleRuntime>["evidence"];
  executableSha256: string;
  backendSha256: string | null;
  corpusSha256: string;
  fixtureSha256: string;
}) {
  if (!options.receiptPath)
    return { qualified: false, reason: "untimed backend observer not supplied" };
  const receipt = JSON.parse(readFileSync(options.receiptPath, "utf8"));
  const a = receipt.runtime?.context,
    b = options.runtime.context;
  const sameContext =
    a?.platform === b.platform &&
    a?.arch === b.arch &&
    a?.cacheBasePhysical === b.cacheBasePhysical &&
    a?.outputParentPhysical === b.outputParentPhysical;
  const qualified =
    options.runtime.createdEmpty &&
    receipt.schema === "vize.cold-cache-custody" &&
    receipt.version === 1 &&
    receipt.qualified === true &&
    sameContext &&
    receipt.providerRootPhysical === realpathSync(options.provider) &&
    receipt.executableSha256 === options.executableSha256 &&
    receipt.backendSha256 === options.backendSha256 &&
    receipt.corpusSha256 === options.corpusSha256 &&
    receipt.fixtureIdentity?.sha256 === options.fixtureSha256;
  return {
    qualified,
    reason: qualified
      ? "matching source/backend/corpus/context observer and fresh owned UUID"
      : "observer identity or cache boundary did not qualify",
    receipt: options.receiptPath,
  };
}
