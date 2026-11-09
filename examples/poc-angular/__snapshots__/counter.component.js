import * as i0 from "@angular/core";
import { Component } from "@angular/core";

function CounterComponent_Conditional_5_Template(rf, ctx) {
  if (rf & 1) {
    i0.ɵɵelementStart(0, "p");
    i0.ɵɵtext(1, "Hot!");
    i0.ɵɵelementEnd();
  }
}

function CounterComponent_Conditional_6_Template(rf, ctx) {
  if (rf & 1) {
    i0.ɵɵelementStart(0, "p");
    i0.ɵɵtext(1, "Keep clicking");
    i0.ɵɵelementEnd();
  }
}

function CounterComponent_For_8_Template(rf, ctx) {
  if (rf & 1) {
    i0.ɵɵelementStart(0, "li");
    i0.ɵɵtext(1);
    i0.ɵɵelementEnd();
  }
  if (rf & 2) {
    const item_r3 = ctx.$implicit;
    i0.ɵɵadvance();
    i0.ɵɵtextInterpolate(item_r3);
  }
}

export class CounterComponent {
  title = "Counter";
  count = 0;
  items = ["write", "compile", "ship"];

  tone() {
    return this.count > 3 ? "hot" : "cold";
  }

  increment() {
    this.count += 1;
  }

  static ɵfac = function CounterComponent_Factory(__ngFactoryType__) {
    return new (__ngFactoryType__ || CounterComponent)();
  };
  static ɵcmp = /*@__PURE__*/ i0.ɵɵdefineComponent({
    type: CounterComponent,
    selectors: [["app-counter"]],
    decls: 9,
    vars: 6,
    consts: [[1, "counter"], ["type", "button"]],
    template: function CounterComponent_Template(rf, ctx) {
      if (rf & 1) {
        i0.ɵɵelementStart(0, "section", 0);
        i0.ɵɵelementStart(1, "h1");
        i0.ɵɵtext(2);
        i0.ɵɵelementEnd();
        i0.ɵɵelementStart(3, "button", 1);
        i0.ɵɵlistener("click", function CounterComponent_button_click_3_listener() { return ctx.increment(); });
        i0.ɵɵtext(4);
        i0.ɵɵelementEnd();
        i0.ɵɵtemplate(5, CounterComponent_Conditional_5_Template, 2, 0, "p");
        i0.ɵɵtemplate(6, CounterComponent_Conditional_6_Template, 2, 0, "p");
        i0.ɵɵelementStart(7, "ul");
        i0.ɵɵrepeaterCreate(8, CounterComponent_For_8_Template, 2, 1, "li", null, i0.ɵɵrepeaterTrackByIdentity);
        i0.ɵɵelementEnd();
        i0.ɵɵelementEnd();
      }
      if (rf & 2) {
        i0.ɵɵadvance(2);
        i0.ɵɵtextInterpolate(ctx.title);
        i0.ɵɵadvance();
        i0.ɵɵclassMap(ctx.tone());
        i0.ɵɵadvance();
        i0.ɵɵtextInterpolate1("Clicked ", ctx.count, " times");
        i0.ɵɵadvance();
        i0.ɵɵconditional(ctx.count > 3 ? 5 : 6);
        i0.ɵɵadvance(3);
        i0.ɵɵrepeater(ctx.items);
      }
    },
    encapsulation: 2,
  });
}
