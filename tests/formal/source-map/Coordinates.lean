/- UTF-8 byte widths and UTF-16 code-unit widths for Unicode scalar values. -/
import Std

namespace Davinci.SourceMap

inductive ScalarWidth where
  | ascii | two | three | astral

def bytes : ScalarWidth → Nat
  | .ascii => 1 | .two => 2 | .three => 3 | .astral => 4

def units : ScalarWidth → Nat
  | .ascii => 1 | .two => 1 | .three => 1 | .astral => 2

def total (width : ScalarWidth → Nat) : List ScalarWidth → Nat
  | [] => 0
  | c :: cs => width c + total width cs

def excess := total (fun c => bytes c - units c)

theorem accounted (xs : List ScalarWidth) :
    total units xs + excess xs = total bytes xs := by
  induction xs with
  | nil => rfl
  | cons c cs ih =>
    cases c <;> simp [total, excess, bytes, units] at ih ⊢ <;> omega

theorem appended (xs ys : List ScalarWidth) :
    total units (xs ++ ys) = total units xs + total units ys := by
  induction xs with
  | nil => simp [total]
  | cons c cs ih => simp [total, ih, Nat.add_assoc]

theorem checkpoint_exact (before tail : List ScalarWidth) :
    total bytes before - excess before + total units tail = total units (before ++ tail) := by
  have h := accounted before
  rw [appended]
  omega

-- The line may start in a different checkpoint from its final column.
theorem column_exact (lineStart column startCheckpoint startTail endCheckpoint endTail : List ScalarWidth)
    (hs : startCheckpoint ++ startTail = lineStart)
    (he : endCheckpoint ++ endTail = lineStart ++ column) :
    (total bytes endCheckpoint - excess endCheckpoint + total units endTail) -
    (total bytes startCheckpoint - excess startCheckpoint + total units startTail) = total units column := by
  rw [checkpoint_exact, checkpoint_exact, hs, he, appended]
  omega

#print axioms column_exact

-- Lines include their terminator bytes. The column reader receives the source
-- from its chosen line and can reject positions inside a surrogate pair or
-- beyond the line, as the LSP decoder does.
private def positionOffset {α : Type} (width : α → Nat) (column : List α → Nat → Option Nat) :
    List α → Nat → Nat → Option Nat
  | [], _, _ => none
  | l :: ls, 0, c => column (l :: ls) c
  | l :: ls, n + 1, c => (positionOffset width column ls n c).map (width l + ·)

private theorem positionOffset_append {α : Type} (width : α → Nat)
    (column : List α → Nat → Option Nat) (before suffix : List α) (line c : Nat) :
    positionOffset width column (before ++ suffix) (before.length + line) c =
      (positionOffset width column suffix line c).map ((before.map width).sum + ·) := by
  induction before with
  | nil => simp
  | cons l ls ih =>
    simp only [List.cons_append, List.length_cons, Nat.succ_add, positionOffset, ih,
      List.map_cons, List.sum_cons]
    cases positionOffset width column suffix line c <;> simp [Nat.add_assoc]

-- The editor drops an invalid start and falls back to one byte past a valid
-- start when the end is invalid. Either endpoint may come first.
private def diagnosticRange (start stop : Option Nat) : Option (Nat × Nat) :=
  start.map fun s => (s, stop.getD (s + 1))

theorem diagnostic_range_rebased {α : Type} (width : α → Nat)
    (column : List α → Nat → Option Nat) (before suffix : List α)
    (startLine startColumn stopLine stopColumn : Nat)
    (anchor : before.length = min startLine stopLine) :
    diagnosticRange (positionOffset width column (before ++ suffix) startLine startColumn)
        (positionOffset width column (before ++ suffix) stopLine stopColumn) =
      (diagnosticRange
        (positionOffset width column suffix (startLine - before.length) startColumn)
        (positionOffset width column suffix (stopLine - before.length) stopColumn)).map
          (fun r => ((before.map width).sum + r.1, (before.map width).sum + r.2)) := by
  have startAt : before.length + (startLine - before.length) = startLine := by omega
  have stopAt : before.length + (stopLine - before.length) = stopLine := by omega
  have startEq := positionOffset_append width column before suffix (startLine - before.length) startColumn
  have stopEq := positionOffset_append width column before suffix (stopLine - before.length) stopColumn
  rw [startAt] at startEq
  rw [stopAt] at stopEq
  rw [startEq, stopEq]
  cases positionOffset width column suffix (startLine - before.length) startColumn <;>
    cases positionOffset width column suffix (stopLine - before.length) stopColumn <;>
    simp [diagnosticRange, Nat.add_assoc]

#print axioms diagnostic_range_rebased
end Davinci.SourceMap
