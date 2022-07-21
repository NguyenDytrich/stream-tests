use std::mem;
use std::time::{Duration, Instant};

use futures_util::stream::StreamExt;
use rodio::{OutputStream, Sink};
use rodio::source::Source;

struct ByteSource {
    pub sample_rate: u32,
    pub channels: u16,
    pub data: Vec<u8>,
}

impl Iterator for ByteSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {

        if self.data.len() < 4 {
            return None
        }

        let bytes = take_byte(&mut self.data);
        match bytes {
            Some(v) => Some(f32::from_be_bytes(v)),
            None => None
        }
    }

}

impl Source for ByteSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.data.len())
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        return None
    }
}

fn take_byte(bits: &mut Vec<u8>) -> Option<[u8; 4]> {
    let mut byte: [u8; 4] = [0; 4];

    if bits.len() < 4 {
        return None
    }

    for i in 0..4 {
        match bits.pop() {
            Some(v) => {
                byte[i] = v;
            },
            None => return None,
        }
    };

    // println!("{:?}", byte);

    Some(byte)
}

#[tokio::main]
async fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // let mut stream = reqwest::get("http://localhost:8000/audio-file/decode-test").await.unwrap().bytes_stream();
    let mut stream = reqwest::get("http://localhost:8000/sine").await.unwrap().bytes_stream();
    let mut frames: Vec<u8> = Vec::new();
    let mut total_frames = 0;

    sink.pause();
    let mut start = Instant::now();

    while let Some(item) = stream.next().await {
        let mut vec = item.unwrap().to_vec();

        // TODO: Enqueue these instead of append
        frames.append(&mut vec);

        if frames.len() >= 44100 * 8 {
            let mut sample = ByteSource{
                channels: 2, 
                sample_rate: 44100,
                data: mem::take(&mut frames),
            };

            // TODO:: Enqueue instead of append
            sample.data.reverse();
            // sink.append(sample);
            sink.append(sample);

            total_frames += 1;

            println!("Loaded 44.1k frame in {:?}", start.elapsed());
            println!("Total samples in sink {:?}", sink.len());
            start = Instant::now();
        }

        // Buffer frames to the sink
        if total_frames > 3 && sink.is_paused() {
            sink.play();
        }
    }

    println!("{}", frames.len());

    if frames.len() > 0 {
        let mut sample = ByteSource {
            channels: 1,
            sample_rate: 44100,
            data: frames.clone(),
        };

        sample.data.reverse();
    
        sink.append(sample);
        sink.play();
    }

    sink.sleep_until_end();
}
