# Component fallthrough attributes

Jimmy Chen ([Q16solver](https://github.com/Q16solver)) supplied the reproductions
for a [forwarded native listener being rejected](https://github.com/ubugeeei-prod/vize/issues/13685)
and an [unknown component attribute being accepted](https://github.com/ubugeeei-prod/vize/issues/13686).

These fixtures keep the two reports as separate contracts: an inferred root
contributes its known listeners, while an attribute neither the component nor
its inferred root declares remains unknown.
