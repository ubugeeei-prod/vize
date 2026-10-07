---
title: "vize:croquis/cf/shallow-deep-access"
---

# `vize:croquis/cf/shallow-deep-access`

A deep property of a `shallowReactive` or `shallowRef` value is read as if it were tracked.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

## Bad

A deep property of a `shallowReactive` or `shallowRef` value is read as if it were tracked.

## Good

Use reactive or ref when the nested fields must be tracked.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](../cross-file.md)
