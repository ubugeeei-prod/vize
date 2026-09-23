import Lean.Data.Json

namespace Impeto
open Lean

-- Addresses encode S3 operation ownership and keyed/positional loop scopes.
-- They are reference identities, never inferred from generated DOM strings.
inductive View where
  | text (value : String)
  | element (address : String) (tag : String) (attributes : List (String × Json))
      (disabled : Bool) (event : Option Json) (children : List View)
deriving BEq

namespace View

partial def json : View -> Json
  | .text value => .str value
  | .element _ tag attributes disabled _ children =>
      let fields := [("tag", .str tag), ("attributes", Json.mkObj attributes),
        ("children", .arr (children.map json).toArray)]
      Json.mkObj (if tag == "button" then fields ++ [("disabled", .bool disabled)] else fields)

def append (nodes : List View) (node : View) : List View :=
  match nodes, node with
  | _, .text "" => nodes
  | .text previous :: rest, .text value => .text (previous ++ value) :: rest
  | _, _ => node :: nodes

def tree (nodes : List View) : Json := .arr (nodes.map json).toArray

end View
end Impeto
