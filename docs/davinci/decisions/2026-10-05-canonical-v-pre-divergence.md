# Canonical v-pre DOM and SSR divergence

Issue: [#7939](https://github.com/ubugeeei-prod/vize/issues/7939), linked to
benchmark qualification [#7856](https://github.com/ubugeeei-prod/vize/issues/7856).

## Decision and actual execution

Keep the expanded canonical corpus strict. The metadata-only benchmark
registration #7860 is blocked by its first real 147-gitlink execution;
ordinary source Check success does not replace that failed execution.
The existing provider/history lane owns a bounded source audit and correction.
The comparator, input, old-error allowlist and budgets must remain unchanged.
The provider cause has not been established.

[Matrix 37256474108, attempt 1](https://github.com/ubugeeei-prod/vize/actions/runs/37256474108)
executed source `d7169333f84bb3d8ddfaebcc348b4b83930d6f40` with all modes
enforced. Its [canonical job 111594541298](https://github.com/ubugeeei-prod/vize/actions/runs/37256474108/job/111594541298)
failed. It selected and hydrated 147 gitlinks while preserving the original
144 ecosystem/App registrations. Older 146-gitlink runs are separate evidence.

DOM parsed 42,998 files, found 42,625 templates and compared 42,609 outputs.
Unreadable files and L2 refusals were zero. The same 16 old-error files remained;
there was exactly one new divergence. SSR compared all 42,625 templates,
selected 42,558 `s4` lanes, and recorded zero error skips, zero rejections and
one divergence at the same path. These selected-lane counts do not complete
native migration. Pug was chained after SSR and did not execute after its
failure; this run provides no Pug acceptance.

## Immutable original input and observed output

The entire [original SFC at upstream 5489aee](https://github.com/pikax/vue-benchmarks/blob/5489aee433cd1054b9d72973457498544da7c467/tests/confirm/fixtures/format/format-v-pre-content.vue)
is 228 bytes, Git blob `6a9e7e6d51eb012c4d4688c7f564c313e15ec357`,
SHA-256 `40f6f59fd7ccdc7e13c33d23c3d643a0bd75758993844a9ae1d306712676dcb0`.
The upstream repository licenses it under MIT. Its source is unchanged:

```vue
<template>
  <div v-pre>
    <span>{{ this is not an expression }}</span>
    <span :class="notEvaluated">RAW_VPRE_TOKEN</span>
  </div>
  <p>{{ realBinding }}</p>
</template>

<script setup>
const realBinding = "ok";
</script>
```

DOM reports `old_len=623 new_len=736 first_diff=167`. Its recorded old helper
window ends `Fragment: _Fragment } = Vue`; the new window adds
`, createTextVNode: _createTextVNode`. The artifact retains the first-difference
windows, not complete DOM generated programs; do not invent the omitted bytes.

The complete selected and legacy SSR programs are in the actual job log.
Their differing `_push` lines are copied verbatim after removing job timestamps;
only the lane labels below are authored:

```text
selected:
  _push(`<!--[--><div> <span>{{ this is not an expression }}</span> <span :class="notEvaluated">RAW_VPRE_TOKEN</span> </div><p>${_ssrInterpolate($setup.realBinding)}</p><!--]-->`)
legacy:
  _push(`<!--[--><div><span>{{ this is not an expression }}</span><span :class="notEvaluated">RAW_VPRE_TOKEN</span></div><p>${_ssrInterpolate($setup.realBinding)}</p><!--]-->`)
```

This establishes whitespace/output differences. It does not establish which
provider is responsible or authorize changing the legacy output contract.

## Custody and independent owners

[Artifact 11323206308](https://github.com/ubugeeei-prod/vize/actions/runs/37256474108/artifacts/11323206308)
is an authenticated 11,160-byte ZIP with official SHA-256
`f6f354cf9dd632fe03a6b704a4ce1a11ebe79cba3ef5529039a8f80e40c6f531`.
It contains the DOM log, selected gitlinks, hydrated statuses and strict
failure summary. The separate complete job log is 112,527 bytes, SHA-256
`e279d5b791f2d9fd809b7e697efe06637fe187a736c48f83b84f3554fa60b3ed`;
that log retains the complete SSR programs and the failed chained-step scope.

| Work                         | Original scope and owner                             | Remaining evidence                                                                              |
| ---------------------------- | ---------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| #7939 provider parity        | Full 228-byte original SFC; provider/history lane    | Cause, meaningful differential controls, source Actions, protected actual delivery              |
| #7860 input registration     | 330 tracked benchmark Vue inputs; coordinator        | Genuine corrected canonical 147-gitlink execution                                               |
| #7910 lint probe             | All 11 original validity plants; existing lint owner | Fresh repaired-head Actions and actual unchanged judge/CLI/JSON outputs                         |
| #7918 typecheck probe        | All 154 original projects; batch/typecheck owner     | Fresh repaired-parent Actions and actual whole diagnostic vectors/provider custody              |
| #7862 mixed-style scope      | Focused original style fixture; compiler/CSS owner   | Focused correction delivery; complete original 17 CSS and 33 runtime validators remain separate |
| #7887 Vapor CSS binding      | Existing P0 compiler owner                           | Its original reproduction and actual provider proof; no compiler fourth Stack layer is admitted |
| #7889 pre/textarea newline   | Existing legacy newline owner                        | Its independent original first-newline contract; shared cause with #7939 is unproven            |
| #7871 formatter preservation | Existing formatter owner                             | Its independent formatter contract; shared cause with #7939 is unproven                         |

Source probe instrumentation and runtime qualification are distinct. The
benchmark metadata/lint/typecheck native Stack has three layers; optional
compiler-recipe publication is deferred for P0 work. Neither this corpus run
nor a source probe admits the original benchmark validators, timing or rank.
Native/default routing, complete fix histories and the 10x target remain
unfinished. Preserve this failed run when qualifying any later correction.
