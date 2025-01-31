//! Types and traits for working with JavaScript functions.

use smallvec::smallvec;

use crate::{
    context::{Context, Cx},
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult},
    types::{
        extract::{TryFromJs, TryIntoJs},
        private::ValueInternal,
        JsFunction, JsObject, JsValue, Value,
    },
};

pub(crate) mod private;

/// A builder for making a JavaScript function call like `parseInt("42")`.
///
/// The builder methods make it convenient to assemble the call from parts:
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsNumber> {
/// # let parse_int: Handle<JsFunction> = cx.global("parseInt")?;
/// let x: f64 = parse_int
///     .bind(&mut cx)
///     .arg("42")?
///     .call()?;
/// # Ok(cx.number(x))
/// # }
/// ```
pub struct BindOptions<'a, 'cx: 'a> {
    pub(crate) cx: &'a mut Cx<'cx>,
    pub(crate) callee: Handle<'cx, JsValue>,
    pub(crate) this: Option<Handle<'cx, JsValue>>,
    pub(crate) args: private::ArgsVec<'cx>,
}

impl<'a, 'cx: 'a> BindOptions<'a, 'cx> {
    /// Set the value of `this` for the function call.
    pub fn this<T: TryIntoJs<'cx>>(&mut self, this: T) -> NeonResult<&mut Self> {
        let v = this.try_into_js(self.cx)?;
        self.this = Some(v.upcast());
        Ok(self)
    }

    /// Replaces the arguments list with the given arguments.
    pub fn args<A: TryIntoArguments<'cx>>(&mut self, a: A) -> NeonResult<&mut Self> {
        self.args = a.try_into_args_vec(self.cx)?;
        Ok(self)
    }

    /// Replaces the arguments list with a list computed from a closure.
    pub fn args_with<R, F>(&mut self, f: F) -> NeonResult<&mut Self>
    where
        R: TryIntoArguments<'cx>,
        F: FnOnce(&mut Cx<'cx>) -> R,
    {
        self.args = f(self.cx).try_into_args_vec(self.cx)?;
        Ok(self)
    }

    /// Add an argument to the arguments list.
    pub fn arg<A: TryIntoJs<'cx>>(&mut self, a: A) -> NeonResult<&mut Self> {
        let v = a.try_into_js(self.cx)?;
        self.args.push(v.upcast());
        Ok(self)
    }

    /// Add an argument to the arguments list, computed from a closure.
    pub fn arg_with<R, F>(&mut self, f: F) -> NeonResult<&mut Self>
    where
        R: TryIntoJs<'cx>,
        F: FnOnce(&mut Cx<'cx>) -> R,
    {
        let v = f(self.cx).try_into_js(self.cx)?;
        self.args.push(v.upcast());
        Ok(self)
    }

    /// Make the function call. If the function returns without throwing, the result value
    /// is converted to a Rust value with `TryFromJs::from_js`.
    pub fn call<R: TryFromJs<'cx>>(&mut self) -> NeonResult<R> {
        let this = self.this.unwrap_or_else(|| self.cx.undefined().upcast());
        let v: Handle<JsValue> = unsafe { self.callee.try_call(self.cx, this, &self.args)? };
        R::from_js(self.cx, v)
    }

    /// Make the function call as a constructor. If the function returns without throwing, the
    /// result value is converted to a Rust value with `TryFromJs::from_js`.
    pub fn construct<R: TryFromJs<'cx>>(&mut self) -> NeonResult<R> {
        let v: Handle<JsValue> = unsafe { self.callee.try_construct(self.cx, &self.args)? };
        R::from_js(self.cx, v)
    }

    /// Make the function call for side effect, discarding the result value. This method is
    /// preferable to [`call()`](BindOptions::call) when the result value isn't needed,
    /// since it doesn't require specifying a result type.
    pub fn exec(&mut self) -> NeonResult<()> {
        let _ignore: Handle<JsValue> = self.call()?;
        Ok(())
    }
}

/// A builder for making a JavaScript function call like `parseInt("42")`.
///
/// The builder methods make it convenient to assemble the call from parts:
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsNumber> {
/// # let parse_int: Handle<JsFunction> = cx.global("parseInt")?;
/// let x: Handle<JsNumber> = parse_int
///     .call_with(&cx)
///     .arg(cx.string("42"))
///     .apply(&mut cx)?;
/// # Ok(x)
/// # }
/// ```
#[deprecated(since = "TBD", note = "use `JsFunction::bind()` instead")]
#[derive(Clone)]
pub struct CallOptions<'a> {
    pub(crate) callee: Handle<'a, JsFunction>,
    pub(crate) this: Option<Handle<'a, JsValue>>,
    pub(crate) args: private::ArgsVec<'a>,
}

impl<'a> CallOptions<'a> {
    /// Set the value of `this` for the function call.
    pub fn this<V: Value>(&mut self, this: Handle<'a, V>) -> &mut Self {
        self.this = Some(this.upcast());
        self
    }

    /// Add an argument to the arguments list.
    pub fn arg<V: Value>(&mut self, arg: Handle<'a, V>) -> &mut Self {
        self.args.push(arg.upcast());
        self
    }

    /// Replaces the arguments list with the given arguments.
    pub fn args<A: Arguments<'a>>(&mut self, args: A) -> &mut Self {
        self.args = args.into_args_vec();
        self
    }

    /// Make the function call. If the function returns without throwing, the result value
    /// is downcast to the type `V`, throwing a `TypeError` if the downcast fails.
    pub fn apply<'b: 'a, V: Value, C: Context<'b>>(&self, cx: &mut C) -> JsResult<'b, V> {
        let this = self.this.unwrap_or_else(|| cx.undefined().upcast());
        let v: Handle<JsValue> = self.callee.call(cx, this, &self.args)?;
        v.downcast_or_throw(cx)
    }

    /// Make the function call for side effect, discarding the result value. This method is
    /// preferable to [`apply()`](CallOptions::apply) when the result value isn't needed,
    /// since it doesn't require specifying a result type.
    pub fn exec<'b: 'a, C: Context<'b>>(&self, cx: &mut C) -> NeonResult<()> {
        let this = self.this.unwrap_or_else(|| cx.undefined().upcast());
        self.callee.call(cx, this, &self.args)?;
        Ok(())
    }
}

/// A builder for making a JavaScript constructor call like `new Array(16)`.
///
/// The builder methods make it convenient to assemble the call from parts:
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsObject> {
/// # let url: Handle<JsFunction> = cx.global("URL")?;
/// let obj = url
///     .construct_with(&cx)
///     .arg(cx.string("https://neon-bindings.com"))
///     .apply(&mut cx)?;
/// # Ok(obj)
/// # }
/// ```
#[deprecated(since = "TBD", note = "use `JsFunction::bind()` instead")]
#[derive(Clone)]
pub struct ConstructOptions<'a> {
    pub(crate) callee: Handle<'a, JsFunction>,
    pub(crate) args: private::ArgsVec<'a>,
}

impl<'a> ConstructOptions<'a> {
    /// Add an argument to the arguments list.
    pub fn arg<V: Value>(&mut self, arg: Handle<'a, V>) -> &mut Self {
        self.args.push(arg.upcast());
        self
    }

    /// Replaces the arguments list with the given arguments.
    pub fn args<A: Arguments<'a>>(&mut self, args: A) -> &mut Self {
        self.args = args.into_args_vec();
        self
    }

    /// Make the constructor call. If the function returns without throwing, returns
    /// the resulting object.
    pub fn apply<'b: 'a, O: Object, C: Context<'b>>(&self, cx: &mut C) -> JsResult<'b, O> {
        let v: Handle<JsObject> = self.callee.construct(cx, &self.args)?;
        v.downcast_or_throw(cx)
    }
}

/// The trait for specifying values to be converted into arguments for a function call.
/// This trait is sealed and cannot be implemented by types outside of the Neon crate.
///
/// **Note:** This trait is implemented for tuples of up to 32 JavaScript values,
/// but for the sake of brevity, only tuples up to size 8 are shown in this documentation.
pub trait TryIntoArguments<'cx>: private::TryIntoArgumentsInternal<'cx> {}

impl<'cx> private::TryIntoArgumentsInternal<'cx> for () {
    fn try_into_args_vec(self, _cx: &mut Cx<'cx>) -> NeonResult<private::ArgsVec<'cx>> {
        Ok(smallvec![])
    }
}

impl<'cx, T, E> private::TryIntoArgumentsInternal<'cx> for Result<T, E>
where
    T: private::TryIntoArgumentsInternal<'cx>,
    E: TryIntoJs<'cx>,
{
    fn try_into_args_vec(self, cx: &mut Cx<'cx>) -> NeonResult<private::ArgsVec<'cx>> {
        match self {
            Ok(v) => v.try_into_args_vec(cx),
            Err(err) => err.try_into_js(cx).and_then(|err| cx.throw(err)),
        }
    }
}

impl<'cx, T, E> TryIntoArguments<'cx> for Result<T, E>
where
    T: TryIntoArguments<'cx>,
    E: TryIntoJs<'cx>,
{
}

macro_rules! impl_into_arguments_expand {
    {
        $(#[$attrs:meta])?
        [ $($prefix:ident ),* ];
        [];
    } => {};

    {
        $(#[$attrs:meta])?
        [ $($prefix:ident),* ];
        [ $head:ident $(, $tail:ident)* ];
    } => {
        $(#[$attrs])?
        impl<'cx, $($prefix: TryIntoJs<'cx> + 'cx, )* $head: TryIntoJs<'cx> + 'cx> private::TryIntoArgumentsInternal<'cx> for ($($prefix, )* $head, ) {
            #[allow(non_snake_case)]
            fn try_into_args_vec(self, cx: &mut Cx<'cx>) -> NeonResult<private::ArgsVec<'cx>> {
                let ($($prefix, )* $head, ) = self;
                Ok(smallvec![ $($prefix.try_into_js(cx)?.upcast(),)* $head.try_into_js(cx)?.upcast() ])
             }
         }

         $(#[$attrs])?
         impl<'cx, $($prefix: TryIntoJs<'cx> + 'cx, )* $head: TryIntoJs<'cx> + 'cx> TryIntoArguments<'cx> for ($($prefix, )* $head, ) {}

        impl_into_arguments_expand! {
            $(#[$attrs])?
            [ $($prefix, )* $head ];
            [ $($tail),* ];
        }
   }
}

macro_rules! impl_into_arguments {
    {
        [ $($show:ident),* ];
        [ $($hide:ident),* ];
    } => {
        impl_into_arguments_expand! { []; [ $($show),* ]; }
        impl_into_arguments_expand! { #[doc(hidden)] [ $($show),* ]; [ $($hide),* ]; }
    }
}

impl_into_arguments! {
    // Tuples up to length 8 are included in the docs.
    [V1, V2, V3, V4, V5, V6, V7, V8];

    // Tuples up to length 32 are not included in the docs.
    [
        V9, V10, V11, V12, V13, V14, V15, V16,
        V17, V18, V19, V20, V21, V22, V23, V24,
        V25, V26, V27, V28, V29, V30, V31, V32
    ];
}

/// The trait for specifying arguments for a function call. This trait is sealed and cannot
/// be implemented by types outside of the Neon crate.
///
/// **Note:** This trait is implemented for tuples of up to 32 JavaScript values,
/// but for the sake of brevity, only tuples up to size 8 are shown in this documentation.
pub trait Arguments<'a>: private::ArgumentsInternal<'a> {}

impl<'a> private::ArgumentsInternal<'a> for () {
    fn into_args_vec(self) -> private::ArgsVec<'a> {
        smallvec![]
    }
}

impl<'a> Arguments<'a> for () {}

macro_rules! impl_arguments_expand {
    {
        $(#[$attrs:meta])?
        [ $($prefix:ident),* ];
        [];
    } => {};

    {
        $(#[$attrs:meta])?
        [ $($prefix:ident),* ];
        [ $head:ident $(, $tail:ident)* ];
    } => {
        $(#[$attrs])?
        impl<'a, $($prefix: Value, )* $head: Value> private::ArgumentsInternal<'a> for ($(Handle<'a, $prefix>, )* Handle<'a, $head>, ) {
            #[allow(non_snake_case)]
            fn into_args_vec(self) -> private::ArgsVec<'a> {
                let ($($prefix, )* $head, ) = self;
                smallvec![$($prefix.upcast(),)* $head.upcast()]
             }
         }

         $(#[$attrs])?
         impl<'a, $($prefix: Value, )* $head: Value> Arguments<'a> for ($(Handle<'a, $prefix>, )* Handle<'a, $head>, ) {}

         impl_arguments_expand! {
            $(#[$attrs])?
            [ $($prefix, )* $head ];
            [ $($tail),* ];
         }
    };
}

macro_rules! impl_arguments {
    {
        [ $($show:ident),* ];
        [ $($hide:ident),* ];
    } => {
        impl_arguments_expand! { []; [ $($show),* ]; }
        impl_arguments_expand! { #[doc(hidden)] [ $($show),* ]; [ $($hide),* ]; }
    }
}

impl_arguments! {
    // Tuples up to length 8 are included in the docs.
    [V1, V2, V3, V4, V5, V6, V7, V8];

    // Tuples up to length 32 are not included in the docs.
    [
        V9, V10, V11, V12, V13, V14, V15, V16,
        V17, V18, V19, V20, V21, V22, V23, V24,
        V25, V26, V27, V28, V29, V30, V31, V32
    ];
}
