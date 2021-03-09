use crate::common;
use crate::io;

pub(super) const SAMPLING_RATE: usize = 44100;
const INITIAL_SAMPLE_BUCKET_SIZE: f32 =
    (common::DEFAULT_FPS as f32 * common::CPU_CYCLES_PER_FRAME as f32) / SAMPLING_RATE as f32;
pub(super) const BUFFER_SIZE: usize = 2000;

pub(super) struct AudioSampleBuffer {
    size: usize,
    sum: f32,
    bucket_size: f32,
    target_bucket_size: f32,
    buffer: [io::AudioSampleFormat; BUFFER_SIZE],
}

impl AudioSampleBuffer {
    pub fn new() -> Self {
        AudioSampleBuffer {
            size: 0,
            sum: 0.0,
            bucket_size: 0.0,
            buffer: [0.0; BUFFER_SIZE],
            target_bucket_size: INITIAL_SAMPLE_BUCKET_SIZE,
        }
    }

    pub fn add(&mut self, sample: io::AudioSampleFormat) {
        if 1.0 + self.bucket_size >= self.target_bucket_size && self.size < BUFFER_SIZE {
            let bucket_diff = self.target_bucket_size - self.bucket_size;
            let bucket_diff_comp = 1.0 - bucket_diff;
            self.sum += bucket_diff * sample;
            let target_sample = self.sum / self.target_bucket_size.floor();
            self.buffer[self.size] = target_sample;
            self.size += 1;
            self.sum = bucket_diff_comp * sample;
            self.bucket_size = bucket_diff_comp;
        } else {
            self.sum += sample;
            self.bucket_size += 1.0;
        }
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_byte_size(&self) -> usize {
        std::mem::size_of::<io::AudioSampleFormat>() * self.get_size()
    }

    pub fn get_samples(&self) -> &[io::AudioSampleFormat] {
        &self.buffer[..self.size]
    }

    pub fn reset(&mut self, fps: u16) {
        self.size = 0;
        self.sum = 0.0;
        self.bucket_size = 0.0;
        self.target_bucket_size =
            (fps as f32 * common::CPU_CYCLES_PER_FRAME as f32) / SAMPLING_RATE as f32;
    }
}
