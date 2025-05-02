use parslay::{Rgba, WidgetExt};

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        parslay::button(|| "Hello, World!").background_color(Rgba::from([192, 192, 192, 255]))
    })?;
    Ok(())
}
