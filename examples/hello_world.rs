fn main() -> parslay::Result<()> {
    parslay::launch(|| parslay::label(|| "Hello, World!"))?;
    Ok(())
}
