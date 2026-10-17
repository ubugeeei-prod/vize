import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import path from "node:path";
import { derivePublicationPlan } from "../public_acceptance/plan.ts";
import { rejectOverrides } from "../public_acceptance/installed.ts";
import { verifyPublication } from "../public_acceptance/registry.ts";
import { publicInputs } from "../public_acceptance/inputs.ts";
import {
  admission,
  api,
  collectOriginal,
  files,
  verifiedRecovery,
  verifyIntegration,
  verifyConsumerDirectory,
  type Original,
} from "./custody.ts";
import { assertOracleFiles } from "./oracles.ts";
import { revalidatePublicArchives, type InstalledArchive } from "./archive.ts";
import {
  assertFresh,
  consumerAttempt,
  identity,
  object,
  positive,
  repository,
  roles,
  same,
  sha256,
  type Json,
} from "./policy.ts";

rejectOverrides();
same(process.env.GITHUB_REPOSITORY, repository, "own repository required");
same(process.env.GITHUB_REF, "refs/heads/main", "reviewed main only");
const policySha = process.env.GITHUB_SHA ?? "";
assert.match(policySha, /^[0-9a-f]{40}$/);
const [operation, destination] = process.argv.slice(2);
const root = realpathSync(process.cwd());
const save = (directory: string, name: string, value: unknown) =>
  writeFileSync(path.join(directory, name), JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
const load = (file: string) => object(JSON.parse(readFileSync(file, "utf8")));

if (operation === "prepare" || operation === "finalize") {
  const expected = identity(process.env);
  assert.ok(destination, "fresh evidence directory required");
  const output = path.resolve(destination);
  assert.ok(!output.startsWith(root + path.sep), "evidence must be outside checkout");
  if (operation === "prepare") {
    mkdirSync(output, { recursive: false });
    save(output, "request.json", {
      identity: expected,
      policySha,
      requestedAt: new Date().toISOString(),
      run: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
    });
    const original = await collectOriginal(
      expected,
      policySha,
      path.join(output, "original-log-capture"),
    );
    save(output, "original.json", original);
    writeFileSync(path.join(output, "original-failure.log"), original.log, { flag: "wx" });
    writeFileSync(path.join(output, "consumer.ts"), String(original.blobs.consumerSha256), {
      flag: "wx",
    });
    writeFileSync(
      path.join(output, "consumer-deno.json"),
      String(original.blobs.denoConfigSha256),
      { flag: "wx" },
    );
    const plan = derivePublicationPlan(root, expected.head);
    const publication = await verifyPublication(plan, expected, admission(original));
    save(output, "publication-before.json", publication);
    save(output, "integration.json", verifyIntegration(publication.github, expected));
    console.log("Known frozen consumer fixture admitted; original Release remains failed.");
  } else {
    const original = load(path.join(output, "original.json")) as Original;
    same(original.identity, expected, "original input changed");
    same(original.policy.sha, policySha, "policy changed during requalification");
    const consumers: Json[] = [];
    for (const role of roles) {
      const file = path.join(output, "consumers", role, "consumer.json");
      const packet = verifyConsumerDirectory(path.dirname(file), original);
      assertOracleFiles(path.dirname(file), original.policy, role === "deno-linux");
      save(
        output,
        `revalidated-${role}.json`,
        await revalidatePublicArchives(
          packet.packages as InstalledArchive[],
          object(object(packet.installed).native) as { path: string; sha256: string },
        ),
      );
      same(packet.role, role, "consumer role substitution");
      consumerAttempt(
        packet.run,
        String(positive(process.env.GITHUB_RUN_ID)),
        positive(process.env.GITHUB_RUN_ATTEMPT),
        policySha,
      );
      consumers.push({ role, sha256: sha256(readFileSync(file)) });
    }
    const live = await collectOriginal(expected, policySha, path.join(output, "final-log-capture"));
    for (const key of [
      "policy",
      "run",
      "jobs",
      "failed",
      "log",
      "logIdentity",
      "logCustody",
      "blobs",
      "artifacts",
      "pin",
      "sourceReceipt",
    ])
      same(original[key], live[key], `original ${key} changed during fresh consumers`);
    const publication = await verifyPublication(
      derivePublicationPlan(root, expected.head),
      expected,
      admission(live),
    );
    save(output, "publication-after.json", publication);
    same(
      verifyIntegration(publication.github, expected),
      load(path.join(output, "integration.json")),
      "original signed integration changed",
    );
    const receipt = {
      schema: "vize-consumer-recovery-v1",
      identity: expected,
      originalConclusion: "failure",
      status: "consumer-requalified",
      recovery: {
        run: String(positive(process.env.GITHUB_RUN_ID)),
        attempt: positive(process.env.GITHUB_RUN_ATTEMPT),
        policySha,
      },
      authority: original.policy.authority,
      originalDelivery: {
        authority:
          "Original annotated tag and actual signed integration merge; supplemental exact-main push Check. No historical merge-group claim.",
        evidenceFile: "integration.json",
      },
      consumers,
      files: files(output),
      completedAt: new Date().toISOString(),
      qualification:
        "Only the digest-bound frozen Deno consumer fixture was requalified. Original R/H/tag/artifacts remain unchanged; original R remains failed. All six original public consumer contracts ran freshly. External installed-product acceptance and source receipt closure are separate.",
    };
    assertFresh(receipt, expected);
    save(output, "receipt.json", receipt);
    console.log(
      "All six public consumers and exact original identities passed requalification; original Release remains failed.",
    );
  }
} else if (operation === "verify") {
  const source = publicInputs(process.env);
  assert.ok(source.recoveryRun, "explicit signed consumer recovery run required");
  const run = api(`actions/runs/${source.run}`);
  const expected = {
    cut: source.cut,
    head: source.head,
    tag: source.tag,
    sourcePr: source.sourcePr,
    run: source.run,
    attempt: positive(run.run_attempt),
  };
  const proof = await verifiedRecovery(expected, source.recoveryRun);
  const publication = await verifyPublication(derivePublicationPlan(root, source.head), source);
  const integration = verifyIntegration(publication.github, expected);
  assert.ok(destination, "verification evidence output required");
  mkdirSync(destination, { recursive: false });
  save(destination, "verified.json", {
    schema: "vize-verified-consumer-recovery-v1",
    verifiedAt: new Date().toISOString(),
    source,
    receipt: proof.receipt,
    packetDirectory: proof.directory,
    publication,
    integration,
  });
  // This operation verifies only; it cannot write a source PR, release, tag or R.
  console.log(
    `Original R=${source.run} remains FAILED; signed recovery=${source.recoveryRun} authenticated. Complete installed public acceptance before closing source #${source.sourcePr}.`,
  );
} else {
  throw new Error("Usage: main.ts prepare|finalize|verify <fresh evidence directory>");
}
