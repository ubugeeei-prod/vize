# DOM property bindings

Jimmy Chen ([Q16solver](https://github.com/Q16solver)) reported and minimized
[`srcObject.prop` being rejected as an unknown attribute](https://github.com/ubugeeei-prod/vize/issues/13688).

The fixture keeps DOM property lookup separate from attribute lookup. It accepts
`srcObject`, checks its value type, rejects an undeclared property, and keeps SVG
property lookup in the SVG namespace.
