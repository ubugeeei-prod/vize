import * as $ from "svelte/internal/client";

var root = $.from_html(`<section class="counter"><h1> </h1><button type="button"> </button><!><input><p> </p><ul><!></ul></section>`);
var root_1 = $.from_html(`<p>Hot!</p>`);
var root_2 = $.from_html(`<p>Keep clicking</p>`);
var root_3 = $.from_html(`<li> </li>`);

export default function Component($$anchor, $$props) {
  let count = $.state(0);
  let name = $.state("world");
  let items = $.state(["write", "compile", "ship"]);
  let tone = $.derived(() => $.get(count) > 3 ? "hot" : "cold");

  function increment() {
    $.set(count, $.get(count) + 1);
  }

  var section = root();
  var h1 = $.child(section);
  var text = $.child(h1);
  var button = $.sibling(h1);
  button.__click = increment;
  var text_1 = $.child(button);
  var node = $.sibling(button);
  var consequent = ($$anchor) => {
    var p = root_1();
    $.append($$anchor, p);
  };
  var alternate = ($$anchor) => {
    var p_1 = root_2();
    $.append($$anchor, p_1);
  };
  $.if(node, ($$render) => {
    if ($.get(count) > 3) $$render(consequent); else $$render(alternate, false);
  });
  var input = $.sibling(node);
  $.bind_value(input, () => $.get(name), ($$value) => $.set(name, $$value));
  var p_2 = $.sibling(input);
  var text_2 = $.child(p_2);
  var ul = $.sibling(p_2);
  var node_1 = $.child(ul);
  $.each(node_1, 1, () => $.get(items), $.index, ($$anchor, item) => {
    var li = root_3();
    var text_3 = $.child(li);
    $.template_effect(() => $.set_text(text_3, `${$.get(item) ?? ""}`));
    $.append($$anchor, li);
  });
  $.template_effect(() => {
    $.set_text(text, `${$$props.title ?? ""}`);
    $.set_attribute(button, "class", $.get(tone));
    $.set_text(text_1, `Clicked ${$.get(count) ?? ""} times`);
    $.set_text(text_2, `Hello ${$.get(name) ?? ""}!`);
  });
  $.append($$anchor, section);
}

$.delegate(["click"]);
