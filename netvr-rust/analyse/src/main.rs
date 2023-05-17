mod parse;

use std::collections::HashMap;

use anyhow::Result;
use parse::{LogFile, Sample};

fn main() -> Result<()> {
    // read file from argv
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];
    let contents = std::fs::read_to_string(filename)?;

    // parse contents using nom
    let (_, lines) = LogFile::parse(&contents)?;

    if false {
        std::fs::write(
            "output.json",
            serde_json::to_string(&serde_json::to_value(lines)?)?,
        )?;
    }

    let best_pair = (0, 0);

    Ok(())
}
