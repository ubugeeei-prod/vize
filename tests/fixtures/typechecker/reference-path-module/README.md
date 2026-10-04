The five authored files preserve the complete minimal reproduction in [#7832](https://github.com/ubugeeei-prod/vize/issues/7832), including the `*?raw` ambient module and the original compiler options. The manifest pins their bytes.

The shared test project applies explicit variants: leading `./`, a `.nuxt/` containing directory, a `paths` side-effect import, and `skipLibCheck`. A numeric assignment is a negative control for ambient type preservation. Native checks compare every raw diagnostic field and attest the original source, configuration and diagnosing session; production batch and source-built CLI checks compare complete diagnostic vectors.

An unresolved path with a matching import alias must remain unresolved. Triple-slash path references do not consult `paths` or package resolution. These fixtures do not replace any legacy product with a native provider.
