/** Version of the machine-readable SFC authoring contract shape. */
export const VIZE_UI_SFC_AUTHORING_CONTRACT_SCHEMA_VERSION = 1;

/** One rule that the SFC component authoring audit can report. */
export interface SfcAuthoringRuleContract {
  /** Stable identifier emitted by the audit gate and consumed by CI tooling. */
  readonly id: string;

  /** Short label for documentation and generated reports. */
  readonly title: string;

  /** Normative requirement for a public source-owned component. */
  readonly requirement: string;

  /** Required artifact pattern or authored source fact that satisfies the rule. */
  readonly evidence: readonly string[];

  /** Actionable remediation shown to component authors. */
  readonly remediation: string;
}

/** One quality gate in the source-owned component definition of done. */
export interface SfcQualityGateContract<RuleId extends string = string> {
  /** Stable identifier for manifests, CI policy, and generated documentation. */
  readonly id: string;

  /** Short label for documentation and generated reports. */
  readonly title: string;

  /** Why this gate exists in the component contract. */
  readonly requirement: string;

  /** Concrete artifacts or checks that provide reviewable evidence. */
  readonly evidence: readonly string[];

  /** Authoring-audit rules that enforce this gate in `auditComponentAuthoring`. */
  readonly enforcedByRules: readonly RuleId[];
}

/** Rule catalog emitted by the SFC component authoring gate. */
export const VIZE_UI_SFC_AUTHORING_RULES = [
  {
    id: "explicit-sfc",
    title: "Explicit Vue SFC source",
    requirement:
      'Public components are authored as real `.vue` files with `<template>`, `<script setup lang="ts">`, and scoped `<style>` blocks.',
    evidence: ["*.vue"],
    remediation:
      "Keep public component sources as canonical Vue SFCs with explicit template, typed script setup, and scoped style sections.",
  },
  {
    id: "behavior-table",
    title: "Normative behavior table",
    requirement:
      "Every public component declares its state, input, and outcome contract in a reviewable behavior table.",
    evidence: ["*.behavior.md"],
    remediation: "Add a `*.behavior.md` file that references the SFC filename.",
  },
  {
    id: "interaction-test",
    title: "Mounted interaction evidence",
    requirement:
      "Every public component has mounted-DOM tests that exercise the behavior contract instead of inspecting source text.",
    evidence: ["*.test.ts importing the component SFC"],
    remediation: "Add a `*.test.ts` file that imports the SFC and exercises observable behavior.",
  },
  {
    id: "prop-default-doc",
    title: "Documented prop defaults",
    requirement:
      "Every public prop documents its default value in the prop JSDoc so hover and generated docs explain first-render behavior.",
    evidence: ["defineProps<T> prop JSDoc with `@default`"],
    remediation: "Add an `@default` tag to each public prop's documentation comment.",
  },
  {
    id: "source-regex-behavior",
    title: "No source-regex behavior proof",
    requirement:
      "Behavior evidence must come from mounted runtime output; source-text assertions are reserved for unobservable source contracts.",
    evidence: ["mounted-DOM assertion", "`source-contract:` pragma for source-only invariants"],
    remediation:
      "Move the assertion to mounted DOM behavior or annotate the source assertion with a `source-contract:` pragma.",
  },
] as const satisfies readonly SfcAuthoringRuleContract[];

/** Stable rule ids emitted by `auditComponentAuthoring`. */
export type SfcAuthoringRuleId = (typeof VIZE_UI_SFC_AUTHORING_RULES)[number]["id"];

/** Quality gates that make the SFC authoring contract reviewable in one PR. */
export const VIZE_UI_SFC_QUALITY_GATES = [
  {
    id: "canonical-sfc-source",
    title: "Canonical SFC source",
    requirement:
      "The authored `.vue` file is the public component source; generated compiler output is not the API source.",
    evidence: ["*.vue", "`auditComponentAuthoring` explicit SFC parse"],
    enforcedByRules: ["explicit-sfc"],
  },
  {
    id: "behavior-contract",
    title: "Behavior contract",
    requirement:
      "State, input, accessibility, and failure-mode expectations are captured before implementation is accepted.",
    evidence: ["*.behavior.md"],
    enforcedByRules: ["behavior-table"],
  },
  {
    id: "mounted-interaction",
    title: "Mounted interaction proof",
    requirement:
      "Behavior gates are proven through mounted DOM/runtime assertions rather than string inspection.",
    evidence: ["*.test.ts", "`source-contract:` escape hatch for source-only invariants"],
    enforcedByRules: ["interaction-test", "source-regex-behavior"],
  },
  {
    id: "api-default-documentation",
    title: "API default documentation",
    requirement:
      "Public props expose their first-render defaults through documentation comments and editor hover.",
    evidence: ["defineProps<T> JSDoc `@default` tags"],
    enforcedByRules: ["prop-default-doc"],
  },
] as const satisfies readonly SfcQualityGateContract<SfcAuthoringRuleId>[];

/** Published contract consumed by UI package checks and future install manifests. */
export const VIZE_UI_SFC_AUTHORING_CONTRACT = {
  schemaVersion: VIZE_UI_SFC_AUTHORING_CONTRACT_SCHEMA_VERSION,
  packageName: "@vizejs/ui",
  sourceKind: "vue-sfc",
  stability: "stable",
  rules: VIZE_UI_SFC_AUTHORING_RULES,
  qualityGates: VIZE_UI_SFC_QUALITY_GATES,
} as const;
