//! Thread-local storage for JavaScript threads.
//!
//! At runtime, an instance of a Node.js addon can contain its own local storage,
//! which can then be shared and accessed as needed from Rust in a Neon module. This can
//! be useful for setting up long-lived state that needs to be shared between calls
//! of an addon's APIs.
//!
//! For example, an addon may wish to track the [thread ID][threadId] of each of its
//! instances:
//!
//! ```
//! # use neon::prelude::*;
//! # use neon::thread::LocalKey;
//! static THREAD_ID: LocalKey<u32> = LocalKey::new();
//!
//! pub fn thread_id(cx: &mut Cx) -> NeonResult<u32> {
//!     THREAD_ID.get_or_try_init(cx, |cx| {
//!         let require: Handle<JsFunction> = cx.global("require")?;
//!         let worker: Handle<JsObject> = require
//!             .bind(cx)
//!             .arg("node:worker_threads")?
//!             .call()?;
//!         let thread_id: f64 = worker.prop(cx, "threadId").get()?;
//!         Ok(thread_id as u32)
//!     }).cloned()
//! }
//! ```
//!
//! ### The Addon Lifecycle
//!
//! For some use cases, a single shared global constant stored in a `static` variable
//! might be sufficient:
//!
//! ```
//! static MY_CONSTANT: &'static str = "hello Neon";
//! ```
//!
//! This variable will be allocated when the addon is first loaded into the Node.js
//! process. This works fine for single-threaded applications, or global thread-safe
//! data.
//!
//! However, since the addition of [worker threads][workers] in Node v10,
//! modules can be instantiated multiple times in a single Node process. So even
//! though the dynamically-loaded binary library (i.e., the Rust implementation of
//! the addon) is only loaded once in the running process, its [`#[main]`](crate::main)
//! function can be executed multiple times with distinct module objects, one per application
//! thread:
//!
//! ![The Node.js addon lifecycle, described in detail below.][lifecycle]
//!
//! This means that any thread-local data needs to be initialized separately for each
//! instance of the addon. This module provides a simple container type, [`LocalKey`],
//! for allocating and initializing thread-local data. (Technically, this data is stored in the
//! addon's [module instance][environment], which is equivalent to being thread-local.)
//!
//! A common example is when an addon needs to maintain a reference to a JavaScript value. A
//! reference can be [rooted](crate::handle::Root) and stored in a static, but references cannot
//! be used across separate threads. By placing the reference in thread-local storage, an
//! addon can ensure that each thread stores its own distinct reference:
//!
//! ```
//! # use neon::prelude::*;
//! # use neon::thread::LocalKey;
//! # fn initialize_my_datatype<'cx, C: Context<'cx>>(cx: &mut C) -> JsResult<'cx, JsFunction> { unimplemented!() }
//! static MY_CONSTRUCTOR: LocalKey<Root<JsFunction>> = LocalKey::new();
//!
//! pub fn my_constructor<'cx, C: Context<'cx>>(cx: &mut C) -> JsResult<'cx, JsFunction> {
//!     let constructor = MY_CONSTRUCTOR.get_or_try_init(cx, |cx| {
//!         let constructor: Handle<JsFunction> = initialize_my_datatype(cx)?;
//!         Ok(constructor.root(cx))
//!     })?;
//!     Ok(constructor.to_inner(cx))
//! }
//! ```
//!
//! Notice that if this code were implemented without a `LocalKey`, it would panic whenever
//! one thread stores an instance of the constructor and a different thread attempts to
//! access it with the call to [`to_inner()`](crate::handle::Root::to_inner).
//!
//! ### When to Use Thread-Local Storage
//!
//! Single-threaded applications don't generally need to worry about thread-local data.
//! There are two cases where Neon apps should consider storing static data in a
//! `LocalKey` storage cell:
//!
//! - **Multi-threaded applications:** If your Node application uses the `Worker`
//!   API, you'll want to store any static data that might get access from multiple
//!   threads in thread-local data.
//! - **Libraries:** If your addon is part of a library that could be used by multiple
//!   applications, you'll want to store static data in thread-local data in case the
//!   addon ends up instantiated by multiple threads in some future application.
//!
//! ### Why Not Use Standard TLS?
//!
//! Since the JavaScript engine may not tie JavaScript threads 1:1 to system threads,
//! it is recommended to use this module instead of the Rust standard thread-local storage
//! when associating data with a JavaScript thread.
//!
//! [environment]: https://nodejs.org/api/n-api.html#environment-life-cycle-apis
//! [lifecycle]: https://raw.githubusercontent.com/neon-bindings/neon/main/doc/lifecycle.png
//! [workers]: https://nodejs.org/api/worker_threads.html
//! [threadId]: https://nodejs.org/api/worker_threads.html#workerthreadid

use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use once_cell::sync::OnceCell;

use crate::context::Context;
use crate::lifecycle::LocalCell;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn next_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// A JavaScript thread-local container that owns its contents, similar to
/// [`std::thread::LocalKey`] but tied to a JavaScript thread rather
/// than a system thread.
///
/// ### Initialization and Destruction
///
/// Initialization is dynamically performed on the first call to one of the `init` methods
/// of `LocalKey`, and values that implement [`Drop`] get destructed when
/// the JavaScript thread exits, i.e. when a worker thread terminates or the main thread
/// terminates on process exit.
#[derive(Default)]
pub struct LocalKey<T> {
    _type: PhantomData<T>,
    id: OnceCell<usize>,
}

impl<T> LocalKey<T> {
    /// Creates a new local value. This method is `const`, so it can be assigned to
    /// static variables.
    pub const fn new() -> Self {
        Self {
            _type: PhantomData,
            id: OnceCell::new(),
        }
    }

    fn id(&self) -> usize {
        *self.id.get_or_init(next_id)
    }
}

impl<T: Any + Send + 'static> LocalKey<T> {
    /// Gets the current value of the cell. Returns `None` if the cell has not
    /// yet been initialized.
    pub fn get<'cx, 'a, C>(&self, cx: &'a mut C) -> Option<&'cx T>
    where
        C: Context<'cx>,
    {
        // Unwrap safety: The type bound LocalKey<T> and the fact that every LocalKey has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: Option<&T> =
            LocalCell::get(cx, self.id()).map(|value| value.downcast_ref().unwrap());

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        unsafe { std::mem::transmute::<Option<&'a T>, Option<&'cx T>>(r) }
    }

    /// Gets the current value of the cell, initializing it with the result of
    /// calling `f` if it has not yet been initialized.
    pub fn get_or_init<'cx, 'a, C, F>(&self, cx: &'a mut C, f: F) -> &'cx T
    where
        C: Context<'cx>,
        F: FnOnce() -> T,
    {
        // Unwrap safety: The type bound LocalKey<T> and the fact that every LocalKey has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: &T = LocalCell::get_or_init(cx, self.id(), || Box::new(f()))
            .downcast_ref()
            .unwrap();

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        unsafe { std::mem::transmute::<&'a T, &'cx T>(r) }
    }

    /// Gets the current value of the cell, initializing it with the result of
    /// calling `f` if it has not yet been initialized. Returns `Err` if the
    /// callback triggers a JavaScript exception.
    ///
    /// # Panics
    ///
    /// During the execution of `f`, calling any methods on this `LocalKey` that
    /// attempt to initialize it will panic.
    pub fn get_or_try_init<'cx, 'a, C, E, F>(&self, cx: &'a mut C, f: F) -> Result<&'cx T, E>
    where
        C: Context<'cx>,
        F: FnOnce(&mut C) -> Result<T, E>,
    {
        // Unwrap safety: The type bound LocalKey<T> and the fact that every LocalKey has a unique
        // id guarantees that the cell is only ever assigned instances of type T.
        let r: &T = LocalCell::get_or_try_init(cx, self.id(), |cx| Ok(Box::new(f(cx)?)))?
            .downcast_ref()
            .unwrap();

        // Safety: Since the Box is immutable and heap-allocated, it's guaranteed not to
        // move or change for the duration of the context.
        Ok(unsafe { std::mem::transmute::<&'a T, &'cx T>(r) })
    }
}

impl<T: Any + Send + Default + 'static> LocalKey<T> {
    /// Gets the current value of the cell, initializing it with the default value
    /// if it has not yet been initialized.
    pub fn get_or_init_default<'cx, 'a, C>(&self, cx: &'a mut C) -> &'cx T
    where
        C: Context<'cx>,
    {
        self.get_or_init(cx, Default::default)
    }
}
