use anyhow::Result;
use clap::{Arg, Command};

use quick_xml::Writer;
use std::fs::File;
mod event;
mod transaction;

const CSV_FNAME: &str = "statement.csv";

fn main() -> Result<()> {
    let matches = Command::new("WiseImport")
        .version("0.1")
        .author("Jakub Labath. <jakub@labath.ca>")
        .about("Converts Wise CSV statement into OFX format")
        .arg(
            Arg::new("statement file")
                .long("statement")
                .help("Downloaded as standard statement in CSV format")
                .takes_value(true)
                .default_value(CSV_FNAME),
        )
        .arg(
            Arg::new("account")
                .long("account")
                .help("Account ID to be used in OFX output")
                .takes_value(true)
                .default_value("wise001"),
        )
        .arg(
            Arg::new("output file")
                .long("out")
                .help("Output file to write OFX into (stdout will be used if ommitted)")
                .takes_value(true),
        )
        .get_matches();

    //read input
    let transactions = transaction::read_transactions(
        matches.value_of("statement file").unwrap_or("invalid.csv"),
    )?;
    //prepare output
    let iter = event::make_iter(
        matches.value_of("account").unwrap_or("wise001"),
        transactions,
    );
    //write out output
    let fout = make_out_file(matches.value_of("output file"))?;
    let mut out = Writer::new(fout);
    for event in iter {
        out.write_event(event)?;
    }
    Ok(())
}

fn make_out_file(fname: Option<&str>) -> Result<Box<dyn std::io::Write>, std::io::Error> {
    let w: Box<dyn std::io::Write> = match fname {
        Some(name) => {
            let f = File::create(name)?;
            Box::new(f)
        }
        None => Box::new(std::io::stdout()),
    };
    Ok(w)
}
