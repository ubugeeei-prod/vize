/** Independently authored control vectors for component input ownership. */
export type Diagnostic = {
  file: string;
  line: number;
  column: number;
  severity: number;
  code: number;
  message: string;
};
export type Project = {
  id: string;
  inputs: { file: string; source: string }[];
  diagnostics: Diagnostic[];
  replaceParent?: (source: string) => string;
  mapperStdout?: string;
  ordinaryForm?: boolean;
  inlineInputs?: Record<string, string>;
};

/** Keep the reviewer's authored program, including the optional-property CFA. */
export function relocationControls(): Project[] {
  const setup = `<script setup lang="ts">
import Comp from './Comp.vue'
const state: { value?: number } = {}
state.value = 1
const noop = () => {}
</script>
<template>
`;
  const component = `<script setup lang="ts">
defineProps<{ pick: (x: number) => boolean }>()
defineEmits<{ save: [] }>()
</script>
<template><div /></template>
`;
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
  return [
    ["parameter-type-query-delete", "(field: typeof state.value) => !!field", "delete state.value"],
    [
      "parameter-indexed-type-query-delete",
      "(field: typeof state['value']) => !!field",
      "delete state.value",
    ],
    ["return-type-query-delete", "(field): typeof state.value => field", "delete state.value"],
    ["body-type-query-delete", "(field) => field as typeof state.value", "delete state.value"],
    ["parameter-type-query-stable", "(field: typeof state.value) => !!field", "state.value"],
    ["untyped-delete", "(field) => !!field", "delete state.value"],
    ["untyped-stable", "(field) => !!field", "state.value"],
  ].flatMap(([id, callback, effect]) =>
    [false, true].map((expected) => ({
      id: `relocation-${id}-${expected ? "expect" : "ordinary"}`,
      inputs: [],
      inlineInputs: {
        "tsconfig.json": config,
        "src/Comp.vue": component,
        "src/Min.vue":
          setup +
          (expected ? "  <!-- @vue-expect-error -->\n" : "") +
          `  <Comp :pick="${callback}" @save="noop" />\n  <div>{{ ${effect} }}</div>\n</template>\n`,
      },
      diagnostics: [],
    })),
  );
}

export function directiveControls(base: Project): Project[] {
  const expect = (source: string) =>
    source.replace("  <Child3", "  <!-- @vue-expect-error -->\n  <Child3");
  const slotError: Diagnostic = {
    file: "src/Min.vue",
    line: 9,
    column: 38,
    severity: 1,
    code: 2339,
    message: "Property 'missing' does not exist on type 'string[]'.",
  };
  return [
    { ...base, id: "expect-used", replaceParent: expect, diagnostics: [] },
    {
      ...base,
      id: "expect-unused",
      replaceParent: (source) => expect(source).replace("items[0].length", "items.length"),
      diagnostics: [
        {
          file: "src/Min.vue",
          line: 7,
          column: 3,
          severity: 1,
          code: 2578,
          message: "Unused '@vue-expect-error' directive.",
        },
      ],
      mapperStdout: "src/Min.vue(7,8): error vize4: Unused '@vue-expect-error' directive\n",
    },
    {
      ...base,
      id: "expect-slot-body",
      replaceParent: (source) =>
        expect(source).replace("<span>x</span>", "<span>{{ items.missing }}</span>"),
      diagnostics: [slotError],
    },
    {
      ...base,
      id: "ignore-slot-body",
      replaceParent: (source) =>
        expect(source)
          .replace("@vue-expect-error", "@vue-ignore")
          .replace("<span>x</span>", "<span>{{ items.missing }}</span>"),
      diagnostics: [slotError],
    },
    {
      ...base,
      id: "expect-foreign-sibling",
      replaceParent: (source) =>
        expect(source).replace("  </Child3>", "  </Child3>\n  <div>{{ items.missing }}</div>"),
      diagnostics: [{ ...slotError, line: 11, column: 17 }],
    },
  ];
}

export function formDirectiveControls(base: Project): Project[] {
  const ordinary = (source: string) =>
    source
      .replace("<art>", "<template>")
      .replace("</art>", "</template>")
      .replaceAll(/<variant name="[0-9]+">|<\/variant>/gu, "");
  const expect = (source: string) =>
    ordinary(source).replace(
      '  <GenericForm :initial-state="{ role: null',
      '  <!-- @vue-expect-error -->\n  <GenericForm :initial-state="{ role: null',
    );
  const callback = (source: string) =>
    expect(source).replace(
      ':initial-state="{ role: null, tags: [] }" :schema="schema"',
      ':initial-state="{ role: null, tags: [] }" :schema="schema" :validate-form-field-on="(field) => !!field"',
    );
  return [
    { ...base, ordinaryForm: true, id: "form-expect-used", replaceParent: expect, diagnostics: [] },
    {
      ...base,
      id: "form-expect-unused",
      ordinaryForm: true,
      replaceParent: (source) => expect(source).replace("role: null", "role: 'ok'"),
      diagnostics: [
        {
          file: "src/Form.vue",
          line: 13,
          column: 3,
          severity: 1,
          code: 2578,
          message: "Unused '@vue-expect-error' directive.",
        },
      ],
      mapperStdout: "src/Form.vue(13,8): error vize4: Unused '@vue-expect-error' directive\n",
    },
    {
      ...base,
      id: "form-callback-expect-used",
      ordinaryForm: true,
      replaceParent: callback,
      diagnostics: [],
    },
    {
      ...base,
      id: "form-callback-expect-unused",
      ordinaryForm: true,
      replaceParent: (source) => callback(source).replace("role: null", "role: 'ok'"),
      diagnostics: [
        {
          file: "src/Form.vue",
          line: 13,
          column: 3,
          severity: 1,
          code: 2578,
          message: "Unused '@vue-expect-error' directive.",
        },
      ],
      mapperStdout: "src/Form.vue(13,8): error vize4: Unused '@vue-expect-error' directive\n",
    },
    {
      ...base,
      id: "form-callback-expect-foreign-sibling",
      ordinaryForm: true,
      replaceParent: (source) => {
        const lines = callback(source).split("\n");
        const owner = lines.findIndex((line) => line.includes("role: null"));
        if (owner < 0) throw new Error("missing independently authored middle Form owner");
        lines.splice(owner + 1, 0, "  <div>{{ schema.missing }}</div>");
        return lines.join("\n");
      },
      diagnostics: [
        {
          file: "src/Form.vue",
          line: 15,
          column: 18,
          severity: 1,
          code: 2339,
          message:
            "Property 'missing' does not exist on type 'Schema<{ role: string; tags: string[]; }>'.",
        },
      ],
    },
  ];
}
