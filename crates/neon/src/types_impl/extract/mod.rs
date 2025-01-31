//! Traits and utilities for extract Rust data from JavaScript values.
//!
//! The full list of included extractors can be found on [`TryFromJs`].
//!
//! ## Extracting Handles
//!
//! JavaScript arguments may be extracted into a Rust tuple.
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//! fn greet(mut cx: FunctionContext) -> JsResult<JsString> {
//!     let (greeting, name): (Handle<JsString>, Handle<JsString>) = cx.args()?;
//!     let message = format!("{}, {}!", greeting.value(&mut cx), name.value(&mut cx));
//!
//!     Ok(cx.string(message))
//! }
//! ```
//!
//! ## Extracting Native Types
//!
//! It's also possible to extract directly into native Rust types instead of a [`Handle`].
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//! fn add(mut cx: FunctionContext) -> JsResult<JsNumber> {
//!     let (a, b): (f64, f64) = cx.args()?;
//!
//!     Ok(cx.number(a + b))
//! }
//! ```
//!
//! ## Extracting [`Option`]
//!
//! It's also possible to mix [`Handle`], Rust types, and even [`Option`] for
//! handling `null` and `undefined`.
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//! fn get_or_default(mut cx: FunctionContext) -> JsResult<JsValue> {
//!     let (n, default_value): (Option<f64>, Handle<JsValue>) = cx.args()?;
//!
//!     if let Some(n) = n {
//!         return Ok(cx.number(n).upcast());
//!     }
//!
//!     Ok(default_value)
//! }
//! ```
//!
//! ## Additional Extractors
//!
//! In some cases, the expected JavaScript type is ambiguous. For example, when
//! trying to extract an [`f64`], the argument may be a `Date` instead of a `number`.
//! Newtype extractors are provided to help.
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//! # #[cfg(feature = "napi-5")]
//! # use neon::types::JsDate;
//!
//! # #[cfg(feature = "napi-5")]
//! fn add_hours(mut cx: FunctionContext) -> JsResult<JsDate> {
//!     const MS_PER_HOUR: f64 = 60.0 * 60.0 * 1000.0;
//!
//!     let (Date(date), hours): (Date, f64) = cx.args()?;
//!     let date = date + hours * MS_PER_HOUR;
//!
//!     cx.date(date).or_throw(&mut cx)
//! }
//! ```
//!
//! ## Overloaded Functions
//!
//! It's common in JavaScript to overload function signatures. This can be implemented with
//! [`FunctionContext::args_opt`] or [`Context::try_catch`].
//!
//! ```
//! # use neon::{prelude::*, types::extract::*};
//!
//! fn add(mut cx: FunctionContext, a: f64, b: f64) -> Handle<JsNumber> {
//!     cx.number(a + b)
//! }
//!
//! fn concat(mut cx: FunctionContext, a: String, b: String) -> Handle<JsString> {
//!     cx.string(a + &b)
//! }
//!
//! fn combine(mut cx: FunctionContext) -> JsResult<JsValue> {
//!     if let Some((a, b)) = cx.args_opt()? {
//!         return Ok(add(cx, a, b).upcast());
//!     }
//!
//!     let (a, b) = cx.args()?;
//!
//!     Ok(concat(cx, a, b).upcast())
//! }
//! ```
//!
//! Note well, in this example, type annotations are not required on the tuple because
//! Rust is able to infer it from the type arguments on `add` and `concat`.

use crate::{
    context::{Context, Cx, FunctionContext},
    handle::Handle,
    result::{JsResult, NeonResult},
    types::{JsValue, Value},
};

pub use self::{
    boxed::Boxed,
    buffer::{
        ArrayBuffer, BigInt64Array, BigUint64Array, Buffer, Float32Array, Float64Array, Int16Array,
        Int32Array, Int8Array, Uint16Array, Uint32Array, Uint8Array,
    },
    error::{Error, TypeExpected},
    with::with,
};

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use self::json::Json;

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub mod json;

mod boxed;
mod buffer;
mod either;
mod error;
mod private;
mod try_from_js;
mod try_into_js;
mod with;

/// Extract Rust data from a JavaScript value
pub trait TryFromJs<'cx>
where
    Self: private::Sealed + Sized,
{
    type Error: TryIntoJs<'cx>;

    /// Extract this Rust type from a JavaScript value
    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>>;

    /// Same as [`TryFromJs`], but all errors are converted to JavaScript exceptions
    fn from_js(cx: &mut Cx<'cx>, v: Handle<'cx, JsValue>) -> NeonResult<Self> {
        match Self::try_from_js(cx, v)? {
            Ok(v) => Ok(v),
            Err(err) => {
                let err = err.try_into_js(cx)?;

                cx.throw(err)
            }
        }
    }
}

/// Convert Rust data into a JavaScript value
pub trait TryIntoJs<'cx>
where
    Self: private::Sealed,
{
    /// The type of JavaScript value that will be created
    type Value: Value;

    /// Convert `self` into a JavaScript value
    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value>;
}

#[cfg_attr(docsrs, doc(cfg(feature = "napi-5")))]
#[cfg(feature = "napi-5")]
/// Wrapper for converting between [`f64`] and [`JsDate`](super::JsDate)
pub struct Date(pub f64);

/// Trait specifying values that may be extracted from function arguments.
///
/// **Note:** This trait is implemented for tuples of up to 32 values, but for
/// the sake of brevity, only tuples up to size 8 are shown in this documentation.
pub trait FromArgs<'cx>: private::FromArgsInternal<'cx> {}

// Convenience implementation for single arguments instead of needing a single element tuple
impl<'cx, T> private::FromArgsInternal<'cx> for T
where
    T: TryFromJs<'cx>,
{
    fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self> {
        let (v,) = private::FromArgsInternal::from_args(cx)?;

        Ok(v)
    }

    fn from_args_opt(cx: &mut FunctionContext<'cx>) -> NeonResult<Option<Self>> {
        if let Some((v,)) = private::FromArgsInternal::from_args_opt(cx)? {
            Ok(Some(v))
        } else {
            Ok(None)
        }
    }
}

impl<'cx, T> FromArgs<'cx> for T where T: TryFromJs<'cx> {}

// N.B.: `FromArgs` _could_ have a blanket impl for `T` where `T: FromArgsInternal`.
// However, it is explicitly implemented in the macro in order for it to be included in docs.
macro_rules! from_args_impl {
    ($(#[$attrs:meta])? [$($ty:ident),*]) => {
        $(#[$attrs])?
        impl<'cx, $($ty,)*> FromArgs<'cx> for ($($ty,)*)
        where
            $($ty: TryFromJs<'cx>,)*
        {}

        #[allow(non_snake_case)]
        impl<'cx, $($ty,)*> private::FromArgsInternal<'cx> for ($($ty,)*)
        where
            $($ty: TryFromJs<'cx>,)*
        {
            fn from_args(cx: &mut FunctionContext<'cx>) -> NeonResult<Self> {
                let [$($ty,)*] = cx.argv();

                Ok(($($ty::from_js(cx, $ty)?,)*))
            }

            fn from_args_opt(cx: &mut FunctionContext<'cx>) -> NeonResult<Option<Self>> {
                let [$($ty,)*] = cx.argv();

                Ok(Some((
                    $(match $ty::try_from_js(cx, $ty)? {
                        Ok(v) => v,
                        Err(_) => return Ok(None),
                    },)*
                )))
            }
        }
    }
}

macro_rules! from_args_expand {
    ($(#[$attrs:meta])? [$($head:ident),*], []) => {};

    ($(#[$attrs:meta])? [$($head:ident),*], [$cur:ident $(, $tail:ident)*]) => {
        from_args_impl!($(#[$attrs])? [$($head,)* $cur]);
        from_args_expand!($(#[$attrs])? [$($head,)* $cur], [$($tail),*]);
    };
}

macro_rules! from_args {
    ([$($show:ident),*], [$($hide:ident),*]) => {
        from_args_expand!([], [$($show),*]);
        from_args_expand!(#[doc(hidden)] [$($show),*], [$($hide),*]);
    };
}

// Implement `FromArgs` for tuples up to length `32`. The first list is included
// in docs and the second list is `#[doc(hidden)]`.
from_args!(
    [T1, T2, T3, T4, T5, T6, T7, T8],
    [
        T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26,
        T27, T28, T29, T30, T31, T32
    ]
);
