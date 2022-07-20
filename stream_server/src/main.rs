use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use rocket::{get, routes};
use rocket::tokio::time::{self, Duration};
use rocket::response::stream::{ByteStream, TextStream};

use rodio::Decoder;
use rodio::source::{SineWave, Source, SamplesConverter};


#[get("/ping")]
fn ping() -> &'static str {
    "Pong"
}

#[get("/infinite-hellos")]
fn hello() -> TextStream![&'static str] {
    TextStream! {
        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            yield "hello";
            interval.tick().await;
        }
    }
}

#[get("/byte-stream")]
fn bytes() -> ByteStream![Vec<u8>] {
    ByteStream! {
        let mut interval = time::interval(Duration::from_secs(1));
        for i in 0..10u8 {
            yield vec![i, i+1, i+2];
            interval.tick().await;
        }
    }
}

#[get("/sine")]
fn sine() -> ByteStream![Vec<u8>] {
    ByteStream! {
        let mut source = SineWave::new(440.0).take_duration(Duration::from_secs(1)).repeat_infinite().buffered();
        loop {
            match source.next() {
                Some(v) => {
                    let bytes = v.to_be_bytes().to_vec();
                    println!("{:?}", bytes);
                    yield bytes;
                },
                None => break,
            };
        }
    }
}

#[get("/sine-text")]
fn sine_text() -> TextStream![String] {
    TextStream! {
        let mut source = SineWave::new(440.0).take_duration(Duration::from_secs(1)).buffered();
        loop {
            match source.next() {
                Some(v) => {
                    let bytes = v.to_string();
                    println!("{:?}", bytes);
                    yield bytes;
                },
                None => break,
            };
        }
    }
}


#[get("/byte/stream")]
fn byte_stream() -> ByteStream![Vec<u8>] {
    ByteStream! {
        let mut interval = time::interval(Duration::from_secs(1));
        for i in 0..10u8 {
            let bytes = vec![i, i + 1, i + 2];
            println!("{:?}", bytes);
            yield bytes;
            interval.tick().await;
        }
    }
}

#[get("/audio-file/decode-test")]
fn decode_test() -> String {
    let hey_listen: BufReader<File>;
    let decoder: Decoder<BufReader<File>>;

    let path = "shame.mp3";


    match File::open(path) {
        Ok(v) => hey_listen = BufReader::new(v),
        Err(_) => panic!()
    };

    match Decoder::new_mp3(hey_listen) {
        Ok(v) => decoder = v,
        Err(_) => panic!()
    };

    let mut samples: SamplesConverter<Decoder<BufReader<File>>, f32> = decoder.convert_samples();

    let start = Instant::now();
    let mut sample_count = 0;
    let mut frame_count = 0;

    loop {
        match samples.next() {
            Some(v) => {
                sample_count += 1;
                if sample_count > 44100 * 2 {
                    frame_count += 1;
                    sample_count = 0;
                }
            },
            None => break
        }
    }

    format!("Decoded file ({:?}) in {:?}. ({:?} frames and {:?} samples)", path, start.elapsed(), frame_count, sample_count)

}

#[get("/audio-file/stream")]
fn audio_file() -> ByteStream![Vec<u8>] {
    let hey_listen: BufReader<File>;
    let mut decoder: Decoder<BufReader<File>>;


    match File::open("impact_prelude.mp3") {
        Ok(v) => hey_listen = BufReader::new(v),
        Err(_) => panic!()
    };

    match Decoder::new_mp3(hey_listen) {
        Ok(v) => decoder = v,
        Err(_) => panic!()
    };

    let mut samples: SamplesConverter<Decoder<BufReader<File>>, f32> = decoder.convert_samples();

    let mut frame_count = 0;
    let mut start = Instant::now();

    ByteStream! {
        loop {
            match samples.next() {
                Some(v) => {
                    frame_count += 1;
                    if frame_count >= 44100 * 2 {
                        println!("44.1k frame sent in {:?}", start.elapsed());
                        start = Instant::now();
                        frame_count = 0;
                    }
                    yield v.to_be_bytes().to_vec();
                }
                None => break
            };
        }
    }

}

#[rocket::main]
async fn main() {
    let _server = rocket::build()
        .mount("/", routes![ping, hello, bytes, sine, sine_text, byte_stream, audio_file, decode_test])
        .launch()
        .await;
}
