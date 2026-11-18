## Summary

The Real Project Matrix LSP lifecycle fails on element, vue-calendar, vue-lottie and vue-multiselect with "dependency repair must restore diagnostics exactly" (schedule run 44107635850, also 43880794806 and 43024501823).

Root cause: every `.vue` edit publishes twice at the edited version. `publish_changed_document` first sends the lint publication with the last native family moved to the new text (#11579), then schedules the complete pass, which publishes again at the same version. The oracle compared the first, partial packet. When the probe-era type pass landed during the importer overlay step, the moved Options API messages still printed the probe prop:

```
- Property 'highlighter' does not exist on type 'CreateComponentPublicInstanceWithMixins<ToResolvedProps<ExtractPropTypes<{ demoName: ...; }>, {}>, ...>'
+ Property 'highlighter' does not exist on type 'CreateComponentPublicInstanceWithMixins<ToResolvedProps<ExtractPropTypes<{ demoName: ...; vizeOracleProbe: { type: BooleanConstructor; default: boolean; }; }>, {}>, ...>'
```

A local wire trace of vue-multiselect shows `child v4` published first with `vizeOracleProbe` in 2 messages, then a second `child v4` that equals the baseline exactly. The server converges. Only the harness read the wrong packet.

## Change

- The dependency repair now waits for the same-version publication that restores the baseline. The file lifecycle repair already does this ("repair diagnostics wait must skip incomplete same-version payloads"). A repair that never restores still fails, and the failure lists the observed packets.
- The fake session can emit the moved probe-era answer once (`partial`) or in every publication (`terminal`). Two new tests cover both cases. The `partial` test fails on the old harness.

## Evidence

- Local `tests/tooling/real-project-lsp.test.ts` on element, vue-calendar, vue-lottie and vue-multiselect: 4/4 failed before, 0 failed after (28.6 s).
- `node --test` on the authored-oracle, file-lifecycle and importer-overlay suites: 23/23 pass. `tsc -p tests` is clean.

No server, canon or LSP runtime code changes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
