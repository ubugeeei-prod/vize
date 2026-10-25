# Installed editor Router provider

This fixture expands the pinned public Router 5.4.0 provider with both `.art.vue`
input shapes: an ordinary SFC `<template>` (also present in the extension's basic
fixture), and a proper `<art><variant>` document. The complete declaration was
produced by the real scanner; whole-project reference checking passes. The
runtime helper regenerates and compares every declaration byte before hosting.

The original installed-host capture is retained by Actions 40178143159, artifact 12592699376. It demonstrates that the ordinary-template Art shape has no native
type diagnostics or typed query results. A separate public 0.562.0 proper-variant
probe produces typed completions and template diagnostics, but script property
hover is null, definition selects the local variable, and references omit authored
usages. These are required failures, not qualified Art support.

The public completion adapter follows pinned VS Code 1.107.1 behavior:

- [CompletionsAdapter](https://github.com/microsoft/vscode/blob/1.107.1/src/vs/workbench/api/common/extHostLanguageFeatures.ts#L1072)
  supplies default word insert/replace ranges.
- [CompletionItem.to](https://github.com/microsoft/vscode/blob/1.107.1/src/vs/workbench/api/common/extHostTypeConverters.ts#L1129)
  returns insertion text and an explicit whitespace flag.
- [_inflateSuggestDto](https://github.com/microsoft/vscode/blob/1.107.1/src/vs/workbench/api/browser/mainThreadLanguageFeatures.ts#L521)
  supplies the completion label when the provider omits insertion text.

Native packets remain unmodified. Closed controls reject shifted public ranges,
changed insertion text, and changed whitespace behavior. Whole packets and native
compiler traffic are retained before assertions, including failed Art cases.

The installed-client ownership witness adds two ordinary Vue controls outside the
page scanner. It compares complete directive and CSS completion items through
the real client and public editor API across OS-delivered tsconfig changes. It
retains mixed-owner refusals, a URI-bound native completion and an authored edit
that must block stale completion resolution before forwarding to the compiler.
These controls do not change any scanner declaration or relax the Art failures.
