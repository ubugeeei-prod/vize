/- The first and last LSP line starts in a borrowed byte segment. -/
import Std

namespace Davinci.SourceMap.LineBreaks

/-- Every byte other than ASCII CR or LF is text, including UTF-8 continuation bytes. -/
inductive ByteClass where
  | text | cr | lf
  deriving DecidableEq

/-- Enumerate positions after line endings. The LSP iterator's initial zero is omitted. -/
def starts (offset : Nat) : List ByteClass → List Nat
  | [] => []
  | byte :: tail =>
    let rest := starts (offset + 1) tail
    match byte with
    | .text => rest
    | .lf => (offset + 1) :: rest
    | .cr => if tail.head? = some .lf then rest else (offset + 1) :: rest

/-- A forward search stops at the first CR or LF and consumes both bytes of CRLF. -/
def firstStart (offset : Nat) : List ByteClass → Option Nat
  | [] => none
  | .text :: tail => firstStart (offset + 1) tail
  | .lf :: _ => some (offset + 1)
  | .cr :: tail => some (if tail.head? = some .lf then offset + 2 else offset + 1)

/-- A reverse search returns the position after the rightmost CR or LF. -/
def lastStart (offset : Nat) : List ByteClass → Option Nat
  | [] => none
  | byte :: tail =>
    let rest := lastStart (offset + 1) tail
    match byte with
    | .text => rest
    | .cr | .lf => some (rest.getD (offset + 1))

theorem first_exact (bytes : List ByteClass) (offset : Nat) :
    firstStart offset bytes = (starts offset bytes).head? := by
  induction bytes generalizing offset with
  | nil => rfl
  | cons byte tail ih =>
    cases byte with
    | text => exact ih (offset + 1)
    | lf => rfl
    | cr =>
      cases tail with
      | nil => rfl
      | cons next rest => cases next <;> rfl

theorem last_exact (bytes : List ByteClass) (offset : Nat) :
    lastStart offset bytes = (starts offset bytes).getLast? := by
  induction bytes generalizing offset with
  | nil => rfl
  | cons byte tail ih =>
    cases byte with
    | text => exact ih (offset + 1)
    | lf => simp [lastStart, starts, ih, List.getLast?_cons]
    | cr =>
      by_cases paired : tail.head? = some .lf
      · cases tail with
        | nil => simp at paired
        | cons next rest =>
          have eq : next = .lf := Option.some.inj paired
          subst next
          change some ((lastStart (offset + 1) (.lf :: rest)).getD (offset + 1)) =
            (starts (offset + 1) (.lf :: rest)).getLast?
          rw [ih]
          simp [starts, List.getLast?_cons]
      · simp [lastStart, starts, paired, ih, List.getLast?_cons]

#print axioms first_exact
#print axioms last_exact
end Davinci.SourceMap.LineBreaks
