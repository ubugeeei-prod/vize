Reported by Jimmy Chen ([@Q16solver](https://github.com/Q16solver)) in
[#11915](https://github.com/ubugeeei-prod/vize/issues/11915), from the
[Esposter camera component](https://github.com/Esposter/Esposter/blob/ece0c72567e3031749f95637d49017e8a3698257/apps/web/app/components/Visual/Gem/Index.vue#L8).

An inline constructor must receive the unwrapped class: `new (_unref(Vector3))(...)`.
`new _unref(Vector3)(...)` constructs the helper first and calls the returned class
without `new`. The same precedence error affects member constructors.

The source integration test compiles every original matrix SFC through the public
adapter and executes the full component beside the official Vue compiler. It
checks imported classes/functions, mutable and factory-created classes, member
constructors, refs, plain classes, and ordinary calls. The original broken
expression is also executed as a negative control. The pinned SSR module is the
separate-template control observed with the public 0.691.0 CLI; it is compared
byte for byte and makes no inline-runtime acceptance claim.

After publishing, install `@vizejs/native@<released-version>`, `vue@3.5.44`, and
`@vue/compiler-sfc@3.5.44` into a fresh directory and run:

```sh
NODE_ENV=production node tests/tooling/new-unref-constructor-runtime.mjs /absolute/provider/directory
```

The command must exit zero. Its JSON retains the exact loaded native version,
Vue version, complete generated/reference modules, and the nine matching runtime
objects. A published package is accepted only after this fresh installed run.
