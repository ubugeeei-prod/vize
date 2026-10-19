// Release entry point. Locally it dispatches the Release workflow; in CI it
// picks the next version, stamps it and pushes the tag the workflow builds.
import { appendFileSync } from "node:fs";
import { collectStdout, println, run, runMain, writeStderr } from "../command_io.ts";
import { cliArgs, nonEmptyEnv, readFileToString } from "../command_runtime.ts";
import { cadenceDecision, readCadenceInput } from "./cadence.ts";
import { stampVersion } from "./stamp.ts";
import {
  highestReleaseVersion,
  isUnclaimedVersion,
  nextReleaseVersion,
  workspaceVersionFromContent,
} from "./versions.ts";

const USAGE = `Usage:
  main.ts dispatch [version] dispatch the Release workflow on main (\`vp run release\`)
  main.ts next               print the next release version
  main.ts stamp <version>    stamp <version> into the working tree
  main.ts cut [version]      CI only: stamp, commit and push tag v<version>
  main.ts cadence            CI only: decide whether a scheduled run releases
`;

function fail(message: string): number {
  writeStderr(message);
  return 1;
}

function workspaceVersion(): string | null {
  const cargoToml = readFileToString("Cargo.toml");
  return cargoToml === null ? null : workspaceVersionFromContent(cargoToml);
}

/** Release tags already on the remote, including burned ones. */
async function remoteTagVersions(): Promise<string[] | null> {
  const output = await collectStdout("git", ["ls-remote", "--tags", "--refs", "origin", "v*"]);
  if (output.status !== 0) return null;
  return output.stdout
    .split("\n")
    .map((line) => line.split("refs/tags/")[1])
    .filter((tag): tag is string => tag !== undefined && tag !== "");
}

/** Versions of the `vize` npm package, which every release publishes. */
async function npmVersions(): Promise<string[] | null> {
  try {
    const response = await fetch("https://registry.npmjs.org/vize", {
      headers: { accept: "application/vnd.npm.install-v1+json" },
    });
    if (!response.ok) return null;
    const body = (await response.json()) as { versions?: Record<string, unknown> };
    return Object.keys(body.versions ?? {});
  } catch {
    return null;
  }
}

async function claimedVersions(current: string): Promise<string[] | null> {
  const [tags, published] = await Promise.all([remoteTagVersions(), npmVersions()]);
  return tags === null || published === null ? null : [current, ...tags, ...published];
}

/** The requested version if it is still unclaimed, else the next minor. */
async function resolveVersion(
  current: string,
  requested: string | undefined,
): Promise<string | null> {
  const claimed = await claimedVersions(current);
  if (claimed === null) {
    writeStderr("Failed to read release tags or npm versions\n");
    return null;
  }
  if (requested === undefined) return nextReleaseVersion(claimed);
  if (isUnclaimedVersion(requested, claimed)) return requested;
  writeStderr(`${requested} is not above every existing tag and npm version\n`);
  return null;
}

async function git(...args: string[]): Promise<boolean> {
  return (await run("git", args)) === 0;
}

/** Stamps, commits and pushes the release tag. The tag push is the lock. */
async function cut(current: string, requested: string | undefined): Promise<number> {
  const version = await resolveVersion(current, requested);
  if (version === null) return 1;
  const tag = `v${version}`;
  println(`Releasing ${tag} (workspace manifests say ${current})`);
  if (!(await stampVersion(current, version))) return 1;
  const ok =
    (await git(
      "-c",
      "user.name=github-actions[bot]",
      "-c",
      "user.email=41898282+github-actions[bot]@users.noreply.github.com",
      "commit",
      "--all",
      "--quiet",
      "--no-verify",
      "-m",
      `chore(release): ${tag}`,
    )) &&
    (await git("tag", tag)) &&
    (await git("push", "origin", `refs/tags/${tag}`));
  if (!ok) return fail(`Failed to commit or push ${tag}\n`);
  const sha = (await collectStdout("git", ["rev-parse", "HEAD"])).stdout.trim();
  const output = nonEmptyEnv("GITHUB_OUTPUT");
  if (output !== undefined) {
    appendFileSync(output, `version=${version}\ntag=${tag}\nsha=${sha}\n`);
  }
  println(`Pushed ${tag} at ${sha}`);
  return 0;
}

/** Writes release=true|false and the green main sha for a scheduled run. */
async function cadence(): Promise<number> {
  const tags = await remoteTagVersions();
  if (tags === null) return fail("Failed to read release tags\n");
  const highest = highestReleaseVersion(tags);
  const input = await readCadenceInput(highest === null ? null : `v${highest}`);
  const decision = cadenceDecision(input);
  println(
    `${decision.release ? "Releasing" : "Skipping"} ${input.greenSha ?? "-"}: ${decision.reason}`,
  );
  const output = nonEmptyEnv("GITHUB_OUTPUT");
  if (output !== undefined) {
    appendFileSync(output, `release=${decision.release}\nsha=${input.greenSha ?? ""}\n`);
  }
  return 0;
}

export async function release(args: readonly string[]): Promise<number> {
  const [command, value, ...rest] = args;
  if (rest.length > 0) return fail(USAGE);
  const current = workspaceVersion();
  if (current === null) return fail("Failed to read the workspace version from Cargo.toml\n");
  switch (command) {
    case "next": {
      const version = await resolveVersion(current, undefined);
      if (version === null) return 1;
      println(version);
      return 0;
    }
    case "cadence":
      return cadence();
    case "stamp":
      if (value === undefined) return fail(USAGE);
      return (await stampVersion(current, value)) ? 0 : 1;
    case "cut":
      if (nonEmptyEnv("GITHUB_ACTIONS") !== "true") return fail("cut runs only in CI\n");
      return cut(current, value);
    case "dispatch": {
      if (value !== undefined && !/^\d+\.\d+\.\d+$/.test(value)) return fail(USAGE);
      const inputs = value === undefined ? [] : ["-f", `version=${value}`];
      return run("gh", ["workflow", "run", "release.yml", "--ref", "main", ...inputs]);
    }
    default:
      return fail(USAGE);
  }
}

if (import.meta.main) {
  runMain(() => release(cliArgs()));
}
