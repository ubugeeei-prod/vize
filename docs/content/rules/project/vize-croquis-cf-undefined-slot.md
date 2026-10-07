---
title: "vize:croquis/cf/undefined-slot"
---

# `vize:croquis/cf/undefined-slot`

A parent fills a slot the child does not expose.

Default severity: Not emitted  
Applies to: Analyzed component graph and the supported facts described below  
Automatic fix: None; review related files and apply the repair  
Options: No per-code options; supported CLI findings accept severity overrides

This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.

## Bad

A parent fills a slot the child does not expose.

## Good

Use a slot the child declares, or add that slot to the child.

[Public explanation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Cross-file index](../cross-file.md)
