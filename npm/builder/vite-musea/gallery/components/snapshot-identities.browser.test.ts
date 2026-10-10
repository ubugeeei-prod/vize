import assert from "node:assert/strict";
import { mkdir, readFile, writeFile, rm, readdir, copyFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { PNG } from "pngjs";
import type { ArtFileInfo } from "../../src/types/index.ts";
import { buildHostedGallery } from "./hosted-tests.browser-fixtures.ts";
import { runCli } from "./hosted-vrt.browser-process.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output = path.join(repository, "artifacts/musea-snapshot-identities");

async function buildMachine(name: string) {
  const root = path.join(output, name);
  const arts: ArtFileInfo[] = [];
  for (const side of ["left", "right"]) {
    const source = await readFile(
      path.join(
        repository,
        "tests/tooling/fixtures/musea/snapshot-collision",
        side,
        "Button.art.vue",
      ),
      "utf8",
    );
    const artPath = path.join(root, side, "Button.art.vue");
    await mkdir(path.dirname(artPath), { recursive: true });
    await writeFile(artPath, source);
    const title = source.match(/title="([^"]+)"/)![1];
    const template = source.match(/<variant[^>]*>([\s\S]*?)<\/variant>/)![1];
    arts.push({
      path: artPath,
      metadata: { title, tags: [], status: "ready" },
      variants: [{ name: "Default", template, isDefault: true, skipVrt: false }],
      hasScript: false,
      hasScriptSetup: false,
      styleCount: 0,
    });
  }
  const host = await buildHostedGallery(root, 0, arts);
  const manifest = path.join(host.dist, "__musea__/api/static.json");
  const payload = JSON.parse(await readFile(manifest, "utf8"));
  assert.equal(payload.snapshotIdentityVersion, 1);
  assert.equal(payload.snapshotIdentities[arts[0].path], "left/Button.art.vue");
  assert.equal(payload.snapshotIdentities[arts[1].path], "right/Button.art.vue");
  // The hosted capture cannot inspect these original build-machine sources.
  await Promise.all([...arts, host.art].map((art) => rm(art.path)));
  return { ...host, arts, manifest, payload };
}

await test(
  "same-basename hosted Arts retain independent baselines across build-machine relocation",
  { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1" },
  async () => {
    await rm(output, { recursive: true, force: true });
    await mkdir(output, { recursive: true });
    const machineA = await buildMachine("machine-a");
    const observations: unknown[] = [];
    const args = [
      "--gallery-url",
      machineA.url,
      "--config",
      "vite.config.mjs",
      "--output",
      "reports",
      "--json",
      "--ci",
    ];
    async function capture(stage: string, expectedStatus: number, captureArgs = args) {
      const result = await runCli(output, captureArgs);
      assert.equal(result.status, expectedStatus, `${result.stdout}\n${result.stderr}`);
      const report = JSON.parse(
        await readFile(path.join(output, "reports/vrt-report.json"), "utf8"),
      );
      await copyFile(
        path.join(output, "reports/vrt-report.json"),
        path.join(output, `${stage}.json`),
      );
      observations.push({ stage, ...result, summary: report.summary });
      assert.equal(report.summary.errors, 0);
      return report;
    }
    const optionsUrl = pathToFileURL(
      fileURLToPath(new URL("../../src/plugin/options.ts", import.meta.url)),
    ).href;
    await writeFile(
      path.join(output, "vite.config.mjs"),
      `import { attachMuseaOptions } from ${JSON.stringify(optionsUrl)}; export default { plugins: [attachMuseaOptions({name:"vite-plugin-musea"},{vrt:{viewports:[{width:320,height:200,name:"desktop"},{width:375,height:240,name:"mobile"}],workers:4,capture:{waitForNetwork:false,settleTime:0}}})] };`,
    );
    let machineB: Awaited<ReturnType<typeof buildMachine>> | undefined;
    try {
      const first = await capture("baseline", 0);
      assert.equal(first.summary.new, 16);
      const left = first.results.filter(
        (result: { artPath: string }) => result.artPath === machineA.arts[0].path,
      );
      const right = first.results.filter(
        (result: { artPath: string }) => result.artPath === machineA.arts[1].path,
      );
      assert.equal(left.length, 2);
      assert.equal(right.length, 2);
      assert.ok(
        left.every((result: { snapshotPath: string }) =>
          /^snapshot-[a-f0-9]{64}\.png$/.test(path.basename(result.snapshotPath)),
        ),
      );
      assert.ok(
        first.results
          .filter((result: { artPath: string }) => result.artPath === machineA.art.path)
          .every((result: { snapshotPath: string }) =>
            path.basename(result.snapshotPath).startsWith("Host--"),
          ),
      );
      assert.notEqual(left[0].snapshotPath, right[0].snapshotPath);
      const leftPng = PNG.sync.read(await readFile(path.resolve(output, left[0].snapshotPath)));
      const rightPng = PNG.sync.read(await readFile(path.resolve(output, right[0].snapshotPath)));
      assert.notDeepEqual(
        leftPng.data,
        rightPng.data,
        "blue and red Art pixels cannot share a baseline",
      );
      const repeated = await capture("repeat", 0);
      assert.equal(repeated.summary.passed, 16);
      const indexPath = path.join(output, "reports/snapshots/identities.json");
      const initialIndex = await readFile(indexPath, "utf8");
      machineB = await buildMachine("machine-b");
      const relocatedArgs = args.map((arg) => (arg === machineA.url ? machineB!.url : arg));
      const relocated = await capture("relocated", 0, relocatedArgs);
      assert.equal(relocated.summary.passed, 16);
      assert.equal(relocated.summary.new, 0);
      assert.equal(await readFile(indexPath, "utf8"), initialIndex);
      assert.deepEqual(
        relocated.results.map((result: { snapshotPath: string }) => result.snapshotPath).sort(),
        first.results.map((result: { snapshotPath: string }) => result.snapshotPath).sort(),
      );
      const rightPreview = machineB.payload.previews[machineB.arts[1].path].Default;
      const rightHtml = path.join(machineB.dist, rightPreview.replace(/^\/site\//, ""));
      await writeFile(
        rightHtml,
        (await readFile(rightHtml, "utf8")).replace(
          "</head>",
          "<style>.musea-variant div{background:#00ff00!important}</style></head>",
        ),
      );
      const changed = await capture("changed", 1, relocatedArgs);
      assert.equal(changed.summary.failed, 2);
      assert.ok(
        changed.results
          .filter((result: { status: string }) => result.status === "failed")
          .every(
            (result: { artPath: string; diffPercentage: number }) =>
              result.artPath === machineB!.arts[1].path && result.diffPercentage > 0,
          ),
      );
      const ambiguous = await runCli(output, [
        "approve",
        "Button/*",
        ...relocatedArgs.filter((arg) => arg !== "--ci"),
      ]);
      assert.equal(ambiguous.status, 1, `${ambiguous.stdout}\n${ambiguous.stderr}`);
      assert.match(ambiguous.stderr, /Ambiguous approval pattern/);
      const approvedFirst = await runCli(output, [
        "approve",
        "right/Button/*",
        ...relocatedArgs.filter((arg) => arg !== "--ci"),
      ]);
      assert.equal(approvedFirst.status, 0, `${approvedFirst.stdout}\n${approvedFirst.stderr}`);
      // Introduce independent left changes; an ambiguous basename pattern must refuse both.
      const leftPreview = machineB.payload.previews[machineB.arts[0].path].Default;
      const leftHtml = path.join(machineB.dist, leftPreview.replace(/^\/site\//, ""));
      await writeFile(
        leftHtml,
        (await readFile(leftHtml, "utf8")).replace(
          "</head>",
          "<style>.musea-variant div{background:#ff00ff!important}</style></head>",
        ),
      );
      await writeFile(rightHtml, (await readFile(rightHtml, "utf8")).replace("#00ff00", "#ffff00"));
      const refused = await runCli(output, [
        "approve",
        "Button/*",
        ...relocatedArgs.filter((arg) => arg !== "--ci"),
      ]);
      assert.equal(refused.status, 1, `${refused.stdout}\n${refused.stderr}`);
      assert.match(refused.stderr, /Ambiguous approval pattern/);
      const approved = await runCli(output, [
        "approve",
        "right/Button/*",
        ...relocatedArgs.filter((arg) => arg !== "--ci"),
      ]);
      assert.equal(approved.status, 0, `${approved.stdout}\n${approved.stderr}`);
      assert.match(approved.stdout, /Approved 2 snapshot/);
      const partial = await capture("approved-right", 1, relocatedArgs);
      assert.equal(partial.summary.failed, 2);
      assert.ok(
        partial.results
          .filter((result: { status: string }) => result.status === "failed")
          .every((result: { artPath: string }) => result.artPath === machineB!.arts[0].path),
      );
      // Removing a colliding Art preserves the survivor's hash and reserved ownership.
      machineB.payload.arts = machineB.payload.arts.filter(
        (art: { path: string }) => art.path !== machineB!.arts[0].path,
      );
      await writeFile(machineB.manifest, JSON.stringify(machineB.payload));
      const subset = await capture("subset", 0, relocatedArgs);
      assert.equal(subset.summary.total, 14);
      assert.equal(subset.summary.passed, 14);
      const cleaned = await runCli(output, [
        "clean",
        ...relocatedArgs.filter((arg) => arg !== "--ci"),
      ]);
      assert.equal(cleaned.status, 0, `${cleaned.stdout}\n${cleaned.stderr}`);
      assert.match(cleaned.stdout, /Cleaned 2 orphaned/);
      assert.equal(
        await readFile(indexPath, "utf8"),
        initialIndex,
        "clean keeps removed ownership reserved",
      );
      const names = await readdir(path.join(output, "reports/snapshots"));
      assert.equal(names.filter((name) => name.endsWith(".png")).length, 14);
      assert.ok(!names.includes("identities.lock"), "each CLI released its lock");
      observations.push({ stage: "approval", ambiguous, refused, approved, cleaned });
    } finally {
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify(
          { observations, machineA: machineA.requests, machineB: machineB?.requests },
          null,
          2,
        ),
      );
      await machineA.close();
      await machineB?.close();
    }
  },
);
