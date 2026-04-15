use crate::expense::Expense;
use chrono::NaiveDate;
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
