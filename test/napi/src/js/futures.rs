use std::future::Future;

use neon::{
    prelude::*,
    types::{
        buffer::TypedArray,
        extract::{self, Error, Json, TryIntoJs},
    },
};

use crate::runtime;

// Accepts two functions that take no parameters and return numbers.
// Resolves with the sum of the two numbers.
// Purpose: Test the `Future` implementation on `JoinHandle`
pub fn lazy_async_add(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let get_x = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let get_y = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let channel = cx.channel();
    let runtime = runtime(&mut cx)?;
    let (deferred, promise) = cx.promise();

    runtime.spawn(async move {
        let result = channel
            .send(move |mut cx| {
                let get_x = get_x.into_inner(&mut cx);
                let get_y = get_y.into_inner(&mut cx);

                let x: Handle<JsNumber> = get_x.call_with(&cx).apply(&mut cx)?;
                let y: Handle<JsNumber> = get_y.call_with(&cx).apply(&mut cx)?;

                Ok((x.value(&mut cx), y.value(&mut cx)))
            })
            .await
            .map(|(x, y)| x + y);

        deferred.settle_with(&channel, move |mut cx| {
            let result = result.or_throw(&mut cx)?;

            Ok(cx.number(result))
        });
    });

    Ok(promise)
}

// Accepts a function that returns a `Promise<Float64Array>`.
// Resolves with the sum of all numbers.
// Purpose: Test `JsPromise::to_future`.
pub fn lazy_async_sum(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let nums = cx
        .argument::<JsFunction>(0)?
        .call_with(&cx)
        .apply::<JsPromise, _>(&mut cx)?
        .to_future(&mut cx, |mut cx, nums| {
            let nums = nums
                .or_throw(&mut cx)?
                .downcast_or_throw::<JsTypedArray<f64>, _>(&mut cx)?
                .as_slice(&cx)
                .to_vec();

            Ok(nums)
        })?;

    let (deferred, promise) = cx.promise();
    let channel = cx.channel();
    let runtime = runtime(&mut cx)?;

    runtime.spawn(async move {
        let result = nums.await.map(|nums| nums.into_iter().sum::<f64>());

        deferred.settle_with(&channel, move |mut cx| {
            let result = result.or_throw(&mut cx)?;

            Ok(cx.number(result))
        });
    });

    Ok(promise)
}

#[neon::export]
async fn async_fn_add(a: f64, b: f64) -> f64 {
    a + b
}

#[neon::export(async)]
fn async_add(a: f64, b: f64) -> impl Future<Output = f64> {
    async move { a + b }
}

#[neon::export]
async fn async_fn_div(a: f64, b: f64) -> Result<f64, Error> {
    if b == 0.0 {
        return Err(Error::from("Divide by zero"));
    }

    Ok(a / b)
}

#[neon::export(async)]
fn async_div(cx: &mut FunctionContext) -> NeonResult<impl Future<Output = Result<f64, Error>>> {
    let (a, b): (f64, f64) = cx.args()?;

    Ok(async move {
        if b == 0.0 {
            return Err(Error::from("Divide by zero"));
        }

        Ok(a / b)
    })
}

#[neon::export(async)]
fn async_with_events(
    cx: &mut FunctionContext,
    Json(data): Json<Vec<(f64, f64)>>,
) -> NeonResult<impl Future<Output = impl for<'cx> TryIntoJs<'cx>>> {
    fn emit(cx: &mut Cx, state: &str) -> NeonResult<()> {
        cx.global::<JsObject>("process")?
            .call_method_with(cx, "emit")?
            .arg(cx.string("async_with_events"))
            .arg(cx.string(state))
            .exec(cx)
    }

    emit(cx, "start")?;

    Ok(async move {
        let res = data.into_iter().map(|(a, b)| a * b).collect::<Vec<_>>();

        extract::with(move |cx| -> NeonResult<_> {
            emit(cx, "end")?;
            res.try_into_js(cx)
        })
    })
}

#[neon::export]
async fn await_callback(ch: Channel, cb: Root<JsFunction>) -> Result<Root<JsObject>, Error> {
    let res = ch
        .send(move |mut cx| {
            let this = cx.undefined();

            cb.into_inner(&mut cx)
                .call(&mut cx, this, [])
                .and_then(|v| v.downcast_or_throw::<JsObject, _>(&mut cx))
                .map(|v| v.root(&mut cx))
        })
        .await?;

    Ok(res)
}
