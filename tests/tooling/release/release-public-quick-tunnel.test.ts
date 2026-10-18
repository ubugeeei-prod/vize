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
  tunnelSetupRefusal,
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

// Projection of retained artifact12086476844 from actual acceptance38994019210:
// publication.source excludes recoveryRun; signed recovery is under github.consumerRecovery.
const recoveredCustody = () => {
  const source = {
    cut: "014bd42cacdf3d026fb75162cfa80dce76d78315",
    head: "03882089eff75664abe99ba71868e33739496b01",
    tag: "v0.457.0",
    sourcePr: "9196",
    run: "38805261385",
    recoveryRun: "38993331275",
  };
  const { recoveryRun, ...original } = source;
  return {
    source,
    preparation: { source: { ...source }, version: "0.457.0" },
    publication: {
      schema: "vize-public-release-acceptance-v1",
      success: true,
      source: original,
      github: {
        target: source.head,
        runStatus: "completed",
        runConclusion: "failure",
        releaseId: 1,
        jobs: [
          {
            run: Number(source.run),
            head: source.head,
            attempt: 4,
            status: "completed",
            conclusion: "failure",
          },
        ],
        consumerRecovery: {
          schema: "vize-consumer-recovery-v1",
          identity: { ...original, attempt: 4 },
          recovery: {
            run: recoveryRun,
            attempt: 1,
            policySha: "d57cf724d2caea6cf47f814250d18801f6b75e15",
          },
          originalConclusion: "failure",
          status: "consumer-requalified",
          completedAt: new Date().toISOString(),
          consumers: [
            "node22-linux",
            "node24-linux",
            "node24-macos",
            "node24-windows",
            "deno-linux",
            "bun-linux",
          ].map((role) => ({ role })),
        },
      },
    },
  };
};

type RecoveryPacket = ReturnType<typeof recoveredCustody>;
const assertRecovered = (packet: RecoveryPacket) =>
  assertTunnelReleaseCustody(packet.source, packet.preparation, packet.publication);

void test("authenticated explicit recovery binds the failed original attempt without rewriting its conclusion", () => {
  const packet = recoveredCustody();
  assertRecovered(packet);
  assert.equal(packet.publication.github.runConclusion, "failure");
  assert.equal(Reflect.get(packet.publication.source, "recoveryRun"), undefined);
  for (const key of ["cut", "head", "tag", "sourcePr", "run", "attempt"]) {
    const drift = recoveredCustody();
    Reflect.set(drift.publication.github.consumerRecovery.identity, key, "changed");
    assert.throws(() => assertRecovered(drift), `original recovery identity: ${key}`);
  }
});

void test("absent, partial, stale, foreign and truthy recovery claims cannot admit a failed Release", () => {
  const changes: ((packet: RecoveryPacket) => void)[] = [
    (p) => {
      Reflect.deleteProperty(p.source, "recoveryRun");
      p.preparation.source = { ...p.source };
    },
    (p) => {
      p.preparation.source.recoveryRun = "1";
    },
    (p) => {
      p.source.recoveryRun = "1";
      p.preparation.source.recoveryRun = "1";
    },
    (p) => {
      p.source.recoveryRun = "true";
      p.preparation.source.recoveryRun = "true";
    },
    (p) => {
      Reflect.deleteProperty(p.publication.github, "consumerRecovery");
    },
    (p) => {
      Reflect.set(p.publication.github, "consumerRecovery", true);
    },
    (p) => {
      p.publication.github.runConclusion = "success";
    },
    (p) => {
      p.publication.github.consumerRecovery.schema = "changed";
    },
    (p) => {
      p.publication.github.consumerRecovery.status = "pending";
    },
    (p) => {
      p.publication.github.consumerRecovery.originalConclusion = "success";
    },
    (p) => {
      p.publication.github.consumerRecovery.completedAt = new Date(
        Date.now() - 25 * 3600_000,
      ).toISOString();
    },
    (p) => {
      p.publication.github.consumerRecovery.completedAt = new Date(
        Date.now() + 3600_000,
      ).toISOString();
    },
    (p) => {
      p.publication.github.consumerRecovery.recovery.run = "1";
    },
    (p) => {
      p.publication.github.consumerRecovery.recovery.attempt = 0;
    },
    (p) => {
      p.publication.github.consumerRecovery.recovery.policySha = "main";
    },
    (p) => {
      p.publication.github.consumerRecovery.identity.attempt = 0;
    },
    (p) => {
      p.publication.github.consumerRecovery.identity.attempt = 3;
    },
    (p) => {
      p.publication.github.jobs = [];
    },
    (p) => {
      p.publication.github.jobs.push({ ...p.publication.github.jobs[0] });
    },
    (p) => {
      p.publication.github.jobs[0].run = 1;
    },
    (p) => {
      p.publication.github.jobs[0].head = "a".repeat(40);
    },
    (p) => {
      p.publication.github.jobs[0].status = "in_progress";
    },
    (p) => {
      p.publication.github.jobs[0].attempt = 3;
    },
  ];
  for (const change of changes) {
    const packet = recoveredCustody();
    change(packet);
    assert.throws(() => assertRecovered(packet));
  }
  const unsolicited = custody();
  Reflect.set(
    unsolicited.publication.github,
    "consumerRecovery",
    recoveredCustody().publication.github.consumerRecovery,
  );
  assert.throws(
    () => assertTunnelReleaseCustody(source, unsolicited.preparation, unsolicited.publication),
    /explicit recovery run required/u,
  );
  for (let index = 0; index < 6; index++) {
    for (const replacement of ["missing", "duplicate", "foreign"]) {
      const packet = recoveredCustody();
      const consumers = packet.publication.github.consumerRecovery.consumers;
      if (replacement === "missing") consumers.splice(index, 1);
      else
        consumers[index].role =
          replacement === "duplicate" ? consumers[(index + 1) % 6].role : replacement;
      assert.throws(() => assertRecovered(packet), `missing/duplicate/foreign role ${index}`);
    }
  }
});

void test("setup refusal retains a useful safe cause while discarding raw values and HTTP errors", () => {
  let failure: unknown;
  try {
    assert.equal("credential-secret", "different", "exact authenticated recovery run required");
  } catch (error) {
    failure = error;
  }
  const refusal = tunnelSetupRefusal(failure, "authenticated release custody");
  assert.deepEqual(refusal.cause, {
    phase: "authenticated release custody",
    reason: "exact authenticated recovery run required",
  });
  assert.doesNotMatch(JSON.stringify(refusal.cause), /credential-secret|different/u);
  const remote = tunnelSetupRefusal(
    new TypeError("https://host/?token=credential-secret"),
    "official pinned metadata",
  );
  assert.deepEqual(remote.cause, { phase: "official pinned metadata", reason: "TypeError" });
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
    /node vrt_observer\.ts "\$PUBLIC_VERSION" "\$PUBLIC_OUTPUT\/vrt\.json" 3>&2 > "\$PUBLIC_OUTPUT\/vrt\.log" 2>&1/u,
  );
  assert.match(observerStep, /if \[ "\$vrt_status" -eq 0 \]; then exit 1; fi/u);
  assert.match(observerStep, /exit "\$vrt_status"/u);
});
