# Source-built TSX boundary failure on #7982

Exact unchanged authored language control from PR #8094 source2d7901367b.
Check37416588029 Rust1/job112117854230 reports parser/sfc Error:
`Malformed <script> block: the closing tag is missing.`
Its required browser-global warning count remains1. The paired JSX case uses
the same authored body and is retained separately; old loop stopped at TSX.
No execution or acceptance is inferred for the unexecuted JSX arm.
