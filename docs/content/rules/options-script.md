---
title: Script Rule Options
---

# Script Rule Options

Script rule option examples cover restricted globals, restricted members, and custom event casing.. The main [Rule Options](./options.md) page lists the complete option table,
unknown-field behavior, and scoped replacement semantics.

## `script/no-restricted-globals`

Use this rule when runtime globals must go through project-owned wrappers. Without options, the
rule reports `process`, `localStorage`, and `sessionStorage`. A non-empty `globals` list replaces
that built-in list; an empty list falls back to the built-ins so a config typo cannot silently
disable the rule.

```json
{
  "linter": {
    "rules": { "script/no-restricted-globals": "error" },
    "ruleOptions": {
      "script/no-restricted-globals": {
        "globals": [
          { "name": "process", "message": "Read env through useRuntimeConfig()." },
          { "name": "localStorage" }
        ]
      }
    }
  }
}
```

Bad:

```ts
const flag = process.env.FEATURE_FLAG;
const token = localStorage.getItem("auth.token");
```

Good:

```ts
const flag = useRuntimeConfig().featureFlag;
const token = authStorage.read("auth.token");
```

## `script/no-restricted-members`

Use this rule for project-local member access bans, such as moving browser APIs behind SSR-safe
helpers. The rule has no built-in deny list; it only reports when `members` is non-empty and the
rule is enabled. Each entry matches a bare identifier receiver plus a static property name.

```json
{
  "linter": {
    "rules": { "script/no-restricted-members": "error" },
    "ruleOptions": {
      "script/no-restricted-members": {
        "members": [
          { "object": "window", "property": "localStorage", "message": "Use authStorage." },
          { "object": "globalThis", "property": "process" }
        ]
      }
    }
  }
}
```

Bad:

```ts
const token = window.localStorage.getItem("auth.token");
const env = globalThis.process.env;
```

Good:

```ts
const token = authStorage.read("auth.token");
const env = readServerEnv();
```

## `script/custom-event-name-casing`

Use this option when emitted custom events should follow one casing convention across both script
and template usage. The default is `camelCase`; set `kebab-case` for projects that author emitted
event names in their template-facing form.

```json
{
  "linter": {
    "rules": { "script/custom-event-name-casing": "error" },
    "ruleOptions": {
      "script/custom-event-name-casing": { "casing": "kebab-case" }
    }
  }
}
```

Bad with the config above:

```ts
const emit = defineEmits(["saveItem"]);
emit("saveItem");
```

Good:

```ts
const emit = defineEmits(["save-item"]);
emit("save-item");
```
