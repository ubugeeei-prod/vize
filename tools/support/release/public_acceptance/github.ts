import type { PublicationOptions } from "./registry.ts";

type ObjectValue = Record<string, unknown>;
const repository = "ubugeeei-prod/vize";
const github = `https://api.github.com/repos/${repository}`;
function requireValue(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function object(value: unknown): ObjectValue {
  requireValue(
    value && typeof value === "object" && !Array.isArray(value),
    "GitHub object required",
  );
  return value as ObjectValue;
}

/** Authenticate public immutable pin receipts, without a commit-signature or moving-branch claim. */
export async function verifyGithub(
  options: PublicationOptions,
  json: (url: string) => Promise<ObjectValue>,
) {
  const sourcePr = String(options.sourcePr),
    run = String(options.run);
  const tagRef = await json(`${github}/git/ref/tags/${options.tag}`);
  const tagObject = object(tagRef.object);
  requireValue(
    tagRef.ref === `refs/tags/${options.tag}` &&
      tagObject.type === "tag" &&
      typeof tagObject.sha === "string" &&
      /^[0-9a-f]{40}$/.test(tagObject.sha),
    "public annotated tag required",
  );
  const tag = await json(`${github}/git/tags/${tagObject.sha}`);
  const target = object(tag.object);
  requireValue(
    tag.sha === tagObject.sha &&
      tag.tag === options.tag &&
      target.type === "commit" &&
      target.sha === options.head,
    "annotated tag target H mismatch",
  );
  requireValue(typeof tag.message === "string", "annotated pin receipt message required");
  const annotation = tag.message.split(/\r?\n/);
  const fields = {
    Release: options.tag,
    "Release-Mode": "pinned",
    "Release-PR": `#${sourcePr}`,
    "Validated-run": run,
    "Validated-head": options.head,
    "Validated-base": options.cut,
  };
  for (const [name, value] of Object.entries(fields)) {
    const prefix = name === "Release" ? "Release " : `${name}: `;
    const matches = annotation.filter((line) =>
      name === "Release" ? /^Release(?:\s|$)/.test(line) : new RegExp(`^${name}\\s*:`).test(line),
    );
    requireValue(
      matches.length === 1 && matches[0] === prefix + value,
      `annotated pin field mismatch: ${name}`,
    );
  }
  const source = await json(`${github}/pulls/${sourcePr}`);
  const sourceHead = object(source.head);
  requireValue(
    String(source.number) === sourcePr &&
      sourceHead.sha === options.head &&
      sourceHead.ref === `release/${options.tag}` &&
      object(sourceHead.repo).full_name === repository &&
      object(source.base).ref === "main" &&
      object(object(source.base).repo).full_name === repository &&
      source.merged === false &&
      source.merged_at === null &&
      (source.state === "open" || source.state === "closed"),
    "own-repository exact-H source PR required",
  );
  requireValue(typeof source.body === "string", "immutable source PR body required");
  const marker = (name: string) => {
    const body = source.body as string;
    const hits = [...body.matchAll(new RegExp(`<!-- ${name}: ([^\\n<>]+) -->`, "g"))];
    const occurrences = [...body.matchAll(new RegExp(`<!--\\s*${name}\\s*:`, "g"))];
    requireValue(
      hits.length === 1 && occurrences.length === 1,
      `missing/ambiguous source marker: ${name}`,
    );
    return hits[0][1];
  };
  requireValue(
    marker("vize-release-pin") === "immutable-v1" &&
      marker("vize-release-pin-cut") === options.cut &&
      marker("vize-release-pin-head") === options.head,
    "immutable source marker C/H mismatch",
  );
  const integrationPr = marker("vize-release-integration");
  requireValue(
    /^[1-9][0-9]*$/.test(integrationPr) && Number.isSafeInteger(Number(integrationPr)),
    "positive source integration marker required",
  );
  const releaseRun = await json(`${github}/actions/runs/${run}`);
  requireValue(
    String(releaseRun.id) === run &&
      object(releaseRun.repository).full_name === repository &&
      object(releaseRun.head_repository).full_name === repository &&
      releaseRun.head_sha === options.head &&
      releaseRun.head_branch === `release/${options.tag}` &&
      releaseRun.event === "workflow_dispatch" &&
      releaseRun.display_title ===
        `Pinned Release ${options.tag} PR #${sourcePr} @ ${options.head}` &&
      releaseRun.path === ".github/workflows/release.yml" &&
      releaseRun.status === "completed" &&
      releaseRun.conclusion === "success",
    "terminal own-repository exact-H/source-PR Pinned Release run required",
  );
  requireValue(
    Number.isSafeInteger(releaseRun.run_attempt) && Number(releaseRun.run_attempt) > 0,
    "positive Release attempt required",
  );
  const jobs: ObjectValue[] = [];
  const ids = new Set<number>();
  let expected = 0;
  for (let page = 1; page <= 10; page++) {
    const result = await json(
      `${github}/actions/runs/${run}/jobs?filter=latest&per_page=100&page=${page}`,
    );
    requireValue(
      Number.isSafeInteger(result.total_count) &&
        Number(result.total_count) > 0 &&
        Number(result.total_count) <= 1000 &&
        Array.isArray(result.jobs) &&
        result.jobs.length > 0 &&
        result.jobs.length <= 100,
      "bounded complete Release jobs response required",
    );
    if (page === 1) expected = Number(result.total_count);
    requireValue(result.total_count === expected, "Release job total changed during read");
    for (const value of result.jobs) {
      const job = object(value);
      requireValue(
        typeof job.id === "number" &&
          Number.isSafeInteger(job.id) &&
          job.id > 0 &&
          !ids.has(job.id) &&
          String(job.run_id) === run &&
          typeof job.run_attempt === "number" &&
          Number.isSafeInteger(job.run_attempt) &&
          job.run_attempt > 0 &&
          job.run_attempt <= Number(releaseRun.run_attempt) &&
          job.head_sha === options.head &&
          job.status === "completed" &&
          (job.conclusion === "success" || job.conclusion === "skipped"),
        "terminal exact-H/R Release job required",
      );
      ids.add(job.id);
      jobs.push({
        id: job.id,
        name: job.name,
        status: job.status,
        conclusion: job.conclusion,
        run: job.run_id,
        attempt: job.run_attempt,
        head: job.head_sha,
      });
    }
    requireValue(jobs.length <= expected, "Release job total exceeded");
    if (jobs.length === expected) break;
  }
  requireValue(jobs.length === expected, "complete paginated Release jobs required");
  const finalRun = await json(`${github}/actions/runs/${run}`);
  requireValue(
    [
      "id",
      "head_sha",
      "head_branch",
      "event",
      "display_title",
      "path",
      "run_attempt",
      "status",
      "conclusion",
    ].every((key) => finalRun[key] === releaseRun[key]) &&
      object(finalRun.repository).full_name === repository &&
      object(finalRun.head_repository).full_name === repository,
    "Release attempt changed during jobs read",
  );
  const release = await json(`${github}/releases/tags/${options.tag}`);
  // target_commitish is a creation hint ignored when the immutable tag exists.
  requireValue(
    release.tag_name === options.tag &&
      release.draft === false &&
      release.prerelease === false &&
      typeof release.published_at === "string" &&
      Number.isFinite(Date.parse(release.published_at)) &&
      typeof release.id === "number" &&
      Number.isSafeInteger(release.id) &&
      release.id > 0,
    "published GitHub Release identity mismatch",
  );

  return { tagObject, tag, source, releaseRun, release, jobs, integrationPr };
}
