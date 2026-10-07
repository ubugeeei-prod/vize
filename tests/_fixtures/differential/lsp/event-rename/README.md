# Original event rename transactions

The complete original #8010 report and its Toggle/App source blocks are frozen
under `8010/`. Toggle is also byte-identical to the event source in #8011,
whose complete report is preserved under `8011/`. The earlier library-refusal
corpus and all existing controls remain unchanged.

The independent `UpdatedToggle` and `UpdatedApp` goldens change only the event
key, emit-string content, and parent listener name from `change` to the reported
`update`. The three origins run with LF and CRLF through the actual stdio CLI.
Each requires exactly three full references and three edits, applies the actual
returned transaction, writes both files, and checks version-2 diagnostics.
Independent golden installation then requires empty version-3 diagnostics.
Complete equality rejects extra library/generated targets, missing sites,
duplicates, truncated ranges, and changes to the handler, DOM click, or payload.

The original reports describe compiler options without literal configuration
bytes. The test uses the existing pinned-Vue cross-file project helper and
retains its actual generated configuration in each capture. The complete
reported `src/` source bytes are installed as project-root Vue files; this is a
declared harness layout, not a claim of literal original configuration identity.

This corpus qualifies the original event transaction only after actual current
source/native execution. Configured in-root runtime assets and bare-script
exporter-origin routing in #8010, slots in #8011, and standard stock tsgo
camel/kebab Content Mapper acceptance in #4075 remain separate obligations.
