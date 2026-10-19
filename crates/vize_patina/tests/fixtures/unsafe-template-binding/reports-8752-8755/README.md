# Public template-expression reports #8752–#8755

The two complete semantic oracle packets are copied byte for byte from the
independent expectations authored before any candidate execution. Together they
contain 25 full SFCs, ordered diagnostic vectors, stats, and JSON formatter
envelopes. They cover the four original reports and safe/unsafe neighbors,
including multiline object calls, conditional arms, typed slot declarations,
unsafe descendant values, CRLF/Unicode ranges, and inert internal-marker strings.

`public_expression_tests.rs` runs 24 self-contained scoped cases and the unchanged
RouterView original through real `lint_sfc_with_corsa`. It compares every
diagnostic field, counts, and the complete JSON formatter output. No target-rule
filter hides runtime failures or other findings. `VIZE_TEST_REQUIRE_TSGO` makes a
missing runtime fatal and cannot be combined with `VIZE_TEST_DISABLE_TSGO`.

The RouterView case uses this private fixture workspace's genuine vue-router
5.3.1 dependency. When the workspace is not installed, the tests assemble the
same project in a temporary directory instead: vue-router from the
authenticated members in `reference/` and Vue 3.5.42 from the repository's
pnpm store. A checkout with no install at all skips these tests, unless
`VIZE_TEST_REQUIRE_TSGO` makes that fatal. Shared preflight compares the entire installed package.json,
all 16 declaration files, and the license byte for byte with the authenticated
registry references, and requires the isolated Vue 3.5.42 package. The normal
test refuses a missing/wrong package or missing declarations. A missing resolved
import that yields an unsafe descendant also
fails the complete expected packet. No router stub or alternate installed
version replaces the reported source. This dependency execution is pending;
the presence of its authored zero-finding vector is not acceptance evidence.

The original API issue packets, Markdown bodies, SFCs, reporter identity, and
source custody remain in `original-reports/`. Its `vize.config.json` and
`tsconfig.json` preserve the literal JSON fences from report #8752, with their
trailing newline. The archived initial test preparation had a separately ignored
Router lane; it was never executed and has been replaced by the normal required
lane before candidate observation.

The 25 scoped tests use Incremental plus only
`type/no-unsafe-template-binding` at Warn. The separate unchanged
`original-config-oracles.json` contains four complete original Opinionated plus
typeAware packets, with no rule overrides. Its source proof pins the ordered
140 template, 29 script, and 10 CSS preset members and 199 whole source bodies.
The four original-config tests use their physical original filenames so Corsa
reads the literal reporter compiler configuration. No observed output authored
or changed these expected vectors.

Every role prints its complete API, formatter, and status packets before the
final whole-vector assertion. Missing runtime, unrelated diagnostics, range,
order, help, fix, labels, or count differences remain failures. The scoped
24-case loop collects all results before asserting, so its first difference
cannot prevent later whole packets from being retained.

`afterCli` and `afterFormatter` supply the complete formatter JSON contracts;
these Rust tests do not invoke a CLI process or a public npm installation. The
physical project path is the sole filename adjustment in the Router scoped
and four-original API/formatter expectations. All semantic fields and the
archived 25 scoped oracles remain unchanged. Whole public CLI and release
acceptance remain separate work.

The initial 22-case source-authored packet and three owner-type supplements have
not been recaptured. No runtime/native/API/CLI/Actions pass is recorded by this
fixture preparation. The separate checker-document test verifies that inert
script strings remain a complete contiguous source block.
