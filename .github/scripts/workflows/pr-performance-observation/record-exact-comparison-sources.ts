import { requiredEnv } from "../shared/files.ts";
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
const git = (...args: string[]) =>
  execFileSync("git", ["-C", "head", ...args], { encoding: "utf8" }).trim();
writeFileSync(
  "performance-observation/source-pair.json",
  JSON.stringify(
    {
      schema: 1,
      event: process.env.EVENT_NAME,
      pullRequest: process.env.OBSERVATION_PR || null,
      base: requiredEnv("BASE_SHA"),
      head: requiredEnv("HEAD_SHA"),
      driver: process.env.DRIVER_SHA,
      driverTree: execFileSync("git", ["-C", "driver", "rev-parse", "HEAD^{tree}"], {
        encoding: "utf8",
      }).trim(),
      mergeBase: git("merge-base", requiredEnv("BASE_SHA"), requiredEnv("HEAD_SHA")),
      comparison:
        process.env.EVENT_NAME === "pull_request"
          ? "exact PR base and authored head, independent measurement driver"
          : "explicit immutable source pair, independent measurement driver",
      changedPaths: git(
        "diff",
        "--name-only",
        "-z",
        requiredEnv("BASE_SHA"),
        requiredEnv("HEAD_SHA"),
      )
        .split("\0")
        .filter(Boolean),
    },
    null,
    2,
  ) + "\n",
);
