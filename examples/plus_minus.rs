use parslay::reactive::RwSignal;
use parslay::{Rgba, WidgetExt};

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        let mut counter = RwSignal::new(0);

        parslay::hstack(
            4,
            (
                parslay::button("-")
                    .padding(4)
                    .border_radius(4)
                    .background_color(Rgba::hex("#C0C0C0").unwrap())
                    .on_click(move |_, _| counter -= 1),
                parslay::dyn_label(move || format!("{counter}")).padding(4),
                parslay::button("+")
                    .padding(4)
                    .border_radius(4)
                    .background_color(Rgba::hex("#C0C0C0").unwrap())
                    .on_click(move |_, _| counter += 1),
            ),
        )
        .padding(8)
        .border_radius(8)
        .background_color(Rgba::WHITE)
    })?;
    Ok(())
}
