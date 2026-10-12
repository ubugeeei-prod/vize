# Raw Git source authority

Issue: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).

## Problem

The shared warm editor source reader accepted Git replacement objects. A real
private repository reproduced two ways to borrow a reviewed body: replacing
the requested commit with another commit, or replacing its blob after the tree
lookup. Both returned the reviewed SHA-256 while the literal requested source
contained different complete bytes. This affected source qualification, rather
than native editor behavior.

## Decision

Disable replacement objects in both existing Git reads in `gitBodyDigest`:
the tree lookup and the complete blob read. Preserve the regular `100644`
mode requirement, exact path, absence handling and whole-body digest. The
sixty-body path manifest, all four existing qualification laws and the existing
default migration and mode refusal laws remain unchanged.

Two genuine private Git controls retain active commit and blob replacements.
Both fail on the original reader and pass on the corrected reader. A normal
Git read still demonstrates the borrowed bytes; a replacement-disabled read
and the shared reader must return the requested source bytes. This prevents
the regression from being hidden by a mock digest.

The repair starts independently from actual `main` at
`5459e40a23ca7e647ace8ebd20e1fe00c9294437`. It imports no unqualified native graph
or SDK parent. Existing graph and SDK successors must bind the complete changed
reader body explicitly when composing this actual merge.

## Qualification

The first exact-head original400 run `38199886718`, job `114650727981`,
refused the new companion as an unqualified harness path before native build
or provider execution. Preserve that raw failure. Its limited successor binds
the complete before/after reader, added laws and companion, plus the unchanged
sixty-body manifest and both original law files. Only these exact doc/test
paths join the inherited harness after all six pairs match. Unknown, omitted,
duplicate, changed-body and mode refusals remain mandatory; the original
workflow classifiers, corpus bytes, recipes, whole oracles and budgets remain.
The actual-source law prepares only the literal original full commit above if
missing in a shallow checkout, with a depth-one replacement-disabled fetch;
all six complete raw body pairs remain required after setup.

Fresh exact-head Actions and the protected merge queue remain required before
delivery. Source laws grant no native, resource or speed credit. The retained
cold 71.162-second result, RunData timeout and 129.21484375 MiB server peak
remain unresolved under the original workloads and budgets.

## Protected source composition

Exact `293ec2d` passed its PR checks, native phases and original400 whole-packet
comparison. Protected candidate `d8d74a00` then failed the new six-body positive
in tooling shard 3/4. Actual main `dd188515` supplies the legitimate #8735
default membership law: its complete body is different from the historical law.
All other original six-pair tree entries remain exact. The failed candidate was
removed from the queue; its complete raw log and source comparison are retained.

Keep the original six pairs and refusals. Their positive reads the actual raw
`293ec2d` after bodies, including the complete historical seven-law file. Add a
separate finite incoming tuple with the complete current law and its authentic
runtime API, preset registry and membership custody. Require every named body
and absence on both sides; reject unknown, missing, duplicate or changed bodies.
The existing semantic authenticators still check the whole incoming family and
five membership insertions. They do not make the old measured migration host
eligible. Fresh source, native, original400 and protected qualification remain
required; no recipe, RPC, corpus, deadline or resource cap changes.

## NoDeep source composition

Exact `3dda8f9d` passed source checks, native phases and the original400 comparison.
The actual NoDeep merge `9764c8e2` then changed five bodies in the current family.
Keep the immutable 79-pair manifests and all historical refusal laws. Authenticate
the complete raw `3dda8f9d` module and its imports before executing its original
four laws in an isolated source fixture; its historical result grants no current
source or runtime credit.

The separate current tuple binds actual main `eafe5bed`, containing that merge,
all 87 recursive family and declared helper bodies, the final NoDeep twelve-body
authority and its actual producer. Complete before/after bytes and regular modes remain mandatory, with
unknown, missing, duplicate and replacement borrowing refused. Fresh source,
native, original400 and protected qualification remain required for this tuple.

## Shallow source preparation

Exact `6354e9b9` passed native phases and original400, but both tooling shards
failed source laws. The historical fixture lacked literal `dd188515`, the
independent inverse lacked `3dda8f9d`, and shallow markers hid real ancestors
from `merge-base`. Preserve their complete failures and the separate successes.
Prepare each required literal source independently before its law. A bounded
raw first-parent proof authenticates complete commit bytes and parent identities;
missing objects use exact depth-one fetches. Disconnected or replaced ancestry
cannot borrow authority, and no shallow marker or workload budget changes.

Keep the original 102-pair manifests, guard and refusal bodies. Their separate
setup successor binds every previous body and only the named source-law repairs.
The historical ASCII caller comparison follows complete current ten-body source
authentication; the original, reviewed SDK and reviewed reader frames remain
distinct, with no partial or mixed frame accepted. These checks grant no native,
resource or speed credit. Fresh normal Actions and protected delivery remain
required; the original cold, RunData and RSS blockers remain unresolved.
