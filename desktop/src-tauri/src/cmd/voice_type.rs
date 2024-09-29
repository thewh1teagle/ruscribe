/*
As long as the stream active, it will write to the buffer
If start/stop called the last stream will be stop.
If tab changed, the stream will stop.
*/
use super::audio::AudioDevice;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, Stream};
use eyre::{bail, Context, ContextCompat, Result};
use once_cell::sync::Lazy;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use screencapturekit_sys::as_ptr::AsMutPtr;
use std::ops::Mul;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

static HOLDING_KEYS: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static STREAM_ACTIVE: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static STREAM: Lazy<Mutex<Option<StreamHandle>>> = Lazy::new(|| Mutex::new(None));

struct StreamHandle(Stream);
unsafe impl Send for StreamHandle {}
unsafe impl Sync for StreamHandle {}

static BUFFER: Lazy<Mutex<AllocRingBuffer<i32>>> =
    Lazy::new(|| Mutex::new(AllocRingBuffer::with_capacity_power_of_2(16000 * 60))); // maximum of 1 minute

fn audio_resample(data: &[f32], sample_rate0: u32, sample_rate: u32, channels: u16) -> Vec<f32> {
    use samplerate::{convert, ConverterType};
    convert(
        sample_rate0 as _,
        sample_rate as _,
        channels as _,
        ConverterType::SincBestQuality,
        data,
    )
    .unwrap_or_default()
}

#[tauri::command]
pub fn start_voice_capture(device: AudioDevice) -> Result<()> {
    // use enigo::{Enigo, Settings};
    // let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let host = cpal::default_host();
    let device_id: usize = device.id.parse().context("Failed to parse device ID")?;
    let device = host.devices()?.nth(device_id).context("Failed to get device by ID")?;
    let config = device.default_input_config().context("Failed to get default input config")?;

    let err_fn = move |err| {
        tracing::error!("An error occurred on stream: {}", err);
    };

    let stream = match config.sample_format() {
        cpal::SampleFormat::I8 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| {
                tracing::debug!("Writing input data (I8)");
                write_input_data::<i8, i8>(data)
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| {
                tracing::debug!("Writing input data (I16)");
                write_input_data::<i16, i16>(data)
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::I32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| {
                tracing::debug!("Writing input data (I32)");
                write_input_data::<i32, i32>(data)
            },
            err_fn,
            None,
        )?,
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| {
                tracing::debug!("Writing input data (F32)");
                write_input_data::<f32, f32>(data)
            },
            err_fn,
            None,
        )?,
        sample_format => {
            bail!("Unsupported sample format '{}'", sample_format)
        }
    };
    *STREAM.lock().unwrap() = Some(StreamHandle(stream));
    Ok(())
}

fn write_input_data<T, U>(input: &[T])
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T> + Mul<Output = U> + Copy,
{
    // if let Ok(mut guard) = writer.try_lock() {
    //     if let Some(writer) = guard.as_mut() {
    //         for &sample in input.iter() {
    //             let sample: U = U::from_sample(sample);
    //             writer.write_sample(sample).ok();
    //         }
    //     }
    // }
}

#[tauri::command]
pub fn stop_voice_capture() -> Result<()> {
    let mut stream = STREAM.lock().unwrap();
    let stream = stream.as_mut().unwrap();
    stream.0.pause()?;
    Ok(())
}

#[tauri::command]
pub fn set_voice_type(val: bool) {
    HOLDING_KEYS.store(val, Ordering::Relaxed);
}
