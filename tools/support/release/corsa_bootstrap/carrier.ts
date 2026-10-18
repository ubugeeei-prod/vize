import assert from "node:assert/strict";
import { readFileSync, mkdirSync, writeFileSync, existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { publishedCrates } from "../publish_crates/main.ts";
import { restorePublishManifest, stagePublishManifest } from "../publish_crates/manifest.ts";
import {
  candidate,
  assertPublisher,
  crates,
  hasPause,
  positive,
  record,
  schema,
  type Candidate,
  type Crate,
  type Published,
  type Receipt,
} from "./policy.ts";
import { authenticate, assertFullQualification, commands } from "./main.ts";
import {
  archive,
  boundedRequest,
  canonicalOwner,
  configs,
  ensurePublisher,
  exactVersion,
  intendedOwner,
  namespace,
  sha256,
  type Request,
} from "./registry.ts";

export type Steps = {
  owner: () => Promise<void>;
  inventory: (name: Crate) => Promise<"missing" | "existing">;
  adopt: (name: Crate) => Promise<void>;
  publish: (name: Crate) => Promise<void>;
  verify: (name: Crate) => Promise<Published>;
  consumer: () => Promise<Receipt["consumer"]>;
};

/** Serial side effects: no dependent publication after any failed barrier. */
export async function bootstrap(steps: Steps) {
  await steps.owner();
  const states = new Map<Crate, "missing" | "existing">();
  // Authenticate all existing partial namespaces BEFORE the first new write.
  for (const name of crates) {
    const state = await steps.inventory(name);
    states.set(name, state);
    if (state === "existing") await steps.adopt(name);
  }
  const published: Published[] = [];
  for (const name of crates) {
    if (states.get(name) === "missing") await steps.publish(name);
    published.push(await steps.verify(name));
  }
  return { published, consumer: await steps.consumer() };
}

export function assertConsumer(value: unknown, source: Candidate, published: Published[]) {
  const row = record(value);
  assert.equal(row.sourceSha, source.head);
  assert.equal(row.version, source.tag.slice(1));
  assert.equal(row.sdkOnlyRegistryAcceptance, true);
  assert.equal(
    row.publicRegistryAcceptance,
    false,
    "bootstrap must not claim full seven-crate acceptance",
  );
  assert.equal(row.sixDefaultAndDistributedCompiledAndTested, true);
  assert.equal(row.canonDefaultNativeCompiled, false);
  assert.equal(row.sourcePatches, false);
  assert.equal(row.sourceReplacement, false);
  assert.ok(Array.isArray(row.archives));
  assert.deepEqual(row.archives.map((entry) => record(entry).name).sort(), [...crates].sort());
  for (const entry of row.archives) {
    const archive = record(entry);
    const own = published.find((item) => item.name === archive.name);
    assert.equal(archive.version, own?.version);
    assert.equal(archive.checksum, own?.checksum);
    assert.equal(record(record(archive.vcs).git).sha1, source.head);
  }
}

async function carry(source: Candidate, output: string, deadline: number, request: Request) {
  const root = process.cwd();
  const io = commands(root, deadline);
  const environment = process.env;
  assert.equal(environment.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(environment.GITHUB_SHA, source.head);
  assert.equal(environment.GITHUB_REF, `refs/heads/${source.branch}`);
  assert.equal(environment.GITHUB_REPOSITORY, source.repository);
  assert.equal(environment.VIZE_CORSA_CONFIRM, `first-publication:${source.head}:${source.tag}`);
  const run = Number(environment.GITHUB_RUN_ID),
    attempt = Number(environment.GITHUB_RUN_ATTEMPT);
  const owner = Number(environment.VIZE_CORSA_OWNER_ID);
  const ownerLogin = environment.GITHUB_ACTOR ?? "";
  assert.equal(environment.GITHUB_TRIGGERING_ACTOR, ownerLogin);
  const repositoryOwner = Number(environment.GITHUB_REPOSITORY_OWNER_ID);
  const qualification = Number(environment.VIZE_CORSA_QUALIFICATION_RUN);
  for (const id of [run, attempt, owner, repositoryOwner, qualification])
    positive(id, "explicit identity");
  const token = environment.CARGO_REGISTRY_TOKEN ?? "";
  assert.ok(token, "protected short-lived six-name publish-new/trusted-publishing token required");
  const pr = authenticate(source, io, true);
  assert.equal(
    record(pr.user).login,
    ownerLogin,
    "source author and real bootstrap maintainer must agree",
  );
  assert.ok(hasPause(String(pr.body), source));
  const authorize = () => {
    authenticate(source, io, true);
    assertFullQualification(
      io.api(`repos/${source.repository}/actions/runs/${qualification}`),
      source,
      io,
    );
  };
  authorize();
  assert.equal(record(record(io.api(`repos/${source.repository}`)).owner).id, repositoryOwner);
  assert.deepEqual(
    publishedCrates.filter((name) => name.startsWith("vize_corsa")),
    crates,
    "H must carry exactly the reviewed six-name catalogue",
  );
  const consumer = "tools/support/compat/github/checked-corsa-public-consumer.ts";
  assert.ok(
    readFileSync(consumer, "utf8").includes('"sdk-only"'),
    "reviewed existing-consumer SDK mode required before any write",
  );
  assert.ok(!existsSync(output), "fresh custody directory required");
  mkdirSync(output, { recursive: true });
  const custody = (name: string, body: unknown) =>
    writeFileSync(path.join(output, name), `${JSON.stringify(body, null, 2)}\n`);
  const version = source.tag.slice(1);
  const packagePath = (name: Crate) =>
    path.join(
      environment.CARGO_TARGET_DIR ?? path.join(root, "target"),
      "package",
      `${name}-${version}.crate`,
    );
  const packageSource = (name: Crate) => {
    const archivePath = packagePath(name);
    if (!existsSync(archivePath)) {
      // Resuming a partial publication still reproduces the exact reviewed
      // publisher staging; a same-version registry hit is never a blind skip.
      const staged = stagePublishManifest(name);
      assert.ok(staged, "cannot stage reviewed package manifest");
      try {
        io.run("cargo", ["package", "--allow-dirty", "--no-verify", "-p", name], undefined, true);
      } finally {
        assert.ok(restorePublishManifest(staged), "manifest/lock restoration failed");
      }
    }
    const bytes = readFileSync(archivePath);
    const vcs = record(
      JSON.parse(
        io.run("tar", ["-xOf", archivePath, `${name}-${version}/.cargo_vcs_info.json`]).toString(),
      ),
    );
    assert.equal(record(vcs.git).sha1, source.head);
    assert.equal(vcs.path_in_vcs, `vendor/${name.slice(5)}`);
    return bytes;
  };
  const adopted = new Map<Crate, string>();
  let completed: Published[] = [];
  const result = await bootstrap({
    owner: async () => {
      await canonicalOwner(owner, ownerLogin, request);
      await intendedOwner(owner, token, request);
    },
    inventory: (name) => namespace(name, request),
    adopt: async (name) => {
      // The per-crate endpoint demands direct user ownership. Refuse any name
      // whose exact first version/source cannot be reproduced from this H.
      const existing = await configs({ crate: name }, token, request);
      assert.ok(existing.length <= 1, "unexpected partial publisher inventory");
      if (existing.length === 1) assertPublisher(existing[0], name, repositoryOwner);
      await canonicalOwner(owner, ownerLogin, request, name);
      const checksum = await exactVersion(name, version, request);
      assert.equal(sha256(packageSource(name)), checksum);
      adopted.set(name, checksum);
    },
    publish: async (name) => {
      assert.equal(
        await namespace(name, request),
        "missing",
        "namespace changed before publication",
      );
      authorize();
      // Existing H helper owns manifest staging and serial registry resolution.
      io.run(
        "node",
        ["tools/support/release/publish_crates/main.ts", "--crate", name],
        undefined,
        true,
      );
    },
    verify: async (name) => {
      const checksum = await exactVersion(name, version, request);
      assert.equal(
        sha256(packageSource(name)),
        checksum,
        "registry package differs from exact-H staged archive",
      );
      const bytes = await archive(name, version, checksum, request);
      writeFileSync(path.join(output, `${name}-${version}.crate`), bytes);
      // Even partial resumption crosses the real crates.io dependency barrier.
      io.run("cargo", ["info", "--registry", "crates-io", `${name}@${version}`], undefined, true);
      const publisher = await ensurePublisher(name, repositoryOwner, token, request, authorize);
      await canonicalOwner(owner, ownerLogin, request, name);
      const row = { name, version, checksum, source: source.head, publisher };
      completed.push(row);
      custody("partial.json", {
        schema,
        candidate: source,
        run,
        attempt,
        qualification,
        owner,
        ownerLogin,
        repositoryOwner,
        published: completed,
        adopted: [...adopted.keys()],
        complete: false,
      });
      return row;
    },
    consumer: async () => {
      authenticate(source, io, true);
      const consumerOutput = path.join(
        path.dirname(output),
        `corsa-sdk-consumer-${run}-${attempt}`,
      );
      io.run(
        "node",
        [
          consumer,
          "--sdk-only",
          "--root",
          root,
          "--output",
          consumerOutput,
          "--version",
          version,
          "--source-sha",
          source.head,
        ],
        undefined,
        true,
      );
      const bytes = readFileSync(path.join(consumerOutput, "evidence", "receipt.json"));
      assertConsumer(JSON.parse(bytes.toString()), source, completed);
      return { mode: "sdk-only", success: true, receiptSha256: sha256(bytes) };
    },
  });
  completed = result.published;
  authorize();
  const receipt: Receipt = {
    schema,
    candidate: source,
    run,
    attempt,
    qualification,
    owner,
    ownerLogin,
    repositoryOwner,
    ...result,
  };
  custody("receipt.json", receipt);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [repository, number, branch, head, cut, tag, output, ...extra] = process.argv.slice(2);
  assert.equal(extra.length, 0);
  assert.ok(output && path.isAbsolute(output));
  const deadline = Number(process.env.VIZE_RELEASE_OPERATOR_DEADLINE_UNIX_MILLIS);
  assert.ok(Number.isSafeInteger(deadline) && deadline > Date.now());
  assert.ok(deadline - Date.now() <= 900_000, "carrier cannot exceed fifteen minutes");
  await carry(
    candidate({ repository, number: Number(number), branch, head, cut, tag }),
    output,
    deadline,
    boundedRequest(deadline),
  ).catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
