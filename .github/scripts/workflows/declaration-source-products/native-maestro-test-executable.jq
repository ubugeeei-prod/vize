select(.reason == "compiler-artifact" and .target.name == "vize_maestro" and .executable != null) | .executable
