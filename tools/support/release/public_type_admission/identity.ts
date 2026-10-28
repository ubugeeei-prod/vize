import { SOURCE_ADMISSION } from "./source-admission.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { installedRuntime } from "./installed-runtime.ts";
import { integer, parse, record, string, write, type Runtime } from "./model.ts";
export async function runtime(args: string[]): Promise<Runtime> {
  const [workspaceArg, version, outputArg, identityArg] = args;
  assert.ok(workspaceArg && version && outputArg && identityArg);
  assert.match(version, /^0\.\d+\.0$/u);
  for (const excluded of SOURCE_ADMISSION.explicitlyExcludedVersions)
    assert.notEqual(version, excluded, "10950 is absent from this publication");
  const workspace = fs.realpathSync(workspaceArg),
    output = path.resolve(outputArg);
  fs.mkdirSync(output, { recursive: true });
  const eligibility = fs.readFileSync(identityArg);
  fs.writeFileSync(path.join(output, "release-eligibility.txt"), eligibility, { flag: "wx" });
  const delivery = record(parse(eligibility));
  assert.equal(delivery.schema, "vize.template-ref.first-containing-release.v1");
  assert.equal(delivery.version, version);
  assert.equal(delivery.firstContainingTag, "v" + version);
  assert.equal(delivery.pullRequest, 10950);
  assert.equal(delivery.publicationConfirmed, true);
  const candidate = string(delivery.sourceSha),
    required = string(delivery.requiredSource);
  assert.match(candidate, /^[0-9a-f]{40}$/u);
  assert.match(required, /^[0-9a-f]{40}$/u);
  assert.equal(
    required,
    SOURCE_ADMISSION.requiredSource,
    "eligibility must name the actual merged 10950 source",
  );
  write(path.join(output, "source-admission.txt"), SOURCE_ADMISSION);
  const comparison = record(delivery.sourceComparison);
  assert.equal(string(record(comparison.base_commit).sha), required);
  assert.equal(string(record(comparison.merge_base_commit).sha), required);
  assert.equal(
    comparison.html_url,
    "https://github.com/ubugeeei-prod/vize/compare/" + required + "..." + candidate,
  );
  assert.ok(comparison.status === "ahead" || comparison.status === "identical");
  assert.equal(integer(comparison.behind_by), 0);
  const tag = record(delivery.tag);
  assert.equal(tag.name, "v" + version);
  assert.equal(string(record(tag.commit).sha), candidate);
  const first = record(delivery.firstContainingPublication);
  assert.equal(first.tag_name, "v" + version);
  assert.equal(first.draft, false);
  assert.ok(string(first.published_at).length > 0);
  return installedRuntime({ workspace, version, output, candidate, delivery });
}
