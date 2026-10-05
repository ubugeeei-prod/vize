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
    children: [
      {
        kind: 19,
        name: "h1",
        range: { start: position(8, 2), end: position(8, 24) },
        selectionRange: range(8, 3, 5),
      },
      {
        kind: 19,
        name: "button",
        range: { start: position(9, 2), end: position(9, 45) },
        selectionRange: range(9, 3, 9),
      },
      {
        kind: 19,
        name: "p",
        range: { start: position(10, 2), end: position(13, 6) },
        selectionRange: range(10, 3, 4),
        children: [
          {
            kind: 19,
            name: "a",
            range: { start: position(11, 10), end: position(11, 83) },
            selectionRange: range(11, 11, 12),
          },
        ],
      },
    ],
  },
  {
    detail: "ts",
    kind: 2,
    name: "script setup",
    // `</script>` on line 5 is 9 characters wide.
    range: { start: position(0, 0), end: position(5, 9) },
    selectionRange: range(0, 1, 7),
    children: [
      {
        kind: 14,
        name: "visitCount",
        range: { start: position(3, 0), end: position(3, 25) },
        selectionRange: range(3, 6, 16),
      },
      {
        kind: 14,
        name: "doubled",
        range: { start: position(4, 0), end: position(4, 52) },
        selectionRange: range(4, 6, 13),
      },
    ],
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
