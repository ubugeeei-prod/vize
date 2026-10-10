import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { derivePublicationPlan } from "../../../tools/support/release/public_acceptance/plan.ts";
import {
  jsrAuthorityPaths,
  jsrJobNames,
} from "../../../tools/support/release/public_acceptance/jsr.ts";
import { gh, publicFixture, sha256, version } from "./release-public-acceptance-fixtures.ts";

const repository = fileURLToPath(new URL("../../../", import.meta.url));
type Context = Parameters<typeof publicFixture>[0];

function replaceHead(value: unknown, previous: string, head: string): unknown {
  if (typeof value === "string") return value.replaceAll(previous, head);
  if (Buffer.isBuffer(value)) return value;
  if (Array.isArray(value)) return value.map((item) => replaceHead(item, previous, head));
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, replaceHead(item, previous, head)]),
    );
  }
  return value;
}

/** Real temporary raw Git authority and actual producer bytes; all public responses are inert. */
export function jsrPublicFixture(context: Context, enabled = true) {
  const fixture = publicFixture(context);
  for (const name of jsrAuthorityPaths)
    fixture.write(name, fs.readFileSync(path.join(repository, name), "utf8"));
  fixture.write(
    "jsr/vize/channel.json",
    JSON.stringify({ schema: "vize-jsr-channel-v1", enabled }),
  );
  fixture.write("LICENSE", "Fixture license: byte-preserving 日本語\n");
  const workflow = fs.readFileSync(
    path.join(fixture.root, ".github/workflows/release.yml"),
    "utf8",
  );
  fixture.write(
    ".github/workflows/release.yml",
    workflow +
      "      - run: moon run --target native tools/moon/cmd/publish_npm_package -- npm/cli --provenance\n" +
      "      - run: moon run --target native tools/moon/cmd/publish_npm_package -- npm/builder/vite --provenance\n",
  );
  for (const [directory, name] of [
    ["npm/cli", "vize"],
    ["npm/builder/vite", "@vizejs/vite-plugin"],
  ]) {
    fixture.write(`${directory}/package.json`, JSON.stringify({ name, version }));
    const bytes = Buffer.from(`public npm fixture ${name}@${version}`);
    const url = `https://registry.npmjs.org/${encodeURIComponent(name)}/-/${version}.tgz`;
    fixture.responses.set(`https://registry.npmjs.org/${encodeURIComponent(name)}/${version}`, {
      name,
      version,
      dist: {
        tarball: url,
        integrity: `sha512-${createHash("sha512").update(bytes).digest("base64")}`,
      },
    });
    fixture.responses.set(url, bytes);
  }
  fixture.git("add", ".");
  fixture.git("commit", "--amend", "--no-edit", "-q");
  const head = fixture.git("rev-parse", "HEAD");
  for (const [url, value] of fixture.responses)
    fixture.responses.set(url, replaceHead(value, fixture.head, head));
  const plan = derivePublicationPlan(fixture.root, head);
  const staged = `${fixture.root}-jsr`;
  context.after(() => fs.rmSync(staged, { recursive: true, force: true }));
  const producer = pathToFileURL(
    path.join(repository, "tools/support/release/jsr/prepare.mjs"),
  ).href;
  execFileSync(process.execPath, [
    "--input-type=module",
    "-e",
    `const {preparePackage}=await import(process.argv[1]); preparePackage(process.argv[2],{root:process.argv[3]});`,
    producer,
    staged,
    fixture.root,
  ]);
  const sources = new Map(
    fs.readdirSync(staged).map((name) => [`/${name}`, fs.readFileSync(path.join(staged, name))]),
  );
  const metadataUrl = "https://jsr.io/@vizejs/vize/meta.json";
  const versionMetadataUrl = `https://jsr.io/@vizejs/vize/${version}_meta.json`;
  fixture.responses.set(metadataUrl, {
    scope: "vizejs",
    name: "vize",
    githubRepository: { owner: "ubugeeei-prod", name: "vize" },
    versions: { [version]: {} },
  });
  fixture.responses.set(versionMetadataUrl, {
    exports: plan.jsr!.exports,
    manifest: Object.fromEntries(
      [...sources].map(([name, bytes]) => [
        name,
        { size: bytes.length, checksum: `sha256-${sha256(bytes)}` },
      ]),
    ),
  });
  for (const [name, bytes] of sources)
    fixture.responses.set(`https://jsr.io/@vizejs/vize/${version}${name}`, bytes);
  const jobsUrl = gh(`/actions/runs/${fixture.run}/jobs?filter=latest&per_page=100&page=1`);
  const jobs = jsrJobNames.map((name, index) => ({
    id: 100 + index,
    name,
    run_id: fixture.run,
    run_attempt: 1,
    head_sha: head,
    status: "completed",
    conclusion: "success",
  }));
  fixture.responses.set(jobsUrl, { total_count: jobs.length, jobs });
  return {
    ...fixture,
    head,
    plan,
    sources,
    metadataUrl,
    versionMetadataUrl,
    jobsUrl,
    jobs,
    options: { ...fixture.options, head },
  };
}
