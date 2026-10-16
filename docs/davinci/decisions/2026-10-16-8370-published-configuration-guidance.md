# Published configuration guidance

Issue [#8370](https://github.com/ubugeeei-prod/vize/issues/8370) moved the setup
journey to existing Vite and TypeScript configuration in
[#8447](https://github.com/ubugeeei-prod/vize/pull/8447). The five localized
Getting Started and Configuration pages still said native Vite discovery and
configuration-free init were awaiting the next release after those features
were published.

The replacement notes name the verified `vize@0.457.0` baseline, recommend
updating the integration and editor together, and retain the detailed reference
for discovery, precedence and optional dedicated formats. The editor project-root
description also stops referring to a future release. Code examples, links and
bookmarks are unchanged.

The public npm CLI was installed in a fresh temporary directory. A
`vite.config.mjs` containing `vize.linter` selected the empty `incremental` preset
and enabled `vue/this-in-template` at Error severity; lint returned exactly that
error. Actual `init --yes --no-lint --no-bundler --fmt --typecheck --no-editor
--no-install` added formatting and type-checking scripts without creating any
`vize.config.*`. This verifies those published entry points; it does not substitute
for the broader tool/editor acceptance tracked in #8371.

The existing native-rendered journey checks cover all five localized recipes,
172 original heading targets, 42 complete reference examples and the complete
dedicated workflow recipe. The publication-note assertion now rejects the stale
future-release claim in every locale. Protected Actions, Docs deployment and
deployed reading acceptance complete this documentation delivery.
