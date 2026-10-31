# Vue 2 edit-cycle diagnostic context custody

The legacy dialect editor suite consumes `successor.json` as complete CLI
expectations. The data-key-only suite consumes the explicit `display-successor.json`
that preserves that packet and changes only its script diagnostic type display.
They retain their original exact LSP publications, authored ranges,
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

The later generated-type display change in #11045 is recorded separately in
`display-custody.json`: only `dataKey.files[0].diagnostics[1]` changes from
`type '__VizeThis'` to `type '$This'`. All earlier packets, custody hashes, source
archives and licenses remain byte-identical. The complete current CLI packet was
observed at source `09cc1727c687fa2baa30ecd1dcab854ec72ca597` in
[edit-cycle Actions41112686389](https://github.com/ubugeeei-prod/vize/actions/runs/41112686389/job/124258822990).
The build receipt and complete LSP session wires are retained in artifact13003633335;
the whole CLI comparison and exact display difference are in the job log whose
SHA256 is pinned in the display receipt. This is a reviewed finite display
successor, with no runtime normalization of diagnostic messages or ranges.
