---
title: "vize:croquis/cf/circular-reactive-dependency"
---

# `vize:croquis/cf/circular-reactive-dependency`

Reactive computations depend on each other in a cycle.

Default severity: context-dependent  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## Bad

The analyzer's tracked reactive-flow graph contains a cycle: provider A → consumer B → provider A. Both references are the same graph identities, rather than unrelated variables that share a name.

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

## Good

Remove the B → A flow: let A own the source, and let B read a computed value or emit an action instead of feeding that reference back. The tracked flow graph becomes acyclic.

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producer](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Cross-file index](../cross-file.md)
