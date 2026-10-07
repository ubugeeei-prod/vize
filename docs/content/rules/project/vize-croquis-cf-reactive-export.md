---
title: "vize:croquis/cf/reactive-export"
---

# `vize:croquis/cf/reactive-export`

Reactive state is exported from the module.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

## Bad

Reactive state is exported from the module.

## Good

Export a factory that creates the state per caller.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](../cross-file.md)
