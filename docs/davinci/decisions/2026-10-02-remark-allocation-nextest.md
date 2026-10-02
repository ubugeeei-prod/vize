# Remark allocation discovery (2026-10-02)

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The existing L0 `remark_zero_cost` target keeps `harness = false` so libtest's
reporting thread stays outside its process-wide allocation windows. Its old
unconditional `main` did not advertise a case for nextest discovery.

The executable now advertises the actual
`remarks_preserve_zero_cost_and_the_positive_control` case for `--list --format
terse`. Listing, ignored selection and unmatched filters return without entering
the measurement callback. Default execution, an exact match and a substring
match each enter the original measurement callback once. Unsupported arguments
fail, and a selected callback assertion still fails the process.

Only the dispatcher and the callback's function name change. The complete
original measurement body, fixture passes, counting allocator and assertions
remain byte-identical: NoObserver allocates zero, attached timing plus budget
allocates zero, and the consuming positive control allocates exactly three
labels and records three analysis remarks. Argument parsing occurs before that
unchanged measured window. No ceiling, filter, CI command or manifest changes.

The focused tooling law compiles the actual `CASE` and `main` bytes with an
instrumented callback and checks 16 successful/failing invocations. This proves
the command protocol and callback selection. It does not claim actual nextest
discovery or execution of the original allocation body by a hosted worker.

Authored coverage credit requires the fresh exact-head nextest list to contain
the named case and the actual selected worker/JUnit to execute it successfully.
Full suites, strict instruction ceilings and protected queue checks remain
required; discovery-only output receives no measured-body credit.
