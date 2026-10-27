import { member, sfc, type AdmissionCase } from "./type-admission-fixtures.ts";

// Unknown/never are legitimate authored types. Reject accidental widening while
// retaining their actual semantics instead of banning the spelling wholesale.
const unknownProof = (name: string) => `type Value = typeof ${name}.value;
type ValueExact = Assert<Exact<Value, unknown>>;
type ValueAny = Assert<Exact<IsAny<Value>, false>>;
type ValueUnknown = Assert<Exact<IsUnknown<Value>, true>>;
type ValueNever = Assert<Exact<IsNever<Value>, false>>;
type ValueReadonly = Assert<Exact<IsReadonly<typeof ${name}, 'value'>, true>>;`;

export const templateRefEdgeAdmission: AdmissionCase[] = [
  {
    name: "NestedUntyped",
    source: sfc(
      `import { useTemplateRef } from 'vue';
function later() {
  const target = useTemplateRef('not-in-template');
  ${unknownProof("target")}
  void target.value;
}
void later;`,
      "<div />",
    ),
    mutations: [
      member(
        "nested-unknown",
        "void target.value;",
        "target.value?.missingNestedUnknown();",
        "missingNestedUnknown",
      ),
      {
        name: "nested-untyped-readonly",
        before: "void target.value;",
        after: "target.value = null;",
        token: "value",
        code: 2540,
      },
    ],
  },
  {
    name: "ExplicitUnknown",
    source: sfc(
      `import { useTemplateRef as templateRef } from 'vue';
const target = templateRef<unknown>('not-in-template');
${unknownProof("target")}
void target.value;`,
      "<div />",
    ),
    mutations: [
      member(
        "explicit-unknown",
        "void target.value;",
        "target.value?.missingExplicitUnknown();",
        "missingExplicitUnknown",
      ),
      {
        name: "unknown-readonly",
        before: "void target.value;",
        after: "target.value = null;",
        token: "value",
        code: 2540,
      },
    ],
  },
  {
    name: "ExplicitNever",
    source: sfc(
      `import { useTemplateRef as templateRef } from 'vue';
const target = templateRef<never>('not-in-template');
type Value = typeof target.value;
type ValueExact = Assert<Exact<Value, null>>;
type ValueAny = Assert<Exact<IsAny<Value>, false>>;
type ValueUnknown = Assert<Exact<IsUnknown<Value>, false>>;
type ValueNever = Assert<Exact<IsNever<NonNullable<Value>>, true>>;
type ValueReadonly = Assert<Exact<IsReadonly<typeof target, 'value'>, true>>;
void target.value;`,
      "<div />",
    ),
    mutations: [
      member(
        "explicit-never",
        "void target.value;",
        "target.value?.missingExplicitNever();",
        "missingExplicitNever",
      ),
      {
        name: "never-readonly",
        before: "void target.value;",
        after: "target.value = null;",
        token: "value",
        code: 2540,
      },
    ],
  },
];
