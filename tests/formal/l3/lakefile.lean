import Lake
open Lake DSL

package impeto where
  version := v!"0.1.0"

lean_lib Impeto where

@[default_target]
lean_exe impetoRef where
  root := `Main
