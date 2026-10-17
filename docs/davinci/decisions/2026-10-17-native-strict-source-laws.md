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

The initial observation category `KnownObservedExitWarnings` classified exactly
those two captured lines and only the captured physical Linux x64 image SHA256
`4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`.
Empty stderr was `Clean`; every other packet or image/warning combination failed.
The original seven paired controls and raw historical capture remain intact.
This initial acceptance did not establish clean shutdown or editor lifecycle
qualification. The required MQ result below supersedes accepting that category
as the desired runtime outcome.

The observation is from the Batch API child, not the editor process. The LSP
modifier case calls Maestro's diagnostic adapter directly and has no native
shutdown session. Neither result establishes editor transport lifecycle health;
that remains with its existing owner and actual editor-session receipts.

Final qualification uses ordinary source-head Actions and one coordinated
Full Tooling campaign after the disjoint repairs combine. This lane does not
claim the earlier capture qualified its later source or that ordinary PR checks
replace the full campaign.

## Required MQ exposed a real shutdown race

[Merge-group Check 38930902972](https://github.com/ubugeeei-prod/vize/actions/runs/38930902972),
source `ea158a06d93acad9940ffb2c112ef56be800676f`, ran the native-enabled typechecker
cohort in job `117077466284`. Of 2,717 executed tests, 2,716 passed and one failed:
the exact same two-Batch handoff. The complete stdout contract passed before
stderr rejection, but the actual native stderr was one Exit/EOF line followed
by `context canceled`. The physical binary SHA256 was unchanged. The independent
native Canon job on the same source passed with the prior two EOF lines.
Neither result justifies retrying until the race happens to disappear.

The [MQ failure archive](../../../tests/_fixtures/differential/typechecker/native-strict-source-laws/mq-38930902972-failure.json.gz)
retains the exact raw job failure block and JUnit testcase fragment, plus hashes
of the complete originals in artifact `12058479097`. The child stdout was not
included in this cohort's failed-test artifact; the archive explicitly records
that limit, rather than reconstructing it as observed output. The oracle now
includes the complete child packet in any future unknown-stderr panic.

The correction uses the already published `corsa_lsp = 1.14.0` API. Only a full
known physical native image **and** the actual whole initialize response select
Shutdown's null acknowledgement followed by `client.close()` to send EOF. The
whole initialize contract and Linux x64/Darwin arm64 image identities are taken
from the independently owned #9239 source `04d8236db4b46126d43825a34ba44009fde86141`.
No package/vendor migration, stderr normalization, process-observer fallback or
new lifetime-custody claim is introduced. Wrappers and foreign images retain
the existing standard Shutdown/Exit behavior. Cleanup still runs on protocol
failure and preserves the first error.

Four owned protocol controls compare complete request/response/EOF transcripts:
native null ACK then EOF; foreign null ACK then Exit and EOF; remote Shutdown
error then EOF with the complete original error retained; and a non-null ACK
rejected before EOF. Image/initialize controls reject spoofed versions, missing
files and altered capabilities. The actual MQ stderr is an eighth rejected
observation control, not another permitted warning variant.

The existing actual two-Batch law now requires `Clean` after complete native
stdout verification. Even the historically known two EOF rows fail that desired
outcome. Historical packets and classification controls remain separate evidence.
The new source still needs actual native Canon and parallel required-cohort
execution; no old receipt is relabeled as proof of this correction.
