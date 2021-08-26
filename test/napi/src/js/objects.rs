use neon::prelude::*;

pub fn return_js_global_object(mut cx: FunctionContext) -> JsResult<JsObject> {
    Ok(cx.global())
}

pub fn return_js_object(mut cx: FunctionContext) -> JsResult<JsObject> {
    Ok(cx.empty_object())
}

pub fn return_js_object_with_mixed_content(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object: Handle<JsObject> = cx.empty_object();
    let n = cx.number(9000.0);
    js_object.set(&mut cx, "number", n)?;
    let s = cx.string("hello node");
    js_object.set(&mut cx, "string", s)?;
    Ok(js_object)
}

pub fn return_js_object_with_number(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object: Handle<JsObject> = cx.empty_object();
    let n = cx.number(9000.0);
    js_object.set(&mut cx, "number", n)?;
    Ok(js_object)
}

pub fn return_js_object_with_string(mut cx: FunctionContext) -> JsResult<JsObject> {
    let js_object: Handle<JsObject> = cx.empty_object();
    let s = cx.string("hello node");
    js_object.set(&mut cx, "string", s)?;
    Ok(js_object)
}

pub fn return_array_buffer(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let b: Handle<JsArrayBuffer> = cx.array_buffer(16)?;
    Ok(b)
}

pub fn read_array_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsTypedArray<u32>>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let lock = cx.lock();
    let n = buf.try_borrow(&lock).map(|buf| buf[i]).or_throw(&mut cx)?;

    Ok(cx.number(n))
}

pub fn read_array_buffer_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsArrayBuffer>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = buf.as_slice(&cx)[i];

    Ok(cx.number(n as f64))
}

pub fn write_array_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsArrayBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as u32 as usize;
    let x = cx.argument::<JsNumber>(2)?.value(&mut cx) as u8;
    let lock = cx.lock();

    b.try_borrow_mut(&lock)
        .map(|mut slice| {
            slice[i] = x;
        })
        .or_throw(&mut cx)?;

    Ok(cx.undefined())
}

pub fn write_array_buffer_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut buf = cx.argument::<JsArrayBuffer>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = cx.argument::<JsNumber>(2)?.value(&mut cx) as u8;

    buf.as_mut_slice(&mut cx)[i] = n;

    Ok(cx.undefined())
}

pub fn read_typed_array_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsTypedArray<i32>>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = buf.as_slice(&cx)[i];

    Ok(cx.number(n as f64))
}

pub fn write_typed_array_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut buf = cx.argument::<JsTypedArray<i32>>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = cx.argument::<JsNumber>(2)?.value(&mut cx) as i32;

    buf.as_mut_slice(&mut cx)[i] = n;

    Ok(cx.undefined())
}

pub fn read_u8_typed_array(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsTypedArray<u8>>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = buf.as_slice(&cx)[i];

    Ok(cx.number(n as f64))
}

pub fn copy_typed_array(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let source = cx.argument::<JsTypedArray<u32>>(0)?;
    let mut dest = cx.argument::<JsTypedArray<u32>>(1)?;
    let mut run = || {
        let lock = cx.lock();
        let source = source.try_borrow(&lock)?;
        let mut dest = dest.try_borrow_mut(&lock)?;

        dest.copy_from_slice(&source);

        Ok(())
    };

    run().or_throw(&mut cx)?;

    Ok(cx.undefined())
}

pub fn return_uninitialized_buffer(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let b: Handle<JsBuffer> = unsafe { JsBuffer::uninitialized(&mut cx, 16)? };
    Ok(b)
}

pub fn return_buffer(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let b: Handle<JsBuffer> = cx.buffer(16)?;
    Ok(b)
}

pub fn return_external_buffer(mut cx: FunctionContext) -> JsResult<JsBuffer> {
    let data = cx.argument::<JsString>(0)?.value(&mut cx);
    let buf = JsBuffer::external(&mut cx, data.into_bytes());

    Ok(buf)
}

pub fn return_external_array_buffer(mut cx: FunctionContext) -> JsResult<JsArrayBuffer> {
    let data = cx.argument::<JsString>(0)?.value(&mut cx);
    let buf = JsArrayBuffer::external(&mut cx, data.into_bytes());

    Ok(buf)
}

pub fn read_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let b: Handle<JsBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let lock = cx.lock();
    let x = b
        .try_borrow(&lock)
        .map(|slice| slice[i])
        .or_throw(&mut cx)?;

    Ok(cx.number(x))
}

pub fn read_buffer_with_borrow(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let buf = cx.argument::<JsBuffer>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = buf.as_slice(&cx)[i];

    Ok(cx.number(n as f64))
}

pub fn write_buffer_with_lock(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut b: Handle<JsBuffer> = cx.argument(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let x = cx.argument::<JsNumber>(2)?.value(&mut cx) as u8;
    let lock = cx.lock();

    b.try_borrow_mut(&lock)
        .map(|mut slice| slice[i] = x)
        .or_throw(&mut cx)?;

    Ok(cx.undefined())
}

pub fn write_buffer_with_borrow_mut(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut buf = cx.argument::<JsBuffer>(0)?;
    let i = cx.argument::<JsNumber>(1)?.value(&mut cx) as usize;
    let n = cx.argument::<JsNumber>(2)?.value(&mut cx) as u8;

    buf.as_mut_slice(&mut cx)[i] = n;

    Ok(cx.undefined())
}
