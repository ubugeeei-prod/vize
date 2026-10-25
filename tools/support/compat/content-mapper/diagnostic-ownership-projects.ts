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
};

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
  const expect = (source: string) =>
    source.replace(
      '  <GenericForm :initial-state="{ role: null',
      '  <!-- @vue-expect-error -->\n  <GenericForm :initial-state="{ role: null',
    );
  return [
    { ...base, id: "form-expect-used", replaceParent: expect, diagnostics: [] },
    {
      ...base,
      id: "form-expect-unused",
      replaceParent: (source) => expect(source).replace("role: null", "role: 'ok'"),
      diagnostics: [
        {
          file: "src/Form.art.vue",
          line: 13,
          column: 3,
          severity: 1,
          code: 2578,
          message: "Unused '@vue-expect-error' directive.",
        },
      ],
      mapperStdout: "src/Form.art.vue(13,8): error vize4: Unused '@vue-expect-error' directive\n",
    },
    {
      ...base,
      id: "form-callback-expect-used",
      replaceParent: (source) =>
        expect(source).replace(
          ':initial-state="{ role: null, tags: [] }" :schema="schema"',
          ':initial-state="{ role: null, tags: [] }" :schema="schema" :validate-form-field-on="(field) => !!field"',
        ),
      diagnostics: [],
    },
  ];
}
