use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

use crate::{
    image_processing::{
        argb8888_to_dynamic_image, dynamic_image_to_argb8888, resize_image_to_max_edge,
    },
    utils::{get_jni_signature, MethodType, ReturnMethodType},
    Paths,
};
use anyhow::{Context, Result};
use image::GenericImageView;
use jni::{
    objects::{JClass, JValue},
    JNIEnv,
};
use log::debug;
use mistralrs::{
    AutoDeviceMapParams, DefaultSchedulerMethod, Device, DeviceMapSetting, LoraConfig,
    MistralRsBuilder, Model, ModelDType, ModelSource, ModelWeightSource, Ordering, RequestBuilder,
    ResponseOk, SchedulerConfig, TextMessageRole, VisionLoaderBuilder, VisionLoaderType,
    VisionSpecificConfig, XLoraConfig,
};
use tokio::time::interval;

const MAX_IMAGE_LENGTH: u32 = 1024;

static MODEL: OnceLock<Arc<Model>> = OnceLock::new();

#[derive(Clone, Debug)]
/// All local paths and metadata necessary to load a model.
pub struct CustomModelSource {
    pub tokenizer: String,
    pub config: String,
    pub chat_template: Option<String>,
    pub filenames: Vec<ModelWeightSource>,
    pub generation_config: Option<String>,
    pub preprocessor_config: Option<String>,
    pub processor_config: Option<String>,
    pub chat_template_json: Option<String>,
}

impl ModelSource for CustomModelSource {
    fn get_config(&self) -> &String {
        &self.config
    }
    fn get_tokenizer(&self) -> &String {
        &self.tokenizer
    }
    fn get_weights(&self) -> &[ModelWeightSource] {
        &self.filenames
    }
    fn get_adapter_filenames(&self) -> &Option<Vec<(String, PathBuf)>> {
        &None
    }
    fn get_adapter_configs(&self) -> &Option<Vec<((String, String), LoraConfig)>> {
        &None
    }
    fn get_classifier_config(&self) -> &Option<XLoraConfig> {
        &None
    }
    fn get_classifier_path(&self) -> &Option<PathBuf> {
        &None
    }
    fn get_ordering(&self) -> &Option<Ordering> {
        &None
    }
    fn get_chat_template(&self) -> &Option<String> {
        &self.chat_template
    }
    fn get_generation_config(&self) -> Option<&String> {
        self.generation_config.as_ref()
    }
    fn get_lora_preload_adapter_info(&self) -> &Option<HashMap<String, (PathBuf, LoraConfig)>> {
        &None
    }
    fn get_preprocessor_config(&self) -> &Option<String> {
        &self.preprocessor_config
    }
    fn get_processor_config(&self) -> &Option<String> {
        &self.processor_config
    }
    fn get_chat_template_json(&self) -> &Option<String> {
        &self.chat_template_json
    }
}

fn send_chunk(env: &mut JNIEnv<'_>, this: &JClass<'_>, chunk: String) -> Result<()> {
    let method_sig = get_jni_signature(&[MethodType::String], ReturnMethodType::Void);
    let x = env.new_string(chunk)?;
    env.call_method(
        this,
        "process_chunk",
        method_sig.clone(),
        &[JValue::Object(&x)],
    )?;

    Ok(())
}

fn clear_output(env: &mut JNIEnv<'_>, this: &JClass<'_>) -> Result<()> {
    let method_sig = get_jni_signature(&[], ReturnMethodType::Void);
    env.call_method(this, "clear_output", method_sig.clone(), &[])?;

    Ok(())
}

async fn do_inference(
    env: &mut JNIEnv<'_>,
    this: &JClass<'_>,
    model: &Model,
    messages: RequestBuilder,
) -> Result<String> {
    let mut stream = model.stream_chat_request(messages).await?;

    let mut accum = String::new();

    clear_output(env, this)?;

    let mut ticker = interval(Duration::from_millis(10));

    let prompt_start_time = Instant::now();
    let mut decode_start_time = None;
    let mut tokens_generated = 0usize;

    loop {
        tokio::select! {
            // When the stream produces a value:
            maybe_item = stream.next() => {
                match maybe_item {
                    Some(resp) => {
                        let ResponseOk::Chunk(chunk) = resp.as_result()? else {
                            anyhow::bail!("Apparently this isn't a chunk");
                        };

                        if decode_start_time.is_none() {
                            decode_start_time = Some(Instant::now());
                        }

                        let chunk = &chunk.choices[0].delta.content.as_ref().unwrap();
                        accum.push_str(chunk);

                        if tokens_generated == 0 {
                            clear_output(env, this)?;
                        }
                        send_chunk(env, this, chunk.to_string())?;
                        tokens_generated += 1;
                    },
                    None => {
                        // Done with generation
                        send_chunk(env, this, "\n\nStats:".to_string())?;

                        if let Some(decode_start_time) = decode_start_time {
                            send_chunk(env, this, format!(
                                "\nGenerated {} tokens at {:.2} token/s.",
                                tokens_generated,
                                tokens_generated as f32 / Instant::now().duration_since(decode_start_time).as_secs_f32()
                            ))?;
                        }

                        send_chunk(env, this, format!(
                            "\nTotal time: {:.2}s.",
                            Instant::now().duration_since(prompt_start_time).as_secs_f32()
                        ))?;
                        break;
                    }
                }
            }
            // Every second, the timer ticks:
            tick = ticker.tick() => {
                if tokens_generated == 0 {
                    // Clear output again
                    clear_output(env, this)?;

                    send_chunk(env, this, format!(
                        "Processing... {:.2}s",
                        tick.duration_since(prompt_start_time.into()).as_secs_f32()
                    ))?;
                }
            }
        }
    }

    Ok(accum)
}

fn load_phi3_5_vision(src_paths: Paths) -> Result<Arc<Model>> {
    if let Some(model) = MODEL.get() {
        return Ok(model.clone());
    }

    let paths: Box<dyn ModelSource> = Box::new(CustomModelSource {
        tokenizer: fs::read_to_string(&src_paths.tok_path)?,
        config: fs::read_to_string(&src_paths.cfg_path)?,
        chat_template: Some(fs::read_to_string(&src_paths.tok_cfg_path)?),
        filenames: vec![ModelWeightSource::PathBuf(
            src_paths.res_path.to_string().into(),
        )],
        generation_config: Some(fs::read_to_string(&src_paths.gen_cfg_path)?),
        preprocessor_config: Some(fs::read_to_string(&src_paths.pre_proc_path)?),
        processor_config: Some(fs::read_to_string(&src_paths.proc_path)?),
        chat_template_json: None,
    });

    let uqff_path = PathBuf::from(src_paths.uqff_path);
    let model_id = uqff_path
        .parent()
        .context("Expected parent")?
        .to_string_lossy()
        .to_string();
    let uqff_file = uqff_path.file_name().unwrap().to_string_lossy().to_string();

    let config = VisionSpecificConfig {
        use_flash_attn: false,
        prompt_chunksize: None,
        topology: None,
        write_uqff: None,
        from_uqff: Some(ModelWeightSource::PathBuf(uqff_file.into())),
        max_edge: None,
        calibration_file: None,
        imatrix: Some(src_paths.cimatrix_path.into()),
    };

    let loader =
        VisionLoaderBuilder::new(config, None, None, Some(model_id)).build(VisionLoaderType::Phi3V);

    // Load, into a Pipeline
    let pipeline = loader.load_model_from_path(
        &paths,
        &ModelDType::Auto,
        &Device::Cpu,
        true,
        DeviceMapSetting::Auto(AutoDeviceMapParams::default_vision()),
        None,
        None,
    )?;

    let scheduler_method = SchedulerConfig::DefaultScheduler {
        method: DefaultSchedulerMethod::Fixed(1.try_into()?),
    };

    let runner = MistralRsBuilder::new(pipeline, scheduler_method)
        .with_no_kv_cache(false)
        .with_gemm_full_precision_f16(true)
        .with_no_prefix_cache(false);

    let model = Model::new(runner.build());
    MODEL
        .set(Arc::new(model))
        .map_err(|_| anyhow::Error::msg("Only one thread!"))?;

    Ok(MODEL
        .get()
        .context("Expected model to be present!")?
        .clone())
}

pub async fn load_model_generic(
    env: &mut JNIEnv<'_>,
    this: &JClass<'_>,
    src_paths: Paths,
) -> Result<Arc<Model>> {
    let loader_thread = std::thread::spawn(move || load_phi3_5_vision(src_paths));

    clear_output(env, this)?;

    let mut ticker = interval(Duration::from_millis(10));

    let start = Instant::now();

    loop {
        let tick = ticker.tick().await;
        // Clear output again
        clear_output(env, this)?;

        send_chunk(
            env,
            this,
            format!(
                "Loading model... {:.2}s",
                tick.duration_since(start.into()).as_secs_f32()
            ),
        )?;

        if loader_thread.is_finished() {
            clear_output(env, this)?;
            let res = loader_thread
                .join()
                .map_err(|_| anyhow::Error::msg("join failed"))?;
            return res;
        }
    }
}

pub async fn run_inference_vision(
    env: &mut JNIEnv<'_>,
    this: &JClass<'_>,
    paths: Paths,
    prompt: String,
    pixels: Vec<i32>,
    width: u32,
    height: u32,
) -> Result<String> {
    debug!("Loading model.");
    let model = load_model_generic(env, this, paths).await?;
    debug!("Loaded model.");

    let mut image = argb8888_to_dynamic_image(width, height, pixels)
        .context("Failed to decode argb8888_to_dynamic_image")?;
    image = resize_image_to_max_edge(image, MAX_IMAGE_LENGTH);

    debug!("Prompt is `{prompt}`.");
    let messages = RequestBuilder::new().add_image_message(
        TextMessageRole::User,
        prompt,
        image.clone(),
        &model,
    )?;
    debug!("Starting request.");

    // Send the image back to confirm correct decode
    {
        let method_sig = get_jni_signature(
            &[
                MethodType::Arr(Box::new(MethodType::I32)),
                MethodType::I32,
                MethodType::I32,
            ],
            ReturnMethodType::Void,
        );
        let argb_8888_image = dynamic_image_to_argb8888(image.clone());
        let argb_8888_slice = unsafe {
            // Convert to i32 slice (transmute)
            std::slice::from_raw_parts(
                argb_8888_image.as_ptr() as *const i32,
                argb_8888_image.len(),
            )
        };
        let pixels = env.new_int_array(argb_8888_image.len().try_into().unwrap())?;
        env.set_int_array_region(&pixels, 0, argb_8888_slice)?;

        let (w, h) = image.dimensions();
        env.call_method(
            this,
            "process_image_update",
            method_sig.clone(),
            &[
                JValue::Object(&pixels),
                JValue::Int(w as i32),
                JValue::Int(h as i32),
            ],
        )?;
    }

    do_inference(env, this, &model, messages).await
}

pub async fn run_inference_text(
    env: &mut JNIEnv<'_>,
    this: &JClass<'_>,
    paths: Paths,
    prompt: String,
) -> Result<String> {
    debug!("Loading model.");
    let model = load_model_generic(env, this, paths).await?;
    debug!("Loaded model.");

    debug!("Prompt is `{prompt}`.");
    let messages = RequestBuilder::new().add_message(TextMessageRole::User, prompt);
    debug!("Starting request.");

    do_inference(env, this, &model, messages).await
}
