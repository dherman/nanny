use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use neon_runtime::raw::Env;
use neon_runtime::tsfn::ThreadsafeFunction;

use crate::context::{Context, TaskContext};
use crate::result::NeonResult;

type Callback = Box<dyn FnOnce(Env) + Send + 'static>;

/// Channel for scheduling Rust closures to execute on the JavaScript main thread.
///
/// Cloning a `Channel` will create a new channel that shares a backing queue for
/// events.
///
/// # Example
///
/// The following example spawns a standard Rust thread to complete a computation
/// and calls back to a JavaScript function asynchronously with the result.
///
/// ```
/// # use neon::prelude::*;
/// # fn fibonacci(_: f64) -> f64 { todo!() }
/// fn async_fibonacci(mut cx: FunctionContext) -> JsResult<JsUndefined> {
///     // These types (`f64`, `Root<JsFunction>`, `Channel`) may all be sent
///     // across threads.
///     let n = cx.argument::<JsNumber>(0)?.value(&mut cx);
///     let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
///     let channel = cx.channel();
///
///     // Spawn a thread to complete the execution. This will _not_ block the
///     // JavaScript event loop.
///     std::thread::spawn(move || {
///         let result = fibonacci(n);
///
///         // Send a closure as a task to be executed by the JavaScript event
///         // loop. This _will_ block the event loop while executing.
///         channel.send(move |mut cx| {
///             let callback = callback.into_inner(&mut cx);
///             let this = cx.undefined();
///             let null = cx.null();
///             let args = vec![
///                 cx.null().upcast::<JsValue>(),
///                 cx.number(result).upcast(),
///             ];
///
///             callback.call(&mut cx, this, args)?;
///
///             Ok(())
///         });
///     });
///
///     Ok(cx.undefined())
/// }
/// ```

pub struct Channel {
    state: Arc<ChannelState>,
    has_ref: bool,
}

impl std::fmt::Debug for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Channel")
    }
}

impl Channel {
    /// Creates an unbounded channel for scheduling closures on the JavaScript
    /// main thread
    pub fn new<'a, C: Context<'a>>(cx: &mut C) -> Self {
        Self {
            state: Arc::new(ChannelState::new(cx)),
            has_ref: true,
        }
    }

    /// Allow the Node event loop to exit while this `Channel` exists.
    /// _Idempotent_
    pub fn unref<'a, C: Context<'a>>(&mut self, cx: &mut C) -> &mut Self {
        // Already unreferenced
        if !self.has_ref {
            return self;
        }

        self.has_ref = false;
        self.state.unref(cx);
        self
    }

    /// Prevent the Node event loop from exiting while this `Channel` exists. (Default)
    /// _Idempotent_
    pub fn reference<'a, C: Context<'a>>(&mut self, cx: &mut C) -> &mut Self {
        // Already referenced
        if self.has_ref {
            return self;
        }

        self.has_ref = true;
        self.state.reference(cx);
        self
    }

    /// Schedules a closure to execute on the JavaScript thread that created this Channel
    /// Panics if there is a libuv error
    pub fn send<F>(&self, f: F)
    where
        F: FnOnce(TaskContext) -> NeonResult<()> + Send + 'static,
    {
        self.try_send(f).unwrap()
    }

    /// Schedules a closure to execute on the JavaScript thread that created this Channel
    /// Returns an `Error` if the task could not be scheduled.
    pub fn try_send<F>(&self, f: F) -> Result<(), SendError>
    where
        F: FnOnce(TaskContext) -> NeonResult<()> + Send + 'static,
    {
        let callback = Box::new(move |env| {
            let env = unsafe { std::mem::transmute(env) };

            // Note: It is sufficient to use `TaskContext`'s `InheritedHandleScope` because
            // N-API creates a `HandleScope` before calling the callback.
            TaskContext::with_context(env, move |cx| {
                let _ = f(cx);
            });
        });

        self.state.tsfn.call(callback, None).map_err(|_| SendError)
    }

    /// Returns a boolean indicating if this `Channel` will prevent the Node event
    /// loop from exiting.
    pub fn has_ref(&self) -> bool {
        self.has_ref
    }
}

impl Clone for Channel {
    /// Returns a clone of the Channel instance that shares the internal
    /// unbounded queue with the original channel. Scheduling callbacks on the
    /// same queue is faster than using separate channels, but might lead to
    /// starvation if one of the threads posts significantly more callbacks on
    /// the channel than the other one.
    ///
    /// Cloned and referenced Channel instances might trigger additional
    /// event-loop tick when dropped. Channel can be wrapped into an Arc and
    /// shared between different threads/callers to avoid this.
    fn clone(&self) -> Self {
        // Not referenced, we can simply clone the fields
        if !self.has_ref {
            return Self {
                state: self.state.clone(),
                has_ref: false,
            };
        }

        let state = Arc::clone(&self.state);

        // Only need to increase the ref count since the tsfn is already referenced
        state.ref_count.fetch_add(1, Ordering::Relaxed);

        Self {
            state,
            has_ref: true,
        }
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        // Not a referenced event queue
        if !self.has_ref {
            return;
        }

        // It was only us who kept the `ChannelState` alive. No need to unref
        // the `tsfn`, because it is going to be dropped once this function
        // returns.
        if Arc::strong_count(&self.state) == 1 {
            return;
        }

        // The ChannelState is dropped on a worker thread. We have to `unref`
        // the tsfn on the UV thread after all pending closures. Note that in
        // the most of scenarios the optimization in N-API layer would coalesce
        // `send()` with a user-supplied closure and the unref send here into a
        // single UV tick.
        //
        // If this ever has to be optimized a second `Arc` could be used to wrap
        // the `state` and it could be cloned in `try_send` and unref'ed on the
        // UV thread if strong reference count goes to 0.
        let state = Arc::clone(&self.state);

        self.send(move |mut cx| {
            state.unref(&mut cx);
            Ok(())
        });
    }
}

/// Error indicating that a closure was unable to be scheduled to execute on the event loop.
pub struct SendError;

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SendError")
    }
}

impl std::fmt::Debug for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::error::Error for SendError {}

struct ChannelState {
    tsfn: ThreadsafeFunction<Callback>,
    ref_count: AtomicUsize,
}

impl ChannelState {
    fn new<'a, C: Context<'a>>(cx: &mut C) -> Self {
        let tsfn = unsafe { ThreadsafeFunction::new(cx.env().to_raw(), Self::callback) };
        Self {
            tsfn,
            ref_count: AtomicUsize::new(1),
        }
    }

    fn reference<'a, C: Context<'a>>(&self, cx: &mut C) {
        // We can use relaxed ordering because `reference()` can only be called
        // on the Event-Loop thread.
        if self.ref_count.fetch_add(1, Ordering::Relaxed) != 0 {
            return;
        }

        unsafe {
            self.tsfn.reference(cx.env().to_raw());
        }
    }

    fn unref<'a, C: Context<'a>>(&self, cx: &mut C) {
        // We can use relaxed ordering because `unref()` can only be called
        // on the Event-Loop thread.
        if self.ref_count.fetch_sub(1, Ordering::Relaxed) != 1 {
            return;
        }

        unsafe {
            self.tsfn.unref(cx.env().to_raw());
        }
    }

    // Monomorphized trampoline funciton for calling the user provided closure
    fn callback(env: Option<Env>, callback: Callback) {
        if let Some(env) = env {
            callback(env);
        } else {
            crate::context::internal::IS_RUNNING.with(|v| {
                *v.borrow_mut() = false;
            });
        }
    }
}
