use parslay::prelude::*;
use parslay::reactive::{RwSignal, SignalGet as _, SignalWrite as _};
use parslay::widgets::input::InputExt;

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        let celsius = RwSignal::new(0.0);

        hstack(
            4,
            (
                dyn_input(move || format!("{}", celsius.get() as i32))
                    .on_edit(move |this| {
                        if let Ok(c) = this.get_text().parse::<f32>() {
                            *celsius.write_only().write().borrow_mut() = c
                        }
                    })
                    .padding(4),
                label("Celsius = ").padding(4),
                dyn_input(move || format!("{}", (celsius.get() * 9. / 5. + 32.) as i32))
                    .on_edit(move |this| {
                        if let Ok(f) = this.get_text().parse::<f32>() {
                            *celsius.write_only().write().borrow_mut() = (f - 32.) * 5. / 9.
                        }
                    })
                    .padding(4),
                label("Fahrenheit").padding(4),
            ),
        )
        .frame(FrameType::Frame)
        .background_color(Rgba::WHITE)
        .padding(8)
    })?;
    Ok(())
}
