`ssr/no-browser-globals-in-ssr` no longer treats a template name as a browser global when the component may declare it as a prop. Template names resolve on the component instance, so a prop named `open` or `close` is never `window.open` or `window.close`.

Before this change the rule reported such names whenever the props type did not resolve in the file. The glyph lane found it in the Real Project Matrix lint-agreement surface (#12336):
- n8n `N8nPopover/Popover.vue` (`interface Props extends Pick<PopoverRootProps, 'open'>` plus `withDefaults(..., { open: undefined })`)
- elk `ModalError.vue` (`defineProps<ErrorDialogData>()` from an unresolved `#shared/types`)

A template name now counts as a possible prop when any of these holds:
- croquis resolves it as a prop;
- it is a runtime prop key (object or array form) or a destructured prop name;
- it is an authored key of the top-level `withDefaults(defineProps(...), { ... })` object, read from the script setup AST the lint pass already holds, so it also works when the prop's declaration is out of reach (a spread or computed key may name any prop);
- it is a string-literal key of `Pick<_, 'name'>` in the props type, its local aliases or its interface heritage (members of object types are not heritage);
- the props type is imported and does not resolve, so it may declare any name.

The unresolved-shape check moves out of `vue/no-undefined-refs` into a shared `rules/props_shape.rs`, so both rules make the same decision. I checked the other rules that read prop names: they only lose findings when props are unknown, never gain them. Script-side reads are unchanged, since a prop never shadows a global in script.

Tests (`prop_evidence_tests.rs`):
- each evidence source, with the n8n and elk shapes;
- free globals next to complete local props are still reported.

The prop check runs only for a name that is already a browser global, so other identifiers cost nothing extra.

The input-only differential case `linter/ssr-browser-globals-prop-evidence-12336` pins both reductions.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

<!-- codesmith:footer -->
---
<a href="https://backend.blacksmith.sh/track/enable-autofix?expires=1797518637&installation_model_id=422833&pr_number=13278&ref=codesmith_pr_footer&repository=ubugeeei-prod%2Fvize&return_to=https%3A%2F%2Fgithub.com%2Fubugeeei-prod%2Fvize%2Fpull%2F13278&signature=402756fa64e16733114eaa7e885e05f78bcea4421a6fea3ca4dc344e0c99c040"><picture><source media="(prefers-color-scheme: dark)" srcset="https://pr-comments-assets.blacksmith.sh/codesmith/autofix-with-codesmith-dark.svg"><source media="(prefers-color-scheme: light)" srcset="https://pr-comments-assets.blacksmith.sh/codesmith/autofix-with-codesmith-light.svg"><img alt="Autofix with [code]smith" src="https://pr-comments-assets.blacksmith.sh/codesmith/autofix-with-codesmith-dark-3x.png" width="190" height="28"></picture></a>
<sup><a href="https://app.blacksmith.sh/ubugeeei-prod/codesmith/vize/pr/13278?autoLogin=true&ref=codesmith_pr_footer">Open in [code]smith</a> · Need help on this PR? Tag <code>@codesmith-bot</code> with what you need.</sup>

<!-- codesmith:autofix:disabled -->
<!-- /codesmith:footer -->

<!-- This is an auto-generated comment: release notes by coderabbit.ai -->
<!-- coderabbit-ai-generated:v1 -->

## Summary by CodeRabbit

* **Bug Fixes**
  * SSR browser-global checks now recognize names declared as component props, reducing false-positive warnings for props such as `open` and `close`.
  * References to browser globals not declared as props continue to be reported.
  * Undefined-reference checks now account for prop types that use imported or inherited definitions.
* **Documentation**
  * Updated source-code links in the undefined-reference rule documentation.
<!-- end of auto-generated comment: release notes by coderabbit.ai -->
