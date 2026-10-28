import { sfc, proof, member, type AdmissionMutation } from "./type-admission-fixtures.ts";

export type GlobalRefAdmission = {
  name: string;
  ambient: string;
  vueComposable?: boolean;
  source: string;
  mutation: AdmissionMutation;
};

const generic =
  "<T = string, K extends string = string>(key: K): import('vue').ShallowRef<T | null>";
const callable = generic.replace("): import", ") => import");
const importedWitnesses = `type ImportedWitnesses = [Exact<string, string>, IsAny<string>, IsNever<never>, IsUnknown<unknown>, IsReadonly<{ readonly value: 1 }, 'value'>];
void (null as unknown as ImportedWitnesses);`;
const mutableScript = `const target = useTemplateRef('not-a-template-ref');
const explicit = useTemplateRef<{ count: number }>('target');
${proof("target", "string | null", false)}
${proof("explicit", "{ count: number } | null", false)}
target.value = 'mutable';
explicit.value = { count: 2 };
target.value.toUpperCase();`;
const mutableMutation = member(
  "custom-default-member",
  "target.value.toUpperCase();",
  "target.value.missingCustomDefault();",
  "missingCustomDefault",
);

export const globalRefAdmission: GlobalRefAdmission[] = [
  ...(
    [
      ["CustomConst", `const useTemplateRef: ${callable};`],
      ["CustomVar", `var useTemplateRef: ${callable};`],
      ["CustomFunction", `function useTemplateRef${generic};`],
    ] as const
  ).map(([name, ambient]) => ({
    name,
    ambient,
    source: sfc(mutableScript, '<button ref="target" />'),
    mutation: mutableMutation,
  })),
  {
    name: "VueGlobalKey",
    ambient: "const useTemplateRef: typeof import('vue').useTemplateRef;",
    vueComposable: true,
    source: sfc(
      `const target = useTemplateRef('target');
${importedWitnesses}
void target;`,
      '<button ref="target" />',
    ),
    mutation: {
      name: "real-vue-global-missing-key",
      before: "useTemplateRef('target')",
      after: "useTemplateRef('missingVueKey')",
      token: "'missingVueKey'",
      code: 2339,
    },
  },
  {
    name: "CustomUnknown",
    ambient: `const useTemplateRef: <T = unknown, K extends string = string>(key: K) => import('vue').ShallowRef<T | null>;`,
    source: sfc(
      `const target = useTemplateRef('not-a-template-ref');
type Value = Assert<Exact<typeof target.value, unknown>>;
type NotAny = Assert<Exact<IsAny<typeof target.value>, false>>;
type Unknown = Assert<Exact<IsUnknown<typeof target.value>, true>>;
type NotNever = Assert<Exact<IsNever<typeof target.value>, false>>;
type Mutable = Assert<Exact<IsReadonly<typeof target, 'value'>, false>>;
void (null as unknown as [Value, NotAny, Unknown, NotNever, Mutable]);
target.value = 'mutable';
void target.value;`,
      '<button ref="target" />',
    ),
    mutation: member(
      "custom-unknown-member",
      "void target.value;",
      "target.value?.missingCustomUnknown();",
      "missingCustomUnknown",
    ),
  },
  {
    name: "CustomReadonlyDefault",
    ambient: `const useTemplateRef: <T = string, K extends string = string>(key: K) => Readonly<import('vue').ShallowRef<T | null>>;`,
    source: sfc(
      `const target = useTemplateRef('not-a-template-ref');
${proof("target", "string | null")}
target.value?.toUpperCase();`,
      '<button ref="target" />',
    ),
    mutation: {
      name: "custom-readonly",
      before: "target.value?.toUpperCase();",
      after: "target.value = null;",
      token: "value",
      code: 2540,
    },
  },
  {
    name: "CustomOverloads",
    ambient: `function useTemplateRef(key: 'alpha'): { value: { marker: 'alpha' } };
function useTemplateRef(key: string): { value: { marker: 'fallback' } };`,
    source: sfc(
      `const target = useTemplateRef('alpha');
${proof("target", "{ marker: 'alpha' }", false)}
target.value = { marker: 'alpha' };
target.value.marker.toUpperCase();`,
      '<button ref="target" />',
    ),
    mutation: member(
      "custom-overload-member",
      "target.value.marker.toUpperCase();",
      "target.value.missingOverload();",
      "missingOverload",
    ),
  },
  ...(
    [
      ["MissingTypedImport", "<HTMLButtonElement>"],
      ["MissingInferredImport", ""],
    ] as const
  ).map(([name, parameters]) => ({
    name,
    ambient: "",
    source: sfc(
      `import { useTemplateRef } from 'vue';
const target = useTemplateRef${parameters}('target');
${importedWitnesses}
void target;`,
      '<button ref="target" />',
    ),
    mutation: {
      name: "missing-runtime-import",
      before: `import { useTemplateRef } from 'vue';\nconst target = useTemplateRef${parameters}('target');`,
      after: `// runtime import deliberately removed\nconst target = useTemplateRef${parameters}('target');`,
      token: "useTemplateRef",
      code: 2304,
    },
  })),
];
