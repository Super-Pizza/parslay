use parslay::reactive::RwSignal;
use parslay::{FrameType, Rgba, WidgetExt};

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        let mut counter = RwSignal::new(0);

        parslay::hstack(
            4,
            (
                parslay::button("-")
                    .frame(FrameType::Button)
                    .background_color(Rgba::hex("#C0C0C0").unwrap())
                    .padding(4)
                    .on_click(move |_, _| counter -= 1),
                parslay::dyn_label(move || format!("{counter}")).padding(4),
                parslay::button("+")
                    .frame(FrameType::Button)
                    .background_color(Rgba::hex("#C0C0C0").unwrap())
                    .padding(4)
                    .on_click(move |_, _| counter += 1),
            ),
        )
        .frame(FrameType::Frame)
        .background_color(Rgba::WHITE)
        .padding(8)
    })?;
    Ok(())
}
