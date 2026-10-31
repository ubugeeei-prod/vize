# Scoped formatter discovery (#11146)

The Vue inputs retain the issue-authored formatted component and unformatted
story. `vue-options.json` retains the `withVue` options from the reproduction;
the CLI test uses the repository's canonical Vite carrier. The test pins complete
stdout/stderr, exit status, and unchanged configuration/source bytes for glob
and explicit-path `fmt --check` invocations. Pure-global and formatter-only
exclusions have separate typed projection and loader controls.
