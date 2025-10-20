use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

use lite_graphics::{Buffer, Drawable, Overlay, Rect, color::Rgba};

use crate::{
    app::{CursorType, HoverResult},
    reactive::{RwSignal, SignalGet, SignalUpdate, create_effect},
    themes,
    window::Window,
};

use super::{
    ComputedSize, InputEventFn, MouseEventFn, Offset, Size, WidgetBase, WidgetExt, WidgetGroup,
    WidgetInternal,
    button::Button,
    input::{InputBase, InputExt},
    label::{Label, dyn_label},
    stack::{HStack, VStack, vstack},
};

pub struct DropDown<W: WidgetBase + ?Sized + 'static> {
    base: Rc<Label>,

    overlay: Rc<VStack>,
    overlay_pos: Cell<Offset>,

    selected: RwSignal<(String, bool)>,

    default_bg: Cell<Rgba>,
    hovered_bg: Cell<Rgba>,
    clicked_bg: Cell<Rgba>,

    hovered: Cell<Option<Offset>>,
    clicked: Cell<bool>,

    hover_fn: RefCell<Box<MouseEventFn<Self>>>,
    edit_fn: RefCell<Box<InputEventFn<Self>>>,
    click_fn: RefCell<Box<MouseEventFn<Self>>>,
}

impl<W: WidgetBase + 'static> InputBase for DropDown<W> {
    fn handle_key(&self, _: crate::event::Key) {}
}

impl<W: WidgetBase + 'static> InputExt for DropDown<W> {
    fn on_edit<F: FnMut(&Self) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.edit_fn.borrow_mut() = Box::new(f);
        let this = self.clone();
        create_effect(move |_| (this.edit_fn.borrow_mut())(&this.clone()));
        self
    }
}

impl<W: WidgetBase> WidgetBase for DropDown<W> {
    fn set_size(&self, size: Size) {
        self.base.set_size(size);
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn set_pos(&self, pos: Offset) {
        self.base.set_pos(pos);
    }
    fn set_frame(&self, frame: String) {
        self.base.set_frame(frame);
    }
    fn set_background_color(&self, color: Rgba) {
        self.base.set_background_color(color);
        self.default_bg.set(color);
    }
    fn get_background_color(&self) -> Rgba {
        self.base.get_background_color()
    }
    fn set_padding(&self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn get_padding(&self) -> (u32, u32, u32, u32) {
        self.base.get_padding()
    }
    fn set_border_radius(&self, radius: u32) {
        self.base.set_border_radius(radius);
    }
    fn get_border_radius(&self) -> u32 {
        self.base.get_border_radius()
    }
    fn set_color(&self, color: Rgba) {
        self.base.set_color(color);
    }
    fn set_text(&self, text: &str) {
        self.base.set_text(text);
    }
    fn get_text(&self) -> String {
        self.selected.get().0
    }
    fn set_disabled(&self, disable: bool) {
        self.base.set_disabled(disable);
    }
    fn is_disabled(&self) -> bool {
        self.base.is_disabled()
    }
}

impl<W: WidgetExt> WidgetExt for DropDown<W> {
    fn new() -> Rc<Self> {
        let signal = RwSignal::new(("".to_string(), false));
        let this = DropDown {
            base: dyn_label(move || signal.get().0)
                .padding(4)
                .frame(themes::FrameType::Button)
                .background_color(Rgba::hex("#808080").unwrap()),
            overlay: vstack(4, "").background_color(Rgba::hex("#606060").unwrap()),
            overlay_pos: Cell::new(Offset::default()),
            selected: signal,
            default_bg: Cell::new(Rgba::WHITE),
            hovered_bg: Cell::new(Rgba::hex("#808080").unwrap()),
            clicked_bg: Cell::new(Rgba::hex("#a0a0a0").unwrap()),
            hovered: Cell::new(None),
            clicked: Cell::new(false),
            hover_fn: RefCell::new(Box::new(|_, _| {})),
            edit_fn: RefCell::new(Box::new(|_| {})),
            click_fn: RefCell::new(Box::new(|_, _| {})),
        };
        Rc::new(this)
    }

    fn on_hover<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.hover_fn.borrow_mut() = Box::new(f);
        self
    }
    fn on_click<F: FnMut(&Self, Offset) + 'static>(self: Rc<Self>, f: F) -> Rc<Self> {
        *self.click_fn.borrow_mut() = Box::new(f);
        self
    }
}

impl<W: WidgetBase> WidgetInternal for DropDown<W> {
    fn set_font(&self, font: ab_glyph::FontArc) {
        self.base.set_font(font.clone());
        self.overlay.set_font(font);
    }
    fn width_bounds(&self) -> (u32, u32) {
        self.base.width_bounds()
    }
    fn set_width(&self, width: u32) {
        self.base.set_width(width);
    }
    fn height_bounds(&self) -> (u32, u32) {
        self.base.height_bounds()
    }
    fn set_height(&self, height: u32) {
        self.base.set_height(height);
    }
    fn get_computed_size(&self) -> ComputedSize {
        self.base.get_computed_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&self, pos: Offset) {
        self.base.set_offset(pos);
        self.overlay_pos.set(pos);
        self.overlay.set_offset(Offset::default());
    }
    fn get_frame(&self) -> themes::FrameFn {
        self.base.get_frame()
    }
    fn draw_frame(&self, _: &dyn Drawable) {}
    fn draw(&self, buf: &mut dyn Drawable) {
        if self.is_disabled() {
            self.base
                .set_background_color(Rgba::hex("#d0d0d0").unwrap());
        } else if self.clicked.get() {
            self.base.set_background_color(self.clicked_bg.get());
        } else if self.hovered.get().is_some() {
            self.base.set_background_color(self.hovered_bg.get());
        } else {
            self.base.set_background_color(self.default_bg.get());
        }
        self.base.draw(buf);
    }
    fn draw_overlays(&self, buf: &mut Buffer) {
        if self.selected.get().1 {
            let offset = self.overlay_pos.get();
            let width = self.overlay.width_bounds().1;
            self.overlay.set_width(width);
            let height = self.overlay.height_bounds().1;
            self.overlay.set_height(height);
            self.overlay.set_offset(Offset::default());
            let size = self.overlay.get_computed_size();
            let mut overlay = Overlay::new(buf.clone(), Rect::new(offset, size));
            self.overlay.draw(&mut overlay);
            overlay.write();
        }
    }

    fn handle_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>) {
        if self.is_disabled() {
            self.selected.update(|s| s.1 = false);
            return;
        }

        let pos = pos - self.get_offset();
        let size = self.get_computed_size();
        let inside = pos.x >= 0 && pos.y >= 0 && pos.x <= size.w as i32 && pos.y <= size.h as i32;

        self.clicked.set(pressed.is_some() && inside);

        if let Some(w) = pressed {
            if inside {
                *w.focus.borrow_mut() = Some(self.clone());
            }
            return;
        } else if inside {
            (self.click_fn.borrow_mut())(&self.clone(), pos)
        };
        self.selected.update(|s| s.1 = inside);

        // todo: add button handling!
    }
    fn handle_hover(self: Rc<Self>, pos: Offset) -> HoverResult {
        let is_hovered = self.hovered.get().is_some();
        if self.is_disabled() {
            self.hovered.set(None);
            return HoverResult {
                redraw: is_hovered,
                cursor: CursorType::Arrow,
            };
        }

        let pos = pos - self.get_offset();
        let size = self.get_computed_size();
        if pos.x < 0 || pos.y < 0 || pos.x > size.w as i32 || pos.y > size.h as i32 {
            self.clicked.set(false);
            self.hovered.set(None);
            return HoverResult {
                redraw: is_hovered,
                cursor: CursorType::Arrow,
            };
        }

        (self.hover_fn.borrow_mut())(&self.clone(), pos);
        self.hovered.set(Some(pos));

        HoverResult {
            redraw: !is_hovered,
            cursor: CursorType::Pointer,
        }
    }
    fn handle_overlay_hover(self: Rc<Self>, pos: Offset) -> HoverResult {
        let is_hovered = self.hovered.get().is_some();
        if self.is_disabled() {
            self.hovered.set(None);
            return HoverResult {
                redraw: is_hovered,
                cursor: CursorType::Arrow,
            };
        }

        self.overlay
            .clone()
            .handle_hover(pos - self.overlay_pos.get())
    }
    fn handle_overlay_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>) -> bool {
        if self.is_disabled() {
            return false;
        }

        if !self.selected.get().1 {
            return false;
        }
        self.overlay
            .clone()
            .handle_button(pos - self.overlay_pos.get(), pressed);
        let end = self.overlay.get_computed_size() + self.overlay_pos.get();
        pos.x >= self.overlay_pos.get().x
            && pos.y >= self.overlay_pos.get().y
            && pos.x < end.x
            && pos.y < end.y
    }
}

pub fn drop_down<G: WidgetGroup + 'static>(
    initial: &'static str,
    items: G,
) -> Rc<DropDown<HStack>> {
    let signal = RwSignal::new((initial.to_string(), false));
    let overlay = VStack::new_internal(
        4,
        (initial, items).map(|w| {
            if let Ok(l) = (w.clone() as Rc<dyn Any>).downcast::<Label>() {
                Button::new_internal(l)
                    .on_click(move |b, _| signal.set((b.get_text(), false)))
                    .background_color(Rgba::hex("#606060").unwrap())
                    .size(Size::stretch(1, 0))
            } else {
                w.set_background_color(Rgba::hex("#808080").unwrap());
                w
            }
        }),
    )
    .background_color(Rgba::hex("#606060").unwrap())
    .frame(themes::FrameType::Frame)
    .padding(4);
    let this = DropDown {
        base: dyn_label(move || signal.get().0 + " ▾")
            .padding(4)
            .frame(themes::FrameType::Button)
            .background_color(Rgba::hex("#808080").unwrap()),
        overlay,
        overlay_pos: Cell::new(Offset::default()),
        selected: signal,
        default_bg: Cell::new(Rgba::WHITE),
        hovered_bg: Cell::new(Rgba::hex("#808080").unwrap()),
        clicked_bg: Cell::new(Rgba::hex("#a0a0a0").unwrap()),
        hovered: Cell::new(None),
        clicked: Cell::new(false),
        hover_fn: RefCell::new(Box::new(|_, _| {})),
        edit_fn: RefCell::new(Box::new(|_| {})),
        click_fn: RefCell::new(Box::new(|_, _| {})),
    };
    this.base.set_frame(themes::FrameType::Button.to_string());
    this.base.set_padding(4);
    Rc::new(this)
}
