#[derive(Default, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Header {
    volume: i16, // in dB
    volume_curve: String,
    attack_time: Option<i16>,  // Fade-in time
    release_time: Option<i16>, // Fade-out time
    doppler_factor: Option<i16>,
    category: String, // Should be prefixed with hash_ following a HEX hash when exporting to xml/dat
    unk20: u8,        // VirtualiseAsGroup - Stereo panning L-R?
    // - 0 = stereo
    // - 1 = left
    // - 2 = right
    // - 3 = both
    // - 4 = ? both + centre?
    // - 5 =
    // - 6 =
    echo_x: Option<i16>,
    echo_y: Option<i16>,
    echo_z: Option<i16>,
}
