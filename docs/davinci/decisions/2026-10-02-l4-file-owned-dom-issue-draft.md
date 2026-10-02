# Prepared issue comment for #6840

Implemented a genuine file-owned native DOM entry for static structure and
retained literals. `emit_file(&NativeFileAnalysis)` accepts no caller binding
policy or replacement tables. Its private static backend authenticates the
actual file/node/scope/AST/source/coordinate row, then uses the checked span
writer only for literal ASTs with complete empty reference tables.

Admitted references receive a precise typed `RuntimeAccessUnavailable`; FileDependent,
script identity and const/setup syntax never become `_ctx`. L3 grouping,
semantic dependency order and block eligibility drive the same append encoder;
L4 owns checked helpers, patch bits, props and late complete imports. Refusals
return no partial writer and retain the original analysis/source observations.

Eight genuine Program/Vue Component/File/L3 pipelines match every complete
Vue 3.5.35 template-module byte under the recorded six-parameter render options.
Recorded/NoLinks agree. Real runtime execution, entity/Unicode/comment links,
setup scope/binding shadowing, equal numeric foreign owners and L3 special/zero-ref refusals
are covered. Fifty-one L4 unit laws, four genuine file integration laws and
strict whole-source production Clippy pass; the DOM skeleton ratchet falls from
four to three. Evidence uses authenticated source libraries, not a whole-current
Cargo build, hosted Actions or performance result.

Whole-driver/SFC completeness, actual same-file Vue runtime exposure, file
If/For, broader dialects, raw upstream-map equivalence, hoist/cache, unchanged
performance acceptance and final provider/Stack/Actions/queue delivery remain
unfinished. No production route switch; #6840 and #6880 remain open.

Decision: [file-owned native DOM emission](./2026-10-02-l4-file-owned-dom.md),
paired with the central level-restructure record in this same change.
