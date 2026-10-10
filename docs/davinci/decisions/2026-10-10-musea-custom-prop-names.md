# Musea literal custom prop names

Tracking issue: [#8431](https://github.com/ubugeeei-prod/vize/issues/8431).

## Decision

Keep ordinary usage attributes byte-exact. Put names containing template syntax
or directive prefixes into the existing escaped, typed object `v-bind` rather
than interpolating them into attribute syntax. Spaces, dots, quotes, `v-if`,
`@event` and `:handler` remain literal keys.

Editor defaults, merged values and restored values use own-key construction, so
an accepted `__proto__` key cannot change the dictionary's prototype or disappear
from Current Values and saved state. The saved format/version stays unchanged.

If a defined own `__proto__` key exists, bind the complete values object once.
Mixing ordinary attributes with that object invokes Vue's `mergeProps` assignment
path, which changed its prototype and leaked an inherited attribute in the real
clipboard execution regression. One complete object binding keeps the key own
until Vue handles it. Vue's intrinsic reserved-name behavior stays intact:
`v-if` used as a prop resolves to `vIf`; Vue ignores the direct reserved prototype
key. No inherited `retained` attribute is permitted.

## Regression and evidence

The new differential fixture is
`tests/_fixtures/differential/musea/props-usage-names.json`. The existing typed
usage fixture and all its runtime laws remain unchanged.

Source execution and actual Add Prop/clipboard execution initially rendered
`<!--v-if-->` instead of the component. The own-key browser extension then timed
out because Current Values lost the accepted key. The mixed-binding clipboard
regression exposed the inherited attribute; the complete binding corrects it.

Source laws execute the copied markup for both unusual-only and mixed names,
assert full component props and attrs, and check own-key defaults/restoration
without changing the resulting dictionary prototype. The real gallery adds all
fixture names through controls, verifies whole Current Values and exact clipboard
against its display, executes that copied template, saves, reloads the gallery,
and verifies the same complete values and clipboard again. Observations retain
fixture, raw values, expected/received props and both whole clipboard strings.

The corrected combined local source/Chromium contract passed eleven tests,
including the existing palette replacement/storage laws. The complete original
source/copy/scroll/theme and palette response-order regressions remain registered.
Seven changed TypeScript files passed formatting/lint with zero warnings.

This leaf depends on the palette replacement PR in native Stack #8421. Fresh
exact-head Actions, qualified prefix queue admission, protected checks, actual
merge and public publication remain required. Local results grant no release
completion credit.
