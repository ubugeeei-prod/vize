import assert from "node:assert/strict";
import {
  appendFile,
  lstat,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "./vrt_fixtures.ts";
import { verifiedVrtFiles, vrtDiagnosticRecord, type VrtPhase } from "./vrt_secrets.ts";
import { verifiedRetainedScan } from "./vrt_retained_scan.ts";

/** Only post-exit, whole-verified buffers enter the original upload tree. */
export async function promoteVrt(log: string, retainedRoot: string): Promise<Buffer> {
  const privateRoot = await realpath(path.dirname(log));
  const retained = await realpath(retainedRoot);
  assert.ok(privateRoot !== retained && !privateRoot.startsWith(retained + path.sep));
  assert.ok(!retained.startsWith(privateRoot + path.sep));
  const verified = await verifiedVrtFiles(log, retained);
  const { files } = verified;
  const topNames = [...new Set(files.map((file) => file.path.split(path.sep)[0]))];
  for (const name of [...topNames, "vrt-promotion.json"])
    await assert.rejects(lstat(path.join(retained, name)), { code: "ENOENT" });
  const stage = await mkdtemp(path.join(path.dirname(privateRoot), "vize-vrt-promote-"));
  const committed: string[] = [];
  try {
    const mapping = [];
    for (const file of files) {
      assert.ok(!path.isAbsolute(file.path) && !file.path.split(path.sep).includes(".."));
      const bytes = await readFile(path.join(privateRoot, file.path));
      assert.equal(sha256(bytes), file.sha256, "Verified VRT body changed before promotion");
      assert.equal(String(bytes.length), file.size);
      await mkdir(path.dirname(path.join(stage, file.path)), { recursive: true });
      await writeFile(path.join(stage, file.path), bytes, { flag: "wx" });
      mapping.push({
        privateRelative: file.path,
        retainedRelative: file.path,
        sha256: file.sha256,
        bytes: file.size,
      });
    }
    assert.deepEqual((await verifiedVrtFiles(log, retained)).files, files);
    await writeFile(
      path.join(stage, "vrt-promotion.json"),
      JSON.stringify(
        {
          schema: "vize-public-vrt-promotion-v1",
          privateRoot,
          retainedRoot: retained,
          absolutePathsInOriginalEvidencePreserved: true,
          files: mapping,
        },
        null,
        2,
      ) + "\n",
      { flag: "wx" },
    );
    for (const name of [...topNames, "vrt-promotion.json"]) {
      await rename(path.join(stage, name), path.join(retained, name));
      committed.push(name);
    }
    return verified.log;
  } catch {
    const cleanup = await Promise.allSettled(
      committed.map((name) => rm(path.join(retained, name), { recursive: true, force: true })),
    );
    assert.ok(
      cleanup.every((result) => result.status === "fulfilled"),
      "Verified VRT promotion rollback failed",
    );
    throw new Error("Verified VRT promotion failed");
  } finally {
    await rm(stage, { recursive: true, force: true });
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [log, retained] = process.argv.slice(2);
  let privateRoot: string | undefined;
  let retainedSafe = false;
  let phase: VrtPhase = "promotion-inputs";
  const report = (disposition: "entered" | "passed" | "failed") =>
    console.error(vrtDiagnosticRecord(phase, disposition).trimEnd());
  try {
    report("entered");
    assert.ok(log && retained && path.isAbsolute(log) && path.isAbsolute(retained));
    const root = path.dirname(log);
    const info = await lstat(root);
    assert.ok(info.isDirectory() && !info.isSymbolicLink() && (info.mode & 0o777) === 0o700);
    assert.ok(/^vize-public-vrt\.[A-Za-z0-9]+$/.test(path.basename(root)));
    assert.equal(await realpath(root), root);
    assert.ok(root !== retained && !root.startsWith((await realpath(retained)) + path.sep));
    privateRoot = root;
    phase = "promotion-whole-files";
    report("entered");
    const bytes = await promoteVrt(log, retained);
    report("passed");
    retainedSafe = true;
    process.stdout.write(bytes);
  } catch {
    report("failed");
    if (privateRoot && retained) {
      phase = "promotion-retained-fallback";
      report("entered");
      try {
        await verifiedRetainedScan(privateRoot, retained);
        report("passed");
        retainedSafe = true;
      } catch {
        report("failed");
        console.error("Original retained output upload withheld: closed scan was not verified");
      }
    }
    if (retained) {
      try {
        await writeFile(
          path.join(retained, "vrt-refused.json"),
          '{"schema":"vize-public-vrt-retention-refused-v1","success":false}\n',
          { flag: "wx" },
        );
      } catch {
        console.error("VRT fixed refusal evidence could not be retained");
      }
    }
    console.error("VRT observation log withheld: closed evidence was not verified or promoted");
    process.exitCode = 1;
  } finally {
    if (retainedSafe && process.env.GITHUB_OUTPUT) {
      try {
        const info = await lstat(process.env.GITHUB_OUTPUT);
        assert.ok(info.isFile() && !info.isSymbolicLink());
        await appendFile(process.env.GITHUB_OUTPUT, "upload-safe=true\n");
      } catch {
        console.error("VRT upload authorization output could not be retained");
        process.exitCode ||= 1;
      }
    }
    if (privateRoot) {
      try {
        await rm(privateRoot, { recursive: true, force: true });
      } catch {
        phase = "promotion-cleanup";
        report("failed");
        console.error("VRT private staging cleanup failed");
        process.exitCode ||= 1;
      }
    }
  }
}
