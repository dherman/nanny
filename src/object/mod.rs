//! Traits for working with JavaScript objects.

pub(crate) mod class;

use crate::context::Context;
use crate::handle::{Handle, Managed};
use crate::result::{JsResult, NeonResult, Throw};
use crate::types::utf8::Utf8;
use crate::types::{build, JsArray, JsValue, Value};
use neon_runtime;
use neon_runtime::raw;

pub use self::class::{Class, ClassDescriptor};

/// A property key in a JavaScript object.
pub trait PropertyKey {
    unsafe fn get_from(self, out: &mut raw::Local, obj: raw::Local) -> bool;
    unsafe fn set_from(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool;
}

impl PropertyKey for u32 {
    unsafe fn get_from(self, out: &mut raw::Local, obj: raw::Local) -> bool {
        neon_runtime::object::get_index(out, obj, self)
    }

    unsafe fn set_from(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
        neon_runtime::object::set_index(out, obj, self, val)
    }
}

impl<'a, K: Value> PropertyKey for Handle<'a, K> {
    unsafe fn get_from(self, out: &mut raw::Local, obj: raw::Local) -> bool {
        neon_runtime::object::get(out, obj, self.to_raw())
    }

    unsafe fn set_from(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
        neon_runtime::object::set(out, obj, self.to_raw(), val)
    }
}

impl<'a> PropertyKey for &'a str {
    unsafe fn get_from(self, out: &mut raw::Local, obj: raw::Local) -> bool {
        let (ptr, len) = Utf8::from(self).into_small_unwrap().lower();
        neon_runtime::object::get_string(out, obj, ptr, len)
    }

    unsafe fn set_from(self, out: &mut bool, obj: raw::Local, val: raw::Local) -> bool {
        let (ptr, len) = Utf8::from(self).into_small_unwrap().lower();
        neon_runtime::object::set_string(out, obj, ptr, len, val)
    }
}

/// The trait of all object types.
pub trait Object: Value {
    fn get<'a, C: Context<'a>, K: PropertyKey>(
        self,
        _: &mut C,
        key: K,
    ) -> NeonResult<Handle<'a, JsValue>> {
        build(|out| unsafe { key.get_from(out, self.to_raw()) })
    }

    fn get_own_property_names<'a, C: Context<'a>>(self, _: &mut C) -> JsResult<'a, JsArray> {
        build(|out| unsafe { neon_runtime::object::get_own_property_names(out, self.to_raw()) })
    }

    fn set<'a, C: Context<'a>, K: PropertyKey, W: Value>(
        self,
        _: &mut C,
        key: K,
        val: Handle<'_, W>,
    ) -> NeonResult<bool> {
        let mut result = false;
        if unsafe { key.set_from(&mut result, self.to_raw(), val.to_raw()) } {
            Ok(result)
        } else {
            Err(Throw)
        }
    }
}

/// The trait of types that can be a function's `this` binding.
pub unsafe trait This: Managed {
    fn as_this(h: raw::Local) -> Self;
}
