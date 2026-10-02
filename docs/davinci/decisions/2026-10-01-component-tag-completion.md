# Authored component tag completion (#7194)

Tag-name completion in ordinary Vue templates must offer script-setup values,
Options API component registration names, workspace `GlobalComponents` declarations,
and native HTML/SVG elements. The existing directive and built-in snippet fallback
does not meet that contract for `<Chi` beside an imported `Child.vue`.

The native tag provider runs before Corsa template-expression completion. It uses
the resident SFC descriptor and the existing Croquis scope analysis for local names,
filters type-only imports and prop bindings, and offers PascalCase and kebab-case
component names. It accepts opening and closing names, replaces the complete
authored name, and leaves the existing `<` or `</` in place. UTF-16 replacement
ranges include the suffix after the caret. Attribute values, comments, and Vue
expressions continue through their existing providers.
Bare `<` and `</` positions offer element/component names and exclude attribute
directives. The prior directive-snippet assertions at these tag-name positions
are replaced by authored native-name inclusion and directive exclusion controls;
attribute-position directive/event checks retain their existing expectations.

Workspace declaration discovery keeps its `.git` exclusion and existing cache.
Declaration names are cached by disk metadata or the open document revision;
closing/reopening an editor buffer cannot reuse an earlier buffer's names.
Cold declaration reads and parsing share one background batch, while unchanged
requests reuse cached names. Declaration inputs are capped at 4 MiB.
Open buffers are checked before copying their text. A failed background thread
creation logs a warning and retains cached names without panicking the request.

The owning `crates/vize/tests/fixtures/tag-completion` corpus reproduces the Issue.
Unit checks cover exact UTF-16 edits, closing-name suffixes, type-only exclusion,
Options API names, invalid tag contexts, module augmentations and inheritance,
and unsaved/reopened global declarations. An actual stdio LSP integration target
asserts complete selected completion edits for native, imported, global and renamed
components. Terminal Actions validation is required before merging.
