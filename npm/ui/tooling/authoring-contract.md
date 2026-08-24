# Vize UI SFC Authoring Contract

Schema version 1 is exported from `@vizejs/ui-tooling/authoring-contract` as
`VIZE_UI_SFC_AUTHORING_CONTRACT`.

This contract covers public `@vizejs/ui` component sources. It is intentionally
disjoint from individual component-family implementation work: it defines the
reviewable evidence that a component PR must provide before a family can claim
the SFC quality gate.

## Authoring Rules

- `explicit-sfc`: public components stay canonical `.vue` sources with
  `<template>`, `<script setup lang="ts">`, and scoped `<style>` blocks.
- `behavior-table`: every SFC has a `*.behavior.md` table naming the component
  source and describing state, input, and outcome.
- `interaction-test`: every SFC has a mounted interaction test importing the
  component source.
- `prop-default-doc`: every public prop has documentation comments that include
  an `@default` tag for editor hover and generated docs.
- `event-doc`: every public event has documentation comments explaining when it
  fires and what payload it carries.
- `source-regex-behavior`: source-text assertions are not behavior evidence
  unless a nearby `source-contract:` pragma explains why mounted output cannot
  observe the invariant.

## Quality Gates

- `canonical-sfc-source` is enforced by `explicit-sfc`.
- `behavior-contract` is enforced by `behavior-table`.
- `mounted-interaction` is enforced by `interaction-test` and
  `source-regex-behavior`.
- `api-default-documentation` is enforced by `prop-default-doc`.
- `api-event-documentation` is enforced by `event-doc`.

Future source installation manifests, gallery metadata, and CI policy should
consume the exported contract rather than duplicating these identifiers.
