# Separate mounted runtime preparation from observation

Issue [#9067](https://github.com/ubugeeei-prod/vize/issues/9067) recorded a Linux
runtime child that printed the complete passing result and then died with
SIGSEGV. The passing JSON is insufficient: process status and complete streams
remain part of the test result.

The historical cause is unknown. The bounded diagnostic
[38928783457](https://github.com/ubugeeei-prod/vize/actions/runs/38928783457)
retained two historical natural exits, two current guarded exits and two
build-only natural exits. All six passed; explicit zero, nonzero and signal
controls also behaved correctly. This did not reproduce the original fault or
prove its elimination. In particular, the guarded exits skipped native teardown.

## Ownership

`mounted-runtime-producer.ts` owns the real Vite build and any bundler addons. It
uses the same pinned `vueVaporRuntimeEntry`, `configFile: false`, silent logging,
four defines, `write: false`, mode-dependent minification and single ES library
chunk as the former inline `loadRuntime` implementation. It writes the complete
generated code packet and exits naturally. It has no DOM and no exit override.

`mounted-runtime-source.ts` owns the child and both complete byte streams. It
waits for `close`, including stream completion. A nonzero exit, signal, spawn or
input error, any stderr, malformed or noncanonical packet, extra field or empty
code fails. Failures retain the PID, status, signal, text and complete raw byte
arrays. Code written before a producer crash cannot be accepted.

`loadRuntime` still imports that code through the same base64 data URL after
the same two preparation phases. It retains its signature, environment default,
mode defines and runtime module identity. DOM creation, Vue devtools attachment,
mounting, updates, diagnostics and cleanup stay with the observer.

The direct `davinci-mounted-trace.mjs` entry and `patterned-template-runtime.mjs`
explicitly forbid existing `.node` images and future `process.dlopen` calls.
This is an active loader boundary, not an inference from an image census alone.
The SSR patterned branch also arms it, although that branch does not build the
mounted runtime. Neither observer overrides normal exceptions, rejections or
process exit. The dedicated native-exit regression exercises both real producer
modes, module identity, the loader refusal and the observer's natural exit.

## Builds and process count

The old `loadRuntime` had no generated-code or build-promise cache: every call
ran Vite again, even when Node reused an identical data-URL module. The new cache
retains one generated-code promise per development/production mode in each
observer process. Both successes and failures are retained; there is no silent
retry. Entry, flags and other producer inputs are fixed for that invocation.

An invocation requesting only one mode therefore adds one producer process and
runs Vite once. An invocation requesting both modes adds at most two producers
and runs Vite twice. N patterned cases no longer create N producer processes or
repeat N builds. No compilation case adds a separate producer. Existing runtime
calls still import the original code and reuse its data-URL module; this does
not introduce a new Vue module cache or change mount isolation.

The complete source inventory at main `502073de95` is below. Counts exclude the
separate regression and describe builds, not how many scenarios a helper tests.

| Entry/helper                             | Former builds per invocation | New producers/builds           | Other native preparation    |
| ---------------------------------------- | ---------------------------- | ------------------------------ | --------------------------- |
| Direct `davinci-mounted-trace` entry     | 1                            | 1                              | None; addon guard           |
| `patterned-template-runtime`             | N non-SSR cases; 0 for SSR   | 1 for N > 0; 0 for SSR         | None; addon guard           |
| `davinci-upstream-vapor-trace`           | 1                            | 1                              | None                        |
| `davinci-directive-trace`                | 1                            | 1                              | None                        |
| `davinci-suspense-trace`                 | 1                            | 1                              | None                        |
| `davinci-transition-trace`               | 1                            | 1                              | None                        |
| `vapor-attribute-modifiers`              | 1                            | 1                              | None                        |
| `vapor-setup-resolution`                 | 1                            | 1                              | None                        |
| `vapor-sfc-runtime`                      | 1                            | 1                              | None                        |
| `vapor-keyed-fragment-runtime`           | 1                            | 1                              | None                        |
| `jsx-falsy-and-runtime`                  | 1                            | 1                              | None                        |
| `implicit-default-slot-comments`         | 1 for Vapor; otherwise 0     | Same                           | None                        |
| `enum-template-runtime-7893`             | 1 for Vapor; otherwise 0     | Same                           | Vite/Oxc source transform   |
| `vapor-css-vars-runtime`                 | 1                            | 1                              | Vite/Oxc source transform   |
| `vapor-trailing-line-comments`           | 1                            | 1                              | Vite/Oxc source transform   |
| `vapor-component-listeners`              | 1                            | 1                              | Vite/Oxc source transform   |
| `page-meta-runtime`                      | 1                            | 1                              | Vite/Oxc source transform   |
| `ssr-slot-forwarding`                    | 1                            | 1                              | Vue plugin source transform |
| `mounted-runtime-native-exit` regression | 0 (imported Vite only)       | 2 modes, repeated reads cached | None; addon guard           |

These are all 18 direct helper importers plus the shared module's direct entry.
The six native-transform helpers retain their real independent source-transform
oracles. They are not claimed to be addon-free. Moving those transforms across
a process boundary would be a separate change.

## Qualification boundary

The focused source tests cover cache identity, sticky failures and strict full
producer packets, including valid code followed by a real nonzero exit or signal
and non-UTF8 bytes. Existing Rust signal/nonzero and raw stream controls remain.
The existing mounted-runtime comparison workflow runs those exact two Node
tests and the real producer regression after its unchanged Transition comparison.
Its existing artifact retains source hashes, both complete command streams,
status and exactly two distinct producer PIDs/modes/code hashes. Optional producer
capture records both complete streams after close and success receipts only
after strict validation. The native-exit stdout remains exactly `{"passed":1}`.
The real producer regression must pass under that existing Actions runtime setup. Original mounted observations and their complete output oracles
must pass in development and production before this replaces the workaround.
This architecture removes the exit bypass; it does not establish the historical
SIGSEGV's root cause or justify closing #9067 without that qualification.
