import {
  authoredPosition as position,
  authoredRange as range,
} from "../../tooling/support/lsp/authored-ranges.ts";

export const expectedSymbols = [
  {
    kind: 2,
    name: "template",
    // `</template>` on line 14 is 11 characters wide.
    range: { start: position(7, 0), end: position(14, 11) },
    selectionRange: range(7, 1, 9),
  },
  {
    detail: "ts",
    kind: 2,
    name: "script setup",
    // `</script>` on line 5 is 9 characters wide.
    range: { start: position(0, 0), end: position(5, 9) },
    selectionRange: range(0, 1, 7),
  },
  {
    kind: 2,
    name: "style scoped",
    // `<style scoped></style>` is a single line, 22 characters wide.
    range: { start: position(16, 0), end: position(16, 22) },
    selectionRange: range(16, 1, 6),
  },
];

export const expectedFolding = [
  { collapsedText: "template", startLine: 7, endLine: 13, kind: "region" },
  { collapsedText: "script setup", startLine: 0, endLine: 4, kind: "region" },
  { startLine: 10, endLine: 12 },
];
