import assert from "node:assert/strict";
import { test } from "node:test";
import {
  derivePublicationPlan,
  readRawBlob,
} from "../../../tools/support/release/public_acceptance/plan.ts";
import { readRepoFile } from "../support/github-workflows.ts";
import { sourceFixture } from "../support/release-public-acceptance-fixtures.ts";

const legacy =
  "run: node tools/support/release/publish_npm_package_dirs/main.ts npm/native/npm --provenance";
const concurrent =
  'run: node tools/support/release/publish_npm_package_dirs/main.ts npm/native/npm --concurrency 8 --provenance --receipt "$RUNNER_TEMP/native-platform-publish.json"';

type Fixture = ReturnType<typeof sourceFixture>;

function commitInvocation(source: Fixture, invocation: string) {
  const workflow = readRawBlob(source.root, source.head, ".github/workflows/release.yml");
  assert.equal(workflow.toString().split(legacy).length, 2);
  source.git("reset", "--soft", source.cut);
  source.write(".github/workflows/release.yml", workflow.toString().replace(legacy, invocation));
  source.git("add", ".");
  source.git("commit", "-qm", "authored native invocation H");
  return source.git("rev-parse", "HEAD");
}

function targets(plan: ReturnType<typeof derivePublicationPlan>) {
  return {
    version: plan.version,
    npm: plan.npm,
    crates: plan.crates,
    editor: plan.editor,
    githubAssets: plan.githubAssets,
  };
}

test("legacy native source invocation keeps the original complete plan unchanged", (t) => {
  const source = sourceFixture(t);
  assert.deepEqual(derivePublicationPlan(source.root, source.head), source.plan);
  assert.deepEqual(source.plan.npm, [
    { name: "@vizejs/native", version: "0.438.0" },
    { name: "@vizejs/mcp-musea", version: "0.438.0" },
    { name: "@vizejs/native-darwin-arm64", version: "0.438.0" },
    { name: "@vizejs/native-linux-x64-gnu", version: "0.438.0" },
  ]);
});

test("the exact current eight-child invocation derives the same authored native targets", (t) => {
  const workflow = readRepoFile(".github", "workflows", "release.yml");
  const actual = workflow
    .split("\n")
    .filter(
      (line) =>
        !line.trimStart().startsWith("#") &&
        /tools\/support\/release\/publish_npm_package_dirs\/main\.ts\s/.test(line),
    );
  assert.deepEqual(
    actual.map((line) => line.trim()),
    [concurrent],
  );
  const source = sourceFixture(t);
  const head = commitInvocation(source, actual[0].trim());
  const plan = derivePublicationPlan(source.root, head);
  assert.equal(plan.parentCut, source.cut);
  assert.deepEqual(targets(plan), targets(source.plan));
  const authority = plan.authority.blobs.find(
    (blob) => blob.path === ".github/workflows/release.yml",
  );
  assert.equal(authority?.oid, source.git("rev-parse", `${head}:.github/workflows/release.yml`));
});

test("native source commands refuse every unreviewed flag, path, shell suffix and duplicate", (t) => {
  const source = sourceFixture(t);
  const hostile = [
    concurrent.replace("--concurrency 8", "--concurrency 0"),
    concurrent.replace("--concurrency 8", "--concurrency 3"),
    concurrent.replace("--concurrency 8", "--concurrency 5"),
    concurrent.replace("--concurrency 8", "--concurrency 8oops"),
    concurrent.replace("--provenance ", ""),
    concurrent.replace("npm/native/npm", "npm/other/npm"),
    concurrent.replace("native-platform-publish.json", "other.json"),
    concurrent.replace(
      '"$RUNNER_TEMP/native-platform-publish.json"',
      "$RUNNER_TEMP/native-platform-publish.json",
    ),
    concurrent.replace("--receipt", "--ignore-failure --receipt"),
    concurrent + " --retries 1",
    concurrent + " --unknown",
    concurrent + " && true",
    concurrent + "; true",
    legacy + " --ignore-failure",
    legacy + " --concurrency 8",
    "# " + concurrent,
    legacy + "\n      - " + concurrent,
    concurrent + "\n      - " + concurrent,
    legacy + "\n      - " + concurrent + " --ignore-failure",
  ];
  for (const invocation of hostile) {
    const head = commitInvocation(source, invocation);
    assert.throws(
      () => derivePublicationPlan(source.root, head),
      /(?:one native target publication required|unsupported native target publication command)/,
      invocation,
    );
  }
});

test("sources from before the TypeScript port derive the same plan from MoonBit publishers", (t) => {
  const source = sourceFixture(t);
  const workflow = readRawBlob(source.root, source.head, ".github/workflows/release.yml")
    .toString()
    .replaceAll(
      "node tools/support/release/publish_npm_package/main.ts ",
      "moon run --target native tools/moon/cmd/publish_npm_package -- ",
    )
    .replace(
      "node tools/support/release/publish_npm_package_dirs/main.ts ",
      "moon run --target native tools/moon/cmd/publish_npm_package_dirs -- ",
    );
  source.git("reset", "--soft", source.cut);
  source.git("rm", "-q", "--cached", "tools/support/release/publish_crates/main.ts");
  source.write(".github/workflows/release.yml", workflow);
  source.write(
    "tools/moon/cmd/publish_crates/main.mbt",
    'let published_crates : Array[String] = [\n  "vize_l0",\n  "vize_musea",\n]\n',
  );
  source.git("add", ".github/workflows/release.yml", "tools/moon/cmd/publish_crates/main.mbt");
  source.git("commit", "-qm", "MoonBit-era H");
  const head = source.git("rev-parse", "HEAD");
  const plan = derivePublicationPlan(source.root, head);
  assert.deepEqual(targets(plan), targets(source.plan));
  assert.ok(plan.authority.blobs.some((blob) => blob.path.endsWith("publish_crates/main.mbt")));
});
