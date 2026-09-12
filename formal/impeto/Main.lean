import Impeto

open Impeto

structure Fixture where
  folio : String
  trace : String

def fixtures : List Fixture := [
  { folio := "fixtures/static-text.s3.folio", trace := "fixtures/static-text.trace" },
  { folio := "fixtures/dynamic-button.s3.folio", trace := "fixtures/dynamic-button.trace" },
  { folio := "fixtures/control-flow.s3.folio", trace := "fixtures/control-flow.trace" }
]

def joinLines : List String -> String
  | [] => ""
  | line :: rest => line ++ "\n" ++ joinLines rest

def traceText (program : Program) : String :=
  joinLines (referenceTrace program)

def readProgram (path : String) : IO (Except String Program) := do
  let text <- IO.FS.readFile (System.FilePath.mk path)
  pure (Folio.parseProgram text)

def checkPair (folioPath : String) (tracePath : String) : IO UInt32 := do
  match (<- readProgram folioPath) with
  | .error message =>
      IO.eprintln s!"{folioPath}: {message}"
      pure 2
  | .ok program =>
      let actual := traceText program
      let expected <- IO.FS.readFile (System.FilePath.mk tracePath)
      if actual == expected then
        pure 0
      else
        IO.eprintln s!"{tracePath}: trace drift"
        IO.eprintln "expected:"
        IO.eprintln expected
        IO.eprintln "actual:"
        IO.eprintln actual
        pure 1

def checkFixtures : List Fixture -> IO UInt32
  | [] => pure 0
  | fixture :: rest => do
      let code <- checkPair fixture.folio fixture.trace
      if code == 0 then
        checkFixtures rest
      else
        pure code

def printTrace (folioPath : String) : IO UInt32 := do
  match (<- readProgram folioPath) with
  | .error message =>
      IO.eprintln s!"{folioPath}: {message}"
      pure 2
  | .ok program =>
      IO.print (traceText program)
      pure 0

def usage : String :=
  "usage: impetoRef --check-fixtures | --check <s3.folio> <trace> | --trace <s3.folio>"

def main (args : List String) : IO UInt32 := do
  match args with
  | ["--check-fixtures"] => checkFixtures fixtures
  | ["--check", folioPath, tracePath] => checkPair folioPath tracePath
  | ["--trace", folioPath] => printTrace folioPath
  | _ =>
      IO.eprintln usage
      pure 2
