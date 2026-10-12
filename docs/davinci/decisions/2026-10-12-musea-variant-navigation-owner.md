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

Initial ref readiness is checked after that existing `nextTick`, alongside the
generation and current-tab checks. A separate complete-production-block control
retains the original empty-before-render refusal and proves that refs installed
during the existing tick become observed.

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

Nine source ownership controls currently pass locally with no skips. Actual
source-native physical controls, complete Actions and protected integration are
pending. Source CLI retry #8721 remains outside the queue until this independent
repair actually merges and its fresh composition qualifies. Existing public443
and retired unpublished446 grant no current repair or installed-release credit.

## First source refusal

Source15d40 run38193195886 retained all original native browser laws passing, but
the new observer probe failed before qualification: tsx inserted host-only
`__name` references into a serialized browser initialization closure. Artifact
11721962032 SHA256
`b016d85848416274c316e29831b5d5421faceba23629f1692c206ec04f9798af`
passed all1,929 ZIP CRCs. Its whole own packet SHA256
`c20435a496bcc1cd1fafe5002115a9f9d54eae14c03f9ac2471387f34910c9d4`
records zero observers and `ReferenceError: __name is not defined`; those zero
records do not prove a production ref-readiness cause. Actual synthetic checkout
`0b16b8c582c83f370483faa6b4fbb221f8a96a2c` and loaded source GNU provider
SHA256 `72c25b4301a28731010f59c00cde00b6f2d704546c2e487483704b0b497fc8a6`
remain retained. The corrected probe loads raw authored JavaScript by path.
Its separate actual Chromium transport law passes locally on macOS with no
Vize native producer qualification. The original three owned zero-warning lint
refusals are repaired by structural browser types and `node:vm` execution of
the authenticated source block; no warning rule or original oracle is waived.

The same source Check's tooling4 law refused ComponentView growth807 versus
baseline784. No length allowance is added. Move-only commit66f4cf3eb5 extracts
only the owned navigation state/functions into `gallery/utils/variantNavigation.ts`,
with every moved function body byte-exact and the same captured refs/route/tick.
The view's actual watchers and ref cleanup remain. Subsequent formatting and
control adapters execute the complete real helper and view cleanup; the test-only
module-export transport is reversed byte-exact. Fresh qualification remains
required for this composition.

Source0a27842 run38195433366 retained source9 and raw native probe1 passing.
The complete authenticated original artifact11723008109 is 53,111,635 bytes,
SHA256 `ec0642b8d3a5de67f8e9fd5cc8737b0120090836304ceff0949f512c8394c781`,
with all1,931 CRCs valid. Its whole97,962-byte observation packet SHA256
`8ee8385dbba9b74cd290383e99b90b8d0fcdc3d5f6bd5fcdaf7ba680c5072639`
has no page/console errors. Controls2, both rapid directions, direct hash and
keyboard navigation pass with seven actual native observers/deliveries. On
Controls→Keys, all three original previews/IDs render correctly but every old
observer is disconnected and no new Keys observer is constructed. Actual
bundle SHA256 `2d453ee17290fdae2480af172f83988e674af8f27b571898293c8582e90e15bf`
contains the current helper semantics; source checkoutcf9a0fbd and loaded GNU
provider `efe11781ff1a6ded93ddf25950d4f2184eaad8cabe4d3b103acf925e40304a88`
are retained. This is a route ref-readiness refusal, not a transport error or an
initial-mount failure. Place only the existing navigation watcher at `flush: post`
so its same controller setup runs after view refs update; no mounted hook, timer
or additional scheduling delay is added. Fresh physical qualification remains.

All authored physical controls use the original default Chromium viewport.
Its real recorded IO root is y129..346 at1280×720 and post-scroll target ratio
is0.3799, so the unexecuted extra tall viewport is unnecessary. Manual, retained
callback and both compiled perturbation assertions/targets remain intact; no
tall-viewport acceptance is claimed. Competing router requests test rapid
replacement, while committed pending continuations are qualified by the distinct
deterministic complete-source controls.
