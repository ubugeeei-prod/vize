module primitive/PreferConst
open util/ordering[Step] as ord

abstract sig Mode {}
one sig Mutable, Immutable extends Mode {}
sig Name {}
sig Value {}
abstract sig Action { target: one Name, value: one Value }
sig Init extends Action { mode: one Mode }
sig Assignment extends Action {}
sig Step { action: one Action }
fact ExactlyOnce { all a: Action | one action.a }

-- An initializer (including a repeated var initializer) or an assignment can
-- overwrite an existing binding. Replacing an initializer with const makes any
-- later write to that name fail at the corresponding instruction.
pred legal[immutable: set Init, point: Step] {
  all s: ord/prevs[point] + point | s.action in immutable implies
    no t: (ord/nexts[s] & (ord/prevs[point] + point)) | t.action.target = s.action.target
}
fun environment[point: Step]: Name -> Value {
  { n: Name, v: Value |
    some s: ord/prevs[point] + point | s.action.target = n and s.action.value = v and
      no t: ord/nexts[s] & (ord/prevs[point] + point) | t.action.target = n
  }
}
fun result[immutable: set Init, point: Step]: Name -> Value {
  legal[immutable, point] => environment[point] else none -> none
}
pred closed { no Assignment and all n: Name | lone target.n }
fun original: set Init { mode.Immutable }
assert SameTrace {
  closed implies all candidate: Init - original, point: Step |
    legal[original, point] and legal[original + candidate, point] and
    result[original,point] = result[original + candidate,point]
}
check SameTrace for 5 but exactly 5 Step expect 0
pred NonemptyTrace {
  closed and some disj a,b: Init - original |
    a.value != b.value and a.target -> a.value + b.target -> b.value in result[original + a, ord/last]
}
run NonemptyTrace for 4 but exactly 3 Step expect 1
pred LaterWrite {
  some c: Init - original, a: Assignment |
    a.target = c.target and legal[original, ord/last] and not legal[original + c, ord/last] and
    result[original, ord/last] != result[original + c, ord/last]
}
run LaterWrite for 3 but exactly 2 Step expect 1
pred RepeatedName {
  no Assignment and some disj a,b: Init - original |
    a.target = b.target and legal[original,ord/last] and not legal[original + a,ord/last] and
    result[original, ord/last] != result[original + a,ord/last]
}
run RepeatedName for 3 but exactly 2 Step expect 1
