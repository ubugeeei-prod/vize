import Impeto
import Impeto.BehaviorTests
import Impeto.ControlTests
import Impeto.LoopTests

open Impeto

structure Fixture where
  folio : String
  trace : String
  vdomTrace : String
  vaporTrace : String

def fixtures : List Fixture := [
  {
    folio := "fixtures/static-text.s3.folio"
    trace := "fixtures/static-text.trace"
    vdomTrace := "fixtures/static-text.vdom.trace"
    vaporTrace := "fixtures/static-text.vapor.trace"
  },
  {
    folio := "fixtures/dynamic-button.s3.folio"
    trace := "fixtures/dynamic-button.trace"
    vdomTrace := "fixtures/dynamic-button.vdom.trace"
    vaporTrace := "fixtures/dynamic-button.vapor.trace"
  },
  {
    folio := "fixtures/control-flow.s3.folio"
    trace := "fixtures/control-flow.trace"
    vdomTrace := "fixtures/control-flow.vdom.trace"
    vaporTrace := "fixtures/control-flow.vapor.trace"
  },
  {
    folio := "fixtures/rust-lowered-static-dynamic.s3.folio"
    trace := "fixtures/rust-lowered-static-dynamic.trace"
    vdomTrace := "fixtures/rust-lowered-static-dynamic.vdom.trace"
    vaporTrace := "fixtures/rust-lowered-static-dynamic.vapor.trace"
  },
  {
    folio := "fixtures/rust-lowered-control-slots.s3.folio"
    trace := "fixtures/rust-lowered-control-slots.trace"
    vdomTrace := "fixtures/rust-lowered-control-slots.vdom.trace"
    vaporTrace := "fixtures/rust-lowered-control-slots.vapor.trace"
  },
  {
    folio := "fixtures/rust-lowered-loop-keyed.s3.folio"
    trace := "fixtures/rust-lowered-loop-keyed.trace"
    vdomTrace := "fixtures/rust-lowered-loop-keyed.vdom.trace"
    vaporTrace := "fixtures/rust-lowered-loop-keyed.vapor.trace"
  },
  {
    folio := "fixtures/rust-lowered-loop-unkeyed.s3.folio"
    trace := "fixtures/rust-lowered-loop-unkeyed.trace"
    vdomTrace := "fixtures/rust-lowered-loop-unkeyed.vdom.trace"
    vaporTrace := "fixtures/rust-lowered-loop-unkeyed.vapor.trace"
  },
  {
    folio := "fixtures/rust-lowered-loop-nested.s3.folio"
    trace := "fixtures/rust-lowered-loop-nested.trace"
    vdomTrace := "fixtures/rust-lowered-loop-nested.vdom.trace"
    vaporTrace := "fixtures/rust-lowered-loop-nested.vapor.trace"
  }
]

def joinLines : List String -> String
  | [] => ""
  | line :: rest => line ++ "\n" ++ joinLines rest

def traceText (program : Program) : String :=
  joinLines (referenceTrace program)

def backendTraceText (backend : Backend) (program : Program) : String :=
  joinLines ((run backend program).map TraceEvent.format)

def readProgram (path : String) : IO (Except String Program) := do
  let text <- IO.FS.readFile (System.FilePath.mk path)
  pure (Folio.parseProgram text)

def checkTraceText (tracePath : String) (actual : String) : IO UInt32 := do
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

def checkPair (folioPath : String) (tracePath : String) : IO UInt32 := do
  match (<- readProgram folioPath) with
  | .error message =>
      IO.eprintln s!"{folioPath}: {message}"
      pure 2
  | .ok program =>
      let actual := traceText program
      checkTraceText tracePath actual

def checkBackendPair (fixture : Fixture) : IO UInt32 := do
  match (<- readProgram fixture.folio) with
  | .error message =>
      IO.eprintln s!"{fixture.folio}: {message}"
      pure 2
  | .ok program =>
      let vdomCode <- checkTraceText fixture.vdomTrace (backendTraceText .vdom program)
      if vdomCode != 0 then
        pure vdomCode
      else
        checkTraceText fixture.vaporTrace (backendTraceText .vapor program)

def checkFixtures : List Fixture -> IO UInt32
  | [] => pure 0
  | fixture :: rest => do
      let code <- checkPair fixture.folio fixture.trace
      if code == 0 then
        checkFixtures rest
      else
        pure code

def checkBackendFixtures : List Fixture -> IO UInt32
  | [] => pure 0
  | fixture :: rest => do
      let code <- checkBackendPair fixture
      if code == 0 then
        checkBackendFixtures rest
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

def printBackendTrace (backend : Backend) (folioPath : String) : IO UInt32 := do
  match (<- readProgram folioPath) with
  | .error message =>
      IO.eprintln s!"{folioPath}: {message}"
      pure 2
  | .ok program =>
      IO.print (backendTraceText backend program)
      pure 0

def usage : String :=
  "usage: impetoRef --check-fixtures | --check-backend-fixtures | --check-stateful-fixtures | " ++
    "--check <s3.folio> <trace> | --trace <s3.folio> | " ++
    "--trace-vdom <s3.folio> | --trace-vapor <s3.folio>"

def main (args : List String) : IO UInt32 := do
  match args with
  | ["--check-fixtures"] => checkFixtures fixtures
  | ["--check-backend-fixtures"] => checkBackendFixtures fixtures
  | ["--check-stateful-fixtures"] =>
      let code <- BehaviorTests.check
      if code != 0 then pure code
      else
        let code <- ControlTests.check
        if code != 0 then return code
        let code <- Behavior.check "fixtures/rust-lowered-static-dynamic"
        if code != 0 then pure code
        else
          let code <- Behavior.check "fixtures/rust-lowered-control-slots"
          if code != 0 then pure code else LoopTests.check
  | ["--check", folioPath, tracePath] => checkPair folioPath tracePath
  | ["--trace", folioPath] => printTrace folioPath
  | ["--trace-vdom", folioPath] => printBackendTrace .vdom folioPath
  | ["--trace-vapor", folioPath] => printBackendTrace .vapor folioPath
  | _ =>
      IO.eprintln usage
      pure 2
