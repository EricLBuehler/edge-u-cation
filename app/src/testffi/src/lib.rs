use std::panic;

use android_logger::Config;
use anyhow::Result;
use jni::objects::{JClass, JIntArray, JString};
use jni::sys::{jint, jstring};
use jni::JNIEnv;
use log::{debug, error, LevelFilter};
use utils::{jni_to_int_array, jni_to_string};

#[macro_use]
mod utils;
mod image_processing;
mod inference;

pub struct Paths {
    cfg_path: String,
    gen_cfg_path: String,
    uqff_path: String,
    pre_proc_path: String,
    proc_path: String,
    res_path: String,
    tok_path: String,
    tok_cfg_path: String,
    cimatrix_path: String,
}

#[no_mangle]
pub extern "C" fn Java_com_example_mistralrs_FFI_00024Companion_run_1vision(
    mut env: JNIEnv,
    _class: JClass,
    this: JClass,
    prompt: JString,
    pixels: JIntArray,
    width: jint,
    height: jint,
    cfg_path: JString,
    gen_cfg_path: JString,
    uqff_path: JString,
    pre_proc_path: JString,
    proc_path: JString,
    res_path: JString,
    tok_path: JString,
    tok_cfg_path: JString,
    cimatrix_path: JString,
) -> jstring {
    panic::set_hook(Box::new(|panic_info| {
        let payload = {
            panic_info
                .payload()
                .downcast_ref::<String>()
                .map(|s| s.to_string())
        };
        error!(
            "A panic occurred: payload={payload:?} location={:?}",
            panic_info.location()
        );
    }));

    let cfg_path = jtry!(env, jni_to_string(&mut env, &cfg_path));
    let gen_cfg_path = jtry!(env, jni_to_string(&mut env, &gen_cfg_path));
    let uqff_path = jtry!(env, jni_to_string(&mut env, &uqff_path));
    let pre_proc_path = jtry!(env, jni_to_string(&mut env, &pre_proc_path));
    let proc_path = jtry!(env, jni_to_string(&mut env, &proc_path));
    let res_path = jtry!(env, jni_to_string(&mut env, &res_path));
    let tok_path = jtry!(env, jni_to_string(&mut env, &tok_path));
    let tok_cfg_path = jtry!(env, jni_to_string(&mut env, &tok_cfg_path));
    let cimatrix_path = jtry!(env, jni_to_string(&mut env, &cimatrix_path));

    let paths = Paths {
        cfg_path,
        gen_cfg_path,
        uqff_path,
        pre_proc_path,
        proc_path,
        res_path,
        tok_cfg_path,
        tok_path,
        cimatrix_path,
    };

    let pixels = jtry!(env, jni_to_int_array(&mut env, &pixels));

    let prompt = jtry!(env, jni_to_string(&mut env, &prompt));

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Debug)
            .with_tag("mistralrs-ffi"),
    );

    let rt = jtry!(env, tokio::runtime::Runtime::new());

    let result: Result<String> = rt.block_on(async {
        let resp = inference::run_inference_vision(
            &mut env,
            &this,
            paths,
            prompt,
            pixels,
            width as u32,
            height as u32,
        )
        .await?;
        Ok(resp)
    });
    let result = jtry!(env, result);
    debug!("Result is:\n{result}.");
    let x: jstring = jtry!(env, env.new_string(result)).into_raw();
    x
}

#[no_mangle]
pub extern "C" fn Java_com_example_mistralrs_FFI_00024Companion_run_1text(
    mut env: JNIEnv,
    _class: JClass,
    this: JClass,
    prompt: JString,
    cfg_path: JString,
    gen_cfg_path: JString,
    uqff_path: JString,
    pre_proc_path: JString,
    proc_path: JString,
    res_path: JString,
    tok_path: JString,
    tok_cfg_path: JString,
    cimatrix_path: JString,
) -> jstring {
    panic::set_hook(Box::new(|panic_info| {
        let payload = {
            panic_info
                .payload()
                .downcast_ref::<String>()
                .map(|s| s.to_string())
        };
        error!(
            "A panic occurred: payload={payload:?} location={:?}",
            panic_info.location()
        );
    }));

    let cfg_path = jtry!(env, jni_to_string(&mut env, &cfg_path));
    let gen_cfg_path = jtry!(env, jni_to_string(&mut env, &gen_cfg_path));
    let uqff_path = jtry!(env, jni_to_string(&mut env, &uqff_path));
    let pre_proc_path = jtry!(env, jni_to_string(&mut env, &pre_proc_path));
    let proc_path = jtry!(env, jni_to_string(&mut env, &proc_path));
    let res_path = jtry!(env, jni_to_string(&mut env, &res_path));
    let tok_path = jtry!(env, jni_to_string(&mut env, &tok_path));
    let tok_cfg_path = jtry!(env, jni_to_string(&mut env, &tok_cfg_path));
    let cimatrix_path = jtry!(env, jni_to_string(&mut env, &cimatrix_path));

    let paths = Paths {
        cfg_path,
        gen_cfg_path,
        uqff_path,
        pre_proc_path,
        proc_path,
        res_path,
        tok_cfg_path,
        tok_path,
        cimatrix_path,
    };

    let prompt = jtry!(env, jni_to_string(&mut env, &prompt));

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Debug)
            .with_tag("mistralrs-ffi"),
    );

    let rt = jtry!(env, tokio::runtime::Runtime::new());

    let result: Result<String> = rt.block_on(async {
        let resp = inference::run_inference_text(&mut env, &this, paths, prompt).await?;
        Ok(resp)
    });
    let result = jtry!(env, result);
    debug!("Result is:\n{result}.");
    let x: jstring = jtry!(env, env.new_string(result)).into_raw();
    x
}

// Not actually used
#[no_mangle]
pub extern "C" fn Java_com_example_mistralrs_FFI_00024Companion_load_model(
    mut env: JNIEnv,
    _class: JClass,
    this: JClass,
    cfg_path: JString,
    gen_cfg_path: JString,
    uqff_path: JString,
    pre_proc_path: JString,
    proc_path: JString,
    res_path: JString,
    tok_path: JString,
    tok_cfg_path: JString,
    cimatrix_path: JString,
) -> jstring {
    panic::set_hook(Box::new(|panic_info| {
        let payload = {
            panic_info
                .payload()
                .downcast_ref::<String>()
                .map(|s| s.to_string())
        };
        error!(
            "A panic occurred: payload={payload:?} location={:?}",
            panic_info.location()
        );
    }));

    let cfg_path = jtry!(env, jni_to_string(&mut env, &cfg_path));
    let gen_cfg_path = jtry!(env, jni_to_string(&mut env, &gen_cfg_path));
    let uqff_path = jtry!(env, jni_to_string(&mut env, &uqff_path));
    let pre_proc_path = jtry!(env, jni_to_string(&mut env, &pre_proc_path));
    let proc_path = jtry!(env, jni_to_string(&mut env, &proc_path));
    let res_path = jtry!(env, jni_to_string(&mut env, &res_path));
    let tok_path = jtry!(env, jni_to_string(&mut env, &tok_path));
    let tok_cfg_path = jtry!(env, jni_to_string(&mut env, &tok_cfg_path));
    let cimatrix_path = jtry!(env, jni_to_string(&mut env, &cimatrix_path));

    let paths = Paths {
        cfg_path,
        gen_cfg_path,
        uqff_path,
        pre_proc_path,
        proc_path,
        res_path,
        tok_cfg_path,
        tok_path,
        cimatrix_path,
    };

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Debug)
            .with_tag("mistralrs-ffi"),
    );

    let rt = jtry!(env, tokio::runtime::Runtime::new());

    let res = rt.block_on(async { inference::load_model_generic(&mut env, &this, paths).await });
    let _model = jtry!(env, res);

    let x: jstring = jtry!(env, env.new_string("DONE".to_string())).into_raw();
    x
}
