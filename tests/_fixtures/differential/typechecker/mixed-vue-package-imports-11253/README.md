# Mixed Vue and private package imports

`input.json` retains the five original code-fence file bodies from the human report at
https://github.com/ubugeeei-prod/vize/issues/11253. The reporter supplied no lockfile.
The original package metadata is retained; source tests borrow the Actions source CLI
and existing real test Vue/native providers rather than installing that public Vize.
That source scope is separate from the reported Vue 3.5.44 / TypeScript 5.9.3 /
vue-tsc 3.3.14 / Linux WSL2 environment and later untouched public acceptance.

The compile-included native CLI regression consumes all five file bodies. It keeps
the original two TS2322 diagnostics, exercises all five reported import arrangements,
then a main-only error, full repair and restoration in the same project directory.
Every complete process result is retained before assertions. The source-inspected
manifest input does not manufacture a diagnostic pack or native admission.
