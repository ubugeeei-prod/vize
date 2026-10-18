import { println, run, writeStderr } from "../command_io.ts";
import { nonEmptyEnv } from "../command_runtime.ts";

/** Node runner used for release-side JS checks. */
function releaseNodeBin(): string {
  return nonEmptyEnv("VIZE_RELEASE_NODE") ?? "node";
}

const MANIFEST_TESTS = [
  "--test",
  "tests/tooling/package-manifests.test.ts",
  "tests/tooling/bundler-plugin-manifests.test.ts",
  "tests/tooling/published-snapshot-publication.test.ts",
];

/** Runs the manifest alignment test before release files are mutated. */
export async function verifyManifestAlignmentBeforeMutation(): Promise<boolean> {
  println("Verifying manifest alignment before repository mutation...");
  if ((await run(releaseNodeBin(), MANIFEST_TESTS)) !== 0) {
    writeStderr(
      "Error: package manifest or benchmark publication tests failed before repository mutation. Install JS dependencies or fix the manifest drift, then retry.\n",
    );
    return false;
  }
  return true;
}

/** Runs the manifest alignment test against the generated release commit. */
export async function verifyManifestAlignmentAfterCommit(): Promise<boolean> {
  println("Verifying manifest alignment with tests/tooling/package-manifests.test.ts...");
  if ((await run(releaseNodeBin(), MANIFEST_TESTS)) !== 0) {
    writeStderr(
      "Error: package manifest or benchmark publication tests failed against the release commit. Inspect tests/tooling/package-manifests.test.ts, then run `git reset --hard origin/main` after confirming there is no work to keep, fix the drift, and retry.\n",
    );
    return false;
  }
  return true;
}
