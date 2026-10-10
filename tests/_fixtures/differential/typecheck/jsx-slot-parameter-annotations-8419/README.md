# Authored JSX slot parameter contract (#8419)

The nine complete source CLI packets exercise an imported required `number` slot.
Arrow, function and destructured callbacks annotated `string` must report
TS2345. Correct and untyped callbacks are clean; an independent number-body
error retains TS2339. Source text includes Japanese, emoji and CRLF.

`cases.json` contains literal diagnostic expectations. The assignability
messages were independently checked against the original Corsa 7.0.2 typed
callback contract, without using repaired output as the oracle. The source
CLI test compares every report field, including all file/program entries,
compiler options, diagnostic text, coordinates, counts, stderr and exit code.

The public 0.439.0 before-repair archive retains all six complete inputs,
process records, stdout/stderr and native loader journals, plus the original
installation identities. Its 49 regular files have an independent digest
manifest. Observations SHA256:
`65c4946126d029f6ae7a1202d9d5079380775bed79cbca7d7086dfca9b259610`.
Archive SHA256:
`f1bf5ff9270a1dee35fa8966f0bf06207b16d973f0654635576ec9f1467f706b`.
The source cut is C `26031a4fbb30a1e86511919bafc7035b08440ef3`,
H `a26243855bcf81005049252a6e71b6dd55e457f1`, release run `38011924629`.
Receipt SHA256:
`2bd2afdaf90ef9c274955fcc0335adbaa1a8bb3585821dae92850228e0e691a5`.
All six sessions observe the original public native loader's successful return
and automatically selected public Corsa; status-1 Rust termination bypasses
Node's exit event, so that case proves termination from its actual process
result instead. No replacement backend or source package override is used.

This is a bounded correctness fixture. Source Actions, protected merge queue,
actual merge and a subsequent public release remain required. Normal compiler
output and metadata are independently pinned in pre-fix full VDOM/Vapor
snapshots. Broader JSX contracts remain under #1497.
