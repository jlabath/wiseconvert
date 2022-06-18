use crate::transaction::Transaction;
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use rust_decimal::Decimal;
use std::borrow::Cow;
use std::cmp::Ordering;

const FMT: &str = "%Y%m%d%H%M%S%.3f[%z:%Z]";

pub(crate) fn make_iter(
    account: &str,
    transactions: Vec<Transaction>,
) -> impl Iterator<Item = Event<'static>> {
    let now: DateTime<Utc> = Utc::now();
    let fmt_now = now.format(FMT);

    let earliest = earliest_tx(&transactions);
    let latest = latest_tx(&transactions);
    let header = vec![
        Event::Decl(BytesDecl::new(b"1.0", None, None)),
        Event::PI(BytesText::from_escaped_str(
            r#"OFX OFXHEADER="200 VERSION="203" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE""#,
        )),
        Event::Start(BytesStart::owned_name("OFX")),
        Event::Start(BytesStart::owned_name("SIGNONMSGSRSV1")),
        Event::Start(BytesStart::owned_name("SONRS")),
        Event::Start(BytesStart::owned_name("STATUS")),
        Event::Start(BytesStart::owned_name("CODE")),
        Event::Text(BytesText::from_escaped_str("0")),
        Event::End(BytesEnd::owned("CODE".into())),
        Event::Start(BytesStart::owned_name("SEVERITY")),
        Event::Text(BytesText::from_escaped_str("INFO")),
        Event::End(BytesEnd::owned("SEVERITY".into())),
        Event::End(BytesEnd::owned("STATUS".into())),
        Event::Start(BytesStart::owned_name("DTSERVER")),
        Event::Text(BytesText::from_escaped_str(fmt_now.to_string())),
        Event::End(BytesEnd::owned("DTSERVER".into())),
        Event::Start(BytesStart::owned_name("LANGUAGE")),
        Event::Text(BytesText::from_escaped_str("ENG")),
        Event::End(BytesEnd::owned("LANGUAGE".into())),
        Event::End(BytesEnd::owned("SONRS".into())),
        Event::End(BytesEnd::owned("SIGNONMSGSRSV1".into())),
        Event::Start(BytesStart::owned_name("BANKMSGSRSV1")),
        Event::Start(BytesStart::owned_name("STMTTRNRS")),
        Event::Start(BytesStart::owned_name("TRNUID")),
        Event::Text(BytesText::from_escaped_str("0")),
        Event::End(BytesEnd::owned("TRNUID".into())),
        Event::Start(BytesStart::owned_name("STATUS")),
        Event::Start(BytesStart::owned_name("CODE")),
        Event::Text(BytesText::from_escaped_str("0")),
        Event::End(BytesEnd::owned("CODE".into())),
        Event::Start(BytesStart::owned_name("SEVERITY")),
        Event::Text(BytesText::from_escaped_str("INFO")),
        Event::End(BytesEnd::owned("SEVERITY".into())),
        Event::End(BytesEnd::owned("STATUS".into())),
        Event::Start(BytesStart::owned_name("STMTRS")),
        Event::Start(BytesStart::owned_name("CURDEF")),
        Event::Text(BytesText::from_escaped_str("CAD")), //hardcoded ???
        Event::End(BytesEnd::owned("CURDEF".into())),
        Event::Start(BytesStart::owned_name("BANKACCTFROM")),
        Event::Start(BytesStart::owned_name("ACCTID")),
        Event::Text(BytesText::from_escaped_str(account.to_string())),
        Event::End(BytesEnd::owned("ACCTID".into())),
        Event::End(BytesEnd::owned("BANKACCTFROM".into())),
        Event::Start(BytesStart::owned_name("BANKTRANLIST")),
    ];
    let mut body = vec![];
    if let Some(tx) = earliest {
        let txt = nd_dt(&tx.date).format(FMT).to_string();
        body.append(&mut plain_event("DTSTART".into(), txt.into(), false));
    }
    if let Some(tx) = latest {
        let txt = nd_dt(&tx.date).format(FMT).to_string();
        body.append(&mut plain_event("DTEND".into(), txt.into(), false));
    }
    let mut total = Decimal::ZERO;
    for tx in transactions.iter() {
        total += &tx.amount;
        body.append(&mut tx.as_events());
    }
    body.push(Event::End(BytesEnd::owned("BANKTRANLIST".into())));
    //do ledgerbal
    body.push(Event::Start(BytesStart::owned_name("LEDGERBAL")));
    body.append(&mut plain_event(
        "BALAMT".into(),
        total.to_string().into(),
        false,
    ));
    let date_str: String = {
        let date = match latest {
            Some(tx) => nd_dt(&tx.date),
            None => now,
        };
        date.format(FMT).to_string()
    };
    body.append(&mut plain_event("DTASOF".into(), date_str.into(), false));

    body.push(Event::End(BytesEnd::owned("LEDGERBAL".into())));

    let footer = vec![
        Event::End(BytesEnd::owned("STMTRS".into())),
        Event::End(BytesEnd::owned("STMTTRNRS".into())),
        Event::End(BytesEnd::owned("BANKMSGSRSV1".into())),
        Event::End(BytesEnd::owned("OFX".into())),
    ];
    header
        .into_iter()
        .chain(body.into_iter())
        .chain(footer.into_iter())
}

fn earliest_tx(transactions: &[Transaction]) -> Option<&Transaction> {
    let mut r = None;
    for tx in transactions {
        match r {
            None => r = Some(tx),
            Some(old) => match old.date.cmp(&tx.date) {
                Ordering::Greater => r = Some(tx),
                Ordering::Less => (),
                Ordering::Equal => {
                    if old.transfer_wise_id > tx.transfer_wise_id {
                        r = Some(tx)
                    }
                }
            },
        }
    }
    r
}

fn latest_tx(transactions: &[Transaction]) -> Option<&Transaction> {
    let mut r = None;
    for tx in transactions {
        match r {
            None => r = Some(tx),
            Some(old) => match old.date.cmp(&tx.date) {
                Ordering::Less => r = Some(tx),
                Ordering::Greater => (),
                Ordering::Equal => {
                    if old.transfer_wise_id < tx.transfer_wise_id {
                        r = Some(tx)
                    }
                }
            },
        }
    }
    r
}

fn plain_event<'a>(tag_name: Cow<'a, str>, val: Cow<'a, str>, escape: bool) -> Vec<Event<'static>> {
    let tag_str = String::from(tag_name);
    let tag = BytesStart::owned_name(tag_str);
    let end = tag.to_end().into_owned();
    vec![
        Event::Start(tag),
        Event::Text({
            if escape {
                BytesText::from_plain(val.as_bytes()).into_owned()
            } else {
                BytesText::from_escaped_str(val).into_owned()
            }
        }),
        Event::End(end),
    ]
}

fn nd_dt(d: &NaiveDate) -> DateTime<Utc> {
    Utc.ymd(d.year(), d.month(), d.day())
        .and_hms_milli(0, 0, 0, 0)
}

//extension trait for Transaction - could have been a function but having fun
trait TransactionExt {
    /// Convert transaction into xml events.
    ///
    /// Returns a vector of owned xml::Event
    fn as_events(&self) -> Vec<Event<'static>>;
}

impl TransactionExt for Transaction {
    fn as_events(&self) -> Vec<Event<'static>> {
        let ttype = if self.amount > Decimal::ZERO {
            "CREDIT"
        } else {
            "DEBIT"
        };
        let mut result = vec![Event::Start(BytesStart::owned_name("STMTTRN"))];
        result.append(&mut plain_event("TRNTYPE".into(), ttype.into(), false));
        result.append(&mut plain_event(
            "DTPOSTED".into(),
            nd_dt(&self.date).format(FMT).to_string().into(),
            false,
        ));
        result.append(&mut plain_event(
            "TRNAMT".into(),
            self.amount.to_string().into(),
            false,
        ));
        result.append(&mut plain_event(
            "FITID".into(),
            self.transfer_wise_id.as_str().into(),
            false,
        ));
        result.append(&mut plain_event(
            "REFNUM".into(),
            self.payment_reference
                .as_ref()
                .unwrap_or(&self.transfer_wise_id)
                .into(),
            false,
        ));
        result.append(&mut plain_event(
            "NAME".into(),
            self.merchant.as_ref().unwrap_or(&self.description).into(),
            false,
        ));
        result.push(Event::End(BytesEnd::owned("STMTTRN".into())));
        result
    }
}
