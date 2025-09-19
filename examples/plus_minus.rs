use parslay::prelude::*;
use parslay::reactive::RwSignal;

fn main() -> parslay::Result<()> {
    launch(|| {
        let mut counter = RwSignal::new(0);

        hstack(
            4,
            (
                button("-")
                    .frame(FrameType::Button)
                    .background_color(Rgba::hex("#C0C0C0").unwrap())
                    .padding(4)
                    .on_click(move |_, _| counter -= 1),
                dyn_label(move || format!("{counter}")).padding(4),
                button("+")
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
