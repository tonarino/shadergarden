use std::path::Path;

use ffmpeg_next::{
    format::{
        input,
        Pixel,
    },
    media::Type,
    software::scaling::{
        context::Context,
        flag::Flags,
    },
    util::frame::video::Video,
};
use glium::{
    backend::Facade,
    texture::RawImage2d,
    Texture2d,
};

pub struct FrameStream {
    frames: Vec<Texture2d>,
    index:  usize,
}

impl FrameStream {
    pub fn new<F: Facade>(
        filename: &Path,
        width: u32,
        height: u32,
        facade: &F,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        ffmpeg_next::init().unwrap();
        let mut ictx = input(&filename)?;

        let input = ictx.streams().best(Type::Video).expect("no video stream");
        let video_stream_index = input.index();
        let mut decoder = input.codec().decoder().video()?;

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            width,
            height,
            Flags::BILINEAR,
        )?;

        let mut frames = vec![];
        for (stream, packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                decoder.send_packet(&packet)?;
                let mut decoded = Video::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    scaler
                        .run(&decoded, &mut rgb_frame)
                        .expect("scaler failed");
                    let data = rgb_frame.data(0);
                    let raw_image = RawImage2d::from_raw_rgb_reversed(
                        data,
                        (width, height),
                    );
                    frames.push(Texture2d::new(facade, raw_image).unwrap());
                }
            }
        }
        decoder.send_eof()?;
        Ok(Self { frames, index: 0 })
    }

    pub fn next_frame(&mut self) -> &Texture2d {
        let frame = &self.frames[self.index % self.frames.len()];
        self.index += 1;
        frame
    }
}
