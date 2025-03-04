fn main() -> parslay::Result<()> {
    parslay::launch(|| parslay::vstack(12, (parslay::label("Hello"), parslay::label("World!"))))?;
    Ok(())
}
