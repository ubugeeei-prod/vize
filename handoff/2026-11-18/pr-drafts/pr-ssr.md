`ssr/no-browser-globals-in-ssr` no longer treats a template name as a browser global when the component may declare it as a prop. Template names resolve on the component instance, so a prop named `open` or `close` is never `window.open` or `window.close`.

Before this change the rule reported such names whenever the props type did not resolve in the file. The glyph lane found it in the Real Project Matrix lint-agreement surface (#12336):
- n8n `N8nPopover/Popover.vue` (`interface Props extends Pick<PopoverRootProps, 'open'>` plus `withDefaults(..., { open: undefined })`)
- elk `ModalError.vue` (`defineProps<ErrorDialogData>()` from an unresolved `#shared/types`)

A template name now counts as a possible prop when any of these holds:
- croquis resolves it as a prop;
- it is a runtime prop key (object or array form) or a destructured prop name;
- it is an authored `withDefaults` key, through a new `MacroTracker::with_defaults_declares` that also works when the prop's declaration is out of reach;
- it is a string-literal key of `Pick<_, 'name'>` in the props type, its local aliases or its interface heritage (members of object types are not heritage);
- the props type is imported and does not resolve, so it may declare any name.

The unresolved-shape check moves out of `vue/no-undefined-refs` into a shared `rules/props_shape.rs`, so both rules make the same decision. I checked the other rules that read prop names: they only lose findings when props are unknown, never gain them. Script-side reads are unchanged, since a prop never shadows a global in script.

Tests (`prop_evidence_tests.rs`):
- each evidence source, with the n8n and elk shapes;
- free globals next to complete local props are still reported.

The prop check runs only for a name that is already a browser global, so other identifiers cost nothing extra.

The input-only differential case `linter/ssr-browser-globals-prop-evidence-12336` pins both reductions.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
