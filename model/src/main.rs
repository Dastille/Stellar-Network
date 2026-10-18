//! Prints the generated block that `whitepaper/whitepaper.md` embeds.
fn main() -> Result<(), stellar_model::ModelError> {
    println!("{}", stellar_model::render_paper_block()?);
    Ok(())
}
