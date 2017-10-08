use neon::vm::{Call, JsResult};
use neon::mem::Handle;
use neon::js::{JsNumber, JsString, JsObject, Object};
use neon::scope::Scope;

pub fn return_js_global_object(call: Call) -> JsResult<JsObject> {
    let scope = call.scope;
    Ok(scope.global())
}

pub fn return_js_object(call: Call) -> JsResult<JsObject> {
    Ok(JsObject::new(call.scope))
}

pub fn return_js_object_with_mixed_content(call: Call) -> JsResult<JsObject> {
    let scope = call.scope;
    let js_object: Handle<JsObject> = JsObject::new(scope);
    js_object.set("number", JsNumber::new(scope, 9000.0))?;
    js_object.set("string", JsString::new(scope, "hello node").unwrap())?;
    Ok(js_object)
}

pub fn return_js_object_with_number(call: Call) -> JsResult<JsObject> {
    let scope = call.scope;
    let js_object: Handle<JsObject> = JsObject::new(scope);
    js_object.set("number", JsNumber::new(scope, 9000.0))?;
    Ok(js_object)
}

pub fn return_js_object_with_string(call: Call) -> JsResult<JsObject> {
    let scope = call.scope;
    let js_object: Handle<JsObject> = JsObject::new(scope);
    js_object.set("string", JsString::new(scope, "hello node").unwrap())?;
    Ok(js_object)
}
