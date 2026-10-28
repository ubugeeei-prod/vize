import { member, proof, sfc, type AdmissionCase } from "./type-admission-fixtures.ts";

// Unknown/never are legitimate authored types. Reject accidental widening while
// retaining their actual semantics instead of banning the spelling wholesale.
const unknownProof = (name: string) => `type Value = typeof ${name}.value;
type ValueExact = Assert<Exact<Value, unknown>>;
type ValueAny = Assert<Exact<IsAny<Value>, false>>;
type ValueUnknown = Assert<Exact<IsUnknown<Value>, true>>;
type ValueNever = Assert<Exact<IsNever<Value>, false>>;
type ValueReadonly = Assert<Exact<IsReadonly<typeof ${name}, 'value'>, true>>;
void (null as unknown as [ValueExact, ValueAny, ValueUnknown, ValueNever, ValueReadonly]);`;

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
void (null as unknown as [ValueExact, ValueAny, ValueUnknown, ValueNever, ValueReadonly]);
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
  {
    name: "GlobalNestedTyped",
    source: sfc(
      `function later() {
  const target = useTemplateRef<HTMLButtonElement>('not-in-template');
  ${proof("target", "HTMLButtonElement | null")}
  target.value?.click();
}
void later;`,
      "<div />",
    ),
    mutations: [
      member(
        "global-typed-member",
        "target.value?.click();",
        "target.value?.missingGlobalTyped();",
        "missingGlobalTyped",
      ),
      {
        name: "global-typed-readonly",
        before: "target.value?.click();",
        after: "target.value = null;",
        token: "value",
        code: 2540,
      },
    ],
  },
  {
    name: "EscapedGlobalUnknown",
    source: sfc(
      `function later() {
  const target = useTemplateRe\\u0066<unknown>('not-in-template');
  ${unknownProof("target")}
  void target.value;
}
void later;`,
      "<div />",
    ),
    mutations: [
      member(
        "escaped-unknown",
        "void target.value;",
        "target.value?.missingEscapedUnknown();",
        "missingEscapedUnknown",
      ),
      {
        name: "escaped-unknown-readonly",
        before: "void target.value;",
        after: "target.value = null;",
        token: "value",
        code: 2540,
      },
    ],
  },
  {
    name: "EscapedGlobalInferred",
    source: sfc(
      `const target = useTemplateRe\\u0066('target');
${proof("target", "HTMLButtonElement | null")}
target.value?.click();`,
      '<button ref="target" />',
    ),
    mutations: [
      member(
        "escaped-inferred",
        "target.value?.click();",
        "target.value?.missingEscapedInferred();",
        "missingEscapedInferred",
      ),
      {
        name: "escaped-inferred-readonly",
        before: "target.value?.click();",
        after: "target.value = null;",
        token: "value",
        code: 2540,
      },
    ],
  },
];
