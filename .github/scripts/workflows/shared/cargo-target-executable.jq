[ .[] | select(.reason == "compiler-artifact" and .target.name == $name) ]
| last
| .executable
