# Select fixed n8n baseline partitions in GitHub CI (#6830)

Only the existing `original-project-qualification` step in
`.github/actions/test-js-packages-with-history/action.yml` selects
`VIZE_N8N_BASELINE_MODE=partitioned`. Its command, success guard, required failure
behavior and evidence upload/cleanup remain unchanged. Local invocation and the
original manual JS comparison retain the serial baseline default; the explicit
manual baseline experiment still selects its two original arms.

The existing observer partitions the original ordered 1369 inputs into 685 and
684, preserving all six sequential modes per input and the original cache
controls once. Source preseed and both process groups share the original 900s
deadline and combined 32MiB output cap. Product/cache behavior, native identity
checks and the two concurrent real Oxlint hosts remain unchanged.

One complete [Actions pair](https://github.com/ubugeeei-prod/vize/actions/runs/38086887765)
executed immutable source
[33e0b2e](https://github.com/ubugeeei-prod/vize/commit/33e0b2e9b7da6d369ad0e967f0ad698f660cb200)
on the same Linux runner, debug native build, toolchain and licensed inputs.

| Measured phase       |   Serial | Partitioned | Shorter |
| -------------------- | -------: | ----------: | ------: |
| Baseline preparation | 324.054s |    176.196s |   45.6% |
| Complete controller  | 866.485s |    722.436s |   16.6% |

Independent whole-artifact review joined all 8456 retained entries, the complete
1369 inputs, 8214 ordered diagnostic rows per arm, 280 cache controls and all
original real-host, HTML and mixed-directory oracles. Timings remain retained;
temporary paths and native identities were bound before comparison. The complete
artifact SHA-256 is
`52cafb9bd695daa225aeacc6ff8349d42339fed7d366a6daf904927248382ec4`.
The [issue record](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6102831231)
states the same qualification and limits.

This is one serial-first pair, with no statistical, installed-product or whole
CI-gate gain claim. The 3a30528 test-guard correction did not execute another
timing arm. Qualification of this CI opt-in still requires fresh ordinary and
full Check on its final source, including actual partitioned CI behavior.
The unfinished Docs comparison and remaining #6830/#8090 acceptance stay open.
