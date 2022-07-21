use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::time::{Instant, Duration};

use rocket::{get, routes};
use rocket::response::stream::ByteStream;

use rodio::Decoder;
use rodio::source::{SineWave, Source, SamplesConverter};

#[get("/ping")]
fn ping() -> &'static str {
    "Pong"
}

#[get("/sine")]
fn sine() -> ByteStream![Vec<u8>] {
    ByteStream! {
        let mut source = SineWave::new(440.0);
        let mut vec: Vec<u8> = Vec::new();
        loop {
            match source.next() {
                Some(v) => {
                    vec.append(&mut v.to_be_bytes().to_vec());
                    // Sine wave is always 48khz
                    if vec.len() > 48000 {
                        yield std::mem::take(&mut vec);
                    }
                },
                None => break,
            };
        }
    }
}

#[get("/audio-file/decode-test")]
fn decode_test() -> ByteStream![Vec<u8>] {
    let hey_listen: BufReader<File>;
    let decoder: Decoder<BufReader<File>>;

    let path = "impact_prelude.mp3";


    match File::open(path) {
        Ok(v) => hey_listen = BufReader::new(v),
        Err(_) => panic!()
    };

    match Decoder::new_mp3(hey_listen) {
        Ok(v) => decoder = v,
        Err(_) => panic!()
    };

    let mut samples: SamplesConverter<Decoder<BufReader<File>>, f32> = decoder.convert_samples();

    let mut start = Instant::now();
    let total_time = Instant::now();
    let mut frame_count = 0;
    let mut vec: Vec<u8> = Vec::new();

    ByteStream! {
        loop {
            match samples.next() {
                Some(v) => {
                    vec.append(&mut v.to_be_bytes().to_vec());
                    // Stereo 44.1khz 
                    if vec.len() > 44100 * 2 {

                        println!("Processed 44.1k stereo frame in {:?}", start.elapsed());
                        frame_count += 1;
                        start = Instant::now();

                        yield std::mem::take(&mut vec);
                    }
                },
                None => break
            }
        }
        if vec.len() > 0 {
            let mut res = Vec::new();
            std::mem::swap(&mut res, &mut vec);

            println!("Processed remaining samples in {:?}", start.elapsed());
            frame_count += 1;

            yield res;
        }

        println!("Processed {:?} frames in {:?}", frame_count, total_time.elapsed());
    }

}

#[rocket::main]
async fn main() {
    let _server = rocket::build()
        .mount("/", routes![ping, sine, decode_test])
        .launch()
        .await;
}
