The declared-any successor for #11027 reuses the unchanged NoAny/Exact witness,
known-provider repair and unresolved-tag control from
`tests/_fixtures/differential/typechecker/unknown-component-slot-payload-11027`.
Only the imported provider changes between known, declared-any, intrinsic
error-any and repaired stages. The original historical pack remains unchanged.

The declared-any stage must report the authored TS2345. The erroneous provider
and missing registry tag retain their intrinsic error-any behavior. Production
CLI, official mapper, pinned vue-tsc comparison and actual TypeScript type
flags are captured by `canon-slot-provider-any.yml`.
