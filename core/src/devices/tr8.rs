use super::DeviceInterface;
use heapless::String;

#[derive(Clone)]
pub struct TR8 {
    running: bool,
}

impl TR8 {
    pub fn init() -> Self {
        Self { running: false }
    }
}

impl DeviceInterface for TR8 {
    fn run(&mut self) {
        self.running = true;
    }

    fn stop(&mut self) {
        self.running = false;
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn name_to_str(&self) -> String<64> {
        String::try_from("TR-8").unwrap()
    }

    fn manufacturer_to_str(&self) -> String<64> {
        String::try_from("Roland").unwrap()
    }
}

/// (number: u8, name: &'static str)
type RichMidiCC = (u8, &'static str);

pub const TR_8_INTRUMENTS: usize = 11;
pub const TR_8_STEPS: usize = TR_8_INTRUMENTS + 5;
pub const TR_8_PARAM_ELEMS: usize = TR_8_INTRUMENTS + 2;

// relevant notes to check

pub const TR_8_NOTES: [u8; TR_8_STEPS] = [
    36, // BD
    38, // SD
    43, // LT
    47, // MT
    50, // HT
    37, // RS
    39, // HC
    42, // CH
    46, // OH
    49, // CC
    51, // RC
    //
    // | these might be unsupported if 727 update has not been flashed on device
    // v
    35, // BD2
    40, // SD2
    56, // CB
    54, // TB
    0,  // unused
];

// relevant CC numbers to check

pub const TR_8_CC_FADER: [RichMidiCC; TR_8_INTRUMENTS] = [
    (24, "BD"),
    (29, "SD"),
    (48, "LT"),
    (51, "MT"),
    (54, "HT"),
    (57, "RS"),
    (60, "HC"),
    (63, "CH"),
    (82, "OH"),
    (85, "CC"),
    (88, "RC"),
];

pub const TR_8_CC_PARAMS_1ST_ROW: [RichMidiCC; TR_8_PARAM_ELEMS] = [
    (20, "TUNE"),   // BD
    (21, "ATTACK"), // BD
    (25, "TUNE"),   // SD
    (26, "SNAPPY"), // SD
    (46, "TUNE"),   // LT
    (49, "TUNE"),   // MT
    (52, "TUNE"),   // HT
    (55, "TUNE"),   // RS
    (58, "TUNE"),   // HC
    (61, "TUNE"),   // CH
    (80, "TUNE"),   // OH
    (83, "TUNE"),   // CC
    (86, "TUNE"),   // RC
];

pub const TR_8_CC_PARAMS_2ND_ROW: [RichMidiCC; TR_8_PARAM_ELEMS] = [
    (22, "COMP"),  // BD
    (23, "DECAY"), // BD
    (27, "COMP"),  // SD
    (28, "DECAY"), // SD
    (47, "DECAY"), // LT
    (50, "DECAY"), // MT
    (53, "DECAY"), // HT
    (56, "DECAY"), // RS
    (59, "DECAY"), // HC
    (62, "DECAY"), // CH
    (81, "DECAY"), // OH
    (84, "DECAY"), // CC
    (87, "DECAY"), // RC
];
