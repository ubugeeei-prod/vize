import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const sha = /^[0-9a-f]{40}$/;
type Pull = {
  number?: number;
  body?: string | null;
  draft?: boolean;
  merged?: boolean;
  head?: { ref?: string; sha?: string; repo?: { full_name?: string } };
  base?: { ref?: string; repo?: { full_name?: string } };
};
type Event = {
  repository?: { full_name?: string };
  number?: number;
  pull_request?: Pull;
  merge_group?: { head_ref?: string; head_sha?: string; base_ref?: string; base_sha?: string };
};

/** A source receipt must keep its original H even when main moves. */
export function assertPinnedSource(pull: Pull, repository: string | undefined) {
  const body = pull.body ?? "";
  if (!/<!-- vize-release-pin(?:-|:)/.test(body)) return;
  // Integration PRs carry pin-source; their separate catalog authenticates custody.
  if (pull.head?.ref?.startsWith("release-integration/")) return;
  const marker = (name: string) => {
    const rows = body.split("\n").filter((line) => line.startsWith(`<!-- ${name}:`));
    const match = rows.length === 1 && /^<!-- [a-z-]+: (\S+) -->$/.exec(rows[0]);
    if (!match) throw new Error(`Pinned source requires one complete ${name} marker`);
    return match[1];
  };
  const head = marker("vize-release-pin-head");
  const cut = marker("vize-release-pin-cut");
  if (
    !repository ||
    marker("vize-release-pin") !== "immutable-v1" ||
    !sha.test(head) ||
    !sha.test(cut) ||
    head === cut ||
    pull.head?.sha !== head ||
    !pull.head.ref?.startsWith("release/v") ||
    pull.head.repo?.full_name !== repository ||
    pull.base?.repo?.full_name !== repository ||
    pull.base?.ref !== "main" ||
    pull.draft !== true ||
    pull.merged === true
  ) {
    throw new Error("Pinned source must remain an unchanged, unmerged draft receipt at recorded H");
  }
}

/** Select only the queue root or PR itself; never infer custody from a title. */
export function integrationEvent(
  eventName: string | undefined,
  event: Event,
  repository: string | undefined,
  candidate: string | undefined,
  loadPull?: (number: number) => Pull,
) {
  if (!repository || event.repository?.full_name !== repository || !sha.test(candidate ?? "")) {
    throw new Error("Release integration event repository/candidate is missing or changed");
  }
  let number;
  let queueBase;
  let pull: Pull | undefined;
  if (eventName === "pull_request") {
    number = event.number;
    pull = event.pull_request;
  } else if (eventName === "merge_group") {
    const group = event.merge_group;
    const match = /^refs\/heads\/gh-readonly-queue\/main\/pr-([1-9]\d*)-([0-9a-f]{40})$/.exec(
      group?.head_ref ?? "",
    );
    if (
      !group ||
      !match ||
      group.head_sha !== candidate ||
      group.base_ref !== "refs/heads/main" ||
      !sha.test(group.base_sha ?? "")
    ) {
      throw new Error("Release integration requires an unambiguous own main queue candidate");
    }
    number = Number(match[1]);
    queueBase = match[2];
    if (!loadPull) throw new Error("Queue pull request loader is required");
    pull = loadPull(number);
  } else {
    throw new Error(`Unsupported release integration event: ${eventName}`);
  }
  if (
    typeof number !== "number" ||
    !Number.isSafeInteger(number) ||
    number <= 0 ||
    !pull ||
    pull.number !== number
  ) {
    throw new Error("Release integration event PR identity is missing or changed");
  }
  assertPinnedSource(pull, repository);
  const selected = pull.head?.ref?.startsWith("release-integration/");
  if (selected && queueBase !== undefined && event.merge_group?.base_sha !== queueBase) {
    throw new Error("Release integration requires an unambiguous own main queue candidate");
  }
  if (!selected && /<!-- vize-release-pin-source:/.test(pull.body ?? "")) {
    throw new Error("Pinned integration markers require the official integration branch");
  }
  return selected ? String(number) : "";
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const repository = process.env.GITHUB_REPOSITORY;
  const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH!, "utf8"));
  const number = integrationEvent(
    process.env.GITHUB_EVENT_NAME,
    event,
    repository,
    process.env.GITHUB_SHA,
    (number) =>
      JSON.parse(
        execFileSync("gh", ["api", `repos/${repository}/pulls/${number}`], { encoding: "utf8" }),
      ) as Pull,
  );
  appendFileSync(process.env.GITHUB_OUTPUT!, `integration=${number}\n`);
  const message = number
    ? `Pinned integration #${number}: authenticate the exact candidate before merging.\n`
    : "Ordinary PR/queue candidate: no pinned integration to qualify.\n";
  process.stdout.write(message);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, message);
}
