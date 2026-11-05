---
title: "type/project-configuration"
description: "Repair TypeScript project configuration failures that block type-aware lint."
---

# `type/project-configuration`

Type-aware lint reports this error when it cannot load the selected TypeScript project configuration
or resolve its configuration inheritance. It is an operational diagnostic outside the preset rule
catalogue. There is no automatic fix.

## When it is reported

When an enabled type-aware check needs a type query, Vize validates the selected `tsconfig`
before starting its native type-checking session. Validation includes the configuration's
`extends` chain. Unreadable or invalid configuration files and unresolved inherited configurations
prevent those type queries from running.

The message names the selected configuration and explains the failure. It is reported once per
configuration in a linter instance, on the source file that encounters it first, with a zero-length
range at the file's start. It is a project setup error, rather than a type error in that first file.
Other checks that do not need the failed type query can still report their own findings.

## Bad

This `tsconfig.json` refers to a generated configuration that has not been created.

```json
{
  "extends": "./.generated/tsconfig.base.json",
  "include": ["src"]
}
```

## Good

Create the inherited `.generated/tsconfig.base.json` before lint runs. For this small example,
its complete contents can be the following; keep the original `tsconfig.json` above.

```json
{
  "compilerOptions": {
    "strict": true
  }
}
```

In a framework project, use the framework's preparation command to generate its configuration
instead of replacing it with this illustrative base file.

## How to repair it

1. Open the configuration named in the message. Check that the selected `tsconfig` is the one
   intended for the source file; an explicit selection uses `vue.typeChecker.tsconfig`.
2. Repair invalid configuration syntax or inaccessible files. Resolve each missing `extends`
   entry by generating the required file, correcting its path, or installing the package that
   supplies the inherited configuration.
3. Run lint again after preparation finishes. Start a new run after editing configuration so
   the linter validates the repaired configuration afresh.

Do not remove `extends` merely to silence the diagnostic: the inherited compiler options may be
required by the project. If the diagnostic ID is `type/corsa-runtime` instead, inspect native runtime
selection and compatibility in [Type and script rules](../type-and-script.md).

## Related

- [Native lint execution settings](../../guide/lint-execution.md)
- [Shared Vue configuration](../../guide/vue-configuration.md)
- [Type and script rules](../type-and-script.md)
- [Type checking](../../guide/type-checking.md)
- [Diagnostic source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/native_type_aware.rs)
- [Project configuration validation source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/corsa_session/paths/store.rs)
