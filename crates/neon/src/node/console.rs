use std::marker::PhantomData;

use crate::{
    context::Context,
    handle::{Handle, Root},
    object::Object,
    result::{JsResult, NeonResult},
    thread::LocalKey,
    types::{extract::TryIntoJs, function::BindOptions, JsFunction, JsObject},
};

static CONSOLE: LocalKey<Root<JsObject>> = LocalKey::new();
static LOG: LocalKey<Root<JsFunction>> = LocalKey::new();
static ERROR: LocalKey<Root<JsFunction>> = LocalKey::new();
static INFO: LocalKey<Root<JsFunction>> = LocalKey::new();
static WARN: LocalKey<Root<JsFunction>> = LocalKey::new();
static CLEAR: LocalKey<Root<JsFunction>> = LocalKey::new();

pub struct Console<'a, 'cx: 'a, C: Context<'cx>> {
    pub(crate) cx: &'a mut C,
    pub(crate) marker: PhantomData<&'cx ()>,
}

impl<'a, 'cx: 'a, C: Context<'cx>> Console<'a, 'cx, C> {
    fn memo<T, F>(
        &mut self,
        cache: &'static LocalKey<Root<T>>,
        get_container: F,
        key: &str,
    ) -> JsResult<'cx, T>
    where
        T: Object,
        F: FnOnce(&mut Self) -> JsResult<'cx, JsObject>,
    {
        let container = get_container(self)?;
        let v = cache.get_or_try_init(self.cx, |cx| {
            let v: Handle<T> = container.get(cx, key)?;
            Ok(v.root(cx))
        })?;
        Ok(v.to_inner(self.cx))
    }

    fn memo_method<F>(
        &mut self,
        cache: &'static LocalKey<Root<JsFunction>>,
        get_container: F,
        key: &str,
    ) -> NeonResult<BindOptions<'_, 'cx>>
    where
        F: FnOnce(&mut Self) -> JsResult<'cx, JsObject>,
    {
        let container = get_container(self)?;
        let function = self.memo(cache, |_| Ok(container), key)?;
        let mut method = function.bind(self.cx.cx_mut());
        method.this(container)?;
        Ok(method)
    }

    pub(crate) fn new(cx: &'a mut C) -> Self {
        Self {
            cx,
            marker: PhantomData,
        }
    }

    fn console_object(&mut self) -> JsResult<'cx, JsObject> {
        self.memo(&CONSOLE, |c| Ok(c.cx.global_object()), "console")
    }

    pub fn log<T: TryIntoJs<'cx>>(&mut self, msg: T) -> NeonResult<()> {
        self.memo_method(&LOG, |c| c.console_object(), "log")?.arg(msg)?.exec()
    }

    pub fn error<T: TryIntoJs<'cx>>(&mut self, msg: T) -> NeonResult<()> {
        self.memo_method(&ERROR, |c| c.console_object(), "error")?.arg(msg)?.exec()
    }

    pub fn info<T: TryIntoJs<'cx>>(&mut self, msg: T) -> NeonResult<()> {
        self.memo_method(&INFO, |c| c.console_object(), "info")?.arg(msg)?.exec()
    }

    pub fn warn<T: TryIntoJs<'cx>>(&mut self, msg: T) -> NeonResult<()> {
        self.memo_method(&WARN, |c| c.console_object(), "warn")?.arg(msg)?.exec()
    }

    pub fn clear<T: TryIntoJs<'cx>>(&mut self) -> NeonResult<()> {
        self.memo_method(&CLEAR, |c| c.console_object(), "clear")?.exec()
    }
}
