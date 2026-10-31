# Required props with an undeclared component listener

The original `cases.json`, manifest entry and three authored inputs stay frozen.
The child declares only `checked: boolean` and has an input root. It declares
neither an `onChange` prop nor an emitted `change` event. The corrected
contract therefore retains the two real number-to-boolean prop errors, once
without a listener and once with a listener. An undeclared component event does
not acquire the native input's `Event` contract.

`native-event-current.json` is the complete native vector actually observed at
source `90800904bcfbf3ad073ba95349852ee71ffb37e2` in
[Full Rust qualification](https://github.com/ubugeeei-prod/vize/actions/runs/41137319424/job/124342873996).
The test framework is the locked real Vue 3.6.0-beta.10 package and the native
compiler is TypeScript 7.0.2. Both diagnostics are byte-for-byte the last two
original records, retaining file, authored position, severity, code, message
and order. The separate authority supersedes only the two original false
`Event` callback diagnostics. Actual results are never filtered or normalized.

The same original project was checked locally with the existing published
vize 0.608.0, the retained before image, the corrected c647 source image and
official vue-tsc 3.3.11 / TypeScript 6.0.3. All use the same pinned framework.
Published vize and the before image report four errors; the corrected source
and official reference agree on exactly the two original checked-prop errors
at 6:11 and 7:11. Repairing only the two checked values leaves both corrected
source and the official reference clean, while the before image still reports
the two false event errors. These are causal local observations, separate from
final combined source CI and future public release acceptance.

Nine additional same-provider controls retain complete raw process and output
packets. An optional own `onChange` prop with a boolean parameter accepts the
boolean handler. A string parameter rejects both handler sites; its repair is
clean. Declared boolean emits are clean; declared string emits reject both
handler sites until repaired. An optional boolean emitted payload still
rejects a handler that requires boolean; accepting undefined repairs it. All
eight own-listener/emits/optional control packets are byte-identical before
and after the source fix, and every 0/2 result agrees with the official
reference. The corrected no-emits repair is the only changed control.

The Rust selector and typed loader authenticate the complete historical pack,
original manifest context, all three input files and the separate complete
current vector. The effective plan and report identify the selected current
authority explicitly; the original corpus reference and actual capture remain
unchanged. Missing prop errors, restored obsolete event errors, arbitrary
message/range/code changes and unknown contexts are rejected.
