import Std
import Lean.Data.Json

/- The spelling produced by vize_l0::js_string::push_js_string_literal is a
   subset of ECMA-262 §12.9.4 and RFC 8259 §7. Inputs are Unicode scalars.
   https://tc39.es/ecma262/#sec-literals-string-literals
   https://www.rfc-editor.org/rfc/rfc8259#section-7 -/

namespace Davinci.StringLiteral

/-- The three spellings the compiler can put inside a double-quoted literal. -/
inductive Spelling where
  | raw (c : Char)
  | escaped (tag : Char)
  | hexadecimal (high low : Char)

def chars : Spelling → List Char
  | .raw c => [c]
  | .escaped tag => ['\\', tag]
  | .hexadecimal high low => ['\\', 'u', '0', '0', high, low]

def digit (n : Nat) : Char :=
  Char.ofNat (if n < 10 then '0'.toNat + n else 'a'.toNat + n - 10)

def control (n : Nat) : Spelling :=
  match n with
  | 8 => .escaped 'b'
  | 9 => .escaped 't'
  | 10 => .escaped 'n'
  | 12 => .escaped 'f'
  | 13 => .escaped 'r'
  | _ => .hexadecimal (digit (n / 16)) (digit (n % 16))

def emit (c : Char) : Spelling :=
  if c = '"' then .escaped '"'
  else if c = '\\' then .escaped '\\'
  else if c.toNat < 32 then control c.toNat
  else .raw c

/-- Interpret the short escapes and hexadecimal digits used by the emitter. -/
def shortValue : Char → Option Char
  | '"' => some '"'
  | '\\' => some '\\'
  | 'b' => some (Char.ofNat 8)
  | 't' => some (Char.ofNat 9)
  | 'n' => some (Char.ofNat 10)
  | 'f' => some (Char.ofNat 12)
  | 'r' => some (Char.ofNat 13)
  | _ => none

def digitValue (d : Char) : Option Nat :=
  let n := d.toNat
  if '0'.toNat ≤ n ∧ n ≤ '9'.toNat then some (n - '0'.toNat)
  else if 'a'.toNat ≤ n ∧ n ≤ 'f'.toNat then some (n - 'a'.toNat + 10)
  else none

def hexValue (hi lo : Char) : Option Char := do
  let h ← digitValue hi
  let l ← digitValue lo
  return Char.ofNat (16 * h + l)

def value : Spelling → Option Char
  | .raw c => if c = '"' ∨ c = '\\' ∨ c.toNat < 32 then none else some c
  | .escaped tag => shortValue tag
  | .hexadecimal hi lo => hexValue hi lo

theorem control_value : ∀ n : Fin 32, value (control n.val) = some (Char.ofNat n.val) := by
  decide

theorem emit_value (c : Char) : value (emit c) = some c := by
  by_cases quote : c = '"'
  · subst c; rfl
  by_cases slash : c = '\\'
  · subst c; rfl
  by_cases small : c.toNat < 32
  · simpa [emit, quote, slash, small] using control_value ⟨c.toNat, small⟩
  · simp [emit, value, quote, slash, small]

inductive Token where
  | close
  | scalar (c : Char)
  deriving DecidableEq

/-- Read one token from the emitted subset of ECMAScript string characters. -/
def next : List Char → Option (Token × List Char)
  | [] => none
  | c :: rest =>
    if c = '"' then some (.close, rest)
    else if c = '\\' then
      match rest with
      | [] => none
      | tag :: tail =>
        match shortValue tag with
        | some s => some (.scalar s, tail)
        | none =>
          match tag, tail with
          | 'u', '0' :: '0' :: hi :: lo :: more =>
            (hexValue hi lo).map fun s => (.scalar s, more)
          | _, _ => none
    else if c.toNat < 32 then none
    else some (.scalar c, rest)

theorem next_spelling (s : Spelling) (c : Char) (suffix : List Char)
    (h : value s = some c) : next (chars s ++ suffix) = some (.scalar c, suffix) := by
  cases s with
  | raw x =>
    by_cases q : x = '"' ∨ x = '\\' ∨ x.toNat < 32
    · simp [value, q] at h
    · simp [value, q] at h
      subst c
      obtain ⟨q₁, q₂, q₃⟩ := not_or.mp q |>.imp_right not_or.mp
      simp [chars, next, q₁, q₂, q₃]
  | escaped tag =>
    change shortValue tag = some c at h
    simp [chars, next, h]
  | hexadecimal hi lo =>
    change hexValue hi lo = some c at h
    simp [chars, next, shortValue, h]

def body : List Char → List Char
  | [] => []
  | c :: cs => chars (emit c) ++ body cs

/-- The Rust writer copies the original UTF-8 run at an escape and at end of
    input. The model stores the pending scalars newest first. -/
def buffered (pending : List Char) : List Char → List Char
  | [] => pending.reverse
  | c :: cs =>
    match emit c with
    | .raw _ => buffered (c :: pending) cs
    | spelling => pending.reverse ++ chars spelling ++ buffered [] cs

theorem flush_original_run (input pending : List Char) :
    buffered pending input = pending.reverse ++ body input := by
  induction input generalizing pending with
  | nil => simp [buffered, body]
  | cons c cs ih =>
    cases h : emit c with
    | raw x =>
      have v := emit_value c
      rw [h] at v
      simp only [value] at v
      split at v
      · contradiction
      · have same : x = c := Option.some.inj v
        subst x
        simp [buffered, body, h, ih, chars, List.append_assoc]
    | escaped tag => simp [buffered, body, h, ih, List.append_assoc]
    | hexadecimal high low => simp [buffered, body, h, ih, List.append_assoc]

def quoted (input : List Char) := '"' :: buffered [] input ++ ['"']

/-- Fuel counts available characters. The reader consumes the closing quote
    and returns the following source without interpreting it. -/
def readBody : Nat → List Char → Option (List Char × List Char)
  | 0, _ => none
  | fuel + 1, input => do
    let (token, tail) ← next input
    match token with
    | .close => return ([], tail)
    | .scalar c =>
      let (cs, rest) ← readBody fuel tail
      return (c :: cs, rest)

def readLiteral : List Char → Option (List Char × List Char)
  | '"' :: input => readBody input.length input
  | _ => none

theorem source_length (xs : List Char) : xs.length ≤ (body xs).length := by
  induction xs with
  | nil => simp [body]
  | cons x xs ih =>
    have h : 0 < (chars (emit x)).length := by cases emit x <;> simp [chars]
    simp only [body, List.length_cons, List.length_append]
    omega

theorem read_body (xs suffix : List Char) (fuel : Nat) (h : xs.length < fuel) :
    readBody fuel (body xs ++ '"' :: suffix) = some (xs, suffix) := by
  induction xs generalizing fuel with
  | nil =>
    cases fuel with
    | zero => simp at h
    | succ n => simp [body, readBody, next]
  | cons x xs ih =>
    cases fuel with
    | zero => simp at h
    | succ n =>
      have enough : xs.length < n := by simpa using h
      simp [body, List.append_assoc, readBody, next_spelling _ _ _ (emit_value x), ih n enough]

theorem literal_round_trip (input suffix : List Char) :
    readLiteral (quoted input ++ suffix) = some (input, suffix) := by
  simp only [quoted, flush_original_run, List.reverse_nil, List.nil_append, List.cons_append,
    List.append_assoc, readLiteral]
  apply read_body
  have h := source_length input
  simp only [List.length_append, List.length_cons]
  omega

#print axioms literal_round_trip
end Davinci.StringLiteral

open Davinci.StringLiteral

/-- Check the literal bytes returned by the Rust producer against the verified
    spelling and decode them with a following token still in the input. -/
def main (args : List String) : IO Unit := do
  let [path] := args | throw (IO.userError "expected a Rust literal packet")
  IO.FS.withFile path .read fun file => do
    let mut rows := 0
    repeat
      let line ← file.getLine
      if line.isEmpty then break
      let json ← IO.ofExcept (Lean.Json.parse line)
      let (source, literal) ← IO.ofExcept (Lean.fromJson? (α := String × String) json)
      let input := source.toList
      unless literal.toList == quoted input do
        throw (IO.userError s!"different emitted spelling at row {rows + 1}")
      unless readLiteral (literal.toList ++ [';']) == some (input, [';']) do
        throw (IO.userError s!"different decoded value or suffix at row {rows + 1}")
      rows := rows + 1
    unless rows > 0 do throw (IO.userError "empty Rust literal packet")
    IO.println s!"Verified {rows} Rust literal spellings and decoded suffixes."
