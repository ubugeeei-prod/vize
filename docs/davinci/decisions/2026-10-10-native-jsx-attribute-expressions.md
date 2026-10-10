# Retain native JSX in authored attribute expressions

## Decision

The original Ant Design project at 7483836f, unchanged compiler options and the
same TS2322 Vue probe lost semantic diagnostics with explicit JSX enabled.
Corsa process captures identify native syntax errors in generated attribute
values, which abort program semantics; TS5023 belongs to the independent original
unsanitized configuration packet. The semantic-loss cause is native AST retention,
not configuration error handling. Preserve both errors and all original inputs.

Reuse already parsed OXC expressions for container and spread attributes, and
retain direct JSX element/fragment values through the existing native lowerer.
Extract the existing element/fragment enqueue sequence into shared private
helpers: isolate outer pending styles, lower one native root, capture its owned
scoped style, restore outer styles and enqueue the same typed root. Guard each
helper to preserve ordinary compilation. No extra parse, stage or serialization.
Skip attributes already owned by the slot path. Keep the unchanged attribute
expression projection in a separate move-only commit and preserve public APIs.

## Custody and qualification

The newly authored native root-vector law runs unchanged before and after repair
and fails on the original producer. Both normal
runtime VDOM/Vapor full output/map/metadata packets were frozen before behavior
changed and remain exact. The new required-native CLI corpus uses the existing
mandatory target and compares four whole reports: positive, mapped member
negative, semantic-plus-config errors, and explicit false control.

All 104 JSX library laws and four whole native CLI reports pass locally. The
repaired true and false Ant reports both equal the original authored false
report byte for byte: SHA256
996532fc2228d6c9cb58476a669d0492f52a113f279c65bbbc456a85fe328ae4.
Original TS5023, TS2322, four program inputs and every option remain unchanged.
Complete original/repaired packets, input hashes and lossless logs are retained
in the corpus. Repaired observations explicitly identify unsealed source custody;
local results grant no Actions, protected queue, merge, release or speed credit.

This source repair is a genuine native Stack child of the nested-expression
producer. #8371 fresh configuration defaults remain held until actual source
ancestry, full exact-head Actions, protected delivery and installed verification
qualify. Existing historical JSX/configuration oracles are not changed.

Hosted custody routes the same four CLI vectors through the existing JSX capture
root. Passive records include the actual spawned PID, complete command arguments,
source revision and streamed CLI/Corsa executable hashes, alongside unchanged
raw stdout/stderr and full inputs. The original Ant observation remains sealed
at its original false-authority hash; its historical local receipt supplies no
new-head Actions credit. Fresh source/native/full checks remain required.
