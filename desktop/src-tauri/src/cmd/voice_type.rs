use super::audio::AudioDevice;
use once_cell::sync::Lazy;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

static HOLDING_KEYS: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static STREAM_ACTIVE: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
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
pub fn start_voice_capture(device: AudioDevice) {
    use enigo::{Enigo, Settings};
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
}

#[tauri::command]
pub fn stop_voice_capture() {}

#[tauri::command]
pub fn set_voice_type(val: bool) {
    HOLDING_KEYS.store(val, Ordering::Relaxed);
}
