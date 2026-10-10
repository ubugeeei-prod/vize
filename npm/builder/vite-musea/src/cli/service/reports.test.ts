import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { writeSessionReport } from "./reports.ts";

void test("concurrent filesystem readers observe only whole old or new owned reports", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-session-report-"));
  try {
    for (const extension of ["json", "html"]) {
      const file = path.join(root, `owned.${extension}`);
      const bodies = ["A", "B"].map((value) =>
        extension === "json"
          ? JSON.stringify({
              reportOwner: { version: 1, artIdentity: "src/Art.art.vue" },
              value: value.repeat(1048576),
            })
          : `<html><body>${value.repeat(1048576)}</body></html>`,
      );
      await writeFile(file, bodies[0]);
      let reads = 0;
      for (let repeat = 0; repeat < 8; repeat++) {
        let pending = true;
        const writing = writeSessionReport(file, bodies[repeat % 2]).finally(() => {
          pending = false;
        });
        try {
          do {
            const actual = await readFile(file, "utf8");
            assert.ok(bodies.includes(actual), `Reader saw a truncated ${extension} report`);
            if (extension === "json") assert.equal(JSON.parse(actual).reportOwner.version, 1);
            reads++;
          } while (pending);
        } finally {
          await writing;
        }
      }
      assert.ok(reads >= 8);
      assert.equal(await readFile(file, "utf8"), bodies[1]);
    }
    assert.deepEqual((await readdir(root)).sort(), ["owned.html", "owned.json"]);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});

void test("a real report rename failure preserves the existing target and removes its temporary", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-session-report-refusal-"));
  const target = path.join(root, "existing.json");
  try {
    await mkdir(target);
    await writeFile(path.join(target, "retained.txt"), "original directory bytes");
    await assert.rejects(writeSessionReport(target, '{"complete":true}'));
    assert.equal(
      await readFile(path.join(target, "retained.txt"), "utf8"),
      "original directory bytes",
    );
    assert.deepEqual(await readdir(root), ["existing.json"]);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
