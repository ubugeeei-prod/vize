import { gitBodyDigest } from "./warm-type-backed-path-host.ts";
import {
  gitReaderIncomingGuard,
  qualifyGitReaderIncomingHarness,
} from "./warm-type-backed-git-reader-incoming.ts";
import {
  gitReaderNoDeepGuard,
  qualifyGitReaderNoDeepHarness,
} from "./warm-type-backed-git-reader-nodeep.ts";
import {
  gitReaderSetupGuard,
  qualifyGitReaderSetupHarness,
} from "./warm-type-backed-git-reader-setup.ts";

export const gitReaderReviewedSource = "7e5630229d0d34c8b25e423544bad275fb67b925";
export const gitReaderOriginalSource = "5459e40a23ca7e647ace8ebd20e1fe00c9294437";
export const gitReaderHarnessPairs: ReadonlyArray<readonly [string, string | null, string | null]> =
  [
    [
      "tests/performance/support/warm-type-backed-path-host.ts",
      "b73e46382f99176d54dfa9b5945f6126eefedd6ecfb9cbd4c733151c3e2c6c30",
      "99f84b671f904a819b5e6e1e319fd97db2ae537b7ba1c773013b1e0e87cd7940",
    ],
    [
      "tests/tooling/warm-type-backed-git-body-authority.test.ts",
      null,
      "d54d8a937f10667470c78e1c0174ff10797c6929c059f98323fdc6962e078f90",
    ],
    [
      "docs/davinci/decisions/2026-10-12-3952-raw-git-body-authority.md",
      null,
      "2cbe0f10dec99c16a44ccd07fd3f70debc5a2a5ee520f1791164403da9c5de54",
    ],
    [
      "tests/performance/support/warm-type-backed-path-host-manifest.json",
      "36256d2672412efe1681fab3137fab4f641622920360ea690a98a463a7325805",
      "36256d2672412efe1681fab3137fab4f641622920360ea690a98a463a7325805",
    ],
    [
      "tests/tooling/warm-type-backed-path-host.test.ts",
      "3dfad726a861009ea8b906752797f6be8654adbba5955dcb649cb0e48d455d48",
      "3dfad726a861009ea8b906752797f6be8654adbba5955dcb649cb0e48d455d48",
    ],
    [
      "tests/tooling/warm-type-backed-default-migration-host.test.ts",
      "86ce5a8bcbb25c298e59d211910b6aeca3da3e280f9a2109ce0daee6ace2ca0c",
      "86ce5a8bcbb25c298e59d211910b6aeca3da3e280f9a2109ce0daee6ace2ca0c",
    ],
  ];

/** Admit the reviewed reader and its laws only after complete source checks. */
export function qualifyGitReaderHarness(
  changed: readonly string[],
  alreadyQualified: ReadonlySet<string>,
  digest: (side: "before" | "after", file: string) => string | null,
) {
  const required = gitReaderHarnessPairs.slice(0, 3).map(([file]) => file);
  const finite = new Set(required);
  if (
    new Set(changed).size !== changed.length ||
    required.some((file) => !changed.includes(file)) ||
    changed.some((file) => !alreadyQualified.has(file) && !finite.has(file))
  )
    return null;
  for (const [file, before, after] of gitReaderHarnessPairs)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    authority: "complete-reviewed-raw-git-reader-harness",
    reviewedReaderSource: gitReaderReviewedSource,
    completeBodyPairs: gitReaderHarnessPairs.length,
    files: required,
  };
}

/** Keep every inherited classifier and refusal when this exact repair is absent. */
export function gitReaderHarness(
  changed: readonly string[],
  alreadyQualified: ReadonlySet<string>,
  root: string,
  before: string,
  after: string,
): ReadonlySet<string> {
  const inherited = new Set([
    ...alreadyQualified,
    ...changed.filter((file) =>
      /^tests\/performance\/support\/warm-type-backed-[a-z-]+\.ts$/u.test(file),
    ),
  ]);
  const qualified = qualifyGitReaderHarness(changed, inherited, (side, file) =>
    gitBodyDigest(root, side === "before" ? before : after, file),
  );
  const incoming =
    !qualified &&
    gitBodyDigest(root, before, gitReaderIncomingGuard) === null &&
    gitBodyDigest(root, after, gitReaderIncomingGuard) ===
      "931d2770c0f1fafe5465fec409e720918bf8e71b0df5e759d61feec81baa0fba"
      ? qualifyGitReaderIncomingHarness(changed, inherited, (side, file) =>
          gitBodyDigest(root, side === "before" ? before : after, file),
        )
      : null;
  const nodeep =
    !qualified &&
    !incoming &&
    gitBodyDigest(root, before, gitReaderNoDeepGuard) === null &&
    gitBodyDigest(root, after, gitReaderNoDeepGuard) ===
      "ad25378cc7272fb2fde533d6a6796a9462dec3d3e157217abef13a5a69cf6a96"
      ? qualifyGitReaderNoDeepHarness(changed, inherited, (side, file) =>
          gitBodyDigest(root, side === "before" ? before : after, file),
        )
      : null;
  const setup =
    !qualified &&
    !incoming &&
    !nodeep &&
    gitBodyDigest(root, before, gitReaderSetupGuard) === null &&
    gitBodyDigest(root, after, gitReaderSetupGuard) ===
      "16fa61566684e0aa675186be50aa6405c3c88d223c7ad19a1d3747bf4cfad5e6"
      ? qualifyGitReaderSetupHarness(changed, inherited, (side, file) =>
          gitBodyDigest(root, side === "before" ? before : after, file),
        )
      : null;
  const accepted = qualified ?? incoming ?? nodeep ?? setup;
  return accepted ? new Set([...alreadyQualified, ...accepted.files]) : alreadyQualified;
}
