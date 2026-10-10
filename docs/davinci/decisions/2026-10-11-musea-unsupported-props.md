# Musea unsupported Vue props

Tracked in [#8533](https://github.com/ubugeeei-prod/vize/issues/8533).

Vue 3.5.41 does not apply the literal `__proto__` component prop. A pinned
Vue-only runtime control reproduces the exclusion with a null-prototype prop
definition and an ordinary input object containing its own data key. The
official compiler output was inspected separately. This does not establish a
Vize compiler defect.

The actual native compact-props browser fixture retains that own key in editor
state and persistence, but excludes it from preview messages and rendered
props. Successful transport therefore does not prove successful application.

For a resolved `.vue` component declaring this key, the palette exposes an
optional unsupported-key list. Raw inline/custom bridges and ordinary palette
responses keep their existing contract. Static gallery data uses the same
palette handler.

The editor keeps the raw value visible and saved, marks its control as
unsupported, and sends or copies only supported props. Code mode still accepts
ordinary edits with an unchanged unsupported value, and permits explicit
removal. Changing or adding the flagged value reports an error. Copy remains
available with an explicit notice that it contains supported props only.

Actual native dev and built/static HTTP browser controls must cover retained
own keys, per-key refusal, supported editing, preview, copied execution,
persistence and reset, alongside unchanged unflagged bridge controls. Completion
requires an included public release and genuine installed-consumer acceptance;
source evidence alone does not complete props support.
