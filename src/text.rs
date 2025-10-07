use std::{collections::BTreeMap, fmt::Alignment};

use ab_glyph::{Font, FontArc, GlyphId, PxScaleFont, ScaleFont};
use unicode_linebreak::{BreakOpportunity, linebreaks};

use lite_graphics::{
    Offset, Rect,
    color::{Color, Rgba},
    draw::Drawable,
};

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
    breaks: BTreeMap<usize, (BreakOpportunity, u32)>,
    real_words: BTreeMap<usize, u32>,
    cursor: Option<usize>,
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
            real_words: BTreeMap::new(),
            cursor: None,
        }
    }

    fn get_glyphs(
        &self,
        c: char,
        next_c: Option<&(usize, char)>,
        word_end: bool,
    ) -> (GlyphId, GlyphId) {
        let real_c = match c {
            '\n' => '\r',
            c => c,
        };
        let glyph_id = self.font.as_ref().unwrap().glyph_id(real_c);
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
            self.breaks.insert(idx, (opportunity, 0));
        }

        let font = self.font.as_ref().unwrap();
        let scaled = font.as_scaled(font.pt_to_px_scale(self.font_size).unwrap());
        let mut cursor =
            scaled.h_side_bearing(font.glyph_id(self.text.chars().next().unwrap_or(' '))) as u32;
        let height = scaled.height() + scaled.line_gap();
        let mut iter = self.text.char_indices().peekable();
        let mut line_idx = 1;
        while let Some((idx, c)) = iter.next() {
            let glyphs = self.get_glyphs(
                c,
                iter.peek(),
                self.breaks.get(&idx).map(|i| i.0) == Some(BreakOpportunity::Mandatory),
            );

            if self.breaks.contains_key(&idx) {
                let mut breaks = self.breaks.get_mut(&idx);
                breaks.as_deref_mut().unwrap().1 = cursor;
                if breaks.map(|i| i.0) == Some(BreakOpportunity::Mandatory) {
                    line_idx += 1;
                }
                cursor = 0;
            }

            cursor += Self::get_glyph_width(scaled, glyphs, iter.peek().is_none());

            if iter.peek().is_none() && c == '\n' {
                line_idx += 1;
            }
        }
        let Some(opportunity) = self.breaks.get_mut(&self.len()) else {
            self.height = height as u32 * line_idx as u32;
            return;
        };
        opportunity.1 = cursor;
        self.height = height as u32 * line_idx as u32;
    }

    /// Minimum, Maximum allowed width in pixels
    pub fn width_bounds(&self) -> (u32, u32) {
        let min = self.breaks.values().map(|i| i.1).max().unwrap_or_default();
        let max = self
            .breaks
            .values()
            .fold((0, 0), |(acc, max), (opp, width)| {
                if *opp == BreakOpportunity::Mandatory {
                    (0, max.max(acc + width))
                } else {
                    (acc + width, max)
                }
            })
            .1;
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

    pub fn insert<S: AsRef<str>>(&mut self, text: S) {
        self.text.insert_str(self.cursor.unwrap(), text.as_ref());
        self.breaks = BTreeMap::new();
        if self.font.is_some() {
            self.get_text_size();
        }
        self.cursor = Some(self.cursor.unwrap() + text.as_ref().len());
    }

    pub fn remove_front(&mut self) {
        if self.cursor.unwrap() == self.text.len() {
            return;
        }
        self.text.remove(self.cursor.unwrap());
        self.breaks = BTreeMap::new();
        if self.font.is_some() {
            self.get_text_size();
        }
    }

    pub fn move_h(&mut self, shift: i32) {
        if shift >= 0 {
            if self.cursor.unwrap() == self.text.len() {
                return;
            }
            *self.cursor.as_mut().unwrap() += shift as usize
        } else {
            if self.cursor.unwrap() == 0 {
                return;
            }
            *self.cursor.as_mut().unwrap() -= (-shift) as usize
        }
    }

    pub fn remove_back(&mut self) {
        if self.cursor.unwrap() == 0 {
            return;
        }
        *self.cursor.as_mut().unwrap() -= 1;
        self.remove_front();
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn set_text<S: AsRef<str>>(&mut self, text: S) {
        self.text = text.as_ref().to_string();
        self.breaks = BTreeMap::new();
        if self.font.is_some() {
            self.get_text_size();
        }
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_font(&mut self, font: ab_glyph::FontArc) {
        self.font = Some(font);
        self.get_text_size();
    }

    /// Returns `None` if the width is too small.
    #[must_use]
    pub fn set_width(&mut self, width: u32) -> Option<()> {
        if width < self.breaks.values().map(|i| i.1).max().unwrap_or_default() {
            return None;
        }
        if self.width == width && !self.real_words.is_empty() {
            return Some(());
        }
        self.real_words = BTreeMap::new();
        self.real_words.insert(0, 0);
        let mut cursor = 0;
        for word in self.breaks.iter() {
            cursor += word.1.1;
            if cursor > width {
                *self.real_words.last_entry().unwrap().get_mut() = cursor - word.1.1;
            } else if word.1.0 == BreakOpportunity::Mandatory {
                *self.real_words.last_entry().unwrap().get_mut() = cursor;
            } else {
                continue;
            }
            self.real_words.insert(*word.0, 0);
            cursor = 0;
        }
        *self.real_words.last_entry().unwrap().get_mut() = cursor;
        self.real_words.insert(self.text.len(), 0);
        Some(())
    }

    pub(crate) fn get_cursor_pos(&mut self, offs: Offset) {
        let font = self.font.as_ref().unwrap();
        let scaled = font.as_scaled(font.pt_to_px_scale(self.font_size).unwrap());
        let mut cursor =
            scaled.h_side_bearing(font.glyph_id(self.text.chars().next().unwrap_or(' '))) as u32;
        let line = offs.y as f32 / (scaled.height() + scaled.line_gap());
        let initial_idx = self
            .real_words
            .iter()
            .skip(line.max(0.0) as usize)
            .map(|i| *i.0)
            .next()
            .unwrap_or(self.len());
        let mut iter = self.text.char_indices().skip(initial_idx).peekable();
        let mut prev_width = None;
        if offs.x <= 0 {
            self.cursor = Some(initial_idx);
            return;
        }
        while let Some((idx, c)) = iter.next() {
            let glyphs = self.get_glyphs(c, iter.peek(), self.real_words.contains_key(&(idx + 1)));
            let half_width = scaled.h_advance(glyphs.0) as i32 / 2;

            if offs.x > cursor as i32 + half_width {
                if self.real_words.contains_key(&(idx + 1))
                    && (idx + 1 < self.len() || self.real_words.contains_key(&idx))
                {
                    if self.breaks.get(&(idx + 1)).unwrap().0 == BreakOpportunity::Allowed {
                        self.cursor = Some(idx + 1);
                    } else {
                        self.cursor = Some(idx);
                    }

                    return;
                }
            } else if offs.x >= cursor as i32 - prev_width.unwrap_or(0)
                && offs.x <= cursor as i32 + half_width
            {
                self.cursor = Some(idx);
                return;
            }
            cursor += Self::get_glyph_width(scaled, glyphs, iter.peek().is_none());
            prev_width = Some(half_width)
        }
        self.cursor = Some(self.len())
    }

    /// Returns `None` if the width is too small.
    #[must_use]
    pub fn draw(&mut self, buf: &mut dyn Drawable, rect: Rect, bg_color: Rgba) -> Option<()> {
        self.set_width(rect.w)?;

        let text = &self.text;
        buf.subregion(rect);
        let mut line_offs = 0;

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
        word_iter.next().unwrap();
        while let Some((idx, c)) = iter.next() {
            let word_end = *word_iter.peek().unwrap().0 == idx + 1;
            let glyphs = self.get_glyphs(c, iter.peek(), word_end);

            let glyph = glyphs.0.with_scale_and_position(scaled.scale, (0i16, 0));
            let ascent = scaled.ascent() as i32 + line_offs;
            if let Some(q) = font.outline_glyph(glyph) {
                let bounds = q.px_bounds();
                q.draw(|x, y, c| {
                    buf.point(
                        x as i32 + cursor as i32 + bounds.min.x as i32,
                        y as i32 + ascent + bounds.min.y as i32,
                        &bg_color.lerp(self.color, (c * 255.0) as u8).into(),
                    )
                });
            }

            if self.cursor == Some(idx) {
                buf.line_v(
                    Offset {
                        x: cursor as i32,
                        y: line_offs,
                    },
                    scaled.height() as i32,
                    Color::BLACK,
                );
            }

            cursor += Self::get_glyph_width(scaled, glyphs, iter.peek().is_none());

            if word_end {
                let curr_len = word_iter.next().unwrap();
                if word_iter.peek().is_none() && c != '\n' {
                    break;
                }
                line_offs += scaled.height() as i32 + scaled.line_gap() as i32;
                cursor = match self.align {
                    Alignment::Left => 0,
                    Alignment::Center => (rect.w - curr_len.1) / 2,
                    Alignment::Right => rect.w - curr_len.1,
                };
            }
        }
        if self.cursor == Some(self.len()) {
            buf.line_v(
                Offset {
                    x: cursor as i32,
                    y: line_offs,
                },
                scaled.height() as i32,
                Color::BLACK,
            );
        }
        buf.end_subregion();
        Some(())
    }
}
