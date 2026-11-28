# Undeclared listeners under noPropertyAccessFromIndexSignature

[2nofa11](https://github.com/2nofa11) reported
[`vize check` reporting TS4111 on `@click` passed to a component without that event](https://github.com/ubugeeei-prod/vize/issues/13881).

A listener for an event the component does not declare is a fallthrough
listener. Its navigation reference read the event name off an index signature
with dot access, which `noPropertyAccessFromIndexSignature` rejects. The fixture
checks that `@click`, `@blur` and a kebab `@focus-in` stay clean while a
declared event keeps its handler check, as in vue-tsc.
