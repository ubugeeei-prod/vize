import Lake
open Lake DSL

package l3 where
  version := v!"0.1.0"

lean_lib L3 where

@[default_target]
lean_exe l3Ref where
  root := `Main
