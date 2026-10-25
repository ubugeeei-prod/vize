# Source-built experimental route props witness

This fixture uses the unmodified `vuejs/router` source at
`593d7ad9ff99e95c24be28e962717df0fb2be763`, tree
`885476969881154b639ea4e2a0c736ec6fa1796a`. Its manifest says `5.4.0`, but
this is **not the published npm archive**. `source.json` pins the source and
original frozen build recipe. `oracles.json` binds the actual archive, complete
member identities, consumer lock, generated declaration, semantic exports and
independent process packets observed in Actions run `40175166770`.

The workflow builds this provider in Actions, using its own package manager and
unchanged build configuration. The consumer uses `npm ci` and the original
source CLI from the same run. No provider declaration or export is synthesized.

The frozen native oracle comes from released `vize@0.562.0`, not the candidate
source CLI. The reference oracle uses the pinned independent TypeScript host.
All process output is retained before acceptance. Only the known
`programs[0].compilerOptions.rootDir` transport field changes with the temporary
workspace; diagnostic text and all remaining output bytes stay exact.

| Capability                                                              | Witness                                                                                                                |
| ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Actual `_RoutePropsForFilePath` export                                  | Real semantic alias, declaration identity and complete export vector                                                   |
| Experimental `useRoute` export                                          | Absent from this exact source-built provider                                                                           |
| Manual route props helper and explicit `definePage` component generic   | Complete `check` packets: valid, wrong numeric type, wrong custom Date type, missing required component prop, repaired |
| Query default, query array, custom Date parser and hash parameter types | Real scanner declaration and manually explicit check inputs                                                            |
| LF/CRLF, Unicode source, plugin enabled/disabled                        | 20 fixed inputs; 60 reference/released/source check processes                                                          |
| Automatic helper or component-generic projection                        | Not qualified by this fixture                                                                                          |
| Content mapper, LSP and editor protocols                                | Not qualified by this fixture                                                                                          |
| Invalid template expression mutation                                    | Not qualified by this fixture                                                                                          |

The disabled-plugin cases deliberately keep the manually authored helper and
component generic. Their success demonstrates ordinary explicit type behavior;
it does not demonstrate automatic projection or a disabled-project admission
boundary. The executable scanner's supplementary route/resolver sources are
retained outside the authored program. Only its complete generated declaration
is executed by these check controls.

The initial discovery run receives no qualification credit. Final acceptance
requires a fresh source run to reproduce every frozen independent packet. The
mutation tests reject position changes in either direction, changed diagnostic
codes, dropped program members, foreign roots, extra stderr and changed bytes.
