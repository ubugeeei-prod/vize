// vize: virtual MoonBit projection of todo.vue (generated)

struct Todo {
  title : String
  done : Bool
}

let todos : Array[Todo] = [
  { title: "Write the dialect", done: true },
  { title: "Give the talk", done: false },
]

let show_done : Ref[Bool] = { val: true }

fn remaining() -> Int {
  todos.filter(fn(todo) { !todo.done }).length()
}

///|
fn __vize_template() -> Unit {
  __vize_show(remaining())
  __vize_on(fn() { show_done.val = !show_done.val })
  for todo in todos {
    __vize_cond(show_done.val || !todo.done)
    __vize_bind(todo.done)
    __vize_show(todo.title)
  }
  if remaining() > 0 {
  } else {
  }
}

///|
fn[T : Show] __vize_show(value : T) -> Unit {
  ignore(value.to_string())
}

///|
fn __vize_cond(value : Bool) -> Unit {
  ignore(value)
}

///|
fn[T] __vize_bind(value : T) -> Unit {
  ignore(value)
}

///|
fn __vize_on(handler : () -> Unit) -> Unit {
  ignore(handler)
}
