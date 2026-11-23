file: bindings:
let
  names = builtins.attrNames bindings;
in
builtins.replaceStrings (map (name: "@" + name + "@") names) (map (name: bindings.${name}) names) (
  builtins.readFile file
)
