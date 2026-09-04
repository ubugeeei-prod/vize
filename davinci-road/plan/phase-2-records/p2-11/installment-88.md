# P2-11 Installment 88 - Self-Referencing Components

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5622](https://github.com/ubugeeei-prod/vize/pull/5622), merged into
> the installment-84 branch and carried to `origin/main` by
> [#5615](https://github.com/ubugeeei-prod/vize/pull/5615) at `e9923c0f8`.

`component_name`: the SFC's own name, which `extract_component_name` takes
from the file stem. A component tag equal to it verbatim, or after camelize
plus PascalCase, is a self-reference, and the shipped lane asks the runtime to
resolve it as one - `_resolveComponent("Foo", true)`.

The bindings corpus lane derives the name the same way and passes it to both
sides, so the option is exercised over the hydrated tree rather than the
committed battery alone.
