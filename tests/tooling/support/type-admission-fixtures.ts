/** Original, public accuracy probes. Never substitute diagnostic agreement for type evidence. */
export const typeProofs = `export type Assert<T extends true> = T;
declare global { const useTemplateRef: typeof import("vue").useTemplateRef; }
export type Exact<A, B> =
  (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2)
    ? (<T>() => T extends B ? 1 : 2) extends (<T>() => T extends A ? 1 : 2) ? true : false
    : false;
export type IsAny<T> = 0 extends 1 & T ? true : false;
export type IsNever<T> = [T] extends [never] ? true : false;
export type IsUnknown<T> = IsAny<T> extends true ? false
  : unknown extends T ? [keyof T] extends [never] ? true : false : false;
export type IsReadonly<T, K extends keyof T> =
  Exact<Pick<T, K>, { -readonly [P in K]: T[P] }> extends true ? false : true;
`;

export const exposedChild = `<script setup lang="ts">
function focus(mode: 'soft' | 'hard'): 'focused' { return 'focused'; }
const state = { count: 1, ready: true };
defineExpose({ focus, state });
</script>
<template><button>{{ state.count }}</button></template>
`;

export const genericChild = `<script setup lang="ts" generic="T extends { id: number; label: string }">
const { value } = defineProps<{ value: T }>();
defineExpose({ value });
</script>
<template><span>{{ value.id }}</span></template>
`;

const imports = `import type { Assert, Exact, IsAny, IsNever, IsUnknown, IsReadonly } from './proofs';`;

export function proof(name: string, expected: string, readonly = true): string {
  return `type ${name}Value = typeof ${name}.value;
type ${name}Exact = Assert<Exact<${name}Value, ${expected}>>;
type ${name}Any = Assert<Exact<IsAny<${name}Value>, false>>;
type ${name}Unknown = Assert<Exact<IsUnknown<${name}Value>, false>>;
type ${name}Never = Assert<Exact<IsNever<${name}Value>, false>>;
type ${name}Readonly = Assert<Exact<IsReadonly<typeof ${name}, 'value'>, ${readonly}>>;
void (null as unknown as [${name}Exact, ${name}Any, ${name}Unknown, ${name}Never, ${name}Readonly]);`;
}

export function sfc(script: string, template: string, crlf = false): string {
  // The emoji is before every script anchor: byte offsets are not UTF-16 columns.
  const source = `<script setup lang="ts">
${imports}
const unicode = '🌱'; void unicode;
${script}
type AuthoredProofIsChecked = Assert<true>;
void (null as unknown as AuthoredProofIsChecked);
</script>
<template>${template}</template>
`;
  return crlf ? source.replaceAll("\n", "\r\n") : source;
}

export type AdmissionMutation = {
  name: string;
  before: string;
  after: string;
  token: string;
  code: number;
};

export type AdmissionCase = {
  name: string;
  source: string;
  mutations: AdmissionMutation[];
};

export const member = (
  name: string,
  before: string,
  after: string,
  token: string,
): AdmissionMutation => ({
  name,
  before,
  after,
  token,
  code: 2339,
});

export const templateRefAdmission: AdmissionCase[] = [
  {
    name: "TypedDom",
    source: sfc(
      `import { useTemplateRef, useTemplateRef as templateRef } from 'vue';
const button = useTemplateRef<HTMLButtonElement>('button');
const anchor = templateRef<HTMLAnchorElement>('not-in-template');
const input = templateRef<HTMLInputElement, 'input'>('input');
const override = templateRef<{ custom: number }>('override');
${proof("button", "HTMLButtonElement | null")}
${proof("anchor", "HTMLAnchorElement | null")}
${proof("input", "HTMLInputElement | null")}
${proof("override", "{ custom: number } | null")}
button.value?.click();
anchor.value?.href.toUpperCase();
input.value?.select();
override.value?.custom.toFixed();`,
      '<button ref="button" /><input ref="input" /><div ref="override" />',
    ),
    mutations: [
      member(
        "typed-member",
        "button.value?.click()",
        "button.value?.missingTypedButton()",
        "missingTypedButton",
      ),
      member(
        "alias-member",
        "anchor.value?.href.toUpperCase()",
        "anchor.value?.missingAliasedAnchor()",
        "missingAliasedAnchor",
      ),
      member(
        "two-type-arguments",
        "input.value?.select()",
        "input.value?.missingTypedInput()",
        "missingTypedInput",
      ),
      member(
        "authored-override",
        "override.value?.custom.toFixed()",
        "override.value?.missingOverride()",
        "missingOverride",
      ),
      {
        name: "readonly",
        before: "button.value?.click();",
        after: "button.value = null;",
        token: "value",
        code: 2540,
      },
      {
        name: "nullable",
        before: "button.value?.click();",
        after: "button.value.click();",
        token: "button.value",
        code: 18047,
      },
    ],
  },
  {
    name: "InferredDom",
    source: sfc(
      `import { useTemplateRef as r } from 'vue';
const button = r('button');
const input = r('input');
${proof("button", "HTMLButtonElement | null")}
${proof("input", "HTMLInputElement | null")}
void '🌱'; button.value?.click();
input.value?.select();`,
      '<button ref="button" /><input ref="input" />',
      true,
    ),
    mutations: [
      member(
        "inferred-button",
        "button.value?.click()",
        "button.value?.missingInferredButton()",
        "missingInferredButton",
      ),
      member(
        "inferred-input",
        "input.value?.select()",
        "input.value?.missingInferredInput()",
        "missingInferredInput",
      ),
    ],
  },
  {
    name: "InferredComponent",
    source: sfc(
      `import { useTemplateRef } from 'vue';
import Child from './Child.vue';
const child = useTemplateRef('child');
type ChildValue = NonNullable<typeof child.value>;
type ChildAny = Assert<Exact<IsAny<ChildValue>, false>>;
type ChildUnknown = Assert<Exact<IsUnknown<ChildValue>, false>>;
type ChildNever = Assert<Exact<IsNever<ChildValue>, false>>;
type ChildNullable = Assert<Exact<Extract<typeof child.value, null>, null>>;
type ChildReadonly = Assert<Exact<IsReadonly<typeof child, 'value'>, true>>;
type FocusArguments = Assert<Exact<Parameters<ChildValue['focus']>, [mode: 'soft' | 'hard']>>;
type FocusResult = Assert<Exact<ReturnType<ChildValue['focus']>, 'focused'>>;
type StateCount = Assert<Exact<ChildValue['state']['count'], number>>;
type StateReady = Assert<Exact<ChildValue['state']['ready'], boolean>>;
void (null as unknown as [ChildAny, ChildUnknown, ChildNever, ChildNullable, ChildReadonly, FocusArguments, FocusResult, StateCount, StateReady]);
child.value?.focus('soft');
child.value?.state.count.toFixed();`,
      '<Child ref="child" />',
    ),
    mutations: [
      {
        name: "exposed-argument",
        before: "child.value?.focus('soft')",
        after: "child.value?.focus('invalid-focus')",
        token: "'invalid-focus'",
        code: 2345,
      },
      member(
        "exposed-state",
        "child.value?.state.count.toFixed()",
        "child.value?.state.count.missingNumber()",
        "missingNumber",
      ),
    ],
  },
  {
    name: "Mixed",
    source: sfc(
      `import { useTemplateRef } from 'vue';
const inferred = useTemplateRef('button');
const explicit = useTemplateRef<{ custom: number }>('absent');
${proof("inferred", "HTMLButtonElement | null")}
${proof("explicit", "{ custom: number } | null")}
inferred.value?.click();
explicit.value?.custom.toFixed();`,
      '<button ref="button" />',
    ),
    mutations: [
      member(
        "mixed-inferred",
        "inferred.value?.click()",
        "inferred.value?.missingMixedButton()",
        "missingMixedButton",
      ),
      member(
        "mixed-explicit",
        "explicit.value?.custom.toFixed()",
        "explicit.value?.missingMixedExplicit()",
        "missingMixedExplicit",
      ),
    ],
  },
  {
    name: "LocalShadow",
    source: sfc(
      `function useTemplateRef<T>(_key: string): { value: T | null } { return { value: null }; }
const local = useTemplateRef<{ own: number }>('not-a-vue-ref');
${proof("local", "{ own: number } | null", false)}
local.value?.own.toFixed();
local.value = null;`,
      "<div />",
    ),
    mutations: [
      member(
        "local-shadow",
        "local.value?.own.toFixed()",
        "local.value?.missingLocalMember()",
        "missingLocalMember",
      ),
    ],
  },
  {
    name: "NestedTyped",
    source: sfc(
      `import { useTemplateRef } from 'vue';
function authoredScope() {
  const nested = useTemplateRef<HTMLAnchorElement>('nested-not-in-template');
  ${proof("nested", "HTMLAnchorElement | null")}
  nested.value?.href.toUpperCase();
}
void authoredScope;`,
      "<div />",
    ),
    mutations: [
      member(
        "nested-typed",
        "nested.value?.href.toUpperCase()",
        "nested.value?.missingNestedAnchor()",
        "missingNestedAnchor",
      ),
      {
        name: "nested-readonly",
        before: "nested.value?.href.toUpperCase();",
        after: "nested.value = null;",
        token: "value",
        code: 2540,
      },
    ],
  },
  {
    name: "DollarRefs",
    source:
      "<!-- @inferTemplateDollarRefs true -->\n" +
      sfc(
        `import { useTemplateRef } from 'vue';
const button = useTemplateRef<HTMLButtonElement>('button');
${proof("button", "HTMLButtonElement | null")}
button.value?.click();`,
        '<button ref="button" />{{ $refs.button.disabled.valueOf() }}',
      ),
    mutations: [
      member(
        "template-dollar-refs",
        "$refs.button.disabled.valueOf()",
        "$refs.button.missingDollarRef()",
        "missingDollarRef",
      ),
    ],
  },
  {
    name: "GenericComponent",
    source: sfc(
      `import { useTemplateRef } from 'vue';
import Generic from './Generic.vue';
const payload = { id: 1, label: 'ready', extra: true };
const generic = useTemplateRef('generic');
type GenericValue = NonNullable<typeof generic.value>['value'];
type GenericExact = Assert<Exact<GenericValue, typeof payload>>;
type GenericAny = Assert<Exact<IsAny<GenericValue>, false>>;
type GenericUnknown = Assert<Exact<IsUnknown<GenericValue>, false>>;
type GenericNever = Assert<Exact<IsNever<GenericValue>, false>>;
type GenericReadonly = Assert<Exact<IsReadonly<typeof generic, 'value'>, true>>;
void (null as unknown as [GenericExact, GenericAny, GenericUnknown, GenericNever, GenericReadonly]);
generic.value?.value.extra.valueOf();
generic.value?.value.id.toFixed();`,
      '<Generic ref="generic" :value="payload" />',
    ),
    mutations: [
      member(
        "generic-member",
        "generic.value?.value.extra.valueOf()",
        "generic.value?.value.missingGenericMember()",
        "missingGenericMember",
      ),
      member(
        "generic-property",
        "generic.value?.value.id.toFixed()",
        "generic.value?.value.id.missingGenericNumber()",
        "missingGenericNumber",
      ),
    ],
  },
];

export { mutatedAdmission } from "./type-admission-mutation.ts";
