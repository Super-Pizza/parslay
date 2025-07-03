use std::{collections::BTreeMap, fmt::Alignment};

use ab_glyph::{Font, FontArc, GlyphId, PxScaleFont, ScaleFont};
use unicode_linebreak::{
    linebreaks,
    BreakOpportunity::{self, *},
};

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
    real_words: BTreeMap<usize, u32>,
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
            real_words: BTreeMap::new(),
        }
    }

    fn get_glyphs(
        &self,
        c: char,
        next_c: Option<&(usize, char)>,
        word_end: bool,
    ) -> (GlyphId, GlyphId) {
        let glyph_id = self.font.as_ref().unwrap().glyph_id(c);
        let next_c = if word_end {
            ' '
        } else {
            next_c.map(|i| i.1).unwrap_or(' ')
        };
        let next_id = self.font.as_ref().unwrap().glyph_id(next_c);
        (glyph_id, next_id)
    }

    fn get_glyph_width(
        scaled: PxScaleFont<&FontArc>,
        glyphs: (GlyphId, GlyphId),
        end: bool,
    ) -> u32 {
        let mut result = 0;
        if end {
            result += scaled.h_side_bearing(glyphs.0) as u32
        }
        result += scaled.h_advance(glyphs.0) as u32;
        result += scaled.kern(glyphs.0, glyphs.1) as u32;
        result
    }

    /// You must have a font set before calling this.
    fn get_text_size(&mut self) {
        if !self.breaks.is_empty() {
            return;
        }
        let breaks = linebreaks(&self.text);
        for (idx, opportunity) in breaks {
            if opportunity == Allowed {
                self.words.last_mut().unwrap().push((idx, 0));
            }
            if opportunity == Mandatory {
                self.words.last_mut().unwrap().push((idx, 0));
                self.words.push(vec![])
            }
            self.breaks.insert(idx, opportunity);
        }
        let mut word_idx = 0;

        let font = self.font.as_ref().unwrap();
        let scaled = font.as_scaled(font.pt_to_px_scale(self.font_size).unwrap());
        let mut cursor =
            scaled.h_side_bearing(font.glyph_id(self.text.chars().next().unwrap_or(' '))) as u32;
        let height = scaled.height();
        let mut iter = self.text.char_indices().peekable();
        let mut line_idx = 0;
        while let Some((idx, c)) = iter.next() {
            let glyphs = self.get_glyphs(c, iter.peek(), self.breaks.get(&idx) == Some(&Mandatory));
            cursor += Self::get_glyph_width(scaled, glyphs, iter.peek().is_none());

            if !self.breaks.contains_key(&idx) && iter.peek().is_some() {
                continue;
            }
            self.words[line_idx][word_idx].1 = cursor;
            cursor = 0;
            if self.breaks.get(&idx) == Some(&Allowed) {
                word_idx += 1;
            } else {
                line_idx += 1;
            }
        }
        self.height = height as u32;
    }

    /// Minimum, Maximum allowed width in pixels
    pub fn width_bounds(&self) -> (u32, u32) {
        let min = self
            .words
            .iter()
            .flatten()
            .max_by_key(|i| i.1)
            .copied()
            .unwrap_or_default()
            .1;
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

    pub fn insert<S: AsRef<str>>(&mut self, text: S, idx: usize) {
        self.text.insert_str(idx, text.as_ref());
        self.breaks = BTreeMap::new();
        self.words = vec![vec![]];
        if self.font.is_some() {
            self.get_text_size();
        }
    }

    pub fn remove(&mut self, idx: usize) {
        self.text.remove(idx);
        self.breaks = BTreeMap::new();
        self.words = vec![vec![]];
        if self.font.is_some() {
            self.get_text_size();
        }
    }

    pub fn set_text<S: AsRef<str>>(&mut self, text: S) {
        self.text = text.as_ref().to_string();
        self.breaks = BTreeMap::new();
        self.words = vec![vec![]];
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
        if width
            < self
                .words
                .iter()
                .flatten()
                .max_by_key(|i| i.1)
                .copied()
                .unwrap_or_default()
                .1
        {
            return None;
        }
        if self.width == width {
            return Some(());
        }
        self.real_words = BTreeMap::new();
        let mut cursor = 0;
        for line in self.words.iter() {
            self.real_words
                .insert(line.first().copied().unwrap_or_default().0, 0);
            for word in line.iter() {
                if cursor + word.1 > width {
                    *self.real_words.last_entry().unwrap().get_mut() = cursor;
                    self.real_words.insert(word.0, 0);
                    cursor = 0;
                }
                cursor += word.1;
            }
            *self.real_words.last_entry().unwrap().get_mut() = cursor;
            cursor = 0;
        }
        self.real_words.insert(self.text.len(), 0);
        Some(())
    }

    pub(crate) fn get_cursor_pos(&self, offs: Offset) -> usize {
        let font = self.font.as_ref().unwrap();
        let scaled = font.as_scaled(font.pt_to_px_scale(self.font_size).unwrap());
        let mut cursor =
            scaled.h_side_bearing(font.glyph_id(self.text.chars().next().unwrap_or(' '))) as u32;
        let mut iter = self.text.char_indices().peekable();
        while let Some((idx, c)) = iter.next() {
            let glyphs = self.get_glyphs(c, iter.peek(), self.real_words.contains_key(&(idx + 1)));
            cursor += Self::get_glyph_width(scaled, glyphs, iter.peek().is_none());
            let half_width = scaled.h_advance(glyphs.0) as i32 / 2;

            if self.real_words.contains_key(&(idx + 1)) {
                if offs.x >= cursor as i32 - half_width {
                    return idx;
                }
            } else if offs.x >= cursor as i32 - half_width
                && offs.x <= cursor as i32 + scaled.h_advance(glyphs.1) as i32 / 2
            {
                return idx + 1;
            }
            if !self.breaks.contains_key(&idx) {
                continue;
            }
        }
        0
    }

    /// Returns `None` if the width is too small.
    #[must_use]
    pub fn draw(&mut self, buf: &Buffer, rect: Rect, bg_color: Rgba) -> Option<()> {
        self.set_width(rect.w)?;

        let text = &self.text;
        let text_buf = buf.subregion(rect);
        let mut start_pos = Offset::default();

        let font = self.font.as_ref().unwrap();
        let scaled = font.as_scaled(font.pt_to_px_scale(self.font_size).unwrap());
        let mut cursor = match self.align {
            Alignment::Left => 0,
            Alignment::Center => (rect.w - self.real_words.first_entry().unwrap().get()) / 2,
            Alignment::Right => rect.w - self.real_words.first_entry().unwrap().get(),
        };
        cursor += scaled.h_side_bearing(font.glyph_id(text.chars().next().unwrap_or(' '))) as u32;
        let mut iter = text.chars().enumerate().peekable();
        let mut word_iter = self.real_words.iter().peekable();
        while let Some((idx, c)) = iter.next() {
            let glyphs = self.get_glyphs(c, iter.peek(), *word_iter.peek().unwrap().0 == idx + 1);

            let glyph = glyphs.0.with_scale_and_position(scaled.scale, (0i16, 0));
            let ascent = scaled.ascent() as i32 + start_pos.y;
            if let Some(q) = font.outline_glyph(glyph) {
                let bounds = q.px_bounds();
                q.draw(|x, y, c| {
                    text_buf.point(
                        x as i32 + cursor as i32 + bounds.min.x as i32,
                        y as i32 + ascent + bounds.min.y as i32,
                        &bg_color.lerp(self.color, (c * 255.0) as u8),
                    )
                });
            }

            cursor += Self::get_glyph_width(scaled, glyphs, iter.peek().is_none());

            if *word_iter.peek().unwrap().0 == idx + 1 {
                start_pos.y += self.height as i32 + scaled.line_gap() as i32;
                cursor = match self.align {
                    Alignment::Left => 0,
                    Alignment::Center => (rect.w - word_iter.next().unwrap().1) / 2,
                    Alignment::Right => rect.w - word_iter.next().unwrap().1,
                };
            }
        }
        Some(())
    }
}
