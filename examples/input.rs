use parslay::prelude::*;

fn main() -> parslay::Result<()> {
    launch(|| {
        input()
            .frame(FrameType::InputFrame)
            .background_color(Rgba::hex("#c0c0c0").unwrap())
            .padding(8)
    })?;
    Ok(())
}
