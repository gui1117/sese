use rusttype::IntoGlyphId;

macro_rules! try_multiple_time {
    ($e:expr) => (
        {
            let mut error_counter = 0;
            let mut res = $e;
            while res.is_err() {
                ::std::thread::sleep(::std::time::Duration::from_millis(10));
                error_counter += 1;
                if error_counter > 10 {
                    break;
                }
                res = $e;
            }
            res
        }
    )
}

#[inline]
pub fn to_grid(coords: &::na::Vector3<f32>, scale: f32) -> ::na::Vector3<isize> {
    ::na::Vector3::<isize>::from_iterator(coords.iter().map(|&c| (c / scale) as isize))
}

#[inline]
pub fn to_world(coords: &::na::Vector3<isize>, scale: f32) -> ::na::Vector3<f32> {
    let mut res = ::na::Vector3::new(scale, scale, scale) * 0.5;
    for i in 0..3 {
        res[i] += (coords[i] as f32) * scale;
    }
    res
}

pub fn menu_layout(texts: Vec<String>, cursor: Option<usize>, font: &::rusttype::Font<'static>) -> Vec<::rusttype::PositionedGlyph<'static>> {
    let v_metrics = font.v_metrics(::rusttype::Scale::uniform(::CFG.text_scale));
    let y_delta = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    let y_top = (texts.len() / 2) as f32 * y_delta;

    let cursor_string = "◀                        ▶".to_string();
    texts.iter()
        .enumerate()
        .chain( if let Some(cursor) = cursor {
            vec![(cursor, &cursor_string)]
        } else {
            vec![]
        })
        .flat_map(|(i, text)| {
            let y = i as f32 * y_delta - y_top;
            let scale = ::rusttype::Scale::uniform(::CFG.text_scale);
            let point = ::rusttype::point(0.0, y);
            let mut glyphs = font.layout(text, scale, point)
                .map(|x| x.standalone())
                .collect::<Vec<_>>();

            let delta = if let Some(last) = glyphs.last() {
                let w = last.position().x
                    + last.unpositioned().h_metrics().advance_width
                    + glyphs.first().unwrap().unpositioned().h_metrics().left_side_bearing;
                ::rusttype::vector(-w/2.0, 0.0)
            } else {
                ::rusttype::vector(0.0, 0.0)
            };

            let glyphs = glyphs.drain(..)
                .map(|glyph| {
                    let position = glyph.position();
                    glyph.into_unpositioned().positioned(position + delta)
                })
                .collect::<Vec<_>>();

            glyphs
        })
        .collect::<Vec<_>>()
}

pub fn joystick_description_layout(texts: Vec<String>, delim: char, menu_len: usize, font: &::rusttype::Font<'static>) -> Vec<::rusttype::PositionedGlyph<'static>> {
    let v_metrics = font.v_metrics(::rusttype::Scale::uniform(::CFG.text_scale));
    let y_delta = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    let y_top = (menu_len / 2 + texts.len() + 1) as f32 * y_delta;
    let delim_id = delim.into_glyph_id(font);

    texts.iter()
        .enumerate()
        .flat_map(|(i, text)| {
            let y = i as f32 * y_delta - y_top;
            let scale = ::rusttype::Scale::uniform(::CFG.text_scale);
            let point = ::rusttype::point(0.0, y);
            let mut glyphs = font.layout(text, scale, point)
                .map(|x| x.standalone())
                .collect::<Vec<_>>();

            let x_delim = glyphs.iter()
                .find(|g| g.id() == delim_id)
                .unwrap()
                .position().x;
            let delta = ::rusttype::vector(-x_delim, 0.0);

            let glyphs = glyphs.drain(..)
                .map(|glyph| {
                    let position = glyph.position();
                    glyph.into_unpositioned().positioned(position + delta)
                })
                .collect::<Vec<_>>();

            glyphs
        })
        .collect::<Vec<_>>()
}
