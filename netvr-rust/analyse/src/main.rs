mod parse;

use anyhow::Result;

fn main() -> Result<()> {
    // read file from argv
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];
    let contents = std::fs::read_to_string(filename)?;

    // parse contents using nom
    let (_, lines) = parse::file(&contents)?;
    println!("{:#?}", lines.len());
    Ok(())
}
