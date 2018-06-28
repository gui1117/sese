const GEN_PALE_DIVISION: usize = 10;
const GEN_PALE_DELTA: f32 = 0.0;

#[derive(EnumIterator)]
#[repr(usize)]
pub enum GenPale {
    Color0,
    Color1,
    Color2,
    Color3,
    Color4,
    Color5,
    Color6,
    Color7,
    Color8,
    Color9,
    Black,
    White,
}

#[test]
fn alignment() {
    assert!(GenPale::iter_variants().count() == GEN_PALE_DIVISION + 2);
    assert!(GenPale::Black as usize = GEN_PALE_DIVISION);
}

impl GenPale {
    // all colors, not black nor white
    pub fn colors() -> Vec<Self> {
        GenPale::iter_variants().collect::<Vec<_>>()
    }
}

lazy_static! {
   static ref  GEN_PALE_GENERATION: Vec<[f32; 3]> =
       generate_colors(GEN_PALE_DIVISION, GEN_PALE_DELTA, ::CFG.color_black, ::CFG.color_white);
}

impl Into<[f32; 3]> for GenPale {
    fn into(self) -> [f32; 3] {
        GEN_PALE_GENERATION[self as usize]
    }
}

fn generate_colors(division: usize, delta: f32, black: f32, white: f32) -> Vec<[f32; 3]> {
    let mut colors = vec![];
    assert!(black < white);

    // Black
    colors.push([black, black, black]);

    // White
    colors.push([white, white, white]);

    for i in 0..division {
        let color = color_circle((i as f32 + delta) / division as f32);
        colors.push([
            color[0] * (white - black) + black,
            color[1] * (white - black) + black,
            color[2] * (white - black) + black,
        ]);
    }

    // Convert from Srgb to Unorm
    for color in &mut colors {
        let lin_color = ::palette::LinSrgb::new(color[0], color[1], color[2]);
        let srgb_color = ::palette::Srgb::from_encoding(lin_color);
        color[0] = srgb_color.red;
        color[1] = srgb_color.green;
        color[2] = srgb_color.blue;
    }

    colors
}

fn color_circle(x: f32) -> [f32; 3] {
    if x * 6.0 < 1.0 {
        let t = x * 6.0;
        [1.0, t, 0.0]
    } else if x * 6.0 < 2.0 {
        let t = x * 6.0 - 1.0;
        [1.0 - t, 1.0, 0.0]
    } else if x * 6.0 < 3.0 {
        let t = x * 6.0 - 2.0;
        [0.0, 1.0, t]
    } else if x * 6.0 < 4.0 {
        let t = x * 6.0 - 3.0;
        [0.0, 1.0 - t, 1.0]
    } else if x * 6.0 < 5.0 {
        let t = x * 6.0 - 4.0;
        [t, 0.0, 1.0]
    } else {
        let t = x * 6.0 - 5.0;
        [1.0, 0.0, 1.0 - t]
    }
}
