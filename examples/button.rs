use parslay::prelude::*;

fn main() -> parslay::Result<()> {
    launch(|| {
        button("Hello, World!")
            .frame(FrameType::Button)
            .background_color(Rgba::hex("#c0c0c0").unwrap())
            .padding(4)
    })?;
    Ok(())
}
