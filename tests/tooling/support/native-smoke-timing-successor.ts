import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const timingEnvironment =
  "        env:\n          VIZE_SMOKE_TIMING_FILE: ${{ runner.temp }}/packed-smoke-timing.jsonl\n";
const selectors = [
  "      - name: Fresh install and runtime smoke published tarballs\n" +
    "        if: ${{ matrix.platform.target != 'linux-x64-gnu' || matrix.node-version != '24' }}\n",
  "      - name: Fresh packed Nuxt init, check, and LSP smoke\n" +
    "        if: ${{ matrix.platform.target == 'linux-x64-gnu' && matrix.node-version == '24' }}\n",
];
const retainedTiming = [
  "      - name: Retain packed smoke timing",
  "        if: always()",
  "        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7",
  "        with:",
  "          name: packed-smoke-timing-${{ matrix.platform.target }}-node${{ matrix.node-version }}",
  "          path: ${{ runner.temp }}/packed-smoke-timing.jsonl",
  "          if-no-files-found: warn",
  "          retention-days: 14",
  "",
].join("\n");
const selectedProducer = "\n  selected-host-native-producer:\n";

/** Build the exact timing-only successor over the sealed Babel workflow. */
export function packedSmokeTimingSuccessor(historical: string): string {
  assert.equal(
    createHash("sha256").update(historical).digest("hex"),
    "cb090990a287cd13949c5e38ed22d3a2619b3b64c7415b6593f171c02c02574a",
    "the complete Babel successor remains the sealed historical authority",
  );
  let expected = historical;
  for (const selector of selectors) {
    assert.equal(expected.split(selector).length, 2, "one original smoke selector");
    expected = expected.replace(selector, selector + timingEnvironment);
  }
  assert.equal(expected.split(selectedProducer).length, 2, "one selected producer boundary");
  expected = expected.replace(selectedProducer, retainedTiming + selectedProducer);
  return expected;
}

/** Admit only the two timing environments and one retained sidecar over a sealed workflow. */
export function assertPackedSmokeTimingSuccessor(historical: string, current: string): void {
  assert.equal(
    current,
    packedSmokeTimingSuccessor(historical),
    "all workflow bytes outside the exact timing observer stay sealed",
  );
}
