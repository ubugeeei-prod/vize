# Independent typechecking stress axes

The original public corpus varies file count, dependency topology, individual
file size and type complexity independently. It supplements the private primary
benchmark; it contains no copied private source and does not qualify that target.

| Axis           | Default cases                                                                           |
| -------------- | --------------------------------------------------------------------------------------- |
| SFC count      | 2,500, 10,000 or 50,000 actual Vue files                                                |
| Dependencies   | 1,000 nodes; chains, diamonds, strongly connected cycles, package aliases and reexports |
| Individual SFC | 128 KiB, 1 MiB and 10 MiB of declarations and template expressions                      |
| Types          | Conditional/mapped/key-remapped unions; width 4/16/64 and recursive depth 4/16/32       |
| Mixed          | 32 SFCs of 128 KiB, 256 dependency nodes, width 16 and depth 16                         |

Each SFC has exact type, `any`/`unknown`/`never` witnesses and typed DOM refs.
Invalid stages inject unique template-member errors at the beginning, middle
and end of every SFC and an assignment error in every graph node. Poisoned
stages invalidate the actual type proofs. All expected errors retain physical
owners, UTF-16 coordinates and duplicate multiplicity. Alternating CRLF/LF files
include Japanese text and emoji. Size comes from executable expressions and
declarations; even shortened poisoned stages retain the requested byte minimum.
The graph's recursive type witness walks four levels from every node. This is
separate from the explicit recursive-type-depth axis.

The `Type admission scale axes` Action runs a small published-provider smoke on
changes and accepts manual axis selection. Its baseline pins Vize/native,
native TypeScript, Vue, vue-tsc and JavaScript TypeScript versions. Each point
uses a fresh project and tool process; OS page cache is not flushed. Raw output,
status, commands, corpus hashes, image identity, versions and process-resource
receipts are retained, including failed points and legitimate compiler depth
errors. A failed point never becomes an omitted file or an empty successful run.
Wall measurements include the tool subprocess; corpus generation is outside
that interval. Resource maximum RSS is the operating system's process maximum,
not a simultaneous sum across the process tree. Allocation counts are not
measured by this runner. A single point is not a median or a speedup claim.
Receipt schema 2 hashes the selected executable and its entrypoint separately.
For vue-tsc it also records the actually resolved JavaScript TypeScript compiler;
native checker/image fields are explicitly comparison-provider observations.

```sh
node tools/benchmarks/scripts/type-admission-scale-suite.ts \
  --provider /absolute/pinned-provider --out /absolute/new-output \
  --axis smoke --tool all
```

The lower-level runner also supports a same-source CLI through `--binary` and
`--source`. It requires the adjacent, byte-matched differential build receipt.
A source mapper label additionally requires an adjacent native `.source.json`
receipt, matching source revision, image SHA-256 and the immutable build token
read from the actually loaded native image. Installed package metadata alone
does not authorize a source label. The Action is explicitly a published baseline;
it does not claim source CLI/native qualification.
