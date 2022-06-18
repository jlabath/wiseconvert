use anyhow::{Context, Result};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename(deserialize = "TransferWise ID"))]
    pub transfer_wise_id: String,
    #[serde(rename(deserialize = "Date"), deserialize_with = "date_deserializer")]
    pub date: NaiveDate,
    #[serde(rename(deserialize = "Amount"))]
    pub amount: Decimal,
    #[serde(rename(deserialize = "Currency"))]
    pub currency: String,
    #[serde(rename(deserialize = "Description"))]
    pub description: String,
    #[serde(rename(deserialize = "Payment Reference"))]
    pub payment_reference: Option<String>,
    #[serde(rename(deserialize = "Running Balance"))]
    pub running_balance: Decimal,
    #[serde(rename(deserialize = "Exchange From"))]
    pub exchange_from: Option<String>,
    #[serde(rename(deserialize = "Exchange To"))]
    pub exchange_to: Option<String>,
    #[serde(rename(deserialize = "Exchange Rate"))]
    pub exchange_rate: Option<Decimal>,
    #[serde(rename(deserialize = "Payer Name"))]
    pub payer_name: Option<String>,
    #[serde(rename(deserialize = "Payee Name"))]
    pub payee_name: Option<String>,
    #[serde(rename(deserialize = "Payee Account Number"))]
    pub payee_account_number: Option<String>,
    #[serde(rename(deserialize = "Merchant"))]
    pub merchant: Option<String>,
    #[serde(rename(deserialize = "Card Last Four Digits"))]
    pub card_last_four: Option<u16>,
    #[serde(rename(deserialize = "Card Holder Full Name"))]
    pub card_holder_full_name: Option<String>,
    #[serde(rename(deserialize = "Attachment"))]
    pub attachment: Option<String>,
    #[serde(rename(deserialize = "Note"))]
    pub note: Option<String>,
    #[serde(rename(deserialize = "Total fees"))]
    pub total_fees: Decimal,
}

pub(crate) fn read_transactions(fname: &str) -> Result<Vec<Transaction>> {
    let mut r: Vec<Transaction> = vec![];
    let f = File::open(fname).with_context(|| format!("Failed to read from {}", fname))?;
    let br = BufReader::new(f);
    let mut reader = csv::Reader::from_reader(br);

    for record in reader.deserialize() {
        let record: Transaction = record.context("Reading data into Transaction failed")?;
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
