---
title: Announcing the oxlint-plugin-vize Alpha
description: A new Oxlint JS plugin bridge brings Vize Patina diagnostics into a single Oxlint run for Vue SFCs.
---

# Announcing the `oxlint-plugin-vize` Alpha

<div class="blog-post-meta">
  <div class="blog-meta-chip">
    <div>
      <span class="blog-meta-label">Published</span>
      <span class="blog-meta-value">2026-03-26</span>
    </div>
  </div>

  <a class="blog-author-card" href="https://github.com/ubugeeei">
    <img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
    <span class="blog-author-text">
      <span class="blog-meta-label">Author</span>
      <span class="blog-meta-value">ubugeeei</span>
    </span>
  </a>
</div>

Today I am opening up the first alpha of `oxlint-plugin-vize`, a new Oxlint JS plugin bridge for Vize Patina.

The goal is simple: keep [Oxlint](https://oxc.rs/docs/guide/usage/linter) as the main runner for JavaScript and TypeScript rules, while letting Vize contribute Vue-specific diagnostics in the same run. Instead of choosing between Oxlint and Patina, this alpha is about making them work together.

## What It Is

`oxlint-plugin-vize` lets Oxlint execute Patina through Vize's native binding while still using Oxlint's JS plugin model and rule configuration.

That means a single `.oxlintrc.json` can mix rules like:

- Oxlint core rules such as `no-console`
- Oxlint's built-in `vue` plugin
- Vize rules such as `vize/vue/require-v-for-key`
- Patina-backed Vue diagnostics like `vize/vue/no-v-html` and `vize/vue/no-duplicate-attributes`

The plugin uses the `vize` namespace and reads settings from `settings.vize`.

## Why This Alpha Matters

Patina already understands Vue templates well, but many teams want Oxlint to stay at the center of their lint workflow.

This alpha is the first step toward that shape:

- one lint command
- one config file
- one output stream
- Rust-native JavaScript and TypeScript rules alongside Vue template-aware diagnostics

For Vue projects, that combination matters. Template rules like missing `v-for` keys or unsafe `v-html` usage should be able to live next to general-purpose Oxlint rules, instead of requiring a separate lint pass and a separate reporting format.

## Configuration Example

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "locale": "en",
      "helpLevel": "none"
    }
  },
  "rules": {
    "no-console": "warn",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "vize/vue/no-duplicate-attributes": "error"
  }
}
```

The alpha currently supports:

- `settings.vize.locale` for diagnostic language
- `settings.vize.helpLevel` with `"full"`, `"short"`, or `"none"`
- `showHelp` for backward compatibility
- `settings.patina` as a compatibility alias while `settings.vize` becomes the canonical key

## How It Works

The bridge is designed around Oxlint's per-rule execution model rather than fighting it.

- The first enabled Vize rule on a file runs a native Patina pass for that rule only.
- If a second Vize rule is enabled for the same file, the plugin upgrades to one shared full-file Patina pass and reuses the result for the remaining Vize rules.
- File contents and rule results are cached per file and settings for the lifetime of the Oxlint process.

That design keeps the first rule cheap while still avoiding redundant native work once multiple Vize rules are active.

## Diagnostics and Output

One of the hard parts in this integration is location reporting.

Oxlint's JS plugin system currently works from the extracted Vue script program, while many Patina diagnostics originate in `<template>` or other SFC blocks. In this alpha, `oxlint-plugin-vize` keeps the real Vue block and `line:column` inline in the diagnostic message so the output still points you back to the right place in the SFC.

The repository also includes a small `examples/oxlint-vize` example to show mixed output from:

- Oxlint core diagnostics
- Oxlint's built-in Vue support
- Patina-backed Vize diagnostics

## Current Limitations

This is still an alpha, and a few limitations are important to call out clearly:

- Oxlint JS plugins currently rely on the extracted Vue script program, so files without `<script>` or `<script setup>` do not invoke the plugin yet.
- Diagnostic anchors still point at the script program when Oxlint cannot accept the original template range directly.
- The current package targets Node 24+.
- Oxlint's JS plugin support is itself still evolving, so some rough edges here are upstream constraints rather than Vize-only behavior.

## Why Alpha Now

I wanted to get this integration into people's hands early, even before every edge case is polished.

The core shape already feels useful:

- Vize brings Vue-specific lint intelligence
- Oxlint remains the top-level runner
- the configuration surface stays small
- the performance model stays native-first

That is enough to start getting real feedback from Vue users who want a faster lint stack without giving up template-aware checks.

## What Comes Next

The next steps are straightforward:

- improve template location mapping as Oxlint exposes more Vue-aware plugin hooks
- harden the install and publish flow around platform native bindings
- expand documentation and examples for real project setups
- keep refining how Patina help text is surfaced inside Oxlint formatters

This alpha is not the end state. It is the first usable bridge between Oxlint and Vize's Vue linting, and I am excited to see where it goes next.
