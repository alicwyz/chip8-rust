use rodio::{
    Sink,
    OutputStream, 
    OutputStreamBuilder
};

pub struct Audio {
    sink: Sink,
    _stream: OutputStream
}

impl Audio {
    // New audio
    pub fn new() -> Result<Audio, String> {
        
        let stream_handle: OutputStream = match OutputStreamBuilder::open_default_stream() {
            Ok(v) => v,
            Err(err) => {return Err(err.to_string());}
        };

        let sink: Sink = Sink::connect_new(&stream_handle.mixer());

        // Sinewave
        sink.append(rodio::source::SineWave::new(440.0));
        sink.pause();

        Ok(Audio{sink, _stream: stream_handle})
    }

    //Play or pause audio
    pub fn play(&self) {
        self.sink.play();
    }
    pub fn pause(&self) {
        self.sink.pause();
    }
}