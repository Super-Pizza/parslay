fn main() -> simple_gui::Result<()> {
    simple_gui::launch(|| simple_gui::label("Hello, World!"))?;
    Ok(())
}
