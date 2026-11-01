import assert from "node:assert/strict";
import type { Bundle, Request } from "./model.ts";
import type { GithubRead } from "./authority.ts";
import { array, bytes, gitBlob, hash, object, sha256, sourcePath } from "./value.ts";
import path from "node:path";
export async function remoteControllerClosure(
  head: string,
  read: GithubRead,
): Promise<Array<{ path: string; bytes: number; sha256: string; gitBlob: string }>> {
  const ns = "tools/support/release/human_reports_11252_11253_public/",
    pending = [
      ...[
        "run.ts",
        "bootstrap.ts",
        "native-observer.ts",
        "identity-probe.ts",
        "original-inputs.json",
        "after-input-plan.json.txt",
        "pnpm-bootstrap.json",
      ].map((n) => ns + n),
      ".github/workflows/published-human-reports-11252-11253.yml",
    ];
  const seen = new Set<string>(),
    result: Array<{ path: string; bytes: number; sha256: string; gitBlob: string }> = [];
  for (let index = 0; index < pending.length; index++) {
    const name = pending[index]!;
    if (seen.has(name)) continue;
    sourcePath(name);
    seen.add(name);
    const file = object(await read(`contents/${name}?ref=${head}`));
    assert.equal(file.type, "file");
    assert.equal(file.path, name);
    assert.equal(file.encoding, "base64");
    const raw = bytes(String(file.content).replace(/\n/gu, ""));
    assert.equal(gitBlob(raw), file.sha);
    result.push({ path: name, bytes: raw.length, sha256: sha256(raw), gitBlob: String(file.sha) });
    const source = new TextDecoder("utf-8", { fatal: true }).decode(raw);
    for (const match of source.matchAll(
      /(?:from\s*|import\s*|import\()\s*["'](\.{1,2}\/[^"']+)["']/gu,
    ))
      pending.push(path.posix.normalize(path.posix.join(path.posix.dirname(name), match[1]!)));
  }
  return result.sort((a, b) => a.path.localeCompare(b.path));
}
export async function authenticateController(
  controller: Bundle["controller"],
  reviewed: Request["reviewedController"],
  read: GithubRead,
  archived: boolean,
): Promise<unknown> {
  hash(reviewed.head, 40);
  assert.equal(controller.head, reviewed.head);
  assert(reviewed.files.length > 0);
  const run = object(await read(`actions/runs/${controller.run}`));
  assert.equal(object(run.repository).full_name, "ubugeeei-prod/vize");
  assert.equal(run.head_sha, reviewed.head);
  assert.equal(run.head_branch, "main");
  assert.equal(run.event, "workflow_dispatch");
  assert.equal(run.path, ".github/workflows/published-human-reports-11252-11253.yml");
  if (archived) {
    assert.equal(run.status, "completed");
    assert.equal(run.conclusion, "success");
  } else {
    assert.equal(process.env.GITHUB_RUN_ID, controller.run);
    assert.equal(process.env.GITHUB_SHA, controller.head);
  }
  const ancestry = object(await read(`compare/${reviewed.head}...main`));
  assert.equal(object(ancestry.merge_base_commit).sha, reviewed.head);
  assert.equal(ancestry.behind_by, 0);
  const jobs = object(await read(`actions/runs/${controller.run}/jobs?per_page=100&filter=latest`));
  assert.equal(jobs.total_count, array(jobs.jobs).length);
  const selected = array(jobs.jobs)
    .map(object)
    .filter((j) => j.name === controller.job);
  assert.equal(selected.length, 1);
  assert.equal(controller.job, "public-consumer");
  assert.equal(selected[0]?.head_sha, reviewed.head);
  if (archived) assert.equal(selected[0]?.conclusion, "success");
  assert.equal(new Set(reviewed.files.map((f) => f.path)).size, reviewed.files.length);
  assert.equal(controller.files.length, reviewed.files.length);
  const closure = await remoteControllerClosure(reviewed.head, read);
  assert.deepEqual(
    [...reviewed.files].sort((a, b) => a.path.localeCompare(b.path)),
    closure,
    "reviewed controller must bind the independently closed runtime source/data/workflow inventory",
  );
  for (const expected of reviewed.files) {
    sourcePath(expected.path);
    hash(expected.sha256);
    hash(expected.gitBlob, 40);
    const matches = controller.files.filter((f) => f.path.endsWith("/" + expected.path));
    assert.equal(matches.length, 1);
    assert.equal(matches[0]?.sha256, expected.sha256);
    assert.equal(matches[0]?.bytes, expected.bytes);
    const file = object(await read(`contents/${expected.path}?ref=${reviewed.head}`));
    assert.equal(file.type, "file");
    assert.equal(file.path, expected.path);
    const raw = bytes(String(file.content).replace(/\n/gu, ""));
    assert.equal(gitBlob(raw), file.sha);
    assert.equal(file.sha, expected.gitBlob);
    assert.equal(raw.length, expected.bytes);
    assert.equal(sha256(raw), expected.sha256);
  }
  return {
    run: controller.run,
    head: reviewed.head,
    job: selected[0],
    sourceFiles: reviewed.files,
    archivedPathsReadFromCurrentFilesystem: false,
  };
}
