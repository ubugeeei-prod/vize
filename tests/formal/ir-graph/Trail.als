module ir/Trail

sig Op, Kind {}
sig Edge { src, dst: one Op, kind: one Kind }
abstract sig Direction {}
one sig Forward, Reverse extends Direction {}
one sig Query {
  seed: one Op, firstDir, secondDir: one Direction,
  firstKinds, secondKinds: set Kind
}
one sig Index { outgoing, incoming: Op -> Edge }
fact Adjacency {
  Index.outgoing = ~src
  Index.incoming = ~dst
}

fun adjacent[o: Op, d: Direction, selected: set Kind]: set Edge {
  (d = Forward => o.(Index.outgoing) else o.(Index.incoming)) & kind.selected
}
fun target[e: Edge, d: Direction]: one Op { d = Forward => e.dst else e.src }
fun indexed: Edge -> Edge {
  { a, b: Edge |
      a in adjacent[Query.seed, Query.firstDir, Query.firstKinds] and
      b in adjacent[target[a, Query.firstDir], Query.secondDir, Query.secondKinds] - a }
}

// Read both underlying relationships in their authored orientations.
pred connects[e: Edge, d: Direction, x, y: Op] {
  (d = Forward and x = e.src and y = e.dst) or
  (d = Reverse and x = e.dst and y = e.src)
}
fun rawWalks: Edge -> Edge {
  { a, b: Edge | a.kind in Query.firstKinds and b.kind in Query.secondKinds and
    some middle, end: Op | connects[a, Query.firstDir, Query.seed, middle] and
      connects[b, Query.secondDir, middle, end] }
}

assert ExactTrails { indexed = rawWalks - iden }
check ExactTrails for 5 but 6 Edge expect 0
pred ParallelReturn {
  Query.firstDir = Forward and Query.secondDir = Reverse
  some disj a, b: Edge | a.src != a.dst and a.src = b.src and a.dst = b.dst
    and a->b + b->a in indexed
}
run ParallelReturn for 3 but exactly 2 Op, exactly 2 Edge expect 1
pred SingleEdgeReturnsOnlyOnReuse {
  some rawWalks and no indexed
  Query.firstDir != Query.secondDir
  all e: Edge | e.src != e.dst
}
run SingleEdgeReturnsOnlyOnReuse for 3 but exactly 1 Edge expect 1
pred DistinctSelfLoops {
  some disj a, b: Edge | a.src = a.dst and b.src = b.dst and a->b + b->a in indexed
}
run DistinctSelfLoops for 3 but exactly 1 Op, exactly 2 Edge expect 1
