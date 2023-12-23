extern crate ffmpeg_next as ffmpeg;
use std::path::Path;

use anyhow::Result;
use ffmpeg_next::{codec, encoder, filter, format, frame, media, ChannelLayout};

fn filter(
    spec: &str,
    decoder: &codec::decoder::Audio,
    encoder: &codec::encoder::Audio,
) -> Result<filter::Graph, ffmpeg_next::Error> {
    let mut filter = filter::Graph::new();

    let args = format!(
        "time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
        decoder.time_base(),
        decoder.rate(),
        decoder.format().name(),
        decoder.channel_layout().bits()
    );

    filter.add(&filter::find("abuffer").unwrap(), "in", &args)?;
    filter.add(&filter::find("abuffersink").unwrap(), "out", "")?;

    {
        let mut out = filter.get("out").unwrap();

        out.set_sample_format(encoder.format());
        out.set_channel_layout(encoder.channel_layout());
        out.set_sample_rate(encoder.rate());
    }

    filter.output("in", 0)?.input("out", 0)?.parse(spec)?;
    filter.validate()?;

    if let Some(codec) = encoder.codec() {
        if !codec
            .capabilities()
            .contains(ffmpeg_next::codec::capabilities::Capabilities::VARIABLE_FRAME_SIZE)
        {
            filter
                .get("out")
                .unwrap()
                .sink()
                .set_frame_size(encoder.frame_size());
        }
    }

    Ok(filter)
}

// Output_dir should be of type: `awc/proj_name`
fn transcoder(
    ictx: &mut format::context::Input,
    octx: &mut format::context::Output,
    filter_spec: &str,
) -> Result<Transcoder, ffmpeg_next::Error> {
    let input = ictx
        .streams()
        .best(media::Type::Audio)
        .expect("could not find best audio stream");
    let context = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())?;

    let mut decoder = context.decoder().audio()?;
    decoder.set_parameters(input.parameters())?;

    let codec = encoder::find(codec::Id::PCM_S16LE)
        .expect("Could not find wanted output codec")
        .audio()?;
    let global = octx
        .format()
        .flags()
        .contains(ffmpeg_next::format::flag::Flags::GLOBAL_HEADER);

    // Output config
    octx.set_metadata(ictx.metadata().to_owned());
    let mut ost = octx.add_stream(codec)?;
    let context = ffmpeg_next::codec::context::Context::from_parameters(ost.parameters())?;
    let mut encoder = context.encoder().audio()?;

    if global {
        encoder.set_flags(ffmpeg_next::codec::flag::Flags::GLOBAL_HEADER);
    }

    // Set base encoder settings
    encoder.set_rate(decoder.rate() as i32);
    encoder.set_channel_layout(ChannelLayout::MONO);
    encoder.set_channels(ChannelLayout::MONO.channels());
    encoder.set_format(
        codec
            .formats()
            .expect("unknown supported formats")
            .next()
            .unwrap(),
    );
    encoder.set_bit_rate(decoder.bit_rate());
    encoder.set_max_bit_rate(decoder.max_bit_rate());
    encoder.set_time_base((1, decoder.rate() as i32));

    encoder.set_time_base((1, decoder.rate() as i32));
    ost.set_time_base((1, decoder.rate() as i32));
    let encoder = encoder.open_as(codec)?;
    ost.set_parameters(&encoder);

    let filter = filter(filter_spec, &decoder, &encoder)?;

    let in_time_base = decoder.time_base();
    let out_time_base = ost.time_base();

    Ok(Transcoder {
        stream: input.index(),
        decoder,
        encoder,
        in_time_base,
        out_time_base,
        filter,
    })
}

struct PannedTranscoder {
    pub transcoder: Transcoder,
    pub octx: format::context::Output,
}

pub fn split_stereo_to_mono(input: &Path, output_dir: &Path) -> Result<()> {
    let mut ictx = format::input(&input)?;
    let mut loctx = format::output(
        &output_dir
            .join(format!(
                "{}-left.wav",
                input.file_stem().unwrap().to_string_lossy()
            ))
            .as_path(),
    )?;
    let mut roctx = format::output(
        &output_dir
            .join(format!(
                "{}-right.wav",
                input.file_stem().unwrap().to_string_lossy()
            ))
            .as_path(),
    )?;

    let left_transcoder = transcoder(&mut ictx, &mut loctx, "pan=mono|c0=FL")?;
    let right_transcoder = transcoder(&mut ictx, &mut roctx, "pan=mono|c0=FR")?;

    loctx.write_header().unwrap();
    roctx.write_header().unwrap();

    let mut transcoders = vec![
        PannedTranscoder {
            transcoder: left_transcoder,
            octx: loctx,
        },
        PannedTranscoder {
            transcoder: right_transcoder,
            octx: roctx,
        },
    ];

    for (stream, mut packet) in ictx.packets() {
        for t in transcoders.as_mut_slice() {
            if stream.index() == t.transcoder.stream {
                packet.rescale_ts(stream.time_base(), t.transcoder.in_time_base);
                t.transcoder.send_packet_to_decoder(&packet);
                t.transcoder.receive_and_process_decoded_frames(&mut t.octx);
            }
        }
    }

    for t in transcoders.as_mut_slice() {
        t.transcoder.send_eof_to_decoder();
        t.transcoder.receive_and_process_decoded_frames(&mut t.octx);

        t.transcoder.flush_filter();
        t.transcoder.get_and_process_filtered_frames(&mut t.octx);

        t.transcoder.send_eof_to_encoder();
        t.transcoder
            .receive_and_process_encoded_packets(&mut t.octx);

        t.octx.write_trailer().unwrap();
    }
    Ok(())
}

struct Transcoder {
    pub stream: usize,
    decoder: codec::decoder::Audio,
    encoder: codec::encoder::Audio,
    pub in_time_base: ffmpeg_next::Rational,
    out_time_base: ffmpeg_next::Rational,
    filter: filter::Graph,
}

impl Transcoder {
    fn send_frame_to_encoder(&mut self, frame: &ffmpeg_next::Frame) {
        self.encoder.send_frame(frame).unwrap();
    }

    fn send_eof_to_encoder(&mut self) {
        self.encoder.send_eof().unwrap();
    }

    fn receive_and_process_encoded_packets(&mut self, octx: &mut format::context::Output) {
        let mut encoded = ffmpeg_next::Packet::empty();
        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(0);
            encoded.rescale_ts(self.in_time_base, self.out_time_base);
            encoded.write_interleaved(octx).unwrap();
        }
    }
    fn add_frame_to_filter(&mut self, frame: &ffmpeg_next::Frame) {
        self.filter.get("in").unwrap().source().add(frame).unwrap();
    }

    fn flush_filter(&mut self) {
        self.filter.get("in").unwrap().source().flush().unwrap();
    }

    fn get_and_process_filtered_frames(&mut self, octx: &mut format::context::Output) {
        let mut filtered = frame::Audio::empty();
        while self
            .filter
            .get("out")
            .unwrap()
            .sink()
            .frame(&mut filtered)
            .is_ok()
        {
            self.send_frame_to_encoder(&filtered);
            self.receive_and_process_encoded_packets(octx);
        }
    }

    fn send_packet_to_decoder(&mut self, packet: &ffmpeg_next::Packet) {
        self.decoder.send_packet(packet).unwrap();
    }

    fn send_eof_to_decoder(&mut self) {
        self.decoder.send_eof().unwrap();
    }

    fn receive_and_process_decoded_frames(&mut self, octx: &mut format::context::Output) {
        let mut frame = frame::Audio::empty();
        while self.decoder.receive_frame(&mut frame).is_ok() {
            let timestamp = frame.timestamp();
            frame.set_pts(timestamp);
            self.add_frame_to_filter(&frame);
            self.get_and_process_filtered_frames(octx);
        }
    }
}
