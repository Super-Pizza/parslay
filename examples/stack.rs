use parslay::prelude::*;

fn main() -> parslay::Result<()> {
    launch(|| {
        vstack(
            12,
            (
                label("Hello")
                    .frame(FrameType::Box)
                    .background_color(Rgba::RED)
                    .padding(4),
                label("World!")
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
