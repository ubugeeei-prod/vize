# Slot conditions that render their slot

[2nofa11](https://github.com/2nofa11) reported
[TS2774 on `v-if="slots.tip"` around `<slot name="tip" />`](https://github.com/ubugeeei-prod/vize/issues/13879).

A branch that tests a declared slot and renders it through a `<slot>` outlet
calls that slot, so the condition is not an uncalled function check. The
fixture covers a direct and a nested outlet, a `v-else-if` branch and a
`v-if` on the outlet itself, and keeps vue-tsc's TS2774 for a condition whose
branch renders no outlet.
