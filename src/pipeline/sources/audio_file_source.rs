use std::env;
use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;


pub struct AudioFileSource {
    
}

impl AudioFileSource {
    pub fn new(filename: &str) -> AudioFileSource {
        let file = Box::new(File::open(Path::new(filename)).unwrap());
        let mss = MediaSourceStream::new(file, Default::default());
        let hint = Hint::new();

        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        // Probe the media source stream for a format.
        let probed =
            symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts).unwrap();
        
        let mut format = probed.format;
        let track = format.default_track().unwrap();
        
        let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &decoder_opts).unwrap();
        let track_id = track.id;
        
        let mut sample_count = 0;
        let mut sample_buf = None;
    }
}