use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

use lite_graphics::{Buffer, Drawable, Overlay, Rect, color::Rgba};

use crate::{
    FrameType, WidgetGroup,
    app::{CursorType, HoverResult},
    dyn_label,
    reactive::{RwSignal, SignalGet, SignalUpdate},
    themes, vstack,
    window::Window,
};

use super::{
    MouseEventFn, Offset, Size, WidgetBase, WidgetExt, WidgetInternal,
    button::Button,
    input::InputBase,
    label::Label,
    stack::{HStack, VStack},
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
    click_fn: RefCell<Box<MouseEventFn<Self>>>,
}

impl<W: WidgetBase + 'static> InputBase for DropDown<W> {
    fn handle_key(&self, _: crate::event::Key) {}
}

impl<W: WidgetBase> WidgetBase for DropDown<W> {
    fn set_size(&self, size: Size) {
        self.base.set_size(size);
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
    fn set_padding(&self, padding: u32) {
        self.base.set_padding(padding);
    }
    fn set_border_radius(&self, radius: u32) {
        self.base.set_border_radius(radius);
    }
    fn set_color(&self, color: Rgba) {
        self.base.set_color(color);
    }
    fn set_text(&self, text: &str) {
        self.base.set_text(text);
    }
    fn get_background_color(&self) -> Rgba {
        self.base.get_background_color()
    }
    fn get_padding(&self) -> (u32, u32, u32, u32) {
        self.base.get_padding()
    }
    fn get_border_radius(&self) -> u32 {
        self.base.get_border_radius()
    }
    fn get_text(&self) -> String {
        self.selected.get().0
    }
}

impl<W: WidgetExt> WidgetExt for DropDown<W> {
    fn new() -> Rc<Self> {
        let signal = RwSignal::new(("".to_string(), false));
        let this = DropDown {
            base: dyn_label(move || signal.get().0)
                .padding(4)
                .frame(FrameType::Button)
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
    fn compute_size(&self, font: ab_glyph::FontArc) {
        self.base.compute_size(font.clone());
        self.overlay.compute_size(font);
    }
    fn get_size(&self) -> Size {
        self.base.get_size()
    }
    fn get_offset(&self) -> Offset {
        self.base.get_offset()
    }
    fn set_offset(&self, pos: Offset) {
        self.base.set_offset(pos);
        self.overlay_pos.set(pos);
        self.overlay.set_offset(self.overlay_pos.get());
    }
    fn get_frame(&self) -> themes::FrameFn {
        self.base.get_frame()
    }
    fn draw_frame(&self, _: &dyn Drawable) {}
    fn draw(&self, buf: &mut dyn Drawable) {
        if self.clicked.get() {
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
            let size = self.overlay.get_size();
            let mut overlay = Overlay::new(buf.clone(), Rect::new(offset, size));
            self.overlay.draw(&mut overlay);
            overlay.write();
        }
    }

    fn handle_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>) {
        let pos = pos - self.get_offset();

        let inside = pos.x >= 0
            && pos.y >= 0
            && pos.x <= self.get_size().w as i32
            && pos.y <= self.get_size().h as i32;

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
        let pos = pos - self.get_offset();

        let is_hovered = self.hovered.get().is_some();

        if pos.x < 0
            || pos.y < 0
            || pos.x > self.get_size().w as i32
            || pos.y > self.get_size().h as i32
        {
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
        self.overlay
            .clone()
            .handle_hover(pos - self.overlay_pos.get())
    }
    fn handle_overlay_button(self: Rc<Self>, pos: Offset, pressed: Option<Rc<Window>>) -> bool {
        if !self.selected.get().1 {
            return false;
        }
        self.overlay
            .clone()
            .handle_button(pos - self.overlay_pos.get(), pressed);
        let end = self.overlay.get_size() + self.overlay_pos.get();
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
            } else {
                w.set_background_color(Rgba::hex("#808080").unwrap());
                w
            }
        }),
    )
    .background_color(Rgba::hex("#606060").unwrap())
    .frame(FrameType::Frame)
    .padding(4);
    let this = DropDown {
        base: dyn_label(move || signal.get().0 + " ▾")
            .padding(4)
            .frame(FrameType::Button)
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
        click_fn: RefCell::new(Box::new(|_, _| {})),
    };
    this.base.set_frame(themes::FrameType::Button.to_string());
    this.base.set_padding(4);
    Rc::new(this)
}
