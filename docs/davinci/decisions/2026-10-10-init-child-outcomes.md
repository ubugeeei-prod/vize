# Fresh initialization child outcomes (#3956)

Issue: [#3956](https://github.com/ubugeeei-prod/vize/issues/3956).

The seven original fresh-project cells retain their complete authored files,
init plans, package-manager commands and diagnostic vectors. This change
corrects only the process boundary used to qualify those diagnostics.

Previously, the machine check parsed stdout but returned only the exit status.
The broken-project assertion accepted any status other than zero, including
`null`. A real child could write the complete expected JSON and then terminate
with SIGTERM; its diagnostics and error count still satisfied the old law.
An unrelated normal exit seven could satisfy it too.

The machine check now retains the actual spawn result, including PID, status,
signal and complete untrimmed decoded stdout/stderr/output. Before parsing
JSON, it requires no signal and a normal status zero or one. An invalid outcome
is retained on the error and as its cause. A JSON parse failure retains that
same outcome beside its original parse-error cause. Invocation rendering stays
unchanged; presentation trimming does not alter the retained streams.

The clean and repaired phases still require exit zero. The broken phase now
requires exactly exit one and no signal, followed by the original complete
diagnostic projection and error-count assertions. Shared process execution,
Corsa discovery, package materialization and existing project inputs are
unchanged.

Four process-boundary laws use actual Node children: normal exit one with the
whole authored TS2322 JSON; the identical JSON followed by actual SIGTERM;
the identical JSON with normal exit seven; and malformed JSON with normal
exit one. Each child writes its actual PID and argv for independent comparison
with the returned spawn result. Boundary CRLF, spaces and tabs in both streams
are asserted without trimming. These are explicit synthetic process controls,
not installed Vize or package-manager acceptance.

The narrow local controls pass after failing before the guard. Exact-source
Actions, the fresh packed matrix and protected delivery remain required.
No historical v0.441 crash is asserted. Additional JavaScript/checkJs manager
cells, fresh-project LSP/editor hosts and the complete #3956 acceptance remain
unfinished. A future release must qualify its own included source and artifacts.
