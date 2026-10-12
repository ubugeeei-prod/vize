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
The PR stays Draft and outside the merge queue until an authenticated additive
successor and the original full gates pass. The original six compiler failures
remain retained; ten complete template comparisons supplement that corpus.

Source review and input custody are not native execution or release evidence.
