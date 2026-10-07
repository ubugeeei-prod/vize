---
title: "vize:croquis/cf/module-scope-reactive"
---

# `vize:croquis/cf/module-scope-reactive`

Reactive state is created at module scope and shared by every caller.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

## Bad

Reactive state is created at module scope and shared by every caller.

## Good

Create the state inside setup so each component instance owns it.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](../cross-file.md)
