# Attribute entity source budget

Issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).
Pull request: [#8548](https://github.com/ubugeeei-prod/vize/pull/8548).

The current PR tooling check rejected `parser/attribute.rs`: the complete
attribute entity handler increased the file from 347 to 371 lines, crossing
the existing 350-line source budget. The original failure remains in
[run 38098169261](https://github.com/ubugeeei-prod/vize/actions/runs/38098169261/job/114348596398).

Move the existing cold `on_attrib_entity_value_impl` method into
`crates/vize_armature/src/parser/attribute/entity.rs`. The parent file becomes
348 lines. The complete method body, its cold annotation, and its position
in the existing attribute callback route stay unchanged. This introduces
no parser or pipeline stage.

The method previously used `pub(super)` inside `parser::attribute`; in the
nested module it uses `pub(in crate::parser)` to retain the same visibility.
The compatibility default, private full callback, historical BEFORE owner,
whole entity witnesses, and instruction ceilings remain intact.

The module move requires fresh source and instruction-count qualification.
Earlier Actions passes describe their original source heads; they do not
qualify the new head or close the adoption issue.
