import { Component } from "@angular/core";

@Component({
  selector: "app-counter",
  template: `
    <section class="counter">
      <h1>{{ title }}</h1>
      <button type="button" [class]="tone()" (click)="increment()">
        Clicked {{ count }} times
      </button>
      @if (count > 3) {
        <p>Hot!</p>
      } @else {
        <p>Keep clicking</p>
      }
      <ul>
        @for (item of items; track item) {
          <li>{{ item }}</li>
        }
      </ul>
    </section>
  `,
})
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
}
