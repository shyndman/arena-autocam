use palette::Srgba;

pub enum MaterialColorPalette {
    Red,
    Pink,
    Purple,
    DeepPurple,
    Indigo,
    Blue,
    LightBlue,
    Cyan,
    Teal,
    Green,
    LightGreen,
    Lime,
    Yellow,
    Amber,
    Orange,
    DeepOrange,
    Brown,
    BlueGrey,
}

impl core::ops::Deref for MaterialColorPalette {
    type Target = MaterialColorShades;

    fn deref(&self) -> &Self::Target {
        match self {
            MaterialColorPalette::Red => &RED,
            MaterialColorPalette::Pink => &PINK,
            MaterialColorPalette::Purple => &PURPLE,
            MaterialColorPalette::DeepPurple => &DEEP_PURPLE,
            MaterialColorPalette::Indigo => &INDIGO,
            MaterialColorPalette::Blue => &BLUE,
            MaterialColorPalette::LightBlue => &LIGHT_BLUE,
            MaterialColorPalette::Cyan => &CYAN,
            MaterialColorPalette::Teal => &TEAL,
            MaterialColorPalette::Green => &GREEN,
            MaterialColorPalette::LightGreen => &LIGHT_GREEN,
            MaterialColorPalette::Lime => &LIME,
            MaterialColorPalette::Yellow => &YELLOW,
            MaterialColorPalette::Amber => &AMBER,
            MaterialColorPalette::Orange => &ORANGE,
            MaterialColorPalette::DeepOrange => &DEEP_ORANGE,
            MaterialColorPalette::Brown => &BROWN,
            MaterialColorPalette::BlueGrey => &BLUE_GREY,
        }
    }
}

pub enum MaterialAccentPalette {
    RedAccent,
    PinkAccent,
    PurpleAccent,
    DeepPurpleAccent,
    IndigoAccent,
    BlueAccent,
    LightBlueAccent,
    CyanAccent,
    TealAccent,
    GreenAccent,
    LightGreenAccent,
    LimeAccent,
    YellowAccent,
    AmberAccent,
    OrangeAccent,
    DeepOrangeAccent,
}

impl core::ops::Deref for MaterialAccentPalette {
    type Target = MaterialAccentShades;

    fn deref(&self) -> &Self::Target {
        match self {
            MaterialAccentPalette::RedAccent => &RED_ACCENT,
            MaterialAccentPalette::PinkAccent => &PINK_ACCENT,
            MaterialAccentPalette::PurpleAccent => &PURPLE_ACCENT,
            MaterialAccentPalette::DeepPurpleAccent => &DEEP_PURPLE_ACCENT,
            MaterialAccentPalette::IndigoAccent => &INDIGO_ACCENT,
            MaterialAccentPalette::BlueAccent => &BLUE_ACCENT,
            MaterialAccentPalette::LightBlueAccent => &LIGHT_BLUE_ACCENT,
            MaterialAccentPalette::CyanAccent => &CYAN_ACCENT,
            MaterialAccentPalette::TealAccent => &TEAL_ACCENT,
            MaterialAccentPalette::GreenAccent => &GREEN_ACCENT,
            MaterialAccentPalette::LightGreenAccent => &LIGHT_GREEN_ACCENT,
            MaterialAccentPalette::LimeAccent => &LIME_ACCENT,
            MaterialAccentPalette::YellowAccent => &YELLOW_ACCENT,
            MaterialAccentPalette::AmberAccent => &AMBER_ACCENT,
            MaterialAccentPalette::OrangeAccent => &ORANGE_ACCENT,
            MaterialAccentPalette::DeepOrangeAccent => &DEEP_ORANGE_ACCENT,
        }
    }
}

pub struct MaterialColorShades {
    primary: Srgba<u8>,
    shades: &'static [Srgba<u8>],
}

impl MaterialColorShades {
    pub const fn new(primary: Srgba<u8>, shades: &'static [Srgba<u8>]) -> Self {
        Self { primary, shades }
    }

    pub fn primary(&self) -> Srgba<u8> {
        self.primary
    }

    pub fn shade50(&self) -> Srgba<u8> {
        self.shades[0]
    }

    pub fn shade100(&self) -> Srgba<u8> {
        self.shades[1]
    }

    pub fn shade200(&self) -> Srgba<u8> {
        self.shades[2]
    }

    pub fn shade300(&self) -> Srgba<u8> {
        self.shades[3]
    }

    pub fn shade400(&self) -> Srgba<u8> {
        self.shades[4]
    }

    pub fn shade500(&self) -> Srgba<u8> {
        self.shades[5]
    }

    pub fn shade600(&self) -> Srgba<u8> {
        self.shades[6]
    }

    pub fn shade700(&self) -> Srgba<u8> {
        self.shades[7]
    }

    pub fn shade800(&self) -> Srgba<u8> {
        self.shades[8]
    }

    pub fn shade900(&self) -> Srgba<u8> {
        self.shades[9]
    }
}

pub struct MaterialAccentShades {
    primary: Srgba<u8>,
    shades: &'static [Srgba<u8>],
}

impl MaterialAccentShades {
    const fn new(primary: Srgba<u8>, shades: &'static [Srgba<u8>]) -> Self {
        Self { primary, shades }
    }

    pub fn primary(&self) -> Srgba<u8> {
        self.primary
    }

    pub fn shade100(&self) -> Srgba<u8> {
        self.shades[1]
    }

    pub fn shade200(&self) -> Srgba<u8> {
        self.shades[2]
    }

    pub fn shade400(&self) -> Srgba<u8> {
        self.shades[4]
    }

    pub fn shade700(&self) -> Srgba<u8> {
        self.shades[7]
    }
}

const fn srgba_from_argb_u32(value: u32) -> Srgba<u8> {
    let b = (value & 0xFF) as u8;
    let g = (value >> 8 & 0xFF) as u8;
    let r = (value >> 16 & 0xFF) as u8;
    let a = (value >> 24 & 0xFF) as u8;
    Srgba::<u8>::new(r, g, b, a)
}

pub const TRANSPARENT: Srgba<u8> = srgba_from_argb_u32(0x00000000);
pub const BLACK: Srgba<u8> = srgba_from_argb_u32(0xFF000000);
pub const BLACK87: Srgba<u8> = srgba_from_argb_u32(0xDD000000);
pub const BLACK54: Srgba<u8> = srgba_from_argb_u32(0x8A000000);
pub const BLACK45: Srgba<u8> = srgba_from_argb_u32(0x73000000);
pub const BLACK38: Srgba<u8> = srgba_from_argb_u32(0x61000000);
pub const BLACK26: Srgba<u8> = srgba_from_argb_u32(0x42000000);
pub const BLACK12: Srgba<u8> = srgba_from_argb_u32(0x1F000000);
pub const WHITE: Srgba<u8> = srgba_from_argb_u32(0xFFFFFFFF);
pub const WHITE70: Srgba<u8> = srgba_from_argb_u32(0xB3FFFFFF);
pub const WHITE60: Srgba<u8> = srgba_from_argb_u32(0x99FFFFFF);
pub const WHITE54: Srgba<u8> = srgba_from_argb_u32(0x8AFFFFFF);
pub const WHITE38: Srgba<u8> = srgba_from_argb_u32(0x62FFFFFF);
pub const WHITE30: Srgba<u8> = srgba_from_argb_u32(0x4DFFFFFF);
pub const WHITE24: Srgba<u8> = srgba_from_argb_u32(0x3DFFFFFF);
pub const WHITE12: Srgba<u8> = srgba_from_argb_u32(0x1FFFFFFF);

pub const WHITE10: Srgba<u8> = srgba_from_argb_u32(0x1AFFFFFF);

pub const RED: MaterialColorShades = MaterialColorShades::new(
    RED_PRIMARY,
    &[
        srgba_from_argb_u32(0xFFFFEBEE), // 50
        srgba_from_argb_u32(0xFFFFCDD2), // 100
        srgba_from_argb_u32(0xFFEF9A9A), // 200
        srgba_from_argb_u32(0xFFE57373), // 300
        srgba_from_argb_u32(0xFFEF5350), // 400
        RED_PRIMARY,                     // 500
        srgba_from_argb_u32(0xFFE53935), // 600
        srgba_from_argb_u32(0xFFD32F2F), // 700
        srgba_from_argb_u32(0xFFC62828), // 800
        srgba_from_argb_u32(0xFFB71C1C), // 900
    ],
);
const RED_PRIMARY: Srgba<u8> = srgba_from_argb_u32(0xFFF44336);

pub const RED_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _RED_ACCENT_VALUE,
    &[
        srgba_from_argb_u32(0xFFFF8A80), // 100
        _RED_ACCENT_VALUE,               // 200
        srgba_from_argb_u32(0xFFFF1744), // 400
        srgba_from_argb_u32(0xFFD50000), // 700
    ],
);
const _RED_ACCENT_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFF5252);

pub const PINK: MaterialColorShades = MaterialColorShades::new(
    _PINK_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFCE4EC), // 50
        srgba_from_argb_u32(0xFFF8BBD0), // 100
        srgba_from_argb_u32(0xFFF48FB1), // 200
        srgba_from_argb_u32(0xFFF06292), // 300
        srgba_from_argb_u32(0xFFEC407A), // 400
        _PINK_PRIMARY_VALUE,             // 500
        srgba_from_argb_u32(0xFFD81B60), // 600
        srgba_from_argb_u32(0xFFC2185B), // 700
        srgba_from_argb_u32(0xFFAD1457), // 800
        srgba_from_argb_u32(0xFF880E4F), // 900
    ],
);
const _PINK_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFE91E63);

pub const PINK_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _PINK_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFF80AB), // 100
        _PINK_ACCENT_PRIMARY_VALUE,      // 200
        srgba_from_argb_u32(0xFFF50057), // 400
        srgba_from_argb_u32(0xFFC51162), // 700
    ],
);
const _PINK_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFF4081);

pub const PURPLE: MaterialColorShades = MaterialColorShades::new(
    _PURPLE_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFF3E5F5), // 50
        srgba_from_argb_u32(0xFFE1BEE7), // 100
        srgba_from_argb_u32(0xFFCE93D8), // 200
        srgba_from_argb_u32(0xFFBA68C8), // 300
        srgba_from_argb_u32(0xFFAB47BC), // 400
        _PURPLE_PRIMARY_VALUE,           // 500
        srgba_from_argb_u32(0xFF8E24AA), // 600
        srgba_from_argb_u32(0xFF7B1FA2), // 700
        srgba_from_argb_u32(0xFF6A1B9A), // 800
        srgba_from_argb_u32(0xFF4A148C), // 900
    ],
);
const _PURPLE_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF9C27B0);

pub const PURPLE_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _PURPLE_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFEA80FC), // 100
        _PURPLE_ACCENT_PRIMARY_VALUE,    // 200
        srgba_from_argb_u32(0xFFD500F9), // 400
        srgba_from_argb_u32(0xFFAA00FF), // 700
    ],
);
const _PURPLE_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFE040FB);

pub const DEEP_PURPLE: MaterialColorShades = MaterialColorShades::new(
    _DEEP_PURPLE_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFEDE7F6), // 50
        srgba_from_argb_u32(0xFFD1C4E9), // 100
        srgba_from_argb_u32(0xFFB39DDB), // 200
        srgba_from_argb_u32(0xFF9575CD), // 300
        srgba_from_argb_u32(0xFF7E57C2), // 400
        _DEEP_PURPLE_PRIMARY_VALUE,      // 500
        srgba_from_argb_u32(0xFF5E35B1), // 600
        srgba_from_argb_u32(0xFF512DA8), // 700
        srgba_from_argb_u32(0xFF4527A0), // 800
        srgba_from_argb_u32(0xFF311B92), // 900
    ],
);
const _DEEP_PURPLE_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF673AB7);

pub const DEEP_PURPLE_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _DEEP_PURPLE_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFB388FF),   // 100
        _DEEP_PURPLE_ACCENT_PRIMARY_VALUE, // 200
        srgba_from_argb_u32(0xFF651FFF),   // 400
        srgba_from_argb_u32(0xFF6200EA),   // 700
    ],
);
const _DEEP_PURPLE_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF7C4DFF);

pub const INDIGO: MaterialColorShades = MaterialColorShades::new(
    _INDIGO_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFE8EAF6), // 50
        srgba_from_argb_u32(0xFFC5CAE9), // 100
        srgba_from_argb_u32(0xFF9FA8DA), // 200
        srgba_from_argb_u32(0xFF7986CB), // 300
        srgba_from_argb_u32(0xFF5C6BC0), // 400
        _INDIGO_PRIMARY_VALUE,           // 500
        srgba_from_argb_u32(0xFF3949AB), // 600
        srgba_from_argb_u32(0xFF303F9F), // 700
        srgba_from_argb_u32(0xFF283593), // 800
        srgba_from_argb_u32(0xFF1A237E), // 900
    ],
);
const _INDIGO_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF3F51B5);

pub const INDIGO_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _INDIGO_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFF8C9EFF), // 100
        _INDIGO_ACCENT_PRIMARY_VALUE,    // 200
        srgba_from_argb_u32(0xFF3D5AFE), // 400
        srgba_from_argb_u32(0xFF304FFE), // 700
    ],
);
const _INDIGO_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF536DFE);

pub const BLUE: MaterialColorShades = MaterialColorShades::new(
    _BLUE_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFE3F2FD), // 50
        srgba_from_argb_u32(0xFFBBDEFB), // 100
        srgba_from_argb_u32(0xFF90CAF9), // 200
        srgba_from_argb_u32(0xFF64B5F6), // 300
        srgba_from_argb_u32(0xFF42A5F5), // 400
        _BLUE_PRIMARY_VALUE,             // 500
        srgba_from_argb_u32(0xFF1E88E5), // 600
        srgba_from_argb_u32(0xFF1976D2), // 700
        srgba_from_argb_u32(0xFF1565C0), // 800
        srgba_from_argb_u32(0xFF0D47A1), // 900
    ],
);
const _BLUE_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF2196F3);

pub const BLUE_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _BLUE_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFF82B1FF), // 100
        _BLUE_ACCENT_PRIMARY_VALUE,      // 200
        srgba_from_argb_u32(0xFF2979FF), // 400
        srgba_from_argb_u32(0xFF2962FF), // 700
    ],
);
const _BLUE_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF448AFF);

pub const LIGHT_BLUE: MaterialColorShades = MaterialColorShades::new(
    _LIGHT_BLUE_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFE1F5FE), // 50
        srgba_from_argb_u32(0xFFB3E5FC), // 100
        srgba_from_argb_u32(0xFF81D4FA), // 200
        srgba_from_argb_u32(0xFF4FC3F7), // 300
        srgba_from_argb_u32(0xFF29B6F6), // 400
        _LIGHT_BLUE_PRIMARY_VALUE,       // 500
        srgba_from_argb_u32(0xFF039BE5), // 600
        srgba_from_argb_u32(0xFF0288D1), // 700
        srgba_from_argb_u32(0xFF0277BD), // 800
        srgba_from_argb_u32(0xFF01579B), // 900
    ],
);
const _LIGHT_BLUE_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF03A9F4);

pub const LIGHT_BLUE_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _LIGHT_BLUE_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFF80D8FF),  // 100
        _LIGHT_BLUE_ACCENT_PRIMARY_VALUE, // 200
        srgba_from_argb_u32(0xFF00B0FF),  // 400
        srgba_from_argb_u32(0xFF0091EA),  // 700
    ],
);
const _LIGHT_BLUE_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF40C4FF);

pub const CYAN: MaterialColorShades = MaterialColorShades::new(
    _CYAN_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFE0F7FA), // 50
        srgba_from_argb_u32(0xFFB2EBF2), // 100
        srgba_from_argb_u32(0xFF80DEEA), // 200
        srgba_from_argb_u32(0xFF4DD0E1), // 300
        srgba_from_argb_u32(0xFF26C6DA), // 400
        _CYAN_PRIMARY_VALUE,             // 500
        srgba_from_argb_u32(0xFF00ACC1), // 600
        srgba_from_argb_u32(0xFF0097A7), // 700
        srgba_from_argb_u32(0xFF00838F), // 800
        srgba_from_argb_u32(0xFF006064), // 900
    ],
);
const _CYAN_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF00BCD4);

pub const CYAN_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _CYAN_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFF84FFFF), // 100
        _CYAN_ACCENT_PRIMARY_VALUE,      // 200
        srgba_from_argb_u32(0xFF00E5FF), // 400
        srgba_from_argb_u32(0xFF00B8D4), // 700
    ],
);
const _CYAN_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF18FFFF);

pub const TEAL: MaterialColorShades = MaterialColorShades::new(
    _TEAL_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFE0F2F1), // 50
        srgba_from_argb_u32(0xFFB2DFDB), // 100
        srgba_from_argb_u32(0xFF80CBC4), // 200
        srgba_from_argb_u32(0xFF4DB6AC), // 300
        srgba_from_argb_u32(0xFF26A69A), // 400
        _TEAL_PRIMARY_VALUE,             // 500
        srgba_from_argb_u32(0xFF00897B), // 600
        srgba_from_argb_u32(0xFF00796B), // 700
        srgba_from_argb_u32(0xFF00695C), // 800
        srgba_from_argb_u32(0xFF004D40), // 900
    ],
);
const _TEAL_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF009688);

pub const TEAL_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _TEAL_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFA7FFEB), // 100
        _TEAL_ACCENT_PRIMARY_VALUE,      // 200
        srgba_from_argb_u32(0xFF1DE9B6), // 400
        srgba_from_argb_u32(0xFF00BFA5), // 700
    ],
);
const _TEAL_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF64FFDA);

pub const GREEN: MaterialColorShades = MaterialColorShades::new(
    _GREEN_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFE8F5E9), // 50
        srgba_from_argb_u32(0xFFC8E6C9), // 100
        srgba_from_argb_u32(0xFFA5D6A7), // 200
        srgba_from_argb_u32(0xFF81C784), // 300
        srgba_from_argb_u32(0xFF66BB6A), // 400
        _GREEN_PRIMARY_VALUE,            // 500
        srgba_from_argb_u32(0xFF43A047), // 600
        srgba_from_argb_u32(0xFF388E3C), // 700
        srgba_from_argb_u32(0xFF2E7D32), // 800
        srgba_from_argb_u32(0xFF1B5E20), // 900
    ],
);
const _GREEN_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF4CAF50);

pub const GREEN_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _GREEN_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFB9F6CA), // 100
        _GREEN_ACCENT_PRIMARY_VALUE,     // 200
        srgba_from_argb_u32(0xFF00E676), // 400
        srgba_from_argb_u32(0xFF00C853), // 700
    ],
);
const _GREEN_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF69F0AE);

pub const LIGHT_GREEN: MaterialColorShades = MaterialColorShades::new(
    _LIGHT_GREEN_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFF1F8E9), // 50
        srgba_from_argb_u32(0xFFDCEDC8), // 100
        srgba_from_argb_u32(0xFFC5E1A5), // 200
        srgba_from_argb_u32(0xFFAED581), // 300
        srgba_from_argb_u32(0xFF9CCC65), // 400
        _LIGHT_GREEN_PRIMARY_VALUE,      // 500
        srgba_from_argb_u32(0xFF7CB342), // 600
        srgba_from_argb_u32(0xFF689F38), // 700
        srgba_from_argb_u32(0xFF558B2F), // 800
        srgba_from_argb_u32(0xFF33691E), // 900
    ],
);
const _LIGHT_GREEN_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF8BC34A);

pub const LIGHT_GREEN_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _LIGHT_GREEN_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFCCFF90),   // 100
        _LIGHT_GREEN_ACCENT_PRIMARY_VALUE, // 200
        srgba_from_argb_u32(0xFF76FF03),   // 400
        srgba_from_argb_u32(0xFF64DD17),   // 700
    ],
);
const _LIGHT_GREEN_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFB2FF59);

pub const LIME: MaterialColorShades = MaterialColorShades::new(
    _LIME_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFF9FBE7), // 50
        srgba_from_argb_u32(0xFFF0F4C3), // 100
        srgba_from_argb_u32(0xFFE6EE9C), // 200
        srgba_from_argb_u32(0xFFDCE775), // 300
        srgba_from_argb_u32(0xFFD4E157), // 400
        _LIME_PRIMARY_VALUE,             // 500
        srgba_from_argb_u32(0xFFC0CA33), // 600
        srgba_from_argb_u32(0xFFAFB42B), // 700
        srgba_from_argb_u32(0xFF9E9D24), // 800
        srgba_from_argb_u32(0xFF827717), // 900
    ],
);
const _LIME_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFCDDC39);

pub const LIME_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _LIME_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFF4FF81), // 100
        _LIME_ACCENT_PRIMARY_VALUE,      // 200
        srgba_from_argb_u32(0xFFC6FF00), // 400
        srgba_from_argb_u32(0xFFAEEA00), // 700
    ],
);
const _LIME_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFEEFF41);

pub const YELLOW: MaterialColorShades = MaterialColorShades::new(
    _YELLOW_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFFFDE7), // 50
        srgba_from_argb_u32(0xFFFFF9C4), // 100
        srgba_from_argb_u32(0xFFFFF59D), // 200
        srgba_from_argb_u32(0xFFFFF176), // 300
        srgba_from_argb_u32(0xFFFFEE58), // 400
        _YELLOW_PRIMARY_VALUE,           // 500
        srgba_from_argb_u32(0xFFFDD835), // 600
        srgba_from_argb_u32(0xFFFBC02D), // 700
        srgba_from_argb_u32(0xFFF9A825), // 800
        srgba_from_argb_u32(0xFFF57F17), // 900
    ],
);
const _YELLOW_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFFEB3B);

pub const YELLOW_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _YELLOW_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFFFF8D), // 100
        _YELLOW_ACCENT_PRIMARY_VALUE,    // 200
        srgba_from_argb_u32(0xFFFFEA00), // 400
        srgba_from_argb_u32(0xFFFFD600), // 700
    ],
);
const _YELLOW_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFFFF00);

pub const AMBER: MaterialColorShades = MaterialColorShades::new(
    _AMBER_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFFF8E1), // 50
        srgba_from_argb_u32(0xFFFFECB3), // 100
        srgba_from_argb_u32(0xFFFFE082), // 200
        srgba_from_argb_u32(0xFFFFD54F), // 300
        srgba_from_argb_u32(0xFFFFCA28), // 400
        _AMBER_PRIMARY_VALUE,            // 500
        srgba_from_argb_u32(0xFFFFB300), // 600
        srgba_from_argb_u32(0xFFFFA000), // 700
        srgba_from_argb_u32(0xFFFF8F00), // 800
        srgba_from_argb_u32(0xFFFF6F00), // 900
    ],
);
const _AMBER_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFFC107);

pub const AMBER_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _AMBER_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFFE57F), // 100
        _AMBER_ACCENT_PRIMARY_VALUE,     // 200
        srgba_from_argb_u32(0xFFFFC400), // 400
        srgba_from_argb_u32(0xFFFFAB00), // 700
    ],
);
const _AMBER_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFFD740);

pub const ORANGE: MaterialColorShades = MaterialColorShades::new(
    _ORANGE_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFFF3E0), // 50
        srgba_from_argb_u32(0xFFFFE0B2), // 100
        srgba_from_argb_u32(0xFFFFCC80), // 200
        srgba_from_argb_u32(0xFFFFB74D), // 300
        srgba_from_argb_u32(0xFFFFA726), // 400
        _ORANGE_PRIMARY_VALUE,           // 500
        srgba_from_argb_u32(0xFFFB8C00), // 600
        srgba_from_argb_u32(0xFFF57C00), // 700
        srgba_from_argb_u32(0xFFEF6C00), // 800
        srgba_from_argb_u32(0xFFE65100), // 900
    ],
);
const _ORANGE_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFF9800);

pub const ORANGE_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _ORANGE_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFFD180), // 100
        _ORANGE_ACCENT_PRIMARY_VALUE,    // 200
        srgba_from_argb_u32(0xFFFF9100), // 400
        srgba_from_argb_u32(0xFFFF6D00), // 700
    ],
);
const _ORANGE_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFFAB40);

pub const DEEP_ORANGE: MaterialColorShades = MaterialColorShades::new(
    _DEEP_ORANGE_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFBE9E7), // 50
        srgba_from_argb_u32(0xFFFFCCBC), // 100
        srgba_from_argb_u32(0xFFFFAB91), // 200
        srgba_from_argb_u32(0xFFFF8A65), // 300
        srgba_from_argb_u32(0xFFFF7043), // 400
        _DEEP_ORANGE_PRIMARY_VALUE,      // 500
        srgba_from_argb_u32(0xFFF4511E), // 600
        srgba_from_argb_u32(0xFFE64A19), // 700
        srgba_from_argb_u32(0xFFD84315), // 800
        srgba_from_argb_u32(0xFFBF360C), // 900
    ],
);
const _DEEP_ORANGE_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFF5722);

pub const DEEP_ORANGE_ACCENT: MaterialAccentShades = MaterialAccentShades::new(
    _DEEP_ORANGE_ACCENT_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFF9E80),   // 100
        _DEEP_ORANGE_ACCENT_PRIMARY_VALUE, // 200
        srgba_from_argb_u32(0xFFFF3D00),   // 400
        srgba_from_argb_u32(0xFFDD2C00),   // 700
    ],
);
const _DEEP_ORANGE_ACCENT_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFFFF6E40);

pub const BROWN: MaterialColorShades = MaterialColorShades::new(
    _BROWN_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFEFEBE9), // 50
        srgba_from_argb_u32(0xFFD7CCC8), // 100
        srgba_from_argb_u32(0xFFBCAAA4), // 200
        srgba_from_argb_u32(0xFFA1887F), // 300
        srgba_from_argb_u32(0xFF8D6E63), // 400
        _BROWN_PRIMARY_VALUE,            // 500
        srgba_from_argb_u32(0xFF6D4C41), // 600
        srgba_from_argb_u32(0xFF5D4037), // 700
        srgba_from_argb_u32(0xFF4E342E), // 800
        srgba_from_argb_u32(0xFF3E2723), // 900
    ],
);
const _BROWN_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF795548);

pub const GREY: MaterialColorShades = MaterialColorShades::new(
    _GREY_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFFAFAFA), // 50
        srgba_from_argb_u32(0xFFF5F5F5), // 100
        srgba_from_argb_u32(0xFFEEEEEE), // 200
        srgba_from_argb_u32(0xFFE0E0E0), // 300
        srgba_from_argb_u32(0xFFBDBDBD), // 400
        _GREY_PRIMARY_VALUE,             // 500
        srgba_from_argb_u32(0xFF757575), // 600
        srgba_from_argb_u32(0xFF616161), // 700
        srgba_from_argb_u32(0xFF424242), // 800
        srgba_from_argb_u32(0xFF212121), // 900
    ],
);
const _GREY_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF9E9E9E);

pub const BLUE_GREY: MaterialColorShades = MaterialColorShades::new(
    _BLUE_GREY_PRIMARY_VALUE,
    &[
        srgba_from_argb_u32(0xFFECEFF1), // 50
        srgba_from_argb_u32(0xFFCFD8DC), // 100
        srgba_from_argb_u32(0xFFB0BEC5), // 200
        srgba_from_argb_u32(0xFF90A4AE), // 300
        srgba_from_argb_u32(0xFF78909C), // 400
        _BLUE_GREY_PRIMARY_VALUE,        // 500
        srgba_from_argb_u32(0xFF546E7A), // 600
        srgba_from_argb_u32(0xFF455A64), // 700
        srgba_from_argb_u32(0xFF37474F), // 800
        srgba_from_argb_u32(0xFF263238), // 900
    ],
);
const _BLUE_GREY_PRIMARY_VALUE: Srgba<u8> = srgba_from_argb_u32(0xFF607D8B);
