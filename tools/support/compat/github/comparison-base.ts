import { execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const fullSha = /^[0-9a-f]{40}$/;

export function comparisonBase({ event, eventBase, head, checkedOutHead, prHead, commit }) {
  if (!["pull_request", "merge_group"].includes(event)) {
    throw new Error("Unexpected comparison context");
  }
  if (![eventBase, head, checkedOutHead].every((sha) => fullSha.test(sha ?? ""))) {
    throw new Error("Expected full comparison and checkout SHAs");
  }
  if (event === "merge_group") return { base: eventBase, reason: "merge group event base" };
  if (!fullSha.test(prHead ?? "")) throw new Error("Expected full pull request head SHA");
  const parents = commit
    .split("\n\n", 1)[0]
    .split("\n")
    .filter((line) => line.startsWith("parent "))
    .map((line) => line.slice("parent ".length));
  const verified =
    checkedOutHead === head &&
    parents.length === 2 &&
    parents.every((sha) => fullSha.test(sha)) &&
    parents[1] === prHead;
  return verified
    ? { base: parents[0], reason: "verified pull request merge parent" }
    : { base: eventBase, reason: "unverified merge: conservative event base" };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const git = (...args) => execFileSync("git", args, { encoding: "utf8" }).trim();
  const result = comparisonBase({
    event: process.env.GITHUB_EVENT_NAME,
    eventBase: process.env.EVENT_BASE_SHA,
    head: process.env.GITHUB_SHA,
    checkedOutHead: git("rev-parse", "HEAD"),
    prHead: process.env.PR_HEAD_SHA,
    commit: git("cat-file", "-p", "HEAD"),
  });
  if (process.env.GITHUB_OUTPUT) appendFileSync(process.env.GITHUB_OUTPUT, `base=${result.base}\n`);
  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(
      process.env.GITHUB_STEP_SUMMARY,
      `Comparison base: ${result.base}; ${result.reason}.\n`,
    );
  }
  process.stdout.write(`${result.base}\n`);
}
