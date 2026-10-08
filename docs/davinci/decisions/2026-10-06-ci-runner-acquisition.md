# Lightweight CI runner acquisition recovery

The [paired decision on #6830](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6002083995)
records the same diagnosis and source-only qualification.

On 2026-10-05, GitHub marked lightweight `ubuntu-24.04` jobs cancelled
before a runner executed them. Their annotations say: “The job was not
acquired by Runner of type hosted even after multiple attempts.” The
job metadata records `runner_id: 0`; this is an acquisition failure,
not a validator assertion failure:

- [Rust source report 111947602426](https://github.com/ubugeeei-prod/vize/actions/runs/37362947409/job/111947602426)
  on protected candidate `e7af2e331e3e3bc39885c84430284ed8d4f02238`.
- [Rust formatting 111947166222](https://github.com/ubugeeei-prod/vize/actions/runs/37364756190/job/111947166222)
  and [dependency direction 111947166509](https://github.com/ubugeeei-prod/vize/actions/runs/37364756190/job/111947166509)
  on the full check of `440410602417eb8a69cc0c0e4328eea1c39ca5ef`.
- [Zizmor planning 111946936166](https://github.com/ubugeeei-prod/vize/actions/runs/37364686827/job/111946936166)
  on that source head. Its unchanged Blacksmith scanner succeeded.

The same Check runs successfully acquired
`blacksmith-32vcpu-ubuntu-2404` runners for JS checking, security and builds.
Use that existing Ubuntu 24.04 label for eight lightweight jobs: Rust
formatting, PR source planning, Rust source reporting, source reporting,
final test reporting, PR report comments, dependency direction and Zizmor
planning. The jobs retain their complete steps, permissions, conditions,
dependency lists, timeouts, outputs, artifact rules and failure handling.

This repair changes scheduling authority for those jobs. Benchmark,
instruction-count, build, product, release and scheduled product recipes
retain their existing bytes. GitHub's generated optional CodeQL configuration
is outside this change; its separate unknown or cancelled scan evidence is
not converted into successful validation.

Existing workflow semantics tests and a complete YAML comparison must verify
that only the eight runner fields change. After source review, qualify a
fresh exact-head source run and protected candidate; the retained cancelled
jobs do not count as passing gates. Their acquisition message does not
establish the underlying hosted-service cause or guarantee future capacity.
