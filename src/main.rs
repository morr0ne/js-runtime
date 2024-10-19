use anyhow::{bail, Result};
use mozjs::{
    conversions::jsstr_to_string,
    jsapi::{
        Evaluate2, JSAutoRealm, JSClass, JSClassOps, JS_EnumerateStandardClasses,
        JS_GlobalObjectTraceHook, JS_MayResolveStandardClass, JS_NewGlobalObject,
        JS_ResolveStandardClass, OnNewGlobalHookOption, JSCLASS_GLOBAL_FLAGS,
    },
    jsval::{JSVal, UndefinedValue},
    panic::maybe_resume_unwind,
    rooted,
    rust::{transform_str_to_source_text, CompileOptionsWrapper, JSEngine, RealmOptions, Runtime},
};
use std::ptr::{null, null_mut};
use tracing::debug;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    debug!("Initializing JS engine");
    let engine = JSEngine::init().expect("failed to initalize JS engine");
    let runtime = Runtime::new(engine.handle());

    if runtime.cx().is_null() {
        bail!("failed to create JSContext")
    }

    do_stuff(&runtime)?;

    Ok(())
}

static GLOBAL_CLASS_OPS: JSClassOps = JSClassOps {
    addProperty: None,
    delProperty: None,
    enumerate: Some(JS_EnumerateStandardClasses),
    newEnumerate: None,
    resolve: Some(JS_ResolveStandardClass),
    mayResolve: Some(JS_MayResolveStandardClass),
    finalize: None,
    call: None,
    construct: None,
    trace: Some(JS_GlobalObjectTraceHook),
};

static GLOBAL_CLASS: JSClass = JSClass {
    name: c"Global".as_ptr(),
    flags: JSCLASS_GLOBAL_FLAGS,
    cOps: &GLOBAL_CLASS_OPS as *const JSClassOps,
    spec: null(),
    ext: null(),
    oOps: null(),
};

fn do_stuff(rt: &Runtime) -> Result<()> {
    // TODO: allow to pass custom options
    let realm_options = RealmOptions::default();

    rooted!(in(rt.cx()) let global = unsafe {
        JS_NewGlobalObject(rt.cx(), &GLOBAL_CLASS, null_mut(),
                           OnNewGlobalHookOption::FireOnNewGlobalHook,
                           &*realm_options)
    });

    let _ac = JSAutoRealm::new(rt.cx(), global.get());

    let options = unsafe { CompileOptionsWrapper::new(rt.cx(), "noname", 1) };
    let mut source = transform_str_to_source_text("(`hello world, it is ${new Date()}`)");

    rooted!(in(rt.cx()) let mut rval = UndefinedValue());

    if unsafe { !Evaluate2(rt.cx(), options.ptr, &mut source, rval.handle_mut().into()) } {
        maybe_resume_unwind();
        bail!("Failed to evaluate script")
    }

    let value: JSVal = rval.get();

    let res = unsafe { jsstr_to_string(rt.cx(), value.to_string()) };

    println!("{res}");

    Ok(())
}
