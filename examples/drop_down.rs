use parslay::prelude::*;

fn main() -> parslay::Result<()> {
    launch(|| drop_down("Select...", ("Apples", "Bananas")))?;
    Ok(())
}
