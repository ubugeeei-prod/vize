import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, lstat, realpath, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertQuickTunnelDownloadLocation,
  assertQuickTunnelMetadata,
  assertTunnelReleaseCustody,
  metadataObject,
} from "../../../tools/support/release/public_acceptance/vrt_tunnel_metadata.ts";
import { quickTunnelPin } from "../../../tools/support/release/public_acceptance/vrt_tunnel.ts";
import {
  createTunnelAssetDirectory,
  requireReleasedService,
} from "../../../tools/support/release/public_acceptance/vrt_tunnel_setup.ts";

const fixtureUrl = new URL("../fixtures/musea/quick-tunnel-pin.json", import.meta.url);
type PinValues = Parameters<typeof assertQuickTunnelMetadata>[0];
const fixture = async () => JSON.parse(await readFile(fixtureUrl, "utf8")) as PinValues;
const source = {
  cut: "a".repeat(40),
  head: "b".repeat(40),
  tag: "v0.444.0",
  sourcePr: "8510",
  run: "1",
};
const custody = () => ({
  preparation: { source: { ...source }, version: "0.444.0" },
  publication: {
    schema: "vize-public-release-acceptance-v1",
    success: true,
    source: { ...source },
    github: { target: source.head, runStatus: "completed", runConclusion: "success", releaseId: 1 },
  },
});

void test("official metadata projection binds the annotated tag, stable mutable release and whole asset pin", async () => {
  const values = await fixture();
  assert.deepEqual(assertQuickTunnelMetadata(values), {
    immutable: false,
    publishedAt: metadataObject(values.release).published_at,
  });
  assert.equal(metadataObject(values.asset).digest, `sha256:${quickTunnelPin.sha256}`);
  assert.equal(metadataObject(metadataObject(values.tagObject).object).sha, quickTunnelPin.source);
});

void test("every pinned asset field and duplicate release asset refuse metadata drift", async () => {
  const original = await fixture();
  for (const key of Object.keys(metadataObject(original.asset))) {
    const values = structuredClone(original);
    Reflect.set(metadataObject(values.asset), key, "changed");
    assert.throws(() => assertQuickTunnelMetadata(values), `direct asset drift: ${key}`);
    const member = structuredClone(original),
      release = metadataObject(member.release);
    assert.ok(Array.isArray(release.assets));
    Reflect.set(metadataObject(release.assets[0]), key, "changed");
    assert.throws(() => assertQuickTunnelMetadata(member), `release member drift: ${key}`);
  }
  const duplicate = structuredClone(original),
    release = metadataObject(duplicate.release);
  assert.ok(Array.isArray(release.assets));
  release.assets.push(structuredClone(release.assets[0]));
  assert.throws(() => assertQuickTunnelMetadata(duplicate), /one pinned named release asset/u);
});

void test("release publication and both annotated tag identities fail closed", async () => {
  const original = await fixture();
  for (const [key, value] of [
    ["id", 1],
    ["tag_name", "latest"],
    ["draft", true],
    ["prerelease", true],
    ["published_at", null],
    ["immutable", null],
    ["url", "https://example.com"],
    ["html_url", "https://example.com"],
  ] as const) {
    const values = structuredClone(original);
    Reflect.set(metadataObject(values.release), key, value);
    assert.throws(() => assertQuickTunnelMetadata(values), `release drift: ${key}`);
  }
  for (const [section, key, value] of [
    ["tag", "ref", "refs/tags/latest"],
    ["tag", "url", "https://example.com"],
    ["tagObject", "sha", "0".repeat(40)],
    ["tagObject", "tag", "latest"],
    ["tagObject", "url", "https://example.com"],
  ] as const) {
    const values = structuredClone(original);
    Reflect.set(metadataObject(values[section]), key, value);
    assert.throws(() => assertQuickTunnelMetadata(values), `tag drift: ${section}/${key}`);
  }
  for (const section of ["tag", "tagObject"] as const) {
    for (const key of ["sha", "type", "url"]) {
      const values = structuredClone(original);
      Reflect.set(metadataObject(metadataObject(values[section]).object), key, "changed");
      assert.throws(() => assertQuickTunnelMetadata(values), `tag target drift: ${section}/${key}`);
    }
  }
});

void test("asset redirects allow only pinned initial HTTPS and the official release-asset host", () => {
  assert.equal(assertQuickTunnelDownloadLocation(quickTunnelPin.url, true), quickTunnelPin.url);
  const official = "https://release-assets.githubusercontent.com/a/b?owned=signed";
  assert.equal(assertQuickTunnelDownloadLocation(official, false), official);
  for (const value of [
    "http://release-assets.githubusercontent.com/a",
    "https://user@release-assets.githubusercontent.com/a",
    "https://release-assets.githubusercontent.com:8443/a",
    "https://release-assets.githubusercontent.com/a#fragment",
    "https://release-assets.githubusercontent.com.attacker.example/a",
    "https://example.com/a",
    quickTunnelPin.url,
  ])
    assert.throws(() => assertQuickTunnelDownloadLocation(value, false));
  for (const value of [
    quickTunnelPin.url + "?different=1",
    official,
    quickTunnelPin.url.replace("2026.10.0", "latest"),
  ])
    assert.throws(() => assertQuickTunnelDownloadLocation(value, true));
});

void test("authenticated preparation must bind the same frozen official release identity", () => {
  const valid = custody();
  assertTunnelReleaseCustody(source, valid.preparation, valid.publication);
  for (const key of Object.keys(source)) {
    const drift = custody();
    Reflect.set(drift.publication.source, key, "changed");
    assert.throws(() => assertTunnelReleaseCustody(source, drift.preparation, drift.publication));
  }
  for (const [key, value] of [
    ["target", "changed"],
    ["runStatus", "in_progress"],
    ["runConclusion", "failure"],
    ["releaseId", 0],
  ] as const) {
    const drift = custody();
    Reflect.set(drift.publication.github, key, value);
    assert.throws(() => assertTunnelReleaseCustody(source, drift.preparation, drift.publication));
  }
  for (const key of ["schema", "success"] as const) {
    const drift = custody();
    Reflect.set(drift.publication, key, "changed");
    assert.throws(() => assertTunnelReleaseCustody(source, drift.preparation, drift.publication));
  }
  const wrongPreparation = custody();
  wrongPreparation.preparation.version = "0.443.0";
  assert.throws(() =>
    assertTunnelReleaseCustody(source, wrongPreparation.preparation, wrongPreparation.publication),
  );
  assert.throws(() => requireReleasedService("unused", "not-a-commit"));
});

void test("real private directory ownership stays outside upload scope and preserves HOME", async (t) => {
  const home = process.env.HOME,
    root = await mkdtemp(path.join(os.tmpdir(), "vize-tunnel-setup-law-"));
  t.after(() => rm(root, { recursive: true, force: true }));
  const temporary = await realpath(root),
    retained = path.join(temporary, "retained");
  await mkdir(retained);
  const directory = await createTunnelAssetDirectory(temporary, retained);
  assert.equal(await realpath(directory), directory);
  assert.equal((await lstat(directory)).mode & 0o777, 0o700);
  assert.ok(directory.startsWith(temporary + path.sep));
  assert.ok(!directory.startsWith(retained + path.sep));
  await assert.rejects(createTunnelAssetDirectory(retained, retained), /private temporary root/u);
  const inside = path.join(retained, "inside");
  await mkdir(inside);
  await assert.rejects(createTunnelAssetDirectory(inside, retained), /private temporary root/u);
  await assert.rejects(createTunnelAssetDirectory("relative", retained));
  assert.equal(process.env.HOME, home);
});

void test("workflow keeps original VRT eligibility and makes setup a separate precredential phase", async () => {
  const root = fileURLToPath(new URL("../../../", import.meta.url));
  const workflow = await readFile(
    path.join(root, ".github/workflows/release-public-acceptance.yml"),
    "utf8",
  );
  assert.match(workflow, /timeout-minutes: 30/u);
  const setup = workflow.indexOf(
    "      - name: Authenticate and acquire the pinned static-gallery tunnel\n",
  );
  const observer = workflow.indexOf(
    "      - name: Observe installed public Musea VRT and hosted audits\n",
  );
  assert.ok(setup > 0 && observer > setup);
  const condition = (section: string) =>
    section.slice(section.indexOf("        if: "), section.indexOf("        working-directory:"));
  const setupStep = workflow.slice(setup, observer),
    observerStep = workflow.slice(
      observer,
      workflow.indexOf("      - name: Seal installed", observer),
    );
  assert.equal(
    setupStep.slice(setupStep.indexOf("        if: "), setupStep.indexOf("        shell:")),
    condition(observerStep),
  );
  assert.match(
    setupStep,
    /run: node tools\/support\/release\/public_acceptance\/vrt_tunnel_setup\.ts/u,
  );
  assert.match(
    observerStep,
    /node vrt_observer\.ts "\$PUBLIC_VERSION" "\$PUBLIC_OUTPUT\/vrt\.json" > "\$PUBLIC_OUTPUT\/vrt\.log" 2>&1/u,
  );
  assert.match(observerStep, /exit "\$\{vrt_status:-0\}"/u);
});
