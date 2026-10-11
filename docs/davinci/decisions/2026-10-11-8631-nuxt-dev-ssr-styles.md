# Preserve Nuxt page styles before development hydration

Tracking: [#8631](https://github.com/ubugeeei-prod/vize/issues/8631),
[#8602](https://github.com/ubugeeei-prod/vize/pull/8602).

## Decision

Retain the SSR CSS dependency imports from #8599 and the generated SFC stylesheet
URLs and complete `direct` query handling from #8602. These integration repairs
also address #8631's initial development SSR response. Keep their existing Nuxt
HTML guards; stylesheet links use the original generated paths and authored order.

Add a pinned Nuxt 4.5.0 / Vue 3.5.40 / Vize 0.443.0 development fixture with the
reported `<NuxtPage />` hierarchy, a scoped page style, a dark card and green
border. The linked reporter repository currently returns HTTP 404, so this is a
minimal regression derived from the issue's public contract, not a claimed copy
of the unavailable repository. The public reporter identity is Ryota Watanabe
(`wattanx`, GitHub account 43837308).

## Verification

The existing Nuxt scoped-style Actions workflow also runs this regression. Start
stock Vue, public Vize 0.443.0 and candidate adapter development servers from
separate copied projects. Wait for the TCP listener without warming SSR with an
HTTP request, then navigate with JavaScript disabled. Retain the first response,
all stylesheet status/content-type/body receipts and complete computed CSS and
card geometry. Public 0.443.0 must reproduce the unstyled response; stock Vue and
the candidate must match and return working CSS. Candidate hydration must retain
the same authored CSS and dimensions.

Local stock/public/candidate observation passes: public 0.443.0 emits zero
stylesheet links and a transparent card; the candidate has its authored dark
background, green 3px border, 24px padding and 420px width before JavaScript runs,
with identical geometry after hydration. Candidate JavaScript adapters are built
from source; the public 0.443.0 native compiler stays pinned because this repair
changes stylesheet integration. Fresh exact-head Actions and release acceptance
remain separate delivery gates. Original public adapter package bytes, fixture
inputs and candidate adapter hashes are recorded.

Register the fixture's two SFCs in both generated TS-32 remarks corpora and the
C-13 backlog. The full native-phase Actions gate observes 502 files and four new
L2 remarks (two applied, two missed); all existing 500 files and remarks remain
unchanged. Regenerate with `UPDATE_REMARKS_BASELINE=1` and rerun the strict gates.
Incremental equivalence retains the complete old 71-file plane and its original
68/3-file partition accounting, then independently pins the added two-file plane:
12 edit scripts, 60 applied steps, 74 comparisons and 154 blocks, with no mismatch.
The seeded stale-cache control must retain its old 667 defects and observe 20 new
defects. Keep the complete sweeps, exact accounting and regression checks.

## Remaining delivery

Carry #8602 through protected checks and the native Stack merge queue. Rebase the
remaining #8605/#8609 layers after any merged prefix; the original full VueFes
qualifier remains required for those layers and its known-red `dev-speaker`
failure must not be treated as passing. Keep #8631 open until a released package
passes the same development SSR and hydration contract.
