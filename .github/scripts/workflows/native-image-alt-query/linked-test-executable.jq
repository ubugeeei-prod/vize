[ .[] | select(.reason == "compiler-artifact" and .executable != null and .target.name == $name) ]
| last
| .executable
