const FONT_DATA: &'static [u8] = include_bytes!("../font6x8_1bpp.raw");

/// Returns the value of a pixel in a character in the font.
pub(crate) fn character_pixel(c: char, x: u32, y: u32) -> bool {
    let char_per_row = 240 / 6;

    // Char _code_ offset from first char, most often a space
    // E.g. first char = ' ' (32), target char = '!' (33), offset = 33 - 32 = 1
    let char_offset = char_offset(c);
    let row = char_offset / char_per_row;

    // Top left corner of character, in pixels
    let char_x = (char_offset - (row * char_per_row)) * 6;
    let char_y = row * 8;

    // Bit index
    // = X pixel offset for char
    // + Character row offset (row 0 = 0, row 1 = (192 * 8) = 1536)
    // + X offset for the pixel block that comprises this char
    // + Y offset for pixel block
    let bitmap_bit_index = char_x + x + ((char_y + y) * 240);

    let bitmap_byte = bitmap_bit_index / 8;
    let bitmap_bit = 7 - (bitmap_bit_index % 8);

    FONT_DATA[bitmap_byte as usize] & (1 << bitmap_bit) != 0
}

fn char_offset(c: char) -> u32 {
    let fallback = '?' as u32 - ' ' as u32;
    if c < ' ' {
        return fallback;
    }
    if c <= '~' {
        return c as u32 - ' ' as u32;
    }
    if c < '¡' || c > 'ÿ' {
        return fallback;
    }
    c as u32 - ' ' as u32 - 34
}

fn blit_text(text: &str) -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity(text.len() * 2 * 6 * 8);
    for y in 0..8 {
        for x in 0..6 * text.len() {
            let is_set = character_pixel(text.as_bytes()[x / 6] as char, (x % 6) as u32, y);
            data.push(if is_set { 0u8 } else { 255u8 }); // L
            data.push(if is_set { 255u8 } else { 0u8 }); // A
        }
    }
    data
}

pub fn blit_text_span(x: f64, y: f64, text: &str) -> usvg::Image {
    let data: Vec<u8> = blit_text(text);

    let mut out: Vec<u8> = Vec::with_capacity(512);
    let mut encoder = png::Encoder::new(&mut out, (text.len() * 6) as u32, 8);
    encoder.set_color(png::ColorType::GrayscaleAlpha);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(&data).unwrap();
    drop(writer);

    usvg::Image {
        id: "".to_string(),
        transform: usvg::Transform::new_translate(x, y),
        view_box: usvg::ViewBox {
            rect: usvg::Rect::new(0., 0., text.len() as f64, 1.0).unwrap(),
            aspect: usvg::AspectRatio::default(),
        },
        visibility: usvg::Visibility::Visible,
        rendering_mode: usvg::ImageRendering::OptimizeSpeed,
        kind: usvg::ImageKind::PNG(out),
    }
}
