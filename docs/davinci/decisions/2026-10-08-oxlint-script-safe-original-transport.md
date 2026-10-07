# Script-safe original-source Oxlint transport

Tracking: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142) and
[#7903](https://github.com/ubugeeei-prod/vize/issues/7903).

Oxlint's JS plugin host omits Program callbacks for a scriptless Vue file and
stops at an invalid authored script. Executing the original SFC after a synthetic
script does not fix that boundary: it can still prevent Vize callbacks and repeat
the original parser fatal. Running every check on a copied file also changes
native import resolution, project ownership and custom rule filenames.

The wrapper now retains the engine's original target traversal and ordered
native/core/custom/import/type-aware result. It runs configured Vize rules in a
separate carrier phase. The carrier contains one whitespace script with the
original UTF16 coordinates and CRLFs, followed by canonical base64 source bytes
in an HTML comment. The original script is never executable carrier content.
The checksum covers original filename, a NUL separator and complete source;
decoded source must equal the actual original file bytes before native linting.
Payload, filename, mirror, original mutation and non-UTF8 authority failures are
explicit failures. Current selected-rule batching, options keys and bounded
revision cache remain intact.

The two complete reports are joined without diagnostic sorting, normalization or
deduplication. Original diagnostic order, multiplicity, filenames, spans, unknown
fields, reporter layout and original file count remain authoritative. Raw
diagnostic JSON is retained; only top-level diagnostic/count values are joined.
Escaped/nested fields and the existing native JSON snapshot remain exact.
An unavailable bridge must
retain original stdout and stderr while failing the run. This also applies after
original linting when mirror preparation, selection, process startup or output
mapping fails. Owned temporary configurations and carriers are removed.

Standalone HTML-only discovery retains the existing HTML/HTM adapter and its
unchanged native reporter snapshot. Stock Oxlint excludes HTML from its Vue
selection. A mixture of HTML and Vue retains the original report and explicitly
refuses transport; it cannot select the HTML fallback. HTML compatibility grants
no original-project, direct n8n51 or scriptless adoption qualification.

The authored regression corpus contains nine positive Vue inputs, including
scriptless, malformed JS/TS, astral Unicode, CRLF and script-closing text. The
same whole source vectors retain five independent script fatals exactly once,
custom original-path/options packets, native import diagnostics and precise
Vize ranges. The JSON failure fixture includes repeated original fatal packets
and an opaque field; it is an authored transport law, not a native accuracy
oracle. Unit controls cover unavailable bridge reports and post-original
preparation/selection/startup failures with original file/config custody.

The existing JS Actions gate reuses its genuine source-built NAPI receipt. Both
actual Oxlint 1.78.0 and 1.86.0 hosts run import/type-aware/whole-packet controls
against that physical addon. The observer authenticates source HEAD/tree,
toolchain, binary hash and physical loads, retains full original native
inputs/returns and host stdout/stderr/status, and rejects a guard reaching native
linting. It installs only the pinned host tools outside the checkout; it does
not build another addon. Source and protected Actions are pending on this new
head. Reused local binaries with unknown source identity receive no accuracy,
timing or adoption credit.

This changes the wrapper route. The direct frozen 51-rule host invocation still
has a scriptless callback limitation, independently of the licensed corpus's 19
scriptless inputs. A wrapper Program/native-call witness does not qualify that
direct route. Full licensed n8n 1,369-input/51-rule/six-scope acceptance and
installed-release evidence remain required separately.

JSONC/no-config transport remains fail-closed with the actual original report
retained. Nested configs, non-POSIX roots, ambiguous selection filenames,
path-sensitive suppression state, fixes, CLI severity/limit overrides,
unused-disable reporting and unsupported report formats remain unqualified.
Restoring retired Vue parser/filter/v-is/v-pre semantics requires its own
explicit rules and complete source vectors; generic rules are not substitutes.
The wrapper's two lint invocations and selection preflights have an unmeasured
cost. No performance improvement or complete n8n adoption is claimed here.
