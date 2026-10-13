# Reap failed VS Code acquisitions before bounded retries

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Release source v0.448.0 H=`2817a0aa13518dd46c16bc8bf56ac831f68c24dc`
failed the editor host gate in full Check
[38331701024](https://github.com/ubugeeei-prod/vize/actions/runs/38331701024/job/115082649383).
All three upstream VS Code 1.107.1 archive attempts reported `aborted`.
The helper printed the complete error at 01:23:06Z but remained alive until
Actions' existing 300-second acquisition timeout at 01:28:01Z. This retained
failure proves acquisition did not complete; it does not prove the upstream
server or network cause.

Run each unchanged upstream acquisition in an owned child. Keep the exact
1.107.1 default, 30-second idle timeout, upstream three retries, SHA256 validation
and extraction. Write the worker's complete error synchronously and exit on
failure so upstream sockets cannot prevent the next acquisition. Preserve both
raw child streams directly in Actions. Allow at most two children, each bounded
to 140 seconds and together to 285 seconds, with one second to escalate a hung
child from TERM to KILL. On POSIX, reap the entire owned process group on every
outcome and when the parent is interrupted. The existing outer 300-second
Actions bound and every subsequent real editor lifecycle check remain unchanged.
Only an actual successful upstream acquisition returns zero; timeout, missing
command, exhausted failures and interruption stay failures. No failed original
attempt is rewritten or discarded.

The five real subprocess controls cover failure followed by success, terminal
nonzero failure, hung parent and descendant teardown, byte-exact non-UTF8 stdout
and original stderr, and a total deadline that bounds the second hung attempt.
These controls do not establish a successful VS Code download or editor host
acceptance. Fresh exact-head Actions and the full editor host gate are required.

The separately retired source #8793 / v0.448.0 cannot resume. Its actual Release
run38331588881, sourceH above and original Operator38330888861 were retired by
Operator38335263162; latest public v0.443.0 remains the publication baseline.
Matrix38331707741 failed actual lint divergence in all22 shards, LSP in ten
and typecheck in five. Those product/oracle failures, source coverage and native
memory gates remain separate release blockers. This acquisition repair alone
provides no release, installed-consumer or issue-completion credit.

The first hosted check-js run rejected five unawaited test registrations under
the existing zero-warning policy. Await each original registration without
changing its child, assertions or deadlines; fresh successor Actions remain required.
