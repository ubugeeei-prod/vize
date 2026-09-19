# Array-loop reference contract

`rust-lowered-loop-{keyed,unkeyed,nested}` each owns one authored template,
Rust-verified S3 graph/value Folios, scenario, and expected behavior artifact.
The `.behavior.json` observations were specified independently of compiler
output. Only the graph/value/operation-label artifacts have an explicit Rust
regeneration option, `VIZE_UPDATE_LOOP_REFERENCE_FIXTURES=1`.

The `identities` observation pairs each visible `data-id` with the lifetime
number of its actual element. The mounted runner assigns numbers with a
`WeakMap`, in element preorder, including unlabelled ancestors and siblings.
It never derives identity from the key or the `data-id` spelling. Removed
elements must be disconnected; unmount must leave an empty host.

For the flat fixtures, `main` is 0, the initial buttons are 1 and 2, and the
trailing paragraph is 3. Reordering preserves key-associated lifetimes in the
keyed case and positional lifetimes in the unkeyed case. Inserting a third
button allocates 4. Emptying and refilling allocates 5, even when its key was
previously present. The paragraph keeps identity 3 across list-size changes.
The nested fixture additionally moves an item to another parent: its identity
must change although its key is unchanged. Reintroducing a removed parent
also creates new parent and descendant lifetimes.

The reference owns a materialized tree and monotonic lifetime allocator.
Reconciliation matches S3 operation/scoped-iteration addresses within the
previous parent, patches attributes, text and captured event values, inserts
missing elements and drops removed subtrees. Each result is compared against
fresh S3 rendering. All pairs of ordered subsets of three keys exercise
insertion, removal, permutation and empty/refill for keyed and positional
identity, including fresh event values and exact allocation counts.

The input contract is S3 operations and operands. `for-key` is the second
iteration alias (the index for arrays); it does not encode reconciliation
identity. This slice reads the normalized repeated-root `key` binding as that
identity. Loop scopes use typed JSON path components, so string and integer
keys and nested owners stay distinct. No source-template parser runs in Lean.

Supported execution is JSON arrays, simple lexical aliases, direct member
paths, one native root per iteration, nested loops, string/integer keys,
`data-id`/`title`/boolean `disabled` bindings and `record(path)` string payloads.
The fixtures verify index bindings and shadowing of the outer `item` binding,
including restoration after a nested loop. Duplicate/unsupported keys,
malformed operands, ambiguous interaction targets and unsupported expressions
fail closed. Arbitrary JavaScript, destructuring, object/range iteration,
multi-root template loops and scheduling/linearity proofs remain outside this
subset. This reference update algorithm makes no production complexity claim.
