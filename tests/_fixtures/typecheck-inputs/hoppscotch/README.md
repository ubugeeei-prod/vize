This SDL is the complete output of Hoppscotch's `generate-gql-sdl` command at the revision recorded in `provenance.json`. Two independent schema-builder processes produced identical bytes. The original GraphQL codegen accepted every common-package document against it. The upstream MIT license is retained.

The producer uses the pinned backend's Prisma client generator and Nest schema builder. It does not start a database. Those producer steps run only when refreshing this sealed snapshot. Fixture preparation copies and verifies the pinned SDL, then runs the common package's existing GraphQL codegen; it does not run Prisma or start the backend in CI.

This input repairs one missing prerequisite in #13298. It grants no semantic, native-parity or historical execution credit. The existing unavailable-input and diagnostic comparisons remain required.
