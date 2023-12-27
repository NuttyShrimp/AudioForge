#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Header {
    volume: i16,          // in dB
    volume_curve: String, // distance attenuation curves
    volume_curve_distance: i16,
    doppler_factor: Option<i16>,
    category: String, // Should be prefixed with hash_ following a HEX hash when exporting to xml/dat
    attack_time: Option<i16>, // Fade-in time
    release_time: Option<i16>, // Fade-out time
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

impl Default for Header {
    fn default() -> Self {
        Self {
            volume: 100,
            volume_curve: "C2770146".to_string(),
            volume_curve_distance: 5,
            category: "02C7B342".to_string(),
            doppler_factor: Some(0),
            attack_time: Some(0),
            release_time: Some(0),
            unk20: 3,
            echo_x: Some(0),
            echo_y: Some(0),
            echo_z: Some(0),
        }
    }
}
