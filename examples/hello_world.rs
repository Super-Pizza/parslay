use parslay::prelude::*;

fn main() -> parslay::Result<()> {
    launch(|| "Hello, World!")?;
    Ok(())
}
