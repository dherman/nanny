use neon::{prelude::*, types::extract};

fn add1(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let x = cx.argument::<JsNumber>(0)?.value(&mut cx);
    Ok(cx.number(x + 1.0))
}

pub fn return_js_function(mut cx: FunctionContext) -> JsResult<JsFunction> {
    JsFunction::new(&mut cx, add1)
}

pub fn call_js_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let f = cx.argument::<JsFunction>(0)?;
    let args = [cx.number(16.0).upcast()];
    let null = cx.null();
    f.call(&mut cx, null, args)?
        .downcast::<JsNumber, _>(&mut cx)
        .or_throw(&mut cx)
}

pub fn call_js_function_idiomatically(mut cx: FunctionContext) -> JsResult<JsNumber> {
    cx.argument::<JsFunction>(0)?
        .call_with(&cx)
        .this(cx.null())
        .arg(cx.number(16.0))
        .apply(&mut cx)
}

pub fn call_js_function_with_bind(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n: f64 = cx
        .argument::<JsFunction>(0)?
        .bind(&mut cx)
        .args((1, 2, 3))?
        .arg(4)?
        .arg_with(|cx| cx.number(5))?
        .call()?;
    Ok(cx.number(n))
}

pub fn call_js_function_with_bind_and_args_with(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n: f64 = cx
        .argument::<JsFunction>(0)?
        .bind(&mut cx)
        .args_with(|_| (1, 2, 3))?
        .call()?;
    Ok(cx.number(n))
}

pub fn call_js_function_with_bind_and_args_and_with(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n: f64 = cx
        .argument::<JsFunction>(0)?
        .bind(&mut cx)
        .arg(extract::with(|cx| Ok(cx.number(1))))?
        .arg(2)?
        .arg(3)?
        .call()?;
    Ok(cx.number(n))
}

pub fn call_parse_int_with_bind(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let parse_int: Handle<JsFunction> = cx.global("parseInt")?;
    let x: f64 = parse_int.bind(&mut cx).arg("41")?.call()?;
    Ok(cx.number(x + 1.0))
}

pub fn call_js_function_with_bind_and_exec(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    cx.argument::<JsFunction>(0)?.bind(&mut cx).arg(1)?.exec()?;
    Ok(cx.undefined())
}

pub fn call_js_constructor_with_bind(mut cx: FunctionContext) -> JsResult<JsObject> {
    cx.argument::<JsFunction>(0)?
        .bind(&mut cx)
        .args((42, "hello"))?
        .construct()
}

pub fn bind_js_function_to_object(mut cx: FunctionContext) -> JsResult<JsValue> {
    let f = cx.argument::<JsFunction>(0)?;
    let obj = cx.empty_object();
    obj.prop(&mut cx, "prop").set(42)?;
    f.bind(&mut cx).this(obj)?.call()
}

pub fn bind_js_function_to_number(mut cx: FunctionContext) -> JsResult<JsValue> {
    let f = cx.argument::<JsFunction>(0)?;
    f.bind(&mut cx).this(42)?.call()
}

fn get_math_max<'a>(cx: &mut FunctionContext<'a>) -> JsResult<'a, JsFunction> {
    let math: Handle<JsObject> = cx.global("Math")?;
    let max: Handle<JsFunction> = math.get(cx, "max")?;
    Ok(max)
}

pub fn call_js_function_with_zero_args(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?.call_with(&cx).apply(&mut cx)
}

pub fn call_js_function_with_one_arg(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?
        .call_with(&cx)
        .arg(cx.number(1.0))
        .apply(&mut cx)
}

pub fn call_js_function_with_two_args(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?
        .call_with(&cx)
        .arg(cx.number(1.0))
        .arg(cx.number(2.0))
        .apply(&mut cx)
}

pub fn call_js_function_with_three_args(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?
        .call_with(&cx)
        .arg(cx.number(1.0))
        .arg(cx.number(2.0))
        .arg(cx.number(3.0))
        .apply(&mut cx)
}

pub fn call_js_function_with_four_args(mut cx: FunctionContext) -> JsResult<JsNumber> {
    get_math_max(&mut cx)?
        .call_with(&cx)
        .arg(cx.number(1.0))
        .arg(cx.number(2.0))
        .arg(cx.number(3.0))
        .arg(cx.number(4.0))
        .apply(&mut cx)
}

pub fn call_js_function_with_custom_this(mut cx: FunctionContext) -> JsResult<JsObject> {
    let custom_this = cx.empty_object();
    let secret = cx.number(42.0);
    custom_this.set(&mut cx, "secret", secret)?;
    cx.argument::<JsFunction>(0)?
        .call_with(&cx)
        .this(custom_this)
        .apply(&mut cx)
}

pub fn call_js_function_with_implicit_this(mut cx: FunctionContext) -> JsResult<JsValue> {
    cx.argument::<JsFunction>(0)?
        .call_with(&cx)
        .arg(cx.number(42))
        .apply(&mut cx)
}

pub fn exec_js_function_with_implicit_this(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    cx.argument::<JsFunction>(0)?
        .call_with(&cx)
        .arg(cx.number(42))
        .exec(&mut cx)?;
    Ok(cx.undefined())
}

pub fn call_js_function_with_heterogeneous_tuple(mut cx: FunctionContext) -> JsResult<JsArray> {
    cx.global::<JsFunction>("Array")?
        .call_with(&cx)
        .args((cx.number(1.0), cx.string("hello"), cx.boolean(true)))
        .apply(&mut cx)
}

pub fn construct_js_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let f = cx.argument::<JsFunction>(0)?;
    let zero = cx.number(0.0);
    let o = f.construct(&mut cx, [zero.upcast()])?;
    let get_utc_full_year_method: Handle<JsFunction> = o.get(&mut cx, "getUTCFullYear")?;
    let args: Vec<Handle<JsValue>> = vec![];
    get_utc_full_year_method
        .call(&mut cx, o.upcast::<JsValue>(), args)?
        .downcast::<JsNumber, _>(&mut cx)
        .or_throw(&mut cx)
}

pub fn construct_js_function_idiomatically(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let o: Handle<JsObject> = cx
        .argument::<JsFunction>(0)?
        .construct_with(&cx)
        .arg(cx.number(0.0))
        .apply(&mut cx)?;
    let get_utc_full_year_method: Handle<JsFunction> = o.get(&mut cx, "getUTCFullYear")?;
    get_utc_full_year_method
        .call_with(&cx)
        .this(o)
        .apply(&mut cx)
}

pub fn construct_js_function_with_overloaded_result(mut cx: FunctionContext) -> JsResult<JsArray> {
    let f: Handle<JsFunction> = cx.global("Array")?;
    f.construct_with(&cx)
        .arg(cx.number(1))
        .arg(cx.number(2))
        .arg(cx.number(3))
        .apply(&mut cx)
}

pub fn check_string_and_number(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    cx.argument::<JsString>(0)?;
    cx.argument::<JsNumber>(1)?;
    Ok(cx.undefined())
}

pub fn panic(_: FunctionContext) -> JsResult<JsUndefined> {
    panic!("zomg")
}

pub fn panic_after_throw(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    cx.throw_range_error::<_, ()>("entering throw state with a RangeError")
        .unwrap_err();
    panic!("this should override the RangeError")
}

pub fn num_arguments(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n = cx.len();
    Ok(cx.number(n as i32))
}

pub fn return_this(mut cx: FunctionContext) -> JsResult<JsValue> {
    cx.this()
}

pub fn require_object_this(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let this = cx.this::<JsObject>()?;
    let t = cx.boolean(true);
    this.set(&mut cx, "modified", t)?;
    Ok(cx.undefined())
}

pub fn is_argument_zero_some(mut cx: FunctionContext) -> JsResult<JsBoolean> {
    let b = cx.argument_opt(0).is_some();
    Ok(cx.boolean(b))
}

pub fn require_argument_zero_string(mut cx: FunctionContext) -> JsResult<JsString> {
    let s = cx.argument(0)?;
    Ok(s)
}

pub fn execute_scoped(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let mut i = 0;
    for _ in 1..100 {
        cx.execute_scoped(|mut cx| {
            let n = cx.number(1);
            i += n.value(&mut cx) as i32;
        });
    }
    Ok(cx.number(i))
}

pub fn compute_scoped(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let mut i = cx.number(0);
    for _ in 1..100 {
        i = cx.compute_scoped(|mut cx| {
            let n = cx.number(1);
            let left = i.value(&mut cx) as i32;
            let right = n.value(&mut cx) as i32;
            Ok(cx.number(left + right))
        })?;
    }
    Ok(i)
}

// Simple identity function to verify that a handle can be moved to `compute_scoped`
// closure and re-escaped.
pub fn recompute_scoped(mut cx: FunctionContext) -> JsResult<JsValue> {
    let value = cx.argument::<JsValue>(0)?;

    cx.compute_scoped(move |_| Ok(value))
}

pub fn throw_and_catch(mut cx: FunctionContext) -> JsResult<JsValue> {
    let v = cx
        .argument_opt(0)
        .unwrap_or_else(|| cx.undefined().upcast());

    cx.try_catch(|cx| cx.throw(v))
        .map(|_: ()| Ok(cx.string("unreachable").upcast()))
        .unwrap_or_else(Ok)
}

pub fn call_and_catch(mut cx: FunctionContext) -> JsResult<JsValue> {
    let f: Handle<JsFunction> = cx.argument(0)?;
    Ok(cx
        .try_catch(|cx| f.call_with(cx).this(cx.global_object()).apply(cx))
        .unwrap_or_else(|err| err))
}

pub fn get_number_or_default(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n = cx
        .try_catch(|cx| Ok(cx.argument::<JsNumber>(0)?.value(cx)))
        .unwrap_or(0.0);

    Ok(cx.number(n))
}

pub fn assume_this_is_an_object(mut cx: FunctionContext) -> JsResult<JsObject> {
    let this: Handle<JsObject> = cx.this()?;
    let result: Handle<JsObject> = cx.empty_object();
    let object_class: Handle<JsFunction> = cx.global("Object")?;
    let get_prototype_of: Handle<JsFunction> = object_class.get(&mut cx, "getPrototypeOf")?;
    let object_prototype: Handle<JsObject> = object_class.get(&mut cx, "prototype")?;
    let has_own_property: Handle<JsFunction> = object_prototype.get(&mut cx, "hasOwnProperty")?;
    let proto: Result<Handle<JsValue>, Handle<JsValue>> =
        cx.try_catch(|cx| get_prototype_of.call_with(cx).arg(this).apply(cx));
    let proto: Handle<JsValue> = proto.unwrap_or_else(|_| cx.undefined().upcast());
    let has_own: Handle<JsBoolean> = has_own_property
        .call_with(&cx)
        .this(this)
        .arg(cx.string("toString"))
        .apply(&mut cx)?;
    let prop: Handle<JsValue> = this.get(&mut cx, "toString")?;
    result.set(&mut cx, "prototype", proto)?;
    result.set(&mut cx, "hasOwn", has_own)?;
    result.set(&mut cx, "property", prop)?;
    Ok(result)
}

pub fn is_construct(mut cx: FunctionContext) -> JsResult<JsObject> {
    let this = cx.this::<JsObject>()?;
    let construct = matches!(cx.kind(), CallKind::Construct);
    let construct = cx.boolean(construct);
    this.set(&mut cx, "wasConstructed", construct)?;
    Ok(this)
}

// `function caller_with_drop_callback(wrappedCallback, dropCallback)`
//
// `wrappedCallback` will be called each time the returned function is
// called to verify we have successfully dynamically created a function
// from a closure.
//
// `dropCallback` will be called when the closure is dropped to test that
// closures are not leaking. The unit test should pass the test callback here.
pub fn caller_with_drop_callback(mut cx: FunctionContext) -> JsResult<JsFunction> {
    struct Callback {
        f: Root<JsFunction>,
        drop: Option<Root<JsFunction>>,
        channel: Channel,
    }

    // Call `dropCallback` when `Callback` is dropped as a sentinel to observe
    // the closure isn't leaked when the function is garbage collected
    impl Drop for Callback {
        fn drop(&mut self) {
            let callback = self.drop.take();

            self.channel.send(move |mut cx| {
                let this = cx.undefined();
                let args: [Handle<JsValue>; 0] = [];

                // Execute the unit test callback to end the test successfully
                callback
                    .unwrap()
                    .into_inner(&mut cx)
                    .call(&mut cx, this, args)?;

                Ok(())
            });
        }
    }

    let callback = Callback {
        f: cx.argument::<JsFunction>(0)?.root(&mut cx),
        drop: Some(cx.argument::<JsFunction>(1)?.root(&mut cx)),
        channel: cx.channel(),
    };

    JsFunction::new(&mut cx, move |mut cx| {
        let this = cx.undefined();
        let args: [Handle<JsValue>; 0] = [];

        callback.f.to_inner(&mut cx).call(&mut cx, this, args)
    })
}
