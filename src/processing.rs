use crate::expense::Expense;
use chrono::Datelike as _;
use chrono::NaiveDate;
use itertools::Itertools as _;
use std::collections::BTreeMap;

pub fn day_sums<'expenses>(
    expenses: impl IntoIterator<Item = &'expenses Expense>,
) -> BTreeMap<NaiveDate, f64> {
    let mut sums = BTreeMap::new();

    for expense in expenses {
        *sums.entry(expense.date).or_default() += expense.amount;
    }

    sums
}

pub fn years<'expenses>(expenses: impl IntoIterator<Item = &'expenses Expense>) -> Vec<i32> {
    expenses
        .into_iter()
        .map(|expense| expense.date.year())
        .sorted()
        .dedup()
        .collect()
}
