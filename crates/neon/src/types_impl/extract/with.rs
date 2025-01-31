use crate::{
    context::Cx,
    result::JsResult,
    types::{extract::TryIntoJs, Value},
};

struct With<F>(pub F);

/// Wraps a closure that will be lazily evaluated when [`TryIntoJs::try_into_js`] is
/// called.
///
/// Useful for executing arbitrary code on the main thread before returning from a
/// function exported with [`neon::export`](crate::export).
///
/// **Note:** The return type is [`JsResult`]. If you need to return a non-JavaScript type,
/// call [`TryIntoJs::try_into_js`].
///
/// _See [`With`](With#Example) for example usage._
///
/// ## Example
///
/// ```
/// # use neon::{prelude::*, types::extract::{self, TryIntoJs}};
/// use std::time::Instant;
///
/// #[neon::export(task)]
/// fn sum(nums: Vec<f64>) -> impl for<'cx> TryIntoJs<'cx> {
///     let start = Instant::now();
///     let sum = nums.into_iter().sum::<f64>();
///     let log = format!("sum took {} ms", start.elapsed().as_millis());
///
///     extract::with(move |cx| -> NeonResult<_> {
///         cx.global::<JsObject>("console")?
///             .method(cx, "log")?
///             .arg(&log)?
///             .exec()?;
///
///         sum.try_into_js(cx)
///     })
/// }
/// ```
pub fn with<V, F>(f: F) -> impl for<'cx> TryIntoJs<'cx, Value = V>
where
    V: Value,
    for<'cx> F: FnOnce(&mut Cx<'cx>) -> JsResult<'cx, V>,
{
    With(f)
}

impl<'cx, O, F> TryIntoJs<'cx> for With<F>
where
    O: TryIntoJs<'cx>,
    F: FnOnce(&mut Cx<'cx>) -> O,
{
    type Value = O::Value;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        (self.0)(cx).try_into_js(cx)
    }
}

impl<F> super::private::Sealed for With<F> {}
