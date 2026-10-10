import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { buildCaptureIdentity, type SnapshotIdentityJob } from "./snapshot-identity.ts";
import { SnapshotIndex, SNAPSHOT_INDEX_FILE } from "./snapshot-index.ts";

function job(
  artPath = "src/Button.art.vue",
  variantName = "Default",
  name = "small",
  width = 200,
): SnapshotIdentityJob {
  return { art: { path: artPath }, variantName, viewport: { name, width, height: 100 } };
}

async function directory(t: { after: (fn: () => Promise<unknown>) => unknown }): Promise<string> {
  const result = await fs.mkdtemp(path.join(os.tmpdir(), "musea-snapshot-index-"));
  t.after(() => fs.rm(result, { recursive: true, force: true }));
  return result;
}

void test("ordinary new baselines retain legacy names and persist their owner before capture", async (t) => {
  const dir = await directory(t);
  const index = await SnapshotIndex.open(dir);
  try {
    const capture = job();
    const plan = await index.plan([capture]);
    assert.equal(plan.get(buildCaptureIdentity(capture)), "Button--Default--small.png");
    const stored = JSON.parse(await fs.readFile(path.join(dir, SNAPSHOT_INDEX_FILE), "utf8"));
    assert.deepEqual(stored, {
      version: 1,
      owners: { "Button--Default--small.png": buildCaptureIdentity(capture) },
    });
    assert.equal((await fs.readdir(dir)).filter((name) => name.endsWith(".png")).length, 0);
  } finally {
    await index.close();
  }
});

void test("same-basename captures are order-independent, distinct and stable after removal", async (t) => {
  const left = job("left/Button.art.vue");
  const right = job("right/Button.art.vue");
  const plans: Map<string, string>[] = [];
  for (const jobs of [
    [left, right],
    [right, left],
  ]) {
    const dir = await directory(t);
    const index = await SnapshotIndex.open(dir);
    try {
      const plan = await index.plan(jobs);
      for (const name of plan.values()) assert.match(name, /^snapshot-[0-9a-f]{64}\.png$/);
      assert.equal(new Set(plan.values()).size, 2);
      await fs.writeFile(path.join(dir, plan.get(buildCaptureIdentity(left))!), "left pixels");
      await fs.writeFile(path.join(dir, plan.get(buildCaptureIdentity(right))!), "right pixels");
      assert.equal(
        (await index.plan([right])).get(buildCaptureIdentity(right)),
        plan.get(buildCaptureIdentity(right)),
      );
      plans.push(plan);
    } finally {
      await index.close();
    }
    const reopened = await SnapshotIndex.open(dir);
    try {
      assert.equal(
        (await reopened.plan([left])).get(buildCaptureIdentity(left)),
        plans.at(-1)!.get(buildCaptureIdentity(left)),
      );
    } finally {
      await reopened.close();
    }
  }
  assert.deepEqual(plans[0], plans[1]);
});

void test("adding another Art cannot transfer a reserved legacy name, even after its PNG is cleaned", async (t) => {
  const dir = await directory(t);
  const left = job("left/Button.art.vue");
  const right = job("right/Button.art.vue");
  const first = await SnapshotIndex.open(dir);
  const legacyName = (await first.plan([left])).get(buildCaptureIdentity(left))!;
  await fs.writeFile(path.join(dir, legacyName), "old left pixels");
  await fs.unlink(path.join(dir, legacyName));
  await first.close();
  const second = await SnapshotIndex.open(dir);
  try {
    const plan = await second.plan([right, left]);
    assert.equal(plan.get(buildCaptureIdentity(left)), legacyName);
    assert.match(plan.get(buildCaptureIdentity(right))!, /^snapshot-/);
    assert.equal(
      (await second.plan([right])).get(buildCaptureIdentity(right)),
      plan.get(buildCaptureIdentity(right)),
    );
  } finally {
    await second.close();
  }
});

void test("unowned ordinary PNG requires explicit adoption and then remains owned", async (t) => {
  const dir = await directory(t);
  const filename = "Button--Default--small.png";
  await fs.writeFile(path.join(dir, filename), "legacy pixels");
  const unapproved = await SnapshotIndex.open(dir);
  try {
    await assert.rejects(unapproved.plan([job()]), /Explicit adoptLegacySnapshots/);
    await assert.rejects(fs.access(path.join(dir, SNAPSHOT_INDEX_FILE)), { code: "ENOENT" });
    assert.equal(await fs.readFile(path.join(dir, filename), "utf8"), "legacy pixels");
  } finally {
    await unapproved.close();
  }
  const adopter = await SnapshotIndex.open(dir, { adoptLegacySnapshots: true });
  try {
    assert.equal((await adopter.plan([job()])).get(buildCaptureIdentity(job())), filename);
  } finally {
    await adopter.close();
  }
  const owned = await SnapshotIndex.open(dir);
  try {
    assert.equal((await owned.plan([job()])).get(buildCaptureIdentity(job())), filename);
  } finally {
    await owned.close();
  }
});

void test("ambiguous old PNG is never adopted and fresh qualified baselines preserve it", async (t) => {
  const dir = await directory(t);
  const filename = "Button--Default--small.png";
  const captures = [job("left/Button.art.vue"), job("right/Button.art.vue")];
  await fs.writeFile(path.join(dir, filename), "ambiguous pixels");
  const adopter = await SnapshotIndex.open(dir, { adoptLegacySnapshots: true });
  try {
    await assert.rejects(adopter.plan(captures), /Cannot adopt ambiguous/);
  } finally {
    await adopter.close();
  }
  const index = await SnapshotIndex.open(dir);
  try {
    const plan = await index.plan(captures);
    assert.ok([...plan.values()].every((name) => name.startsWith("snapshot-")));
    assert.equal(await fs.readFile(path.join(dir, filename), "utf8"), "ambiguous pixels");
    assert.equal(
      (await index.plan([captures[1]])).get(buildCaptureIdentity(captures[1])),
      plan.get(buildCaptureIdentity(captures[1])),
    );
  } finally {
    await index.close();
  }
});

void test("case, delimiter, empty names and viewport collisions get separate bounded filenames", async (t) => {
  const cases = [
    [job("Button.art.vue"), job("button.art.vue")],
    [job("A--B.art.vue", "C"), job("A.art.vue", "B--C")],
    [job("Button.art.vue", ""), job("Button.art.vue", "unnamed")],
    [
      job("Button.art.vue", "Default", "small", 200),
      job("Button.art.vue", "Default", "small", 400),
    ],
    [job("Button.art.vue"), { ...job(), viewport: { ...job().viewport, deviceScaleFactor: 2 } }],
  ];
  for (const captures of cases) {
    const index = await SnapshotIndex.open(await directory(t));
    try {
      const plan = await index.plan(captures);
      assert.equal(new Set(plan.values()).size, 2);
      assert.ok([...plan.values()].every((name) => /^snapshot-[0-9a-f]{64}\.png$/.test(name)));
    } finally {
      await index.close();
    }
  }
  const index = await SnapshotIndex.open(await directory(t));
  try {
    const long = job(`${"日".repeat(200)}.art.vue`, "長い名前");
    const unicode = job("日本語/ボタン.art.vue", "こんにちは");
    const plan = await index.plan([long, unicode]);
    assert.ok([...plan.values()].every((name) => Buffer.byteLength(name) <= 255));
  } finally {
    await index.close();
  }
});

void test("relocated build paths reuse identities and names through hosted maps", async (t) => {
  const dir = await directory(t);
  const firstArt = job("/machine-a/project/left/Button.art.vue");
  const secondArt = job("/machine-b/project/left/Button.art.vue");
  const firstMap = { [firstArt.art.path]: "left/Button.art.vue" };
  const secondMap = { [secondArt.art.path]: "left/Button.art.vue" };
  const first = await SnapshotIndex.open(dir, { snapshotIdentities: firstMap });
  const firstPlan = await first.plan([firstArt]);
  await first.close();
  const second = await SnapshotIndex.open(dir, {
    projectRoot: "/unrelated",
    snapshotIdentities: secondMap,
  });
  try {
    assert.deepEqual(await second.plan([secondArt]), firstPlan);
  } finally {
    await second.close();
  }
});

void test("Windows build-machine paths use the portable identity basename", async (t) => {
  const dir = await directory(t);
  const capture = job("C:\\build\\project\\src\\Button.art.vue");
  const identities = { [capture.art.path]: "src/Button.art.vue" };
  const index = await SnapshotIndex.open(dir, { snapshotIdentities: identities });
  identities[capture.art.path] = "elsewhere/Changed.art.vue";
  try {
    const identity = buildCaptureIdentity(capture, process.cwd(), {
      [capture.art.path]: "src/Button.art.vue",
    });
    assert.equal((await index.plan([capture])).get(identity), "Button--Default--small.png");
  } finally {
    await index.close();
  }
});

void test("case-insensitive adoption retains the existing filename's actual spelling", async (t) => {
  const dir = await directory(t);
  await fs.writeFile(path.join(dir, "BUTTON--Default--small.png"), "legacy pixels");
  const index = await SnapshotIndex.open(dir, { adoptLegacySnapshots: true });
  try {
    assert.equal(
      (await index.plan([job()])).get(buildCaptureIdentity(job())),
      "BUTTON--Default--small.png",
    );
  } finally {
    await index.close();
  }
  const reopened = await SnapshotIndex.open(dir);
  try {
    assert.equal(
      (await reopened.plan([job()])).get(buildCaptureIdentity(job())),
      "BUTTON--Default--small.png",
    );
  } finally {
    await reopened.close();
  }
});

void test("concurrent plan calls serialize reservations and retain every owner", async (t) => {
  const dir = await directory(t);
  const index = await SnapshotIndex.open(dir);
  try {
    const captures = [
      job("left/Button.art.vue"),
      job("right/Button.art.vue"),
      job("Badge.art.vue"),
    ];
    const plans = await Promise.all(captures.map((capture) => index.plan([capture])));
    assert.equal(new Set(plans.flatMap((plan) => [...plan.values()])).size, 3);
    const repeated = await Promise.all(captures.map((capture) => index.plan([capture])));
    assert.deepEqual(repeated, plans);
    const stored = JSON.parse(await fs.readFile(path.join(dir, SNAPSHOT_INDEX_FILE), "utf8"));
    assert.equal(Object.keys(stored.owners).length, 3);
    assert.ok(!(await fs.readdir(dir)).some((name) => name.endsWith(".tmp")));
  } finally {
    await index.close();
  }
});
