mod expense;

use crate::expense::Expense;
use anyhow::Context;
use clap::Parser;
use std::path::Path;

#[derive(Parser)]
struct Arguments {
    path: Box<Path>,
}

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    let expenses: Vec<_> = csv::Reader::from_path(&arguments.path)
        .with_context(|| format!("reading file: `{}`", arguments.path.display()))?
        .deserialize::<Expense>()
        .collect::<Result<_, _>>()?;

    dbg!(expenses);

    Ok(())
}
