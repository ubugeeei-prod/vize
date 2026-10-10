# Musea custom replacements for deleted props

Tracking issue: [#8418](https://github.com/ubugeeei-prod/vize/issues/8418).

## Decision

A deleted palette name hides the original generated control. An active custom
prop of that name owns its current value and remains visible. Restoring saved
state permits this explicit replacement only when the original palette prop is
deleted; duplicate custom names and collisions with undeleted palette props
remain rejected.

Removing the replacement leaves the original control deleted. Reset clears
custom controls and deletion state, restoring the generated defaults. Saved
state format and version remain unchanged.

## Regression and evidence

The differential fixture is
`tests/_fixtures/differential/musea/palette-replacement.json`. It replaces a text
`tone` prop with a numeric control/default, then edits the numeric value.

The production restoration helper initially discarded the saved replacement;
its new source regression failed while both existing storage/collision laws
passed. The actual `usePalette` composable in Chromium initially omitted the
replacement value from merged props. Its browser regression retains complete
before/after snapshots in the existing observations artifact before assertions.

The corrected source tests pass, including the existing undeleted-collision
law. The browser verifies undeleted duplicate rejection, replacement default
and edit, full saved state, reload, removal, reload-after-removal and Reset,
followed by the existing complete gallery source/copy/scroll/theme contract.

This is a dependent slice of the palette response ownership PR: use a native
Stack, preserve its exact parent, and require fresh source Actions before prefix
queue admission. Protected queue checks, actual merge and publication remain
required. Arbitrary custom-name copied markup remains separate unfinished work.
