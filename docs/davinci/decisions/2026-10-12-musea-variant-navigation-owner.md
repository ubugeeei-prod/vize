# Explicit variant navigation owns its target

Issue: [#8731](https://github.com/ubugeeei-prod/vize/issues/8731).

## Observed failure

Protected Musea run
[38189727825](https://github.com/ubugeeei-prod/vize/actions/runs/38189727825),
job114619686264, checked source
`b4e8a0289255236cedf0ef6b879b0af1b6810431`. Its original literal-key
browser law failed after clicking last, then first: first lost `aria-current`,
last became current, and first remained above the viewport while last was visible.
All three literal names and section IDs remained intact; page and console errors
were empty. This is a navigation failure rather than evidence of dictionary damage.

Original artifact11721336653 is 48,810,252 bytes, SHA256
`94fa026fece33961e675f728caac89fcb845a1284825b7509f7157300d1ea80a`.
All 1,747 ZIP members passed CRC verification. The complete 20,859-byte original
static observation packet is retained in the differential corpus at
`tests/_fixtures/differential/musea/variant-navigation/original-failure.json`,
SHA256 `5053496fd0acf831fcde4b5beba93a911ee1fa8c2f585a36d778ce97f3283f0c`.
Its five physical preview file hashes and complete Controls/Keys fixture sources
were authenticated against the artifact and candidate. Actual emitted gallery
bundle SHA256 was
`c83c0afdf26545582171a3022959df7e0973d152eb5007a2e9c2e2383fc5d149`.

The raw job proves checkout, a source-native build and browser execution, but
does not retain the actually loaded `.node` digest. Specific observer/click event
ordering was not captured. Neither missing native custody nor an inferred event
order is converted into proof of a particular callback causing this failure.

## Decision

Explicit TOC and hash navigation scroll immediately. They invalidate the previous
observer before changing selection, and attach a fresh observer after the existing
Vue `nextTick`. A generation refuses both formerly owned callbacks and superseded
async observer setup. Ordinary manual scrolling keeps the existing intersection
comparator, margins and thresholds.

Hash continuation additionally captures and validates its hash, art path, active
tab and generation after `nextTick`. Executing the whole original production
navigation block with controlled continuation resolution demonstrates that the
old function can scroll after tab/route/unmount/hash/click replacement. The
corrected block must refuse those continuations. This adds no timer, scheduling
delay or new production dependency.

## Qualification

All original literal-key, hosted, capture and navigation browser predicates and
deadlines remain byte-exact. New controls use the unchanged original Controls2
and Keys3 sources. They require physical target geometry and exact aria after
genuine freshly attached native observer delivery, rapid navigation in both
directions, hash and keyboard navigation, manual scrolling, tab/route replacement
and unmount. Observer instrumentation forwards original native entries unchanged.
Deliberate replay of a formerly owned callback is labelled as a causal control,
not an observation of the original unrecorded event sequence.

Isolated copies compile the complete actual gallery with exactly one perturbation:
removing the callback generation guard or restoring smooth explicit scrolling.
All other gallery files, the production build config, original native preview
modules and static payload remain exact. The perturbed browser must expose its
own physical/aria counterexample; missing errors or marker-only success is
insufficient. Raw observer entries, actual scrolling ancestors, HTML, PNG,
source/build inventories and loaded native provider digest are retained.

Eight source ownership controls currently pass locally with no skips. Actual
source-native physical controls, complete Actions and protected integration are
pending. Source CLI retry #8721 remains outside the queue until this independent
repair actually merges and its fresh composition qualifies. Existing public443
and retired unpublished446 grant no current repair or installed-release credit.
