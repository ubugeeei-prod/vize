# Preserve return-check bodies through script prefilters

Tracked in [#8090](https://github.com/ubugeeei-prod/vize/issues/8090).
No additional default rule is enabled.

The default `script/return-in-computed-property` and
`script/return-in-emits-validator` producers already recognize method shorthand
and block arrows. Their dispatch gate demanded one of `=> {`, `=>{`, `function`
or `get(`. Ordinary `computed: { total() {} }` and
`emits: { submit(payload) {} }` methods contain none of those bytes. Newlines,
tabs or comments between an arrow and its body also defeated that gate. Existing
direct rule tests bypassed SFC dispatch and could not establish default coverage.

An authenticated published v0.443.0 Apple ARM64 archive executes all sixteen
carriers and reproduces fourteen missing findings, while retaining the two
already reported findings and all seven complete negative results.
The archive SHA256 is
`6378e10d0fa0568a0c1798b48d99b18a01f3e842f8c765fb11b80b638f39a73e`;
the executable SHA256 is
`9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254`.
Their complete status, stdout, stderr and source are conserved in the corpus's
original public observation. The uncompressed receipt SHA256 is
`40a2108e9b737b9104aa0792aee2c031d7deadbe67ee9618aeb797fad908b883`.

Keep the existing `computed` and component/emits declaration gates, then require
only a `{` byte. Every block body reported by the unchanged producers contains
that byte. The AST owns method and arrow syntax; the byte prefilter must be
conservative. Concise arrows without braces still skip this rule. Comments or
unrelated object braces may admit a scan, but the existing producer preserves
the complete result. This retains the shared OXC parse and adds no parser,
pipeline stage, dependency, native query or budget increase.
Escaped spellings of `computed` or `emits` remain outside their unchanged raw
name gates; this correction qualifies body spellings admitted by those gates.

Sixteen complete authored SFC carriers exercise shorthand methods, writable
getters, comment/newline/tab arrows, nested returns and whole negative controls.
Expected source spans, help, labels and no-fix Error metadata come from the
unchanged rule producers and their original method-span snapshots. API tests
compare complete results for HappyPath, Essential and Ecosystem, plus explicit
off and empty Incremental. The 112 CLI observations additionally retain the
unchanged explicit 51 n8n projection, where neither return rule is selected.
All input bytes and whole JSON/status/stderr packets are checked; no finding
filter or runtime expectation recapture supplies acceptance.

The source CLI test also requires a controlled workload of 64
files, each with 64 valid computed getters and 64 valid emits validators. All
8,192 own value returns use shorthand or trivia spellings rejected by the old
body gate. The default and explicit two-rule-off modes execute serially in
alternating order, with two warmups and six measured trials per mode. They must
preserve identical whole empty diagnostic JSON, source/configuration bytes and
executable digest. Raw timings and complete packets are retained in the source
CLI artifact. This configuration contrast includes startup, config-loading and
entry-selection costs; it is neither an isolated rule timer nor a base/head
regression result. Existing protected instruction/performance gates remain
mandatory and no speedup is claimed.

The pre-existing emits resolver handles Options API declarations. Bare
`defineEmits` validator objects remain outside that producer despite its old
comment claiming support. This correction does not claim that separate coverage;
it remains a rule-coverage boundary under
[#6097](https://github.com/ubugeeei-prod/vize/issues/6097).

The newly admitted rule scan has a real cost. Current source Actions, unchanged
protected instruction/performance gates, actual merge and installed later-release
observations remain required. Preparation and the v0.443.0 before observation
grant no corrected source or release acceptance. #8090 remains open through its
existing all 25 installed/default and enforced real-project obligations.
