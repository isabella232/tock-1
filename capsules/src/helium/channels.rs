// Channels 902-928
// 0x0386 ... 0x039F
//
//         906-908 ~ 40k dev
// | Frequency | CF Value | Fract Freq | 
// | 906.00000 | 0x038A   | 0x0000     |
// | 906.03986 | 0x038A   | 0x0A34     |
// | 906.07970 | 0x038A   | 0x1467     |
// | 906.12032 | 0x038A   | 0x1ECD     |
// | 906.16016 | 0x038A   | 0x2900     |
// | 906.20001 | 0x038A   | 0x3334     |
// | 906.23987 | 0x038A   | 0x3D67     |
// | 906.27969 | 0x038A   | 0x479A     |
// | 906.32031 | 0x038A   | 0x5200     |
// | 906.36017 | 0x038A   | 0x5C34     |
// | 906.37970 | 0x038A   | 0x6134     |
// | 906.42032 | 0x038A   | 0x6B9A     |
// | 906.46016 | 0x038A   | 0x75CD     |
// | 906.50000 | 0x038A   | 0x8000     |
// | 906.53986 | 0x038A   | 0x8A34     |
// | 906.57970 | 0x038A   | 0x8467     |
// | 906.62032 | 0x038A   | 0x9ECD     |
// | 906.66016 | 0x038A   | 0xA900     |
// | 906.70001 | 0x038A   | 0xB334     |
// | 906.73987 | 0x038A   | 0xBD67     |
// | 906.77969 | 0x038A   | 0xC79A     |
// | 906.82031 | 0x038A   | 0xD200     |
// | 906.86017 | 0x038A   | 0xDC34     |
// | 906.87970 | 0x038A   | 0xE134     |
// | 906.92032 | 0x038A   | 0xEB9A     |
// | 906.96016 | 0x038A   | 0xF5CD     |
// | 907.0000  | 0x038B   | 0x0000     |
// | 907.03986 | 0x038B   | 0x0A34     |
// | 907.07970 | 0x038B   | 0x1467     |
// | 907.12032 | 0x038B   | 0x1ECD     |
// | 907.16016 | 0x038B   | 0x2900     |
// | 907.20001 | 0x038B   | 0x3334     |
// | 907.23987 | 0x038B   | 0x3D67     |
// | 907.27969 | 0x038B   | 0x479A     |
// | 907.32031 | 0x038B   | 0x5200     |
// | 907.36017 | 0x038B   | 0x5C34     |
// | 907.37970 | 0x038B   | 0x6134     |
// | 907.42032 | 0x038B   | 0x6B9A     |
// | 907.46016 | 0x038B   | 0x75CD     |
// | 907.50000 | 0x038B   | 0x8000     |
// | 907.53986 | 0x038B   | 0x8A34     |
// | 907.57970 | 0x038B   | 0x8467     |
// | 907.62032 | 0x038B   | 0x9ECD     |
// | 907.66016 | 0x038B   | 0xA900     |
// | 907.70001 | 0x038B   | 0xB334     |
// | 907.73987 | 0x038B   | 0xBD67     |
// | 907.77969 | 0x038B   | 0xC79A     |
// | 907.82031 | 0x038B   | 0xD200     |
// | 907.86017 | 0x038B   | 0xDC34     |
// | 907.87970 | 0x038B   | 0xE134     |
// | 907.92032 | 0x038B   | 0xEB9A     |
// | 907.96016 | 0x038B   | 0xF5CD     |

pub const FREQ_DEVIATIONS: [u16; 26] = [
    0x0000, 0x0A34, 0x1467, 0x1ECD, 0x2900, 0x3334, 0x3D67, 0x479A, 0x5200, 
    0x5C34, 0x6134, 0x6B9A, 0x75CD, 0x8000, 0x8A34, 0x8467, 0x9ECD, 0xA900, 
    0xB334, 0xBD67, 0xC79A, 0xD200, 0xDC34, 0xE134, 0xEB9A, 0xF5CD,
];

#[derive(Copy, Clone, Default)]
pub struct ChannelParams {
    pub center_frequency: u16,
    pub frequency: u16,
    pub fract_freq: u16,
}

impl ChannelParams {
    pub fn new(cf: u16) -> ChannelParams {
        ChannelParams {
            center_frequency: cf,
            frequency: 0,
            fract_freq: 0,
        }
    }

    pub fn hop(&self) -> ChannelParams { 
        ChannelParams {
            center_frequency: self.center_frequency,
            frequency: 0,
            fract_freq: 0,
        }
    } 
}
