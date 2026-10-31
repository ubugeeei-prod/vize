# Direct CSS quote override (#6098)

The three authored whole-output fixtures isolate the existing CSS, SCSS and
Less quote policy. They retain comments, an escaped old delimiter, and an
alternate quote that avoids additional escaping. CLI tests exercise explicit
true/false against config and presets, both quote preferences, three full
write passes, an unchanged configuration file, and final `--check` acceptance.
No formatter algorithm, default, schema or preset assignment changes here.
