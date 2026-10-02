# Checked attached binding construction (#6838)

The restricted L2 builder now provides `RegionBuilder::bind` for an explicit
static name and a retained expression value. The factory accepts the exact
authored argument range and whole directive range. It checks UTF-8, immediate
owner containment, exact nonempty argument bytes and the existing expression
coordinate/source contract before allocating or numbering a binding.

Only an active element or component frame can own a binding. Its private phase
allows bindings before the first successfully minted child. Rejected children
and bindings consume no id and do not advance the phase. Opening a nested owner
does advance its parent's phase, even if that owner has no children. Attaching
after a child or at the root returns a typed ownership error.

The same construction accounting mints the owner first, then attached bindings,
then child-region nodes. `mint_attached` does not advance the child phase. Private
frames retain bindings until their owner closes, including partial owners after
a caught callback unwind. The AST reference is retained unchanged. `finish`
continues to seal directly without a validation or counting walk, serialization
or expression parsing. There is no arbitrary binding-vector insertion API.

Five new laws check exact attached page order and pointer retention, atomic
root/late rejection, source and payload ownership, nested-owner phase, and
caught-unwind retention. The independent arbitrary-parts checker agrees in
test code only. On the actual security-main provider prefix, the real-source
scoped canonical-module harness passes 39 laws, including the lower builder's
pending nested-owner refusal. Whole L2 production Clippy passes with genuine
cached dependencies. Whole-workspace
Actions, unchanged instruction/allocation gates and terminal merge remain
required; these scoped checks are not that evidence.

This provider is the prerequisite for a native Vue directive pattern consumer.
Dynamic arguments, spread bindings, modifiers, shorthand values, event-handler
grammar selection, model/control-flow/slot scopes and all product integration
remain unfinished. No product route or legacy output changes. #6838 stays open.
