module ir/Reach
open util/ordering[Rank]

sig Op {}
sig Edge { src, dst: one Op, kind: one Kind }
abstract sig Kind {}
one sig DomOrder, EffectOrder, DataDependency extends Kind {}
sig Rank {}
one sig Query { seed: one Op, selected: set Kind, reverse: lone Flag }
one sig Flag {}
one sig Answer { rows: set Op, rank: Op -> lone Rank }

fun traversal: Op -> Op {
  let rel = { a, b: Op | some e: Edge | e.kind in Query.selected and e.src = a and e.dst = b } |
    (some Query.reverse => ~rel else rel)
}
pred witnesses {
  Query.seed in Answer.rows
  Answer.rank.Rank = Answer.rows
  all x: Answer.rows - Query.seed | some p: Answer.rows |
    p -> x in traversal and lt[p.(Answer.rank), x.(Answer.rank)]
}
pred closed { Answer.rows.traversal in Answer.rows }

assert Exact { witnesses and closed implies Answer.rows = Query.seed.*traversal }
check Exact for 5 but 6 Edge, exactly 5 Rank expect 0

pred ParallelCycle {
  witnesses and closed
  some Query.reverse
  some disj a, b: Answer.rows | a -> b + b -> a in traversal
  some disj e, f: Edge | e.src = f.src and e.dst = f.dst and e.kind in Query.selected and f.kind in Query.selected
  some Op - Answer.rows
}
run ParallelCycle for 5 but 6 Edge, exactly 5 Rank expect 1
pred MissedSuccessor { witnesses and not closed and some Query.seed.*traversal - Answer.rows }
run MissedSuccessor for 4 but exactly 4 Rank expect 1
pred DisconnectedCycle {
  closed and some Answer.rows - Query.seed.*traversal
  Query.seed in Answer.rows
  all x: Answer.rows - Query.seed | some p: Answer.rows | p -> x in traversal and p != x
}
run DisconnectedCycle for 4 but exactly 4 Rank expect 1
