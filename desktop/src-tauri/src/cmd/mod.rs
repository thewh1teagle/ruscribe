use crate::config::{DEAFULT_MODEL_FILENAME, DEAFULT_MODEL_URL, STORE_FILENAME};
use crate::setup::ModelContext;
use eyre::{bail, Context, ContextCompat, Result};
use serde_json::{json, Value};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{
    window::{ProgressBarState, ProgressBarStatus},
    Manager,
};
use tauri::{State, Wry};
use tauri_plugin_store::{with_store, StoreCollection};
use tokio::sync::Mutex;
use vibe_core::{model::SegmentCallbackData, transcript::Transcript};
pub mod audio;

/// Return true if there's internet connection
/// timeout in ms
#[tauri::command]
pub async fn is_online(timeout: Option<u64>) -> Result<bool> {
    let timeout = std::time::Duration::from_millis(timeout.unwrap_or(2000));
    let targets = ["1.1.1.1:80", "1.1.1.1:53", "8.8.8.8:53", "8.8.8.8:80"];

    let tasks = targets.iter().map(|addr| async move {
        tokio::time::timeout(timeout, tokio::net::TcpStream::connect(addr))
            .await
            .map(|res| res.is_ok())
            .unwrap_or(false)
    });

    Ok(futures::future::join_all(tasks).await.into_iter().any(|res| res))
}

fn set_progress_bar(app_handle: &tauri::AppHandle, progress: Option<f64>) -> Result<()> {
    let window = app_handle.get_webview_window("main").context("get window")?;
    if let Some(progress) = progress {
        log::debug!("set_progress_bar {}", progress);
        window.emit("transcribe_progress", progress).unwrap();
        if progress > 1.0 {
            window.set_progress_bar(ProgressBarState {
                progress: Some(progress as u64),
                status: if cfg!(target_os = "windows") {
                    // It works in Windows without it, and setting it causes it to jump every time.
                    None
                } else {
                    Some(ProgressBarStatus::Indeterminate)
                },
            })?;
        }
    } else {
        window.set_progress_bar(ProgressBarState {
            progress: Some(0),
            status: Some(ProgressBarStatus::None),
        })?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_commit_hash() -> String {
    env!("COMMIT_HASH").to_string()
}

#[tauri::command]
pub fn get_x86_features() -> Option<Value> {
    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_os = "windows"))]
    {
        let features: Value = serde_json::to_value(crate::x86_features::X86features::new()).unwrap_or_default();
        Some(features)
    }

    #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_os = "windows")))]
    {
        None
    }
}
#[tauri::command]
pub async fn download_model(app_handle: tauri::AppHandle, url: Option<String>) -> Result<String> {
    let model_path = if let Some(url) = url.clone() {
        let filename = vibe_core::downloader::get_filename(&url).await?;
        log::debug!("url filename is {}", filename);
        get_models_folder(app_handle.clone())?.join(filename)
    } else {
        get_models_folder(app_handle.clone())?.join(DEAFULT_MODEL_FILENAME)
    };

    let mut downloader = vibe_core::downloader::Downloader::new();
    log::debug!("Download model invoked! with path {}", model_path.display());

    let abort_atomic = Arc::new(AtomicBool::new(false));
    let abort_atomic_c = abort_atomic.clone();

    let app_handle_c = app_handle.clone();

    // allow abort transcription
    let app_handle_d = app_handle_c.clone();
    app_handle.listen("abort_download", move |_| {
        set_progress_bar(&app_handle_d, None).unwrap();
        abort_atomic_c.store(true, Ordering::Relaxed);
    });

    let download_progress_callback = {
        let app_handle = app_handle.clone();
        let abort_atomic = abort_atomic.clone();

        move |current: u64, total: u64| {
            let app_handle = app_handle.clone();

            // Update progress in background
            tauri::async_runtime::spawn(async move {
                let window = app_handle.get_webview_window("main").unwrap();
                let percentage = (current as f64 / total as f64) * 100.0;
                log::debug!("percentage: {}", percentage);
                if let Err(e) = set_progress_bar(&app_handle, Some(percentage)) {
                    log::error!("Failed to set progress bar: {}", e);
                }
                if let Err(e) = window.emit("download_progress", (current, total)) {
                    log::error!("Failed to emit download progress: {}", e);
                }
            });
            // Return the abort signal immediately
            abort_atomic.load(Ordering::Relaxed)
        }
    };

    let download_url = if let Some(url) = url {
        url
    } else {
        DEAFULT_MODEL_URL.to_string()
    };
    downloader
        .download(&download_url, model_path.to_owned(), download_progress_callback)
        .await?;
    set_progress_bar(&app_handle_c, None).unwrap();
    Ok(model_path.to_str().context("to_str")?.to_string())
}

#[tauri::command]
pub async fn transcribe(
    app_handle: tauri::AppHandle,
    options: vibe_core::config::TranscribeOptions,
    model_context_state: State<'_, Mutex<Option<ModelContext>>>,
) -> Result<Transcript> {
    let model_context = model_context_state.lock().await;
    if model_context.is_none() {
        bail!("Please load model first")
    }
    let ctx = model_context.as_ref().unwrap();
    let app_handle_c = app_handle.clone();

    let new_segment_callback = move |data: SegmentCallbackData| {
        app_handle_c
            .clone()
            .emit_to(
                "main",
                "new_segment",
                serde_json::json!({"start": data.start_timestamp, "stop": data.end_timestamp, "text": data.text}),
            )
            .unwrap();
    };
    let abort_atomic = Arc::new(AtomicBool::new(false));
    let abort_atomic_c = abort_atomic.clone();

    // allow abort transcription
    let app_handle_c = app_handle.clone();
    app_handle.listen("abort_transcribe", move |_| {
        let _ = set_progress_bar(&app_handle_c, None);
        abort_atomic_c.store(true, Ordering::Relaxed);
    });

    let abort_callback = move || abort_atomic.load(Ordering::Relaxed);

    let app_handle_c = app_handle.clone();
    let progress_callback = move |progress: i32| {
        // log::debug!("desktop progress is {}", progress);
        let _ = set_progress_bar(&app_handle, Some(progress.into()));
    };

    // prevent panic crash. sometimes whisper.cpp crash without nice errors.
    let unwind_result = catch_unwind(AssertUnwindSafe(|| {
        vibe_core::model::transcribe(
            &ctx.handle,
            &options,
            Some(Box::new(progress_callback)),
            Some(Box::new(new_segment_callback)),
            Some(Box::new(abort_callback)),
        )
    }));
    let _ = set_progress_bar(&app_handle_c, None);
    match unwind_result {
        Err(error) => {
            bail!("transcribe crash: {:?}", error)
        }
        Ok(transcribe_result) => {
            let transcript = transcribe_result.with_context(|| format!("options: {:?}", options))?;
            Ok(transcript)
        }
    }
}

#[tauri::command]
pub fn get_path_dst(src: String, suffix: String) -> Result<String> {
    let src = PathBuf::from(src);
    let src_filename = src.file_name().context("filename")?.to_str().context("stostr")?;
    let src_name = src
        .file_stem()
        .map(|name| name.to_str().context("tosstr"))
        .unwrap_or(Ok(src_filename))?;

    let parent = src.parent().context("parent")?;
    let mut dst_path = parent.join(format!("{}{}", src_name, suffix));

    // Ensure we don't overwrite existing file
    let mut counter = 0;
    while dst_path.exists() {
        dst_path = parent.join(format!("{} ({}){}", src_name, counter, suffix));
        counter += 1;
    }
    Ok(dst_path.to_str().context("tostr")?.into())
}

#[tauri::command]
pub fn get_save_path(src_path: PathBuf, target_ext: &str) -> Result<Value> {
    // Get the file stem (filename without extension)
    let stem = src_path.file_stem().and_then(|s| s.to_str()).unwrap_or_default();

    // Create a new path with the same directory and the new extension
    let mut new_path = src_path.clone();
    new_path.set_file_name(stem);
    new_path.set_extension(target_ext);
    let new_filename = new_path.file_name().map(|s| s.to_str()).unwrap_or(Some("Untitled"));
    // Convert the new path to a string
    let new_path = new_path.to_str().context("to_str")?;
    let named_path = json!({"name": new_filename, "path": new_path});
    Ok(named_path)
}

#[tauri::command]
pub fn get_argv() -> Vec<String> {
    std::env::args().collect()
}

#[tauri::command]
/// Opens folder or open folder of a file
pub async fn open_path(path: PathBuf) -> Result<()> {
    if path.is_file() {
        showfile::show_path_in_file_manager(path);
    } else {
        open::that(path)?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_cuda_version() -> String {
    env!("CUDA_VERSION").to_string()
}

#[tauri::command]
pub fn is_avx2_enabled() -> bool {
    #[allow(clippy::comparison_to_empty)]
    return env!("WHISPER_NO_AVX") != "ON";
}

#[tauri::command]
pub async fn load_model(app_handle: tauri::AppHandle, model_path: String, gpu_device: Option<i32>) -> Result<String> {
    let model_context_state: State<'_, Mutex<Option<ModelContext>>> = app_handle.state();
    let mut state_guard = model_context_state.lock().await;
    if let Some(state) = state_guard.as_ref() {
        // check if new path is different
        if model_path != state.path || gpu_device != state.gpu_device {
            log::debug!("model path or gpu device changed. reloading");
            // reload
            let context = vibe_core::model::create_context(Path::new(&model_path), gpu_device)?;
            *state_guard = Some(ModelContext {
                path: model_path.clone(),
                handle: context,
                gpu_device,
            });
        }
    } else {
        log::debug!("loading model first time");
        let context = vibe_core::model::create_context(Path::new(&model_path), gpu_device)?;
        *state_guard = Some(ModelContext {
            path: model_path.clone(),
            handle: context,
            gpu_device,
        });
    }
    Ok(model_path)
}

#[tauri::command]
pub fn get_models_folder(app_handle: tauri::AppHandle) -> Result<PathBuf> {
    let stores = app_handle.state::<StoreCollection<Wry>>();
    if let Ok(Some(models_folder)) = with_store(app_handle.clone(), stores, STORE_FILENAME, |store| {
        log::debug!("{:?}", store.get("models_folder"));
        Ok(store.get("models_folder").and_then(|p| p.as_str().map(PathBuf::from)))
    }) {
        log::debug!("models folder: {:?}", models_folder);
        return Ok(models_folder);
    }
    let path = app_handle.path().app_local_data_dir().context("Can't get data directory")?;
    Ok(path)
}
