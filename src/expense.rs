use chrono::NaiveDate;
use serde::Deserialize;

// TODO: #[serde(rename_all = "Title Case")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Expense {
    #[serde(alias = "Datum")]
    date: NaiveDate,
    #[serde(alias = "Utgift")]
    expense: Box<str>,
    #[serde(alias = "Klass")]
    class: Box<str>,
    #[serde(alias = "Mängd")]
    amount: f64,
    #[serde(alias = "Plats")]
    location: Box<str>,
}
