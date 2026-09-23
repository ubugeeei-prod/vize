import Impeto.Expression

/-!
Pins the TS-29 expression subset against `fixtures/expression-cases.json`.
Every supported case must evaluate to exactly the recorded JSON value.
`davinci-lean-expression.test.ts` evaluates the same supported cases with
Node's JavaScript engine. Every unsupported case must fail closed here.
-/

namespace Impeto.ExpressionTests
open Lean

def run (text : String) : Except String (Nat × Nat) := do
  let table <- Json.parse text
  let context <- table.getObjVal? "context"
  let mut supported := 0
  let mut unsupported := 0
  for case in (<- (<- table.getObjVal? "cases").getArr?).toList do
    let expr <- (<- case.getObjVal? "expr").getStr?
    let result := Expression.evaluate context expr
    match case.getObjVal? "value", result with
    | .ok expected, .ok actual =>
        if actual != expected then
          throw s!"`{expr}` evaluated to {actual.compress}, expected {expected.compress}"
        supported := supported + 1
    | .ok _, .error message => throw s!"`{expr}` should be supported: {message}"
    | .error _, .ok actual => throw s!"`{expr}` must fail closed, got {actual.compress}"
    | .error _, .error _ =>
        let _ <- (<- case.getObjVal? "unsupported").getStr?
        unsupported := unsupported + 1
  if supported < 50 || unsupported < 30 then throw "expression table is too small"
  pure (supported, unsupported)

def check : IO UInt32 := do
  match run (<- IO.FS.readFile "fixtures/expression-cases.json") with
  | .ok _ => pure 0
  | .error message => IO.eprintln s!"fixtures/expression-cases.json: {message}"; pure 1

end Impeto.ExpressionTests
