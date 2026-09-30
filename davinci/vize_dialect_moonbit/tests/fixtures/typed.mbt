fn __vize_expr_0(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_0(title, count, visible, event_name, save, use_label, items)
  __vize_show(title)
}
fn __vize_expr_1(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_1(title, count, visible, event_name, save, use_label, items)
  __vize_show(count.val)
}
fn __vize_expr_2(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_2(title, count, visible, event_name, save, use_label, items)
  __vize_cond(visible)
}
fn __vize_expr_3(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_3(title, count, visible, event_name, save, use_label, items)
  __vize_name(event_name)
}
fn __vize_expr_4(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_4(title, count, visible, event_name, save, use_label, items)
  __vize_on(save)
}
fn __vize_expr_5(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_5(title, count, visible, event_name, save, use_label, items)
  __vize_on(fn() { count.val += 1 })
}
fn __vize_expr_6(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_6(title, count, visible, event_name, save, use_label, items)
  __vize_bind(use_label())
}
fn __vize_expr_7(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String], item : String) -> Unit {
  @environment.__vize_scope_7(title, count, visible, event_name, save, use_label, items, item)
  __vize_show(item + title)
}
fn __vize_expr_8(count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String], title : Int, index : Int) -> Unit {
  @environment.__vize_scope_8(count, visible, event_name, save, use_label, items, title, index)
  __vize_show(title + index)
}
fn __vize_expr_9(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_9(title, count, visible, event_name, save, use_label, items)
  __vize_show(items[0])
}
fn __vize_expr_10(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_10(title, count, visible, event_name, save, use_label, items)
  __vize_on(fn() { save() })
}
fn __vize_expr_11(title : String, count : Ref[Int], visible : Bool, event_name : String, save : () -> Unit, use_label : () -> String, items : Array[String]) -> Unit {
  @environment.__vize_scope_11(title, count, visible, event_name, save, use_label, items)
  __vize_show("😀" + title)
}

fn[T] __vize_bind(value : T) -> Unit { ignore(value) }
fn[T : Show] __vize_show(value : T) -> Unit { ignore(value.to_string()) }
fn __vize_cond(value : Bool) -> Unit { ignore(value) }
fn __vize_name(value : String) -> Unit { ignore(value) }
fn __vize_on(value : () -> Unit) -> Unit { ignore(value) }
