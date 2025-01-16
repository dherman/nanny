use std::marker::PhantomData;

use crate::{
    context::Context,
    handle::{Handle, Root},
    object::Object,
    result::{JsResult, NeonResult},
    thread::LocalKey,
    types::{JsObject, JsString},
};

static PROCESS: LocalKey<Root<JsObject>> = LocalKey::new();
static VERSIONS: LocalKey<Root<JsObject>> = LocalKey::new();
static VERSION_DATA: LocalKey<VersionData> = LocalKey::new();

pub struct Process<'a, 'cx: 'a, C: Context<'cx>> {
    pub(crate) cx: &'a mut C,
    pub(crate) marker: PhantomData<&'cx ()>,
}

impl<'a, 'cx: 'a, C: Context<'cx>> Process<'a, 'cx, C> {
    // FIXME: this can be abstracted in a private super-trait
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

    pub(crate) fn new(cx: &'a mut C) -> Self {
        Self {
            cx,
            marker: PhantomData,
        }
    }

    fn process_object(&mut self) -> JsResult<'cx, JsObject> {
        self.memo(&PROCESS, |c| Ok(c.cx.global_object()), "process")
    }

    fn versions_object(&mut self) -> JsResult<'cx, JsObject> {
        self.memo(&VERSIONS, |c| c.process_object(), "versions")
    }

    pub fn versions(&mut self) -> NeonResult<Versions<'cx>> {
        let object = self.versions_object()?;

        Versions::new(self.cx, object)
    }
}

pub struct Versions<'cx> {
    pub(crate) object: Handle<'cx, JsObject>,
    pub(crate) data: &'cx VersionData,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct VersionData {
    pub node: String,
    pub modules: String,
    pub napi: String,
    pub unicode: String,
    pub uv: String,
}

impl<'cx> Versions<'cx> {
    fn new<C: Context<'cx>>(cx: &mut C, object: Handle<'cx, JsObject>) -> NeonResult<Self> {
        Ok(Self {
            object,
            data: VERSION_DATA.get_or_try_init(cx, |cx| {
                let node = object.get::<JsString, _, _>(cx, "node")?.value(cx);
                let modules = object.get::<JsString, _, _>(cx, "modules")?.value(cx);
                let napi = object.get::<JsString, _, _>(cx, "napi")?.value(cx);
                let unicode = object.get::<JsString, _, _>(cx, "unicode")?.value(cx);
                let uv = object.get::<JsString, _, _>(cx, "uv")?.value(cx);
                Ok(VersionData {
                    node,
                    modules,
                    napi,
                    unicode,
                    uv,
                })
            })?,
        })
    }

    pub fn object(&self) -> Handle<'cx, JsObject> {
        self.object
    }

    pub fn data(&self) -> &VersionData {
        &self.data
    }
}
