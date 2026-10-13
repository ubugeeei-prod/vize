# Actual native terminal controls

For [#6879](https://github.com/ubugeeei-prod/vize/issues/6879), the anomalous
stderr and nonzero terminal controls use a test-only Python transport around
the authenticated native TypeScript 7.0.2 executable. The production adapter,
SDK protocol, original eight physical inputs and source-built CLI oracle do
not change in this slice.

`tests/_fixtures/corsa-process-receipt/native-control.py` accepts
`--native PATH --mode unexpected|nonutf8|nonzero --control-dir DIR --`
followed by the actual native argv. Corsa's LSP argv is `--lsp --stdio`.
Non-LSP invocations exec the same authenticated native executable unchanged.
The native SHA256 must match the original official Linux x64 or Darwin arm64
7.0.2 body. The wrapper's binary identity remains different and cannot receive
the official native binary's production capability classification.

The wrapper forwards every complete LSP frame unchanged, with one narrowly
recorded transport adaptation. After observing the native response to the
forwarded params-less Shutdown request, it retains the entire params-less,
id-less JSON-RPC 2.0 Exit notification from the SDK and closes native stdin
instead of forwarding that notification. Native's direct Exit dispatch can
race cancellation; this test adapter makes the anomalous packet reproducible
without substituting any diagnostic response. An Exit with params, an ID,
duplicate JSON members or another shape is never translated. This adaptation
is confined to the test wrapper and proves no production Exit behavior.

Both actual native readers drain raw bytes. The control directory retains
whole wrapper stdin, forwarded native stdin, native stdout/stderr, wrapper
stdout/stderr, injected wrapper stderr and the complete translated Exit
frame separately. `control.json` binds actual argv, cwd, native hash, native
PID, wrapper PID, parent PID, acknowledged Shutdown, terminal status, signal,
reap, timeout, kill, reader completeness and errors. `native-child.txt` has
exactly four rows: wrapper PID, native PID, native exit and wrapper exit.
The wrapper and native always have distinct physical process identities.

Unexpected and non-UTF-8 modes emit their exact original prefix before native
starts. After native exits 0 with completed readers and acknowledged shutdown,
all modes inject the exact 34-byte EOF line. Unexpected/non-UTF-8 wrappers exit
0; the nonzero wrapper exits 17 while its real native child exits 0. These
bytes explicitly originate from the wrapper. All genuine native stderr is
also retained and replayed in full, including any cancellation or panic bytes.
No normalization, filtering or ignored native error is used. Timeout or
protocol/reader failure refuses success; a bounded kill and actual reap are
recorded separately from a normal terminal result.

The portable supplemental transport law is:

```sh
python3 tests/_fixtures/corsa-process-receipt/native-control-law.py \
  --native /absolute/path/to/official/native/tsc \
  --output /absolute/path/to/new/evidence-directory
```

It uses all eight frozen original files and the same physical config, obtains
the full real `shared.ts` TS2304 LSP response, completes actual configuration
and watcher responses before shutdown, and checks all three native/wrapper
status pairs. It requires byte-exact stdout forwarding, whole stdin equality
except the separately retained Exit frame, complete stderr and unchanged
physical inputs. It retains full request/response frames and native identity.
This transport law does not claim the entire source-built CLI TS1260/TS2304
oracle, selected-project API authority or native differential acceptance.

Local Darwin arm64 7.0.2 execution passes all three transport modes with native
exit 0, no kill/timeout, completed raw streams and wrapper exits 0/0/17.
An actual-native timeout control also records native SIGKILL 9, actual reap,
wrapper exit 124 and complete raw readers, with no injected EOF line.
The original unquiesced experiment also retained native cancellation stderr
in full rather than replacing it with the expected injected bytes. The parent
delivery change owns the public Rust law, exact composed-head hosted native
acceptance, custody registration and the paired existing issue comment.
