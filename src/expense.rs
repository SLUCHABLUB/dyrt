use chrono::NaiveDate;
use serde::Deserialize;

// TODO: #[serde(rename_all = "Title Case")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Expense {
    #[serde(alias = "Datum")]
    pub date: NaiveDate,
    // #[serde(alias = "Utgift")]
    // pub expense: Box<str>,
    // #[serde(alias = "Klass")]
    // pub class: Box<str>,
    #[serde(alias = "Mängd")]
    pub amount: f64,
    // #[serde(alias = "Plats")]
    // pub location: Box<str>,
}
