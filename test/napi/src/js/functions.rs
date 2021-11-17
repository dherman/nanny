use neon::object::This;
use neon::prelude::*;

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

pub fn construct_js_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let f = cx.argument::<JsFunction>(0)?;
    let zero = cx.number(0.0);
    let o = f.construct(&mut cx, [zero.upcast()])?;
    let get_utc_full_year_method = o
        .get(&mut cx, "getUTCFullYear")?
        .downcast::<JsFunction, _>(&mut cx)
        .or_throw(&mut cx)?;
    let args: Vec<Handle<JsValue>> = vec![];
    get_utc_full_year_method
        .call(&mut cx, o.upcast::<JsValue>(), args)?
        .downcast::<JsNumber, _>(&mut cx)
        .or_throw(&mut cx)
}

trait CheckArgument<'a> {
    fn check_argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V>;
}

impl<'a, T: This> CheckArgument<'a> for CallContext<'a, T> {
    fn check_argument<V: Value>(&mut self, i: i32) -> JsResult<'a, V> {
        self.argument::<V>(i)
    }
}

pub fn check_string_and_number(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    cx.check_argument::<JsString>(0)?;
    cx.check_argument::<JsNumber>(1)?;
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
    Ok(cx.number(n))
}

pub fn return_this(mut cx: FunctionContext) -> JsResult<JsValue> {
    Ok(cx.this().upcast())
}

pub fn require_object_this(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let this = cx.this();
    let this = this.downcast::<JsObject, _>(&mut cx).or_throw(&mut cx)?;
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
        .try_catch(|cx| {
            let global = cx.global();
            let args: Vec<Handle<JsValue>> = vec![];
            f.call(cx, global, args)
        })
        .unwrap_or_else(|err| err))
}

pub fn get_number_or_default(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let n = cx
        .try_catch(|cx| Ok(cx.argument::<JsNumber>(0)?.value(cx)))
        .unwrap_or(0.0);

    Ok(cx.number(n))
}

pub fn is_construct(mut cx: FunctionContext) -> JsResult<JsObject> {
    let this = cx.this();
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
