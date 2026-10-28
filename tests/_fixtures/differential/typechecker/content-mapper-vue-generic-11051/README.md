# `@vue-generic` TSX parsing regression

Pinned from Hebilicious/vize-repro-vue-generic at
`64138773886450b6461361522882098461ef5824`. The first case preserves the
reporter's clean project. The second changes only the authored prop value and
must retain its explicit generic diagnostic, proving that successful TSX
parsing does not bypass type checking.
