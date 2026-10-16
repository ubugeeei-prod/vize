# Historical report: Vapor child HMR loses parent setup state

- Report: <https://github.com/ubugeeei-prod/vize/issues/9180>
- Reporter: [ナイトウコウスケ / naitokosuke](https://github.com/naitokosuke), GitHub user ID 102337893.
- Reported versions: `@vizejs/vite-plugin` 0.443.0 and Vue 3.6.0-rc.10.
- Reported application: `naitokosuke/chirashitabi` at
  `521aca90626bdc37f1c22c6f9deae15a43033ffa`.
- Parent: [`src/app/pages/rooms/[roomId=roomId].vue`](https://github.com/naitokosuke/chirashitabi/blob/521aca90626bdc37f1c22c6f9deae15a43033ffa/src/app/pages/rooms/%5BroomId%3DroomId%5D.vue),
  Git blob `01b9eb491c7ab0f1f43d333e3ed4cdd167b0e1e2`.
- Child: [`src/app/pages/rooms/[roomId=roomId]/canvas.vue`](https://github.com/naitokosuke/chirashitabi/blob/521aca90626bdc37f1c22c6f9deae15a43033ffa/src/app/pages/rooms/%5BroomId%3DroomId%5D/canvas.vue),
  Git blob `1793ede2cc5eff3a20d6a3411a3fcabdcf3671c1`.

The reported parent is `<script setup lang="ts" vapor>` and reads computed
`inRoom` in a conditional template. Its child is reached through `RouterView`,
`KeepAlive`, and a dynamic component. Opening a room canvas in the dev server
and saving the child produces, on each save:

```text
TypeError: Cannot read properties of undefined (reading 'inRoom')
    at render
    at callRender / devRender / runDevRender
    at hmrRerender
    at reload
```

The reporter identified the generated `getCurrentInstance()` / `setupState`
assignment and proposed returning setup bindings as Vue's compiler does.
The application itself does not call `getCurrentInstance()`.

`Parent.vue.txt` and `Child.vue.txt` are independently authored minimal inputs,
not copies of the application. The test runs both a direct child and a
`KeepAlive` / dynamic child, with two child script reloads, reactive updates,
production controls, and unmount. It compiles Vapor with current-source Vize
and compares the full observed tree with the same-source official Vue VDOM
compiler, both under the supported Vue **3.6.0-rc.11** runtime.

The rc.10 application report is retained as historical evidence; this fixture
does not claim execution of that application or current-output compatibility
with the older rc.10 runtime. Installed public-package acceptance remains a
separate release check.
