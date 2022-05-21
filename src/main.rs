use anyhow::{Context, Result};
use chrono::NaiveDate;
use clap::{Arg, Command};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::BufReader;

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
                .help("Output file to write OFX into")
                .takes_value(true),
        )
        .get_matches();

    let _transactions =
        read_transactions(matches.value_of("statement file").unwrap_or("invalid.csv"))?;
    Ok(())
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Transaction {
    #[serde(rename(deserialize = "TransferWise ID"))]
    transfer_wise_id: String,
    #[serde(rename(deserialize = "Date"), deserialize_with = "date_deserializer")]
    date: NaiveDate,
    #[serde(rename(deserialize = "Amount"))]
    amount: Decimal,
    #[serde(rename(deserialize = "Currency"))]
    currency: String,
    #[serde(rename(deserialize = "Description"))]
    description: String,
    #[serde(rename(deserialize = "Payment Reference"))]
    payment_reference: Option<String>,
    #[serde(rename(deserialize = "Running Balance"))]
    running_balance: Decimal,
    #[serde(rename(deserialize = "Exchange From"))]
    exchange_from: Option<String>,
    #[serde(rename(deserialize = "Exchange To"))]
    exchange_to: Option<String>,
    #[serde(rename(deserialize = "Exchange Rate"))]
    exchange_rate: Option<Decimal>,
    #[serde(rename(deserialize = "Payer Name"))]
    payer_name: Option<String>,
    #[serde(rename(deserialize = "Payee Name"))]
    payee_name: Option<String>,
    #[serde(rename(deserialize = "Payee Account Number"))]
    payee_account_number: Option<String>,
    #[serde(rename(deserialize = "Merchant"))]
    merchant: Option<String>,
    #[serde(rename(deserialize = "Card Last Four Digits"))]
    card_last_four: Option<u16>,
    #[serde(rename(deserialize = "Card Holder Full Name"))]
    card_holder_full_name: Option<String>,
    #[serde(rename(deserialize = "Attachment"))]
    attachment: Option<String>,
    #[serde(rename(deserialize = "Note"))]
    note: Option<String>,
    #[serde(rename(deserialize = "Total fees"))]
    total_fees: Decimal,
}

fn read_transactions(fname: &str) -> Result<Vec<Transaction>> {
    let mut r: Vec<Transaction> = vec![];
    let f = File::open(fname).with_context(|| format!("Failed to read from {}", fname))?;
    let br = BufReader::new(f);
    let mut reader = csv::Reader::from_reader(br);

    for record in reader.deserialize() {
        let record: Transaction = record.context("Reading data into Transaction failed")?;
        println!("It is a {:?}.", record);
        r.push(record);
    }

    Ok(r)
}

const FORMAT: &str = "%d-%m-%Y";

pub fn date_deserializer<'de, D>(deserializer: D) -> std::result::Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
}
