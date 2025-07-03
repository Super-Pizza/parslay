use parslay::{FrameType, Rgba, WidgetExt};

fn main() -> parslay::Result<()> {
    parslay::launch(|| {
        parslay::input()
            .frame(FrameType::InputFrame)
            .background_color(Rgba::hex("#c0c0c0").unwrap())
            .padding(4)
    })?;
    Ok(())
}
