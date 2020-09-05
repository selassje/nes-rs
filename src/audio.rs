use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::time::Duration;
use crate::io_sdl::{SAMPLES_QUEUE, SampleFormat,SAMPLE_RATE, BUFFER_SIZE};

pub struct ApuOutput {
 
}

impl AudioCallback for ApuOutput {
    type Channel = SampleFormat;

    fn callback(&mut self, out: &mut [SampleFormat]) {
        let scale = std::u8::MAX / 15;
        let mut queue = SAMPLES_QUEUE.lock().unwrap();
        println!("Numer of sample in queue {}", queue.len());
      
        let mut sample_iter = queue.asc_iter();
        let mut requested_samples = 0;
        for x in out.iter_mut() {
            requested_samples+= 1;
         // *x = get_audio_samples() * scale;
         if let Some(sample) = sample_iter.next() {
             assert!(sample < &16);
            *x = *sample;
         }
         else {
          //   *x = 0;
         }
        
          //*x = 30000;  
          // println!("Sample is {}",*x);
        }
       queue.clear();
       //println!("Requested samples {}", requested_samples);
    }
}

pub struct Audio {
   pub device  : sdl2::audio::AudioDevice<ApuOutput>
}

impl Audio {

    pub fn new(audio_subsystem: &sdl2::AudioSubsystem) -> Self
    {
        let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(1),  
        samples: Some(BUFFER_SIZE as u16)
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            ApuOutput {          
            }}).unwrap();

        Audio {
          device: device,
        }
    }

    pub fn beep(&self) {
    self.device.resume();
    std::thread::sleep(Duration::from_millis(100));
    self.device.pause();
    }
}