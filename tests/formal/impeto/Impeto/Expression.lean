import Lean.Data.Json

/-!
Independent evaluator for the TS-29 expression subset. Template expressions
reach S3 as raw `js` operand text. This module parses a closed JavaScript
subset and evaluates it over JSON state with JavaScript semantics. The subset
has literals, lexical names, member reads, `.length`, `!`, unary `-`, `*`,
`%`, `+`, `-`, relational operators, strict equality, `&&`, `||` and the
conditional operator. Anything else fails closed. That includes calls,
indexing, loose equality, division, floats, `undefined`, arrays or objects in
string or strict-equality positions, and integers beyond 2^53 - 1. Only the
selected operand of `&&`, `||` and `?:` is evaluated, as in JavaScript.
-/

namespace Impeto.Expression
open Lean

inductive Token where
  | num (value : Nat)
  | str (value : String)
  | ident (name : String)
  | punct (text : String)
deriving BEq, Repr, Inhabited

inductive Expr where
  | lit (value : Json)
  | var (name : String)
  | member (target : Expr) (field : String)
  | not (operand : Expr)
  | neg (operand : Expr)
  | binary (op : String) (left right : Expr)
  | and (left right : Expr)
  | or (left right : Expr)
  | cond (test yes no : Expr)
deriving Inhabited

def identStart (c : Char) : Bool := (c.isAlpha && c.toNat < 128) || c == '_' || c == '$'
def identPart (c : Char) : Bool := identStart c || c.isDigit

def reserved : List String := [
  "undefined", "this", "new", "typeof", "instanceof", "in", "of", "void", "delete",
  "function", "class", "return", "var", "let", "const", "await", "yield", "async",
  "__proto__", "constructor", "prototype", "NaN", "Infinity"
]

def puncts : List String := [
  "===", "!==", "==", "!=", "<=", ">=", "&&", "||", "??", "?.",
  "!", "<", ">", "+", "-", "*", "%", "/", "?", ":", "(", ")", ".", "[", "]", ","
]

partial def lexString (quote : Char) : List Char -> String -> Except String (String × List Char)
  | [], _ => throw "unterminated string literal"
  | c :: rest, acc =>
      if c == quote then pure (acc, rest)
      else if c == '\\' then
        match rest with
        | e :: rest =>
            let escaped := match e with
              | 'n' => some '\n' | 't' => some '\t' | '\\' => some '\\'
              | '\'' => some '\'' | '"' => some '"' | _ => none
            match escaped with
            | some ch => lexString quote rest (acc.push ch)
            | none => throw "unsupported string escape"
        | [] => throw "unterminated string escape"
      else if c == '\n' then throw "unsupported line break in string"
      else lexString quote rest (acc.push c)

partial def tokenize : List Char -> Except String (List Token)
  | [] => pure []
  | c :: rest =>
      if c == ' ' || c == '\t' || c == '\n' || c == '\r' then tokenize rest
      else if c.isDigit then
        let digits := (c :: rest).takeWhile Char.isDigit
        let after := (c :: rest).dropWhile Char.isDigit
        if digits.length > 1 && c == '0' then throw "unsupported legacy numeric literal"
        else if after.head?.any (fun n => identPart n || n == '.') then
          throw "unsupported numeric literal"
        else do
          let tail <- tokenize after
          pure (.num (String.ofList digits).toNat! :: tail)
      else if c == '"' || c == '\'' then do
        let (value, after) <- lexString c rest ""
        pure (.str value :: (<- tokenize after))
      else if identStart c then
        let name := String.ofList ((c :: rest).takeWhile identPart)
        let after := (c :: rest).dropWhile identPart
        do pure (.ident name :: (<- tokenize after))
      else
        let text := String.ofList (c :: rest)
        match puncts.find? (fun p => text.startsWith p) with
        | some p => do pure (.punct p :: (<- tokenize ((c :: rest).drop p.length)))
        | none => throw s!"unsupported character `{c}`"

abbrev Parser := StateT (List Token) (Except String)

def peek : Parser (Option Token) := do pure (← get).head?

def advance : Parser Unit := modify List.tail

def expect (p : String) : Parser Unit := do
  if (← peek) == some (.punct p) then advance else throw s!"expected `{p}`"

def binaryLevel (ops : List String) (next : Parser Expr) : Parser Expr := do
  let mut left <- next
  repeat
    match (← peek) with
    | some (.punct p) =>
        if ops.contains p then
          advance
          left := .binary p left (← next)
        else break
    | _ => break
  pure left

mutual
partial def primary : Parser Expr := do
  match (← peek) with
  | some (.num n) => advance; pure (.lit (toJson n))
  | some (.str s) => advance; pure (.lit (.str s))
  | some (.ident "true") => advance; pure (.lit (.bool true))
  | some (.ident "false") => advance; pure (.lit (.bool false))
  | some (.ident "null") => advance; pure (.lit .null)
  | some (.ident name) =>
      if reserved.contains name then throw s!"unsupported name `{name}`"
      advance; pure (.var name)
  | some (.punct "(") => advance; let e <- conditional; expect ")"; pure e
  | _ => throw "unsupported expression form"

partial def memberChain : Parser Expr := do
  let mut target <- primary
  repeat
    if (← peek) == some (.punct ".") then
      advance
      match (← peek) with
      | some (.ident field) =>
          if reserved.contains field then throw s!"unsupported member `{field}`"
          advance
          target := .member target field
      | _ => throw "expected member name"
    else break
  pure target

partial def unary : Parser Expr := do
  match (← peek) with
  | some (.punct "!") => advance; pure (.not (← unary))
  | some (.punct "-") => advance; pure (.neg (← unary))
  | _ => memberChain

partial def logical (op : String) (next : Parser Expr) : Parser Expr := do
  let mut left <- next
  while (← peek) == some (.punct op) do
    advance
    let right <- next
    left := if op == "&&" then .and left right else .or left right
  pure left

partial def conditional : Parser Expr := do
  let multiplicative := binaryLevel ["*", "%"] unary
  let additive := binaryLevel ["+", "-"] multiplicative
  let relational := binaryLevel ["<", "<=", ">", ">="] additive
  let equality := binaryLevel ["===", "!=="] relational
  let test <- logical "||" (logical "&&" equality)
  if (← peek) == some (.punct "?") then
    advance
    let yes <- conditional
    expect ":"
    pure (.cond test yes (← conditional))
  else pure test
end

def parse (text : String) : Except String Expr := do
  let tokens <- tokenize text.toList
  let (expr, rest) <- conditional.run tokens
  if !rest.isEmpty then throw s!"unsupported expression tail in `{text}`"
  pure expr

def maxSafe : Nat := 9007199254740991

def integer (value : Json) : Except String Int := do
  let n <- value.getInt?
  if n.natAbs > maxSafe then throw "integer outside the safe range"
  pure n

def number (n : Int) : Except String Json :=
  if n.natAbs > maxSafe then throw "integer result outside the safe range" else pure (toJson n)

def truthy : Json -> Bool
  | .null => false
  | .bool b => b
  | .num n => n.mantissa != 0
  | .str s => !s.isEmpty
  | _ => true

def primitiveText : Json -> Except String String
  | .null => pure "null"
  | .bool b => pure (if b then "true" else "false")
  | .str s => pure s
  | value@(.num _) => do pure (toString (<- integer value))
  | _ => throw "unsupported string conversion of a structured value"

def strictEqual : Json -> Json -> Except String Bool
  | .null, .null => pure true
  | .bool a, .bool b => pure (a == b)
  | .str a, .str b => pure (a == b)
  | a@(.num _), b@(.num _) => do pure ((<- integer a) == (<- integer b))
  | .arr _, _ | _, .arr _ | .obj _, _ | _, .obj _ => throw "unsupported identity comparison"
  | _, _ => pure false

def member (target : Json) (field : String) : Except String Json :=
  match target, field with
  | .arr items, "length" => pure (toJson items.size)
  | .str s, "length" =>
      if s.toList.all (·.toNat < 0x10000) then pure (toJson s.length)
      else throw "unsupported length of astral text"
  | .obj _, _ => target.getObjVal? field
  | _, _ => throw s!"unsupported member `{field}`"

def binary (op : String) (left right : Json) : Except String Json := do
  if op == "+" && (left matches .str _ || right matches .str _) then
    return .str ((<- primitiveText left) ++ (<- primitiveText right))
  if op == "===" then return .bool (<- strictEqual left right)
  if op == "!==" then return .bool !(<- strictEqual left right)
  let a <- integer left
  let b <- integer right
  match op with
  | "+" => number (a + b)
  | "-" => number (a - b)
  | "*" => number (a * b)
  | "%" => if b == 0 then throw "remainder by zero" else number (a.tmod b)
  | "<" => pure (.bool (a < b))
  | "<=" => pure (.bool (a ≤ b))
  | ">" => pure (.bool (a > b))
  | ">=" => pure (.bool (a ≥ b))
  | _ => throw s!"unsupported operator `{op}`"

def eval (context : Json) : Expr -> Except String Json
  | .lit value => pure value
  | .var name => context.getObjVal? name
  | .member target field => do member (<- eval context target) field
  | .not operand => do pure (.bool !(truthy (<- eval context operand)))
  | .neg operand => do number (-(<- integer (<- eval context operand)))
  | .binary op left right => do binary op (<- eval context left) (<- eval context right)
  | .and left right => do
      let value <- eval context left
      if truthy value then eval context right else pure value
  | .or left right => do
      let value <- eval context left
      if truthy value then pure value else eval context right
  | .cond test yes no => do
      if truthy (<- eval context test) then eval context yes else eval context no

def evaluate (context : Json) (text : String) : Except String Json := do
  eval context (<- parse text)

end Impeto.Expression
