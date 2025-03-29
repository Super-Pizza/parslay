mod line_breaks;

use std::fmt::Alignment;

use ab_glyph::{Font, ScaleFont};
use line_breaks::{line_breaks, Break};
use lite_graphics::{
    draw::{Buffer, Rgba},
    Rect,
};

pub struct Text {
    text: String,
    font: Option<ab_glyph::FontArc>,
    font_size: f32,
    color: Rgba,
    bg_color: Rgba,
    align: Alignment,

    // Internal state
    height: u32,
    breaks: Vec<Break>,
    words: Vec<Vec<(usize, u32)>>, // Lines<Words<offset, width>>
}

impl Text {
    pub fn new<S: AsRef<str>>(text: S, font_size: f32) -> Self {
        Self {
            font_size,
            font: None,
            text: text.as_ref().to_string(),
            color: Rgba::BLACK,
            bg_color: Rgba::WHITE,
            align: Alignment::Left,

            height: 0,
            breaks: vec![],
            words: vec![vec![(0, 0)]],
        }
    }

    pub fn get_text_size(&mut self, font: ab_glyph::FontArc) {
        self.breaks = line_breaks(&self.text);
        for (idx, car) in self.breaks.iter().enumerate() {
            if *car == Break::Maybe {
                self.words.last_mut().unwrap().push((idx + 1, 0));
            }
            if *car == Break::Yes {
                self.words.push(vec![(idx + 1, 0)])
            }
        }
        let mut cursor = 0;
        let mut max_y = 0;
        let mut min_y = i32::MAX;
        let scaled = font.as_scaled(font.pt_to_px_scale(self.font_size).unwrap());
        let mut iter = self.text.chars().enumerate().peekable();
        let mut word_idx = 0;
        let mut line_idx = 0;
        while let Some((idx, c)) = iter.next() {
            let glyph_id = font.glyph_id(c);
            let mut next_c = iter.peek().unwrap_or(&(idx + 1, ' ')).1;
            if self.breaks[idx] == Break::Yes {
                next_c = ' ';
            }
            let next_id = font.glyph_id(next_c);
            let glyph = glyph_id.with_scale_and_position(scaled.scale, (0i16, 0));
            if let Some(q) = font.outline_glyph(glyph) {
                let bounds = q.px_bounds();
                cursor += bounds.max.x as u32;
                max_y = max_y.max(bounds.max.y as i32);
                min_y = min_y.min(bounds.min.y as i32);
                cursor += scaled.kern(glyph_id, next_id) as u32;
            } else {
                cursor += scaled.h_advance(glyph_id) as u32;
            }
            if self.breaks[idx] == Break::No {
                continue;
            }
            self.words[line_idx][word_idx].1 = cursor;
            cursor = 0;
            if self.breaks[idx] == Break::Maybe {
                word_idx += 1;
            } else {
                line_idx += 1;
            }
        }
        self.height = (max_y - min_y) as u32;
        self.font = Some(font);
    }

    /// Minimum, Maximum allowed width in pixels
    pub fn width_bounds(&self) -> (u32, u32) {
        let min = self.words.iter().flatten().min_by_key(|i| i.1).unwrap().1;
        let max = self.words.iter().flatten().map(|i| i.1).sum();
        (min, max)
    }

    pub fn text_height(&self) -> u32 {
        self.height
    }

    pub fn set_align(&mut self, align: Alignment) {
        self.align = align;
    }

    pub fn set_color(&mut self, color: Rgba) {
        self.color = color;
    }

    pub fn set_background_color(&mut self, bg_color: Rgba) {
        self.bg_color = bg_color;
    }

    pub fn set_text<S: AsRef<str>>(&mut self, text: S) {
        self.text = text.as_ref().to_string();
        self.breaks = vec![];
    }

    /// You must call Text::get_text_size atleast once before, otherwise it will panic.
    pub fn draw(&self, buf: &Buffer, rect: Rect) {
        let text = &self.text;
        let font = self.font.as_ref().unwrap();
        let mut real_words = vec![];
        let mut cursor = 0;
        let width = rect.w;
        for line in self.words.iter() {
            for word in line.iter() {
                if cursor == 0 {
                    real_words.push((word.0, 0));
                }
                if cursor + word.1 > width {
                    real_words.last_mut().unwrap().1 = cursor;
                    cursor = 0;
                }
                cursor += word.1;
            }
            real_words.last_mut().unwrap().1 = cursor;
            cursor = 0;
        }
        let mut start_pos = rect.offset();
        let mut cursor = match self.align {
            Alignment::Left => 0,
            Alignment::Center => (width - real_words[0].1) / 2,
            Alignment::Right => width - real_words[0].1,
        } as i32;
        let scaled = font.as_scaled(font.pt_to_px_scale(self.font_size).unwrap());
        let mut iter = text.chars().enumerate().peekable();
        let mut word_idx = 0;
        while let Some((idx, c)) = iter.next() {
            let glyph_id = font.glyph_id(c);
            let mut next_c = iter.peek().unwrap_or(&(idx + 1, ' ')).1;
            if real_words[word_idx + 1].0 == idx + 1 {
                next_c = ' ';
            }
            let next_id = font.glyph_id(next_c);
            let glyph = glyph_id.with_scale_and_position(scaled.scale, (0i16, 0));
            let ascent = scaled.ascent() as i32;
            let descent = scaled.descent() as i32;
            if let Some(q) = font.outline_glyph(glyph) {
                let bounds = q.px_bounds();
                q.draw(|x, y, c| {
                    buf.point(
                        x as i32 + start_pos.x + cursor + bounds.min.x as i32,
                        y as i32 + start_pos.y + ascent + descent + bounds.min.y as i32,
                        self.bg_color.lerp(self.color, (c * 255.0) as u8),
                    )
                });
                cursor += bounds.max.x as i32;
                cursor += scaled.kern(glyph_id, next_id) as i32;
            } else {
                cursor += scaled.h_advance(glyph_id) as i32;
            }
            if real_words[word_idx + 1].0 == idx + 1 {
                word_idx += 1;
                start_pos.y += self.height as i32;
                cursor = match self.align {
                    Alignment::Left => 0,
                    Alignment::Center => (width - real_words[word_idx].1) / 2,
                    Alignment::Right => width - real_words[word_idx].1,
                } as i32;
            }
        }
    }
}
