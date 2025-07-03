use std::{collections::BTreeMap, fmt::Alignment};

use ab_glyph::{Font, ScaleFont};
use unicode_linebreak::{linebreaks, BreakOpportunity};

use lite_graphics::{color::Rgba, draw::Buffer, Offset, Rect};

#[derive(Clone)]
pub struct Text {
    text: String,
    font: Option<ab_glyph::FontArc>,
    font_size: f32,
    color: Rgba,
    align: Alignment,

    // Internal state
    width: u32,
    height: u32,
    breaks: BTreeMap<usize, BreakOpportunity>,
    words: Vec<Vec<(usize, u32)>>, // Lines<Words<offset, width>>
    real_words: Vec<(usize, u32)>,
}

impl Text {
    pub fn new<S: AsRef<str>>(text: S, font_size: f32) -> Self {
        Self {
            font_size,
            font: None,
            text: text.as_ref().to_string(),
            color: Rgba::BLACK,
            align: Alignment::Left,

            width: 0,
            height: 0,
            breaks: BTreeMap::new(),
            words: vec![vec![]],
            real_words: vec![],
        }
    }

    /// You must have a font set before calling this.
    fn get_text_size(&mut self) {
        if !self.breaks.is_empty() {
            return;
        }
        let breaks = linebreaks(&self.text);
        for (idx, opportunity) in breaks {
            if opportunity == BreakOpportunity::Allowed {
                self.words.last_mut().unwrap().push((idx, 0));
            }
            if opportunity == BreakOpportunity::Mandatory {
                self.words.last_mut().unwrap().push((idx, 0));
                self.words.push(vec![])
            }
            self.breaks.insert(idx, opportunity);
        }
        let font = self.font.as_ref().unwrap();
        let scaled = font.as_scaled(font.pt_to_px_scale(self.font_size).unwrap());
        let mut cursor = 0;
        let height = scaled.height();
        let mut iter = self.text.char_indices().peekable();
        let mut word_idx = 0;
        let mut line_idx = 0;
        while let Some((idx, c)) = iter.next() {
            let glyph_id = font.glyph_id(c);
            if idx == 0 {
                cursor += scaled.h_side_bearing(glyph_id) as u32;
            }
            let mut next_c = iter.peek().unwrap_or(&(idx + 1, ' ')).1;
            if self.breaks.get(&idx) == Some(&BreakOpportunity::Mandatory) {
                next_c = ' ';
            }
            let next_id = font.glyph_id(next_c);
            cursor += scaled.h_advance(glyph_id) as u32;
            cursor += scaled.kern(glyph_id, next_id) as u32;
            if iter.peek().is_none() {
                cursor -= scaled.h_side_bearing(glyph_id) as u32;
            }
            if !self.breaks.contains_key(&idx) {
                continue;
            }
            self.words[line_idx][word_idx].1 = cursor;
            cursor = 0;
            if self.breaks.get(&idx) == Some(&BreakOpportunity::Allowed) {
                word_idx += 1;
            } else {
                line_idx += 1;
            }
        }
        self.words[line_idx][word_idx].1 = cursor;
        self.height = height as u32;
    }

    /// Minimum, Maximum allowed width in pixels
    pub fn width_bounds(&self) -> (u32, u32) {
        let min = self.words.iter().flatten().max_by_key(|i| i.1).unwrap().1;
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

    pub fn set_text<S: AsRef<str>>(&mut self, text: S) {
        self.text = text.as_ref().to_string();
        self.breaks = BTreeMap::new();
        self.words = vec![vec![(0, 0)]];
        if self.font.is_some() {
            self.get_text_size();
        }
    }

    pub fn set_font(&mut self, font: ab_glyph::FontArc) {
        self.font = Some(font);
        self.get_text_size();
    }

    /// Returns `None` if the width is too small.
    #[must_use]
    pub fn set_width(&mut self, width: u32) -> Option<()> {
        if width < self.words.iter().flatten().min_by_key(|i| i.1).unwrap().1 {
            return None;
        }
        if self.width == width {
            return Some(());
        }
        self.real_words = vec![];
        let mut cursor = 0;
        for line in self.words.iter() {
            for word in line.iter() {
                if cursor == 0 {
                    self.real_words.push((word.0, 0));
                }
                if cursor + word.1 > width {
                    self.real_words.last_mut().unwrap().1 = cursor;
                    cursor = 0;
                }
                cursor += word.1;
            }
            self.real_words.last_mut().unwrap().1 = cursor;
            cursor = 0;
        }
        self.real_words.push((self.text.len(), 0));
        Some(())
    }

    /// Returns `None` if the width is too small.
    #[must_use]
    pub fn draw(&mut self, buf: &Buffer, rect: Rect, bg_color: Rgba) -> Option<()> {
        self.set_width(rect.w)?;

        let text = &self.text;
        let font = self.font.as_ref().unwrap();

        let text_buf = buf.subregion(rect);
        let mut start_pos = Offset::default();
        let mut cursor = match self.align {
            Alignment::Left => 0,
            Alignment::Center => (rect.w - self.real_words[0].1) / 2,
            Alignment::Right => rect.w - self.real_words[0].1,
        } as i32;
        let scaled = font.as_scaled(font.pt_to_px_scale(self.font_size).unwrap());
        let mut iter = text.char_indices().peekable();
        let mut word_idx = 0;
        while let Some((idx, c)) = iter.next() {
            let glyph_id = font.glyph_id(c);
            if idx == 0 {
                cursor += scaled.h_side_bearing(glyph_id) as i32;
            }

            let glyph = glyph_id.with_scale_and_position(scaled.scale, (0i16, 0));
            let ascent = scaled.ascent() as i32;
            if let Some(q) = font.outline_glyph(glyph) {
                let bounds = q.px_bounds();
                q.draw(|x, y, c| {
                    text_buf.point(
                        x as i32 + cursor + bounds.min.x as i32,
                        y as i32 + ascent + bounds.min.y as i32,
                        &bg_color.lerp(self.color, (c * 255.0) as u8),
                    )
                });
            }

            let mut next_c = iter.peek().unwrap_or(&(idx + 1, ' ')).1;
            if self.real_words[word_idx + 1].0 == idx + 1 {
                next_c = ' ';
            }
            let next_id = font.glyph_id(next_c);

            cursor += scaled.h_advance(glyph_id) as i32;
            cursor += scaled.kern(glyph_id, next_id) as i32;
            if iter.peek().is_none() {
                cursor -= scaled.h_side_bearing(glyph_id) as i32;
            }
            if self.real_words[word_idx + 1].0 == idx + 1 {
                word_idx += 1;
                start_pos.y += self.height as i32 + scaled.line_gap() as i32;
                cursor = match self.align {
                    Alignment::Left => 0,
                    Alignment::Center => (rect.w - self.real_words[word_idx].1) / 2,
                    Alignment::Right => rect.w - self.real_words[word_idx].1,
                } as i32;
            }
        }
        Some(())
    }
}
