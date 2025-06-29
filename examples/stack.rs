use parslay::{FrameType, Rgba, WidgetExt};

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        parslay::vstack(
            12,
            (
                parslay::label("Hello")
                    .frame(FrameType::Box)
                    .background_color(Rgba::RED)
                    .padding(4),
                parslay::label("World!")
                    .frame(FrameType::Box)
                    .background_color(Rgba::RED)
                    .padding(4),
            ),
        )
        .frame(FrameType::Frame)
        .background_color(Rgba::MAGENTA)
        .padding(8)
    })?;
    Ok(())
}
