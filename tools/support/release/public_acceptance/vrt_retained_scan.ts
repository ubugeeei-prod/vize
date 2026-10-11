import assert from "node:assert/strict";
import { lstat, mkdir, readFile, readdir, realpath, rename, writeFile } from "node:fs/promises";
import path from "node:path";
import { inventory } from "./vrt_artifacts.ts";

const name = "vrt-retained-safe.json";
const schema = "vize-public-vrt-retained-safe-v1";

/** Called only after owned writers close, while the private credential buffers still exist. */
export async function scanRetained(
  privateRoot: string,
  retainedRoot: string,
  unsafe: (bytes: string | Uint8Array) => boolean,
): Promise<boolean> {
  const retained = await realpath(retainedRoot);
  assert.equal(privateRoot, await realpath(privateRoot));
  assert.ok(privateRoot !== retained && !privateRoot.startsWith(retained + path.sep));
  assert.ok(!retained.startsWith(privateRoot + path.sep));
  const moves: string[] = [];
  async function inspect(current: string): Promise<void> {
    for (const entry of await readdir(current, { withFileTypes: true })) {
      const file = path.join(current, entry.name);
      const info = await lstat(file);
      assert.ok(!info.isSymbolicLink() && (info.isFile() || info.isDirectory()));
      if (unsafe(entry.name)) moves.push(file);
      else if (info.isDirectory()) await inspect(file);
      else if (unsafe(await readFile(file))) moves.push(file);
    }
  }
  await inspect(retained);
  if (moves.length) {
    const quarantine = path.join(privateRoot, "vrt-quarantined-retained");
    await mkdir(quarantine, { mode: 0o700 });
    for (const [index, file] of moves.entries())
      await rename(file, path.join(quarantine, String(index)));
  }
  const files = await inventory(retained);
  for (const item of files) {
    assert.equal(unsafe(item.path), false, "Unsafe retained filename remains");
    assert.equal(
      unsafe(await readFile(path.join(retained, item.path))),
      false,
      "Unsafe retained body remains",
    );
  }
  assert.deepEqual(await inventory(retained), files);
  const body =
    JSON.stringify(
      {
        schema,
        privateRoot,
        retained,
        ownedWritersClosed: true,
        allRetainedPathsAndBodiesScanned: true,
        quarantinedEntries: moves.length,
        files,
      },
      null,
      2,
    ) + "\n";
  assert.equal(unsafe(body), false);
  await writeFile(path.join(privateRoot, name), body, { flag: "wx", mode: 0o600 });
  return moves.length > 0;
}

/** A boolean or missing receipt cannot authorize upload; verify the entire closed retained tree. */
export async function verifiedRetainedScan(
  privateRoot: string,
  retainedRoot: string,
): Promise<void> {
  assert.equal(privateRoot, await realpath(privateRoot));
  const file = path.join(privateRoot, name);
  const info = await lstat(file);
  assert.ok(info.isFile() && !info.isSymbolicLink() && (info.mode & 0o777) === 0o600);
  const receipt = JSON.parse(await readFile(file, "utf8")) as {
    schema: string;
    privateRoot: string;
    retained: string;
    ownedWritersClosed: boolean;
    allRetainedPathsAndBodiesScanned: boolean;
    quarantinedEntries: number;
    files: Awaited<ReturnType<typeof inventory>>;
  };
  assert.equal(receipt.schema, schema);
  assert.equal(receipt.privateRoot, privateRoot);
  assert.equal(receipt.retained, await realpath(retainedRoot));
  assert.equal(receipt.ownedWritersClosed, true);
  assert.equal(receipt.allRetainedPathsAndBodiesScanned, true);
  assert.ok(Number.isSafeInteger(receipt.quarantinedEntries) && receipt.quarantinedEntries >= 0);
  assert.deepEqual(
    await inventory(retainedRoot),
    receipt.files,
    "Closed native/browser retained output changed before upload authorization",
  );
}
