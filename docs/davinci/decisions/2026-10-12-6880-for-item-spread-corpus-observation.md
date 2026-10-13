# Loop item spreads and corpus observations

Decision for [#6880](https://github.com/ubugeeei-prod/vize/issues/6880) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830),
with the six compiler failures observed by [#8747](https://github.com/ubugeeei-prod/vize/pull/8747).

The shipped compiler routes object `v-bind` and `v-on` spreads on a loop
item through `mergeProps`. L2 must retain that choice even for one spread.
The existing `for_item` flag identifies the current item; ordinary children
inside its loop keep their existing single-spread behavior.

The original [native job](https://github.com/ubugeeei-prod/vize/actions/runs/38211248000/job/114685682328)
at source `bed2094` retained 527 files and 1,174 S2 remarks, including the
80 additions. Its [compiler reach job](https://github.com/ubugeeei-prod/vize/actions/runs/38211248370/job/114685875273)
retained the six failures for `ForLocal`, `RootFor`, and `RootForLocal`, in
both inline and module modes. Whole outputs beyond the retained comparison
windows were not observed.

The first source change carries the exact 25 Theme and 24 Transition Vue inputs.
It adds their file census and only the 80 Theme S2 remarks already retained by
the original failed native job. Removing those additions restores every byte
of the old S2 baseline. The old S3 baseline is preserved.

Before its unchanged assertions, S3 prints complete observations for new files.
Ordinary Actions must supply the missing S3 and Transition S2 rows; this source
change does not bless them or change any assertion, budget, or failure status.
The original six compiler failures remain retained; ten complete template
comparisons supplement that corpus.

The first [ordinary source check](https://github.com/ubugeeei-prod/vize/actions/runs/38218140053)
at `3a97908` passed all three canonical observers and both template tests
(eight item-root and two ordinary-child comparisons). The unchanged S3 file
assertion failed with 551 current files and 171 remarks: its complete new-file
dump contains 49 paths and 20 entries, five Theme and 15 Transition entries.
The additive follow-up preserves every old S3 byte and registers only those
observed paths and entries.

The S2 corpus test requires `legacy-differential` and was absent from all four
PR JUnits. No zero-entry claim is made for Transition S2. After the finite
follow-up passes every exact-head source check, independent squash admission
uses the genuine protected full suite to qualify S2 and all original gates.
A red candidate is promptly dequeued; its exact observations are retained and
only authenticated additions are repaired before fresh qualification.

Source review and input custody are not native execution or release evidence.

The first protected candidate `3183e16` ran the genuine [feature corpus](https://github.com/ubugeeei-prod/vize/actions/runs/38224948265/job/114731832255).
S2 observed 551 files and 1,228 remarks, with 54 additions on 14 Transition
paths. Its complete original log supplies those exact rows. Removing only
the added entry rows restores every byte of the preceding 1,174-row baseline;
the 551-file census and all earlier records are retained.

That candidate also failed MoonBit setup before compilation in
[Nuxt 3](https://github.com/ubugeeei-prod/vize/actions/runs/38224948215/job/114730051710)
and [Musea inline Art](https://github.com/ubugeeei-prod/vize/actions/runs/38224948167/job/114730051998):
the pinned Linux archive GET returned HTTP 403. These failures do not attribute
a product or compiler defect. The candidate was automatically dequeued and
the PR returned to Draft. Fresh source and protected qualification require
the finite S2 additions and a separately qualified installer repair; no
unchanged retry, cache waiver, or MoonBit downgrade is authorized by this record.

The generated [backlog](../plan/remarks-backlog.md) follows the same complete
S2 corpus: 551 files, 1,228 remarks (233 applied, 995 missed), and 27 missed
reasons. The unchanged primary generator was projected from the exact original
corpus to reproduce the old document byte-for-byte, then from the authenticated
80 Theme and 54 Transition additions. This source projection preserves the
original grouping and final assertion; protected execution is still required.

The 7274fd88 source check failed before tooling tests during shared CLI
restore: its unchanged 30-second rustc -vV identity probe timed out.
The outer ZIP digest matched; internal archive and binary custody were not reached.
The child stderr was not retained, so its exact timeout cause remains unknown.
Setup had installed stable, while this project's declaration selects
1.99.0; the same-run producer subsequently downloaded that named toolchain.
The existing producer and consumer setup steps now provision the exact declared
version, components and targets before receipt authentication. Deadlines, strict
identity checks, original recipes and every gate remain unchanged. The failed
packet is retained; fresh source and protected qualification are required.
