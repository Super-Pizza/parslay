use parslay::{Rgba, WidgetBase};

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        parslay::vstack(
            12,
            (
                parslay::label("Hello")
                    .padding(4)
                    .background_color(Rgba::RED),
                parslay::label("World!")
                    .padding(4)
                    .background_color(Rgba::RED),
            ),
        )
        .padding(8)
        .border_radius(8)
        .background_color(Rgba::MAGENTA)
    })?;
    Ok(())
}
