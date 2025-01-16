//! # Environment life cycle APIs
//!
//! These APIs map to the life cycle of a specific "Agent" or self-contained
//! environment. If a Neon module is loaded multiple times (Web Workers, worker
//! threads), these API will be handle data associated with a specific instance.
//!
//! See the [N-API Lifecycle][npai-docs] documentation for more details.
//!
//! [napi-docs]: https://nodejs.org/api/n-api.html#n_api_environment_life_cycle_apis

use std::{
    any::Any,
    marker::PhantomData,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use crate::{
    context::Context,
    event::Channel,
    handle::root::NapiRef,
    sys::{lifecycle, raw::Env, tsfn::ThreadsafeFunction},
    types::promise::NodeApiDeferred,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
/// Uniquely identifies an instance of the module
///
/// _Note_: Since `InstanceData` is created lazily, the order of `id` may not
/// reflect the order that instances were created.
pub(crate) struct InstanceId(u32);

impl InstanceId {
    fn next() -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);

        let next = NEXT_ID.fetch_add(1, Ordering::Relaxed).checked_add(1);
        match next {
            Some(id) => Self(id),
            None => panic!("u32 overflow ocurred in Lifecycle InstanceId"),
        }
    }
}

/// `InstanceData` holds Neon data associated with a particular instance of a
/// native module. If a module is loaded multiple times (e.g., worker threads), this
/// data will be unique per instance.
pub(crate) struct InstanceData {
    id: InstanceId,

    /// Used to free `Root` in the same JavaScript environment that created it
    ///
    /// _Design Note_: An `Arc` ensures the `ThreadsafeFunction` outlives the unloading
    /// of a module. Since it is unlikely that modules will be re-loaded frequently, this
    /// could be replaced with a leaked `&'static ThreadsafeFunction<NapiRef>`. However,
    /// given the cost of FFI, this optimization is omitted until the cost of an
    /// `Arc` is demonstrated as significant.
    drop_queue: Arc<ThreadsafeFunction<DropData>>,

    /// Shared `Channel` that is cloned to be returned by the `cx.channel()` method
    shared_channel: Channel,

    /// Table of user-defined instance-local cells.
    locals: LocalTable,
}

#[derive(Default)]
pub(crate) struct LocalTable {
    cells: Vec<LocalCell>,
}

pub(crate) type LocalCellValue = Box<dyn Any + Send + 'static>;

#[derive(Default)]
pub(crate) enum LocalCell {
    #[default]
    /// Uninitialized state.
    Uninit,
    /// Intermediate "dirty" state representing the middle of a `get_or_try_init` transaction.
    Trying,
    /// Fully initialized state.
    Init(LocalCellValue),
}

impl LocalCell {
    /// Establish the initial state at the beginning of the initialization protocol.
    /// This method ensures that re-entrant initialization always panics (i.e. when
    /// an existing `get_or_try_init` is in progress).
    fn pre_init<F>(&mut self, f: F)
    where
        F: FnOnce() -> LocalCell,
    {
        match self {
            LocalCell::Uninit => {
                *self = f();
            }
            LocalCell::Trying => panic!("attempt to reinitialize Local during initialization"),
            LocalCell::Init(_) => {}
        }
    }

    pub(crate) fn get<'cx, 'a, C>(cx: &'a mut C, id: usize) -> Option<&'a mut LocalCellValue>
    where
        C: Context<'cx>,
    {
        let cell = InstanceData::locals(cx).get(id);
        match cell {
            LocalCell::Init(ref mut b) => Some(b),
            _ => None,
        }
    }

    pub(crate) fn get_or_init<'cx, 'a, C, F>(
        cx: &'a mut C,
        id: usize,
        f: F,
    ) -> &'a mut LocalCellValue
    where
        C: Context<'cx>,
        F: FnOnce() -> LocalCellValue,
    {
        InstanceData::locals(cx)
            .get(id)
            .pre_init(|| LocalCell::Init(f()));

        LocalCell::get(cx, id).unwrap()
    }

    pub(crate) fn get_or_try_init<'cx, 'a, C, E, F>(
        cx: &'a mut C,
        id: usize,
        f: F,
    ) -> Result<&'a mut LocalCellValue, E>
    where
        C: Context<'cx>,
        F: FnOnce(&mut C) -> Result<LocalCellValue, E>,
    {
        // Kick off a new transaction and drop it before getting the result.
        {
            let mut tx = TryInitTransaction::new(cx, id);
            tx.run(|cx| f(cx))?;
        }

        // If we're here, the transaction has succeeded, so get the result.
        Ok(LocalCell::get(cx, id).unwrap())
    }
}

impl LocalTable {
    pub(crate) fn get(&mut self, index: usize) -> &mut LocalCell {
        if index >= self.cells.len() {
            self.cells.resize_with(index + 1, Default::default);
        }
        &mut self.cells[index]
    }
}

/// An RAII implementation of `LocalCell::get_or_try_init`, which ensures that
/// the state of a cell is properly managed through all possible control paths.
/// As soon as the transaction begins, the cell is labelled as being in a dirty
/// state (`LocalCell::Trying`), so that any additional re-entrant attempts to
/// initialize the cell will fail fast. The `Drop` implementation ensures that
/// after the transaction, the cell goes back to a clean state of either
/// `LocalCell::Uninit` if it fails or `LocalCell::Init` if it succeeds.
struct TryInitTransaction<'cx, 'a, C: Context<'cx>> {
    cx: &'a mut C,
    id: usize,
    _lifetime: PhantomData<&'cx ()>,
}

impl<'cx, 'a, C: Context<'cx>> TryInitTransaction<'cx, 'a, C> {
    fn new(cx: &'a mut C, id: usize) -> Self {
        let mut tx = Self {
            cx,
            id,
            _lifetime: PhantomData,
        };
        tx.cell().pre_init(|| LocalCell::Trying);
        tx
    }

    /// _Post-condition:_ If this method returns an `Ok` result, the cell is in the
    /// `LocalCell::Init` state.
    fn run<E, F>(&mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(&mut C) -> Result<LocalCellValue, E>,
    {
        if self.is_trying() {
            let value = f(self.cx)?;
            *self.cell() = LocalCell::Init(value);
        }
        Ok(())
    }

    fn cell(&mut self) -> &mut LocalCell {
        InstanceData::locals(self.cx).get(self.id)
    }

    #[allow(clippy::wrong_self_convention)]
    fn is_trying(&mut self) -> bool {
        matches!(self.cell(), LocalCell::Trying)
    }
}

impl<'cx, 'a, C: Context<'cx>> Drop for TryInitTransaction<'cx, 'a, C> {
    fn drop(&mut self) {
        if self.is_trying() {
            *self.cell() = LocalCell::Uninit;
        }
    }
}

/// Wrapper for raw Node-API values to be dropped on the main thread
pub(crate) enum DropData {
    Deferred(NodeApiDeferred),
    Ref(NapiRef),
}

impl DropData {
    /// Drop a value on the main thread
    fn drop(env: Option<Env>, data: Self) {
        if let Some(env) = env {
            unsafe {
                match data {
                    DropData::Deferred(data) => data.leaked(env),
                    DropData::Ref(data) => data.unref(env),
                }
            }
        }
    }
}

impl InstanceData {
    /// Return the data associated with this module instance, lazily initializing if
    /// necessary.
    ///
    /// # Safety
    /// No additional locking (e.g., `Mutex`) is necessary because holding a
    /// `Context` reference ensures serialized access.
    pub(crate) fn get<'cx, C: Context<'cx>>(cx: &mut C) -> &mut InstanceData {
        let env = cx.env().to_raw();
        let data = unsafe { lifecycle::get_instance_data::<InstanceData>(env).as_mut() };

        if let Some(data) = data {
            return data;
        }

        let drop_queue = unsafe {
            let queue = ThreadsafeFunction::new(env, DropData::drop);
            queue.unref(env);
            queue
        };

        let shared_channel = {
            let mut channel = Channel::new(cx);
            channel.unref(cx);
            channel
        };

        let data = InstanceData {
            id: InstanceId::next(),
            drop_queue: Arc::new(drop_queue),
            shared_channel,
            locals: LocalTable::default(),
        };

        unsafe { &mut *lifecycle::set_instance_data(env, data) }
    }

    /// Helper to return a reference to the `drop_queue` field of `InstanceData`
    pub(crate) fn drop_queue<'cx, C: Context<'cx>>(
        cx: &mut C,
    ) -> Arc<ThreadsafeFunction<DropData>> {
        Arc::clone(&InstanceData::get(cx).drop_queue)
    }

    /// Clones the shared channel and references it since new channels should start
    /// referenced, but the shared channel is unreferenced.
    pub(crate) fn channel<'cx, C: Context<'cx>>(cx: &mut C) -> Channel {
        let mut channel = InstanceData::get(cx).shared_channel.clone();
        channel.reference(cx);
        channel
    }

    /// Unique identifier for this instance of the module
    pub(crate) fn id<'cx, C: Context<'cx>>(cx: &mut C) -> InstanceId {
        InstanceData::get(cx).id
    }

    /// Helper to return a reference to the `locals` field of `InstanceData`.
    pub(crate) fn locals<'cx, C: Context<'cx>>(cx: &mut C) -> &mut LocalTable {
        &mut InstanceData::get(cx).locals
    }
}
