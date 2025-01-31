//! Helper module to add documentation to macros prior to re-exporting.

/// Marks a function as the main entry point for initialization in
/// a Neon module.
///
/// This attribute should only be used _once_ in a module and will
/// be called each time the module is initialized in a context.
///
/// If a `main` function is not provided, all registered exports will be exported. If
/// the `tokio` feature flag is enabled, a multithreaded tokio runtime will also be
/// registered globally.
///
/// ```
/// # use neon::prelude::*;
/// # fn main() {
/// #[neon::main]
/// fn main(mut cx: ModuleContext) -> NeonResult<()> {
///     // Export all registered exports
///     neon::registered().export(&mut cx)?;
///
///     let version = cx.string("1.0.0");
///
///     cx.export_value("version", version)?;
///
///     Ok(())
/// }
/// # }
/// ```
pub use neon_macros::main;

/// Register an item to be exported by the Neon addon
///
/// ## Exporting constants and statics
///
/// ```
/// #[neon::export]
/// static GREETING: &str = "Hello, Neon!";
///
/// #[neon::export]
/// const ANSWER: u8 = 42;
/// ```
///
/// ### Renaming an export
///
/// By default, items will be exported with their Rust name. Exports may
/// be renamed by providing the `name` attribute.
///
/// ```
/// #[neon::export(name = "myGreeting")]
/// static GREETING: &str = "Hello, Neon!";
/// ```
///
/// ### JSON exports
///
/// Complex values may be exported by automatically serializing to JSON and
/// parsing in JavaScript. Any type that implements `serde::Serialize` may be used.
///
/// ```
/// #[neon::export(json)]
/// static MESSAGES: &[&str] = &["hello", "goodbye"];
/// ```
///
/// ## Exporting functions
///
/// Functions may take any type that implements [`TryFromJs`](crate::types::extract::TryFromJs) as
/// an argument and return any type that implements [`TryIntoJs`](crate::types::extract::TryIntoJs).
///
/// ```
/// #[neon::export]
/// fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
/// ```
///
/// ### Naming exported functions
///
/// Conventionally, Rust uses `snake_case` for function identifiers and JavaScript uses `camelCase`.
/// By default, Neon will attempt to convert function names to camel case. For example:
///
/// ```rust
/// #[neon::export]
/// fn add_one(n: f64) -> f64 {
///     n + 1.0
/// }
/// ```
///
/// The `add_one` function will be exported as `addOne` in JavaScript.
///
/// ```js
/// import { addOne } from ".";
/// ```
///
/// [Similar to globals](#renaming-an-export), exported functions can be overridden with the `name`
/// attribute.
///
/// ```rust
/// #[neon::export(name = "addOneSync")]
/// fn add_one(n: f64) -> f64 {
///     n + 1.0
/// }
/// ```
/// Neon uses the following rules when converting `snake_case` to `camelCase`:
///
/// * All _leading_ and _trailing_ underscores (`_`) are preserved
/// * Characters _immediately_ following a _non-leading_ underscore are converted to uppercase
/// * If the identifier contains an _unexpected_ character, **no** conversion is performed and
///   the identifier is used _unchanged_. Unexpected characters include:
///   - Uppercase characters
///   - Duplicate _interior_ (non-leading, non-trailing underscores)
///
/// ### Exporting a function that uses JSON
///
/// The [`Json`](crate::types::extract::Json) wrapper allows ergonomically handling complex
/// types that implement `serde::Deserialize` and `serde::Serialize`.
///
/// ```
/// # use neon::types::extract::Json;
/// #[neon::export]
/// fn sort(Json(mut items): Json<Vec<String>>) -> Json<Vec<String>> {
///     items.sort();
///     Json(items)
/// }
/// ```
///
/// As a convenience, macro uses may add the `json` attribute to automatically
/// wrap arguments and return values with `Json`.
///
/// ```
/// #[neon::export(json)]
/// fn sort(mut items: Vec<String>) -> Vec<String> {
///     items.sort();
///     items
/// }
/// ```
///
/// ### Tasks
///
/// Neon provides an API for spawning tasks to execute asynchronously on Node's worker
/// pool. JavaScript may await a promise for completion of the task.
///
/// ```
/// # use neon::prelude::*;
/// #[neon::export]
/// fn add<'cx>(cx: &mut FunctionContext<'cx>, a: f64, b: f64) -> JsResult<'cx, JsPromise> {
///     let promise = cx
///         .task(move || a + b)
///         .promise(|mut cx, res| Ok(cx.number(res)));
///
///     Ok(promise)
/// }
/// ```
///
/// As a convenience, macro users may indicate that a function should be executed
/// asynchronously on the worker pool by adding the `task` attribute.
///
/// ```
/// #[neon::export(task)]
/// fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
/// ```
///
/// ### Async Functions
///
/// The [`export`] macro can export `async fn`, converting to a JavaScript `Promise`, if a global
/// future executor is registered. See [`neon::set_global_executor`](crate::set_global_executor) for
/// more details.
///
/// ```
/// # #[cfg(all(feature = "napi-6", feature = "futures"))]
/// # {
/// #[neon::export]
/// async fn add(a: f64, b: f64) -> f64 {
///     a + b
/// }
/// # }
/// ```
///
/// A `fn` that returns a [`Future`](std::future::Future) can be annotated with `#[neon::export(async)]`
/// if it needs to perform some setup on the JavaScript main thread before running asynchronously.
///
/// ```
/// # #[cfg(all(feature = "napi-6", feature = "futures"))]
/// # {
/// # use std::future::Future;
/// # use neon::prelude::*;
/// #[neon::export(async)]
/// fn add(a: f64, b: f64) -> impl Future<Output = f64> {
///     println!("Hello from the JavaScript main thread!");
///
///     async move {
///         a + b
///     }
/// }
/// # }
/// ```
///
/// If work needs to be performed on the JavaScript main thread _after_ the asynchronous operation,
/// the [`With`](crate::types::extract::With) extractor can be used to execute a closure before returning.
///
/// ```
/// # #[cfg(all(feature = "napi-6", feature = "futures"))]
/// # {
/// # use neon::types::extract::{self, TryIntoJs};
/// #[neon::export]
/// async fn add(a: f64, b: f64) -> impl for<'cx> TryIntoJs<'cx> {
///     let sum = a + b;
///
///     extract::with(move |cx| {
///         println!("Hello from the JavaScript main thread!");
///
///         sum.try_into_js(cx)
///     })
/// }
/// # }
/// ```
///
/// ### Error Handling
///
/// If an exported function returns a [`Result`], a JavaScript exception will be thrown
/// with the [`Err`]. Any error type that implements [`TryIntoJs`](crate::types::extract::TryIntoJs)
/// may be used.
///
/// ```
/// #[neon::export]
/// fn throw(msg: String) -> Result<(), String> {
///     Err(msg)
/// }
/// ```
///
/// The [`Error`](crate::types::extract::Error) type is provided for ergonomic error conversions
/// from most error types using the `?` operator.
///
/// ```
/// use neon::types::extract::Error;
///
/// #[neon::export]
/// fn read_file(path: String) -> Result<String, Error> {
///     let contents = std::fs::read_to_string(path)?;
///     Ok(contents)
/// }
/// ```
///
/// ### Interact with the JavaScript runtime
///
/// More complex functions may need to interact directly with the JavaScript runtime,
/// for example with [`Context`](crate::context::Context) or handles to JavaScript values.
///
/// Functions may optionally include a [`Cx`](crate::context::Cx) or
/// [`FunctionContext`](crate::context::FunctionContext) argument. Note that unlike functions
/// created with [`JsFunction::new`](crate::types::JsFunction), exported function receive a borrowed
/// context and may require explicit lifetimes.
///
/// ```
/// # use neon::prelude::*;
/// #[neon::export]
/// fn add<'cx>(
///     cx: &mut Cx<'cx>,
///     a: Handle<JsNumber>,
///     b: Handle<JsNumber>,
/// ) -> JsResult<'cx, JsNumber> {
///     let a = a.value(cx);
///     let b = b.value(cx);
///
///     Ok(cx.number(a + b))
/// }
/// ```
///
/// ### Advanced
///
/// The following attributes are for advanced configuration and may not be
/// necessary for most users.
///
/// #### `context`
///
/// The `#[neon::export]` uses a heuristic to determine if the first argument
/// to a function is a _context_ argument.
///
/// * In a function executed on the JavaScript main thread, it looks for `&mut Cx`
///     or `&mut FunctionContext` to determine if the [`Context`](crate::context::Context)
///     should be passed.
/// * In a function executed on another thread, it looks for [`Channel`](crate::event::Channel).
///
/// If the type has been renamed when importing, the `context` attribute can be
/// added to force it to be passed.
///
/// ```
/// use neon::event::Channel as Ch;
/// use neon::context::FunctionContext as FnCtx;
///
/// #[neon::export(context)]
/// fn add(_cx: &mut FnCtx, a: f64, b: f64) -> f64 {
///     a + b
/// }
///
/// #[neon::export(context)]
/// async fn div(_ch: Ch, a: f64, b: f64) -> f64 {
///     a / b
/// }
/// ```
///
/// #### `this`
///
/// The `#[neon::export]` uses a heuristic to determine if an argument to this function is
/// referring to [`this`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/this).
///
/// 1. If the first argument is a [context](#context), use the 0th argument, otherwise use the 1st.
/// 2. If the argument binding is named `this`
/// 3. Or if it is a tuple struct pattern with an element named `this`
///
/// ```
/// use neon::types::extract::Boxed;
///
/// #[neon::export]
/// fn buffer_clone(this: Vec<u8>) -> Vec<u8> {
///     this
/// }
///
/// #[neon::export]
/// fn box_to_string(Boxed(this): Boxed<String>) -> String {
///     this
/// }
/// ```
///
/// If the function uses a variable name other than `this`, the `this` attribute may
/// be added.
///
/// ```
/// #[neon::export(this)]
/// fn buffer_clone(me: Vec<u8>) -> Vec<u8> {
///     me
/// }
/// ```
pub use neon_macros::export;
