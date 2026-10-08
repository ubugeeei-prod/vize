import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const sha = /^[0-9a-f]{40}$/;

/** Select only the queue root or PR itself; never infer custody from a title. */
export function integrationEvent(eventName, event, repository, candidate, pull) {
  if (event.repository?.full_name !== repository || !sha.test(candidate ?? "")) {
    throw new Error("Release integration event repository/candidate is missing or changed");
  }
  let number;
  let queueBase;
  if (eventName === "pull_request") {
    number = event.number;
    pull = event.pull_request;
  } else if (eventName === "merge_group") {
    const group = event.merge_group;
    const match = /^refs\/heads\/gh-readonly-queue\/main\/pr-([1-9]\d*)-([0-9a-f]{40})$/.exec(
      group?.head_ref ?? "",
    );
    if (
      !match ||
      group.head_sha !== candidate ||
      group.base_ref !== "refs/heads/main" ||
      !sha.test(group.base_sha ?? "")
    ) {
      throw new Error("Release integration requires an unambiguous own main queue candidate");
    }
    number = Number(match[1]);
    queueBase = match[2];
    pull = pull(number);
  } else {
    throw new Error(`Unsupported release integration event: ${eventName}`);
  }
  if (!Number.isSafeInteger(number) || number <= 0 || pull?.number !== number) {
    throw new Error("Release integration event PR identity is missing or changed");
  }
  const selected = pull.head?.ref?.startsWith("release-integration/");
  if (selected && queueBase !== undefined && event.merge_group.base_sha !== queueBase) {
    throw new Error("Release integration requires an unambiguous own main queue candidate");
  }
  if (!selected && /<!-- vize-release-pin-source:/.test(pull.body ?? "")) {
    throw new Error("Pinned integration markers require the official integration branch");
  }
  return selected ? String(number) : "";
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const repository = process.env.GITHUB_REPOSITORY;
  const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, "utf8"));
  const number = integrationEvent(
    process.env.GITHUB_EVENT_NAME,
    event,
    repository,
    process.env.GITHUB_SHA,
    (number) =>
      JSON.parse(
        execFileSync("gh", ["api", `repos/${repository}/pulls/${number}`], { encoding: "utf8" }),
      ),
  );
  appendFileSync(process.env.GITHUB_OUTPUT, `integration=${number}\n`);
  const message = number
    ? `Pinned integration #${number}: authenticate the exact candidate before merging.\n`
    : "Ordinary PR/queue candidate: no pinned integration to qualify.\n";
  process.stdout.write(message);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, message);
}
