# Vue 2 edit-cycle diagnostic context custody

The two live Vue 2 editor suites now consume `successor.json` as complete CLI
expectations. They retain their original exact LSP publications, authored ranges,
three dialect configurations, data-key-only case, clean/broken/restored cycle and
source restoration checks. No diagnostic text is stripped before comparison.

`historical.json` preserves the original complete CLI expectations. The two
`*.historical.txt` files preserve the original suite bytes at main
`284c7c4f6a7a1a50d9acec8d877c47aae21cfe42`, independently hash-pinned in
`custody.json`. The custody test constructs the full successor from those
expectations by adding only the reviewed template source and binding context.
It also rejects mutations to the source context, binding, canonical message,
coordinates, diagnostic inventory, script diagnostic and clean packet.

`TransactionTable.vue.txt` is the complete independent upstream source from
[PanJiaChen/vue-element-admin at 6858a9ad67483025f6a9432a926beb9327037be3](https://github.com/PanJiaChen/vue-element-admin/blob/6858a9ad67483025f6a9432a926beb9327037be3/src/views/dashboard/admin/components/TransactionTable.vue).
Its upstream Git blob and SHA256 are pinned; `LICENSE` retains the upstream MIT
license. The template source on line2 and script member on line50 independently
establish both authored diagnostic ranges. The edit variants rename the data key
and optionally its script use; neither edit changes the template line.

The complete successor was reproduced by a source-built CLI from the exact main
revision in [Actions40171762851](https://github.com/ubugeeei-prod/vize/actions/runs/40171762851).
The artifact `vue-context-packets-284c7c4f6a7a1a50d9acec8d877c47aae21cfe42-40171762851`
retains all twelve phases, raw CLI stdout/stderr, provider versions and binary
hashes, source-build receipt, complete JSON-RPC session wires, authored source and
the unchanged historical gates' failing output. All original LSP vectors passed
through the supplemental full edit-cycle replay. Only the template CLI diagnostic
gains the source context introduced in #10335; the script diagnostic is unchanged.

The separate Pinia presentation-contract difference is not calibrated here.
Compatibility comparator, ledger, ratchet, shared minimum and budgets are unchanged.
