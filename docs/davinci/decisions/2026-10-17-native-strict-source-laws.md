# Native source laws and complete diagnostic custody

The controlled [Full Tooling run 38902394825](https://github.com/ubugeeei-prod/vize/actions/runs/38902394825)
reported 33 failing assertions in 27 test files. The committed held parent
`e4ad440e0b53224952d97cb731621cc0ba3ee513` already repairs 20 assertions;
this disjoint lane leaves those repairs with their owners. Its implementation
starts from main `c83021d545c76cd5cf5ee7e15e6ebdeaf533f35a`.

This lane repairs seven CLI macro calls that still name the former Carton
storage owner, two redundant module-path attributes, and five partial assertion
sites. The underlying CLI, Canon, Maestro and Patina product behavior stays
unchanged. Existing source ownership, module discovery and strict assertion
checks remain required, with no new allowlist entries.

## Native authority

[Focused run 38925869757](https://github.com/ubugeeei-prod/vize/actions/runs/38925869757),
job `117057122312`, executed exactly three focused Cargo commands on source
`9e388b842aecf0788ff3a2655da78ce325e62908`, using Rust 1.99.1 and the locked
physical native 7.0.2 image. Artifact `12056562909` contains source-tree,
compiler/environment identity, native SHA256, all original streams and command
logs. The temporary PR-only driver was removed after that one capture job.
An earlier invalid workflow definition, run `38925597561`, executed zero jobs
and is not native evidence.

The durable [capture archive](../../../tests/_fixtures/differential/typechecker/native-strict-source-laws/capture-38925869757.json.gz)
retains 23 whole packet/identity files, each with its original byte length and
SHA256. Build logs and the full Git tree listing remain in the Actions artifact;
the archive records their exact hashes. No captured paths, diagnostic fields,
stdout or stderr were normalized. The custody test checks all 12 workspace
reports against the complete authored receiver inventory and the full child
handoff packet. This historical capture establishes oracle values; final
source-head Actions must still compile and execute the completed assertions.

The CLI oracle compares every file, empty diagnostic receiver, compiler option,
program root and count for single, two-server and automatic modes, with both
clean and broken private dependencies. The LSP oracle compares the entire lint
result: filename, severity, rule, message, byte range, help, labels and fix.
The TS assertion cases retain both whole original inputs and exact source-owned
union spans, together with their existing complete native lint results.

## Required semantic completion and the shutdown frontier

The Canon child must emit both complete native diagnostic packets: TS2322 on
`Injected.vue` plus TS1128 on `post-request.d.ts`, followed by the repaired
packet containing only TS1128. Its whole libtest output must report the exact
selected test, one pass and four filtered tests. Only the validated elapsed
centiseconds are variable. Missing output, extra diagnostics, extra harness
rows and runtime-degradation notices fail.

The actual capture also exposed two raw lines of
`error handling method 'exit': EOF`. They are current shutdown noise, **not a
clean shutdown**. The existing registry SDK sends Exit after Shutdown; the
owned SDK correction in [#9239](https://github.com/ubugeeei-prod/vize/pull/9239)
authenticates the native image and initialize response, requires Shutdown's
null ACK, then closes stdin. Foreign servers retain their protocol behavior.
That product correction and its shipping dependency remain a separate lane.

The temporary observation category `KnownObservedExitWarnings` accepts exactly
the two captured lines and only the captured physical Linux x64 image SHA256
`4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`.
It emits an explicit observation and preserves raw stderr. Empty stderr is
`Clean`; every other stderr packet or image/warning combination fails. Seven
paired controls include foreign-image, missing/extra-row and degradation
counterexamples. This category must be removed when #9239's combined native
source proof establishes clean output for this exact two-Batch case. It grants
no clean-shutdown or editor-session qualification.

The observation is from the Batch API child, not the editor process. The LSP
modifier case calls Maestro's diagnostic adapter directly and has no native
shutdown session. Neither result establishes editor transport lifecycle health;
that remains with its existing owner and actual editor-session receipts.

Final qualification uses ordinary source-head Actions and one coordinated
Full Tooling campaign after the disjoint repairs combine. This lane does not
claim the earlier capture qualified its later source or that ordinary PR checks
replace the full campaign.
