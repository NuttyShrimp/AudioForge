#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Header {
    pub volume: u16,          // in dB
    pub volume_curve: String, // distance attenuation curves
    pub volume_curve_distance: u16,
    pub doppler_factor: Option<u16>,
    pub category: String, // Should be prefixed with hash_ following a HEX hash when exporting to xml/dat
    pub attack_time: Option<u16>, // Fade-in time
    pub release_time: Option<u16>, // Fade-out time
    pub unk20: u8,        // VirtualiseAsGroup - Stereo panning L-R?
    // - 0 = stereo
    // - 1 = left
    // - 2 = right
    // - 3 = both
    // - 4 = ? both + centre?
    // - 5 =
    // - 6 =
    pub echo_x: Option<u16>,
    pub echo_y: Option<u16>,
    pub echo_z: Option<u16>,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            volume: 100,
            volume_curve: "C2770146".to_string(),
            volume_curve_distance: 5,
            category: "02C7B342".to_string(),
            doppler_factor: None,
            attack_time: None,
            release_time: None,
            unk20: 0,
            echo_x: None,
            echo_y: None,
            echo_z: None,
        }
    }
}
