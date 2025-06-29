use parslay::{FrameType, Rgba, WidgetExt};

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        parslay::button("Hello, World!")
            .frame(FrameType::Button)
            .background_color(Rgba::hex("#c0c0c0").unwrap())
            .padding(4)
    })?;
    Ok(())
}
