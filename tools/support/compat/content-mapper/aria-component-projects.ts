/** Original #11144 inputs and bounded authored negative/repair controls. */
import assert from "node:assert/strict";

export const reportedAriaChild = `<script setup lang="ts">
defineProps<{ size?: string | number }>();
</script>

<template>
  <div>{{ size }}</div>
</template>
`;
export const reportedAriaParent = `<script setup lang="ts">
import Child from "./child.vue";
</script>

<template>
  <Child aria-label="x" />
</template>
`;
export const reportedAriaConfig = {
  compilerOptions: {
    strict: true,
    module: "esnext",
    moduleResolution: "bundler",
    target: "esnext",
    jsx: "preserve",
    skipLibCheck: true,
    noEmit: true,
  },
  include: ["src/**/*.ts", "src/**/*.vue"],
  contentMappers: [{ package: "vize", extensions: [".vue"] }],
};
type Phase = { id: string; parent: string; expectedStdout: string; expectedStatus: number };
export type AriaProject = { id: string; child: string; phases: Phase[]; component: boolean };
const typeError = "Type 'false' is not assignable to type 'string | number | undefined'.";
export const ariaPropReference =
  "declare function accept(props: { size?: string | number }): void;\naccept({ size: false });\n";
export const ariaPropReferenceStdout = `reference.ts(2,10): error TS2322: ${typeError}\n`;
const variant = (source: string, unicode: boolean) =>
  unicode
    ? source.replace("  <Child", "  <!-- 日本 🧭 --><Child").replaceAll("\n", "\r\n")
    : source;

function error(parent: string) {
  const token = ':size="false"',
    start = parent.indexOf(token);
  assert.ok(start >= 0 && parent.indexOf(token, start + 1) < 0);
  // Both the literal key and the per-prop synthetic identifier map to the
  // authored attribute name. The independent same-SDK object check owns the
  // complete message and proves that the literal false must be rejected.
  const name = start + 1,
    lines = parent.slice(0, name).split(/\r\n|\n/u);
  return `src/parent.vue(${lines.length},${lines.at(-1)!.length + 1}): error TS2322: ${typeError}\n`;
}

export function ariaComponentProjects(): AriaProject[] {
  const clean = (id: string, parent: string, child = reportedAriaChild, component = true) => ({
    id,
    child,
    component,
    phases: [{ id: "valid", parent, expectedStdout: "", expectedStatus: 0 }],
  });
  const projects: AriaProject[] = [
    clean("original", reportedAriaParent),
    clean("aria-hidden", reportedAriaParent.replace('aria-label="x"', 'aria-hidden="true"')),
    clean(
      "bound-aria",
      reportedAriaParent
        .replace("</script>", 'const label = "x";\n</script>')
        .replace('aria-label="x"', ':aria-label="label"'),
    ),
    clean("no-props", reportedAriaParent, "<template><div /></template>\n"),
    clean(
      "camel-declared-aria",
      reportedAriaParent,
      reportedAriaChild
        .replace("size?: string | number", "ariaLabel?: string")
        .replace("{{ size }}", "{{ ariaLabel }}"),
    ),
    clean(
      "native-control",
      reportedAriaParent
        .replace('import Child from "./child.vue";\n', "")
        .replace("<Child", "<div"),
      reportedAriaChild,
      false,
    ),
    clean(
      "declared-size",
      reportedAriaParent.replace('aria-label="x"', 'aria-label="x" size="20"'),
    ),
    clean("original-crlf-unicode", variant(reportedAriaParent, true)),
  ];
  for (const unicode of [false, true]) {
    const valid = variant(
        reportedAriaParent.replace('aria-label="x"', 'aria-label="x" :size="20"'),
        unicode,
      ),
      invalid = valid.replace(':size="20"', ':size="false"');
    projects.push({
      id: `declared-prop-cycle${unicode ? "-crlf-unicode" : ""}`,
      child: unicode ? reportedAriaChild.replaceAll("\n", "\r\n") : reportedAriaChild,
      component: true,
      phases: [
        { id: "valid", parent: valid, expectedStdout: "", expectedStatus: 0 },
        { id: "invalid", parent: invalid, expectedStdout: error(invalid), expectedStatus: 2 },
        { id: "repaired", parent: valid, expectedStdout: "", expectedStatus: 0 },
      ],
    });
  }
  return projects;
}
