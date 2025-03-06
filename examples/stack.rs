use parslay::{Rgba, WidgetBase};

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        parslay::vstack(
            12,
            (
                parslay::label("Hello").background_color(Rgba::RED),
                parslay::label("World!").background_color(Rgba::RED),
            ),
        )
        .background_color(Rgba::MAGENTA)
    })?;
    Ok(())
}
