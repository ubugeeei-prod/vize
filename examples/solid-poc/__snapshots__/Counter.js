import { template as _$template, insert as _$insert, className as _$className, effect as _$effect, delegateEvents as _$delegateEvents, createComponent as _$createComponent } from "solid-js/web";
const _tmpl$ = /*#__PURE__*/ _$template(`<section class="counter"><h1><!></h1><button type="button">Clicked <!> times</button><!></section>`);
const _tmpl$2 = /*#__PURE__*/ _$template(`<p>Hot!</p>`);
import { createSignal, Show } from "solid-js";

export function Counter(props) {
  const [count, setCount] = createSignal(props.start);
  return (
    (() => {
      const _el$ = _tmpl$();
      const _el$2 = _el$.firstChild;
      const _el$3 = _el$2.firstChild;
      const _el$4 = _el$2.nextSibling;
      const _el$5 = _el$4.firstChild.nextSibling;
      const _el$6 = _el$4.nextSibling;
      _$insert(_el$2, () => props.title, _el$3);
      _$effect(() => _$className(_el$4, count() > 3 ? "hot" : "cold"));
      _el$4.$$click = () => setCount(count() + 1);
      _$insert(_el$4, () => count(), _el$5);
      _$insert(_el$, _$createComponent(Show, { get when() { return count() > 3; }, get children() { return _tmpl$2(); } }), _el$6);
      return _el$;
    })()
  );
}
_$delegateEvents(["click"]);
