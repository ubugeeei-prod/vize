/** Handwritten #10738 controls. Provider messages come from the same-SDK TS run. */
export type SourceSpan = {
  file: string;
  start: number;
  end: number;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  text: string;
};
type Role = "callback-assignment" | "callback-body" | "interpolation" | "unused-expectation";
type ExpectedRole = { role: Role; code: number; span: SourceSpan };
export type CallbackOwnerProject = {
  id: string;
  inlineInputs: Record<string, string>;
  /** Recompute these UTF16 offsets and columns after applying runner variants. */
  coordinateEncoding: "utf16";
  authoredBindings: { attribute: SourceSpan; name: SourceSpan; value: SourceSpan }[];
  reference: {
    file: string;
    source: string;
    bindings: SourceSpan[];
    expectedRoles: ExpectedRole[] | null;
  };
  expected: {
    diagnostics: ExpectedRole[] | null;
    /** Additional to diagnostics: native anchors the comment; mapper anchors @vue. */
    expectation: null | {
      state: "used" | "unused";
      comment: SourceSpan;
      token: SourceSpan;
      reference: SourceSpan;
      nativeUnusedCode: 2578;
      mapperUnusedCode: 4;
    };
  };
  qualification:
    | "same-provider-reference-pending"
    | "known-10739-frontier"
    | "exact-before-head-preservation-pending";
  comparison: "reference-parity" | "complete-before-head-packets";
  /** The mapper packet stays unqualified under this separate issue. */
  mapperFrontier?: 10871;
  sequence?: { family: "callback-return"; phase: "valid" | "error" | "repaired" };
  scopeProbes?: readonly ["nested", "branch", "loop"];
};

const parentFile = "src/Min.vue";
const referenceFile = "reference.ts";

/** SFC render checks run inside setup; retain the original module source too. */
export function callbackSetupReferencePrefix(source: string): string {
  const module = "export {};\n";
  if (!source.startsWith(module))
    throw new Error("reference must retain its authored module header");
  return module + "function setup() {\n" + source.slice(module.length);
}

export function callbackSetupReference(source: string): string {
  return callbackSetupReferencePrefix(source) + "}\nvoid setup;\n";
}
const config =
  JSON.stringify(
    {
      compilerOptions: {
        strict: true,
        noEmit: true,
        module: "preserve",
        moduleResolution: "bundler",
        types: [],
        lib: ["ESNext", "DOM"],
      },
      include: ["src"],
    },
    null,
    2,
  ) + "\n";
const setup = `<script setup lang="ts">
import Comp from './Comp.vue'
const state: { value?: number } = {}
state.value = 1
const noop = () => {}
</script>
<template>
`;
const referenceSetup = `export {};
const state: { value?: number } = {};
state.value = 1;
const noop = () => {};
void noop;
`;

/** Exact occurrence selection is authored input geometry, never generated-code recognition. */
function span(file: string, source: string, text: string, occurrence = 0): SourceSpan {
  let start = -1;
  for (let index = 0; index <= occurrence; index++) {
    start = source.indexOf(text, start + 1);
    if (start < 0) throw new Error(`Missing authored span ${file}: ${text} #${occurrence}`);
  }
  const end = start + text.length;
  const point = (offset: number) => {
    const lines = source.slice(0, offset).split("\n");
    return { line: lines.length, column: (lines.at(-1)?.length ?? 0) + 1 };
  };
  const first = point(start),
    last = point(end);
  return { file, start, end, ...first, endLine: last.line, endColumn: last.column, text };
}
function binding(source: string, callback: string, occurrence = 0) {
  const text = `:pick="${callback}"`;
  const attribute = span(parentFile, source, text, occurrence);
  const at = (offset: number, value: string) => {
    const prior = source.slice(0, attribute.start + offset);
    const count = prior.split(value).length - 1;
    return span(parentFile, source, value, count);
  };
  return { attribute, name: at(1, "pick"), value: at(7, callback) };
}
type Case = {
  id: string;
  callback: string;
  returns?: "boolean" | "number";
  before?: string;
  after?: string;
  referenceBefore?: string;
  referenceAfter?: string;
  directive?: "used" | "unused";
  errors?: readonly ("callback-assignment" | "callback-body" | "interpolation")[];
  frontier?: boolean;
  mapperFrontier?: 10871;
  sequence?: CallbackOwnerProject["sequence"];
};
function project(input: Case): CallbackOwnerProject {
  const returns = input.returns ?? "boolean";
  const component = `<script setup lang="ts">
defineProps<{ pick: (x: number) => ${returns} }>()
defineEmits<{ save: [] }>()
</script>
<template><div /></template>
`;
  const source =
    setup +
    (input.before ?? "") +
    (input.directive ? "  <!-- @vue-expect-error -->\n" : "") +
    `  <Comp :pick="${input.callback}" @save="noop" />\n` +
    (input.after ?? "  <div>{{ delete state.value }}</div>\n") +
    "</template>\n";
  const reference =
    referenceSetup +
    (input.referenceBefore ?? "") +
    (input.directive ? "// @ts-expect-error\n" : "") +
    `const owner: (x: number) => ${returns} = ${input.callback};\nvoid owner;\n` +
    (input.referenceAfter ?? "void (delete state.value);\n");
  const authored = binding(source, input.callback);
  const diagnostics = (input.errors ?? []).map((role): ExpectedRole => ({
    role,
    code: role === "callback-assignment" ? 2322 : 2339,
    span:
      role === "callback-assignment"
        ? authored.name
        : span(
            parentFile,
            source,
            "missing",
            role === "interpolation" && input.errors?.includes("callback-body") ? 1 : 0,
          ),
  }));
  const referenceRoles = diagnostics.map(({ role, code }): ExpectedRole => ({
    role,
    code,
    span:
      role === "callback-assignment"
        ? span(referenceFile, reference, "owner")
        : span(
            referenceFile,
            reference,
            "missing",
            role === "interpolation" && input.errors?.includes("callback-body") ? 1 : 0,
          ),
  }));
  const expectation = input.directive
    ? {
        state: input.directive,
        comment: span(parentFile, source, "<!-- @vue-expect-error -->"),
        token: span(parentFile, source, "@vue-expect-error"),
        reference: span(referenceFile, reference, "// @ts-expect-error"),
        nativeUnusedCode: 2578 as const,
        mapperUnusedCode: 4 as const,
      }
    : null;
  if (expectation?.state === "unused")
    referenceRoles.unshift({
      role: "unused-expectation",
      code: 2578,
      span: expectation.reference,
    });
  return {
    id: `callback-owner-${input.id}`,
    inlineInputs: { "tsconfig.json": config, "src/Comp.vue": component, [parentFile]: source },
    coordinateEncoding: "utf16",
    authoredBindings: [authored],
    reference: {
      file: referenceFile,
      source: reference,
      bindings: [span(referenceFile, reference, "owner")],
      expectedRoles: referenceRoles,
    },
    expected: { diagnostics, expectation },
    qualification: input.frontier ? "known-10739-frontier" : "same-provider-reference-pending",
    comparison: "reference-parity",
    ...(input.mapperFrontier ? { mapperFrontier: input.mapperFrontier } : {}),
    ...(input.sequence ? { sequence: input.sequence } : {}),
  };
}

/** The caller applies LF/CRLF and Unicode prefixes and captures all hosts. */
export function callbackOwnerProjects(): CallbackOwnerProject[] {
  const query = "(field: typeof state.value) => !!field";
  const projects = [
    project({ id: "parameter-query-delete-after", callback: query }),
    project({
      id: "indexed-parameter-query-delete-after",
      callback: "(field: typeof state['value']) => !!field",
    }),
    project({
      id: "return-query-delete-after",
      returns: "number",
      callback: "(field: number): typeof state.value => field",
    }),
    project({
      id: "body-as-query-delete-after",
      returns: "number",
      callback: "(field: number) => field as typeof state.value",
    }),
    project({
      id: "body-satisfies-query-delete-after",
      returns: "number",
      callback: "(field: number) => (field satisfies typeof state.value)",
    }),
    project({
      id: "parameter-query-delete-before",
      callback: query,
      before: "  <div>{{ delete state.value }}</div>\n",
      after: "  <div>{{ state.value }}</div>\n",
      referenceBefore: "void (delete state.value);\n",
      referenceAfter: "void state.value;\n",
      errors: ["callback-assignment"],
    }),
    project({
      id: "return-valid",
      callback: "(field: number) => !!field",
      sequence: { family: "callback-return", phase: "valid" },
    }),
    project({
      id: "return-error",
      callback: "(field: number) => { field.missing; return 'bad' }",
      errors: ["callback-assignment", "callback-body"],
      frontier: true,
      sequence: { family: "callback-return", phase: "error" },
    }),
    project({
      id: "return-repaired",
      callback: "(field: number) => field > 0",
      sequence: { family: "callback-return", phase: "repaired" },
    }),
    project({
      id: "expect-used",
      callback: "(field: number) => 'bad'",
      directive: "used",
      frontier: true,
    }),
    project({
      id: "expect-unused-with-foreign-interpolation",
      callback: query,
      directive: "unused",
      after: "  <div>{{ delete state.value }}</div>\n  <div>{{ state.missing }}</div>\n",
      referenceAfter: "void (delete state.value);\nvoid state.missing;\n",
      errors: ["interpolation"],
      mapperFrontier: 10871,
    }),
    project({
      id: "prior-v-if-chain",
      callback: query,
      before:
        '  <div v-if="state.value > 2">a</div>\n  <div v-else-if="state.value > 1">b</div>\n  <div v-else>c</div>\n',
      referenceBefore:
        "if (state.value > 2) { void 'a'; } else if (state.value > 1) { void 'b'; } else { void 'c'; }\n",
    }),
  ];
  const unsupported = project({ id: "unsupported-scope-preservation", callback: query });
  const source =
    setup +
    `  <section>
    <Comp :pick="${query}" @save="noop" />
    <div>{{ delete state.value }}</div>
  </section>
  <section v-if="state.value">
    <Comp :pick="${query}" @save="noop" />
    <div>{{ delete state.value }}</div>
  </section>
  <section v-for="entry in [state]">
    <Comp :pick="(field: typeof entry.value) => !!field" @save="noop" />
    <div>{{ delete entry.value }}</div>
  </section>
</template>
`;
  const reference =
    referenceSetup +
    `{
  const nested: (x: number) => boolean = ${query};
  void nested;
  void (delete state.value);
}
if (state.value) {
  const branch: (x: number) => boolean = ${query};
  void branch;
  void (delete state.value);
}
for (const entry of [state]) {
  const loop: (x: number) => boolean = (field: typeof entry.value) => !!field;
  void loop;
  void (delete entry.value);
}
`;
  unsupported.inlineInputs[parentFile] = source;
  unsupported.authoredBindings = [
    binding(source, query),
    binding(source, query, 1),
    binding(source, "(field: typeof entry.value) => !!field"),
  ];
  unsupported.reference = {
    file: referenceFile,
    source: reference,
    bindings: ["nested", "branch", "loop"].map((name) => span(referenceFile, reference, name)),
    expectedRoles: null,
  };
  unsupported.expected = { diagnostics: null, expectation: null };
  unsupported.qualification = "exact-before-head-preservation-pending";
  unsupported.comparison = "complete-before-head-packets";
  unsupported.scopeProbes = ["nested", "branch", "loop"];
  // No reference-parity claim: capture its complete reference vector, but require
  // byte-exact before/head transform and native packets for unsupported scopes.
  projects.push(unsupported);
  const preserve = (
    id: string,
    sibling = "",
    declarations = "",
    before = "",
    referenceBefore = "",
    vueOnly = "",
    opening = "",
  ) => {
    const control = project({
      id: `unsupported-${id}`,
      callback: query,
      before,
      referenceBefore: declarations + referenceBefore,
      referenceAfter: (sibling ? `void (${sibling});\n` : "") + "void (delete state.value);\n",
    });
    let source = control.inlineInputs[parentFile].replace(
      "const noop = () => {}\n",
      declarations + vueOnly + "const noop = () => {}\n",
    );
    if (sibling)
      source = source.replace(`:pick="${query}"`, `:pick="${query}" :other="${sibling}"`);
    if (opening) source = source.replace("<Comp :pick=", `<Comp ${opening} :pick=`);
    control.inlineInputs[parentFile] = source;
    control.inlineInputs["src/Comp.vue"] = control.inlineInputs["src/Comp.vue"].replace(
      "pick: (x: number) => boolean",
      "pick: (x: number) => boolean; other?: unknown",
    );
    control.authoredBindings = [binding(source, query)];
    control.reference.expectedRoles = null;
    control.expected = { diagnostics: null, expectation: null };
    control.qualification = "exact-before-head-preservation-pending";
    control.comparison = "complete-before-head-packets";
    projects.push(control);
  };
  preserve("sibling-assignment", "(state.value = undefined)");
  preserve("sibling-update", "state.value++");
  preserve("sibling-call", "mutate()", "function mutate() { delete state.value }\n");
  preserve(
    "sibling-assertion-call",
    "assertNumber(state.value)",
    "function assertNumber(value: unknown): asserts value is number { if (typeof value !== 'number') throw new Error('number') }\n",
  );
  preserve(
    "sibling-member-getter-read",
    "reader.value",
    "const reader = { get value() { delete state.value; return 1 } }\n",
  );
  preserve("sibling-object-getter-literal", "{ get value() { delete state.value; return 1 } }");
  preserve(
    "prior-root-v-for-mutation",
    "",
    "",
    '  <div v-for="entry in [1]">{{ entry }}{{ delete state.value }}</div>\n',
    "for (const entry of [1]) { void entry; void (delete state.value); }\n",
  );
  preserve(
    "prior-root-slot-deferred-mutation",
    "",
    "",
    '  <slot :other="() => { delete state.value }" />\n',
    "const slotPayload = { other: () => { delete state.value; } };\nvoid slotPayload;\n",
    "defineSlots<{ default(props: { other: () => void }): any }>()\n",
  );
  preserve(
    "dynamic-event-argument-mutation",
    "",
    "",
    "",
    "void (delete state.value);\n",
    "",
    '@[delete(state.value)]="noop"',
  );
  preserve(
    "argumentless-event-object-mutation",
    "",
    "",
    "",
    "void (delete state.value);\n",
    "",
    'v-on="(delete state.value, {})"',
  );
  return projects;
}
