# Per-document project configuration in the editor

The current configuration sources and reference-project ownership are recorded
in [the October 17 decision](./2026-10-17-8371-vite-tsconfig-editor-config.md).
The dedicated-file discovery described below is historical.

Issue: [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).

The language server assigns each open document to its nearest project root
inside the selected workspace folder. A `package.json`, `tsconfig*.json`,
`jsconfig.json`, `vite.config.*` or `vize.config.*` establishes that root, and
the document uses that project's configuration for formatting, type checking,
linting and navigation. Project contexts are created lazily on first use.

Concurrent requests keep their own project graph and initialization options.
Watched configuration changes refresh the affected importers and invalidate
their project caches before diagnostics. Completion resolution accepts only
items issued by the current project session and document generation; any
other item is returned unchanged.

This change lands the production code and behavior tests of #8476 without its
source custody tests and recorded receipts, which no longer apply after the
custody retirement in #6830. Nothing about the editor behavior described in
#8476 is dropped.
