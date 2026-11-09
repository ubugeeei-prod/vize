import assert from "node:assert/strict";

// CI jobs stage only the native binaries they built through one script, since
// @napi-rs/cli 3.10 `pre-publish` rejects every empty foreign platform package.
// Sealed workflow chains invert each staging step to its former text.
const stage = "node tools/support/github/stage-built-native-packages.ts npm/native";
const createDirs = "vp exec napi create-npm-dirs";
const prePublish = "vp exec napi pre-publish -t npm --no-gh-release --skip-optional-publish";
const copyHostBinary = [
  "for binary in vize-vitrine.*.node; do",
  '  [ -f "$binary" ] || continue',
  '  target="${binary#vize-vitrine.}"',
  '  target="${target%.node}"',
  '  if [ -d "npm/$target" ]; then',
  '    cp "$binary" "npm/$target/"',
  "  fi",
  "done",
];
const smokeCopyNote = [
  "# `napi pre-publish --skip-optional-publish` does not move the host",
  "# platform's .node binary into its per-platform npm subdir. Without",
  "# that move, the per-platform tarball that the smoke test installs is",
  "# empty of the actual binding, and the umbrella package fails to load",
  "# at runtime. Copy the host's .node into the matching subdir so the",
  "# smoke test exercises a real install + runtime path.",
];
const muslEnv = "        env:\n          NATIVE_PLATFORM: ${{ matrix.platform }}\n";
const lines = (commands: readonly string[]) =>
  commands.map((command) => `          ${command}\n`).join("");
const inNative = (commands: readonly string[]) =>
  `        working-directory: npm/native\n        run: |\n${lines(commands)}`;
const stagedRun = `        run: ${stage}\n`;

type Step = readonly [current: string, former: string];

const conformanceSteps: readonly Step[] = [
  [stagedRun, inNative([createDirs, prePublish, ...copyHostBinary])],
  [
    lines([stage]),
    lines([
      `(cd npm/native && ${createDirs})`,
      `(cd npm/native && ${prePublish})`,
      "cp npm/native/vize-vitrine.linux-x64-gnu.node npm/native/npm/linux-x64-gnu/",
    ]),
  ],
];

const nativeSmokeSteps: readonly Step[] = [
  [stagedRun, inNative([createDirs, prePublish, ...smokeCopyNote, ...copyHostBinary])],
  [
    `${muslEnv}        run: |\n${lines([
      stage,
      'test -f "npm/native/npm/$NATIVE_PLATFORM/vize-vitrine.$NATIVE_PLATFORM.node"',
    ])}`,
    `        working-directory: npm/native\n${muslEnv}        run: |\n${lines([
      createDirs,
      prePublish,
      'test -f "vize-vitrine.$NATIVE_PLATFORM.node"',
      'cp "vize-vitrine.$NATIVE_PLATFORM.node" "npm/$NATIVE_PLATFORM/"',
    ])}`,
  ],
];

function invert(workflow: string, steps: readonly Step[]): string {
  for (const [current, former] of steps) {
    assert.equal(workflow.split(current).length, 2, `one staged native step: ${current.trim()}`);
    workflow = workflow.replace(current, () => former);
  }
  return workflow;
}

/** Restores content-mapper-conformance.yml's former napi pre-publish staging. */
export function restoreConformanceNativeStaging(workflow: string): string {
  return invert(workflow, conformanceSteps);
}

/** Restores native-smoke.yml's former napi pre-publish staging. */
export function restoreNativeSmokeStaging(workflow: string): string {
  return invert(workflow, nativeSmokeSteps);
}
