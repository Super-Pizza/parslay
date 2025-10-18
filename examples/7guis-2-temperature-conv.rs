use parslay::prelude::*;
use parslay::reactive::{SignalGet as _, SignalUpdate as _};
use parslay::widgets::input::InputExt;

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        let celsius = RwSignal::new(0.0);

        hstack(
            4,
            (
                dyn_input(move || format!("{}", celsius.get() as i32))
                    .on_edit(move |this| {
                        if let Ok(val) = this.get_text().parse::<f32>() {
                            celsius.update(|c| *c = val)
                        }
                    })
                    .padding(4),
                label("Celsius = ").padding(4),
                dyn_input(move || format!("{}", (celsius.get() * 9. / 5. + 32.) as i32))
                    .on_edit(move |this| {
                        if let Ok(val) = this.get_text().parse::<f32>() {
                            celsius.update(|c| *c = (val - 32.) * 5. / 9.)
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
