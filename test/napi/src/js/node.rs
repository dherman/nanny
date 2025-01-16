use neon::node::Node;
use neon::prelude::*;

pub fn call_console_log_and_error(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    cx.console().log("This is cx.console().log()!")?;
    cx.console().error("This is cx.console().error()!")?;
    Ok(cx.undefined())
}

pub fn get_node_version(mut cx: FunctionContext) -> JsResult<JsString> {
    let versions = cx.process().versions()?;
    let data = versions.data();
    Ok(cx.string(&data.node))
}
