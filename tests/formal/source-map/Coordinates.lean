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
end Davinci.SourceMap
