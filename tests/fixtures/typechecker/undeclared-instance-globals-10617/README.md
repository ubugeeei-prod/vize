# Undeclared instance globals: actual consumer reproduction

The original three reported files retain the issue's compiler options, Vue
augmentation and SFC source. The public control uses immutable `vize@0.566.0`
and the actual official `typescript@7.1.0-beta` mapper host. The report omits a Vue
version, so this bounded corpus explicitly pins real `vue@3.5.35`; it does not
claim to reproduce the consumer's entire dependency graph.

The original published mapper reports only `missing` at 6:43. The isolated
undeclared-only program is false-clean. The minimal plain packaged checker also
masks that name; the report's correct application-level CLI result depends on
additional application context and must not be generalized to this fixture.

Independent handwritten function-local public Vue context programs are checked
by the actual beta and TypeScript 5.9.3. They declare no invented library exports.
The two compilers agree on the complete declared/undeclared messages, including
TS2551 and the `$declared` suggestion. The local scope preserves the library's
actual diagnostic type display. Raw SFC-reference output is retained separately:
its virtual SFC graph falsely loses `$declared` in this installed graph and is
not used as an acceptance oracle.

The forty authored cases cover original/isolated/valid/repaired programs,
repeated, optional, call and loop-source accesses, intentional any, declared
unknown, typed plugin values, setup/loop/slot shadows, CSS modules, own props,
Options API discovery, authored suppression comments, object shorthand, typeof value
reads, object/member/computed keys, raw strings/comments/template literals, callback
parameters/destructuring, and mixed loop/slot scopes. Opening-node comments
retain their existing child-negative/unused-expect behavior; separate owned
interpolation and attribute cases include an unrelated invalid sibling. Unicode
is added inside the existing interpolation so directive ownership is preserved. LF plus CRLF with
Unicode/emoji exercise diagnostic and UTF8 protocol coordinates. Options API
results are explicitly discovery-only; a clean-before/error-after result is
recorded as a regression rather than receiving acceptance credit.

The source package is a private protocol adapter to the receipted CLI and the
actual source-owned helper declarations. It is not fresh public-package
acceptance. Keep complete candidate/public generated text, mappings, process
packets, profiles and size observations before assertions. The first campaign is
explicit discovery with no full-packet qualification credit: reviewed complete
transform goldens and mutation controls must be added after the actual capture.
No performance claim follows from the individual timings.

The provisional `source-packets.json` records only 20 unchanged LF transforms from
the first source capture. Its `complete: false` flag prevents final qualification.
These objects are byte-exact evidence of the earlier producer, including the
complete directive payload; all final inputs must be recaptured after the inline
instance-read fix. No workspace-root or generated-code normalization is used.
