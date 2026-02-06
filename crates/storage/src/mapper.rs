use domain::{DomainError, EntryKind};
use rusty_money::{Money, iso};

pub fn to_money(amount_cents: i64) -> Money<'static, iso::Currency> {
    Money::from_minor(amount_cents, iso::USD)
}

pub fn from_money(money: &Money<'static, iso::Currency>) -> i64 {
    let s = money.amount().to_string();
    if let Some(dot) = s.find('.') {
        let (int, frac) = s.split_at(dot);
        let frac = &frac[1..];
        let mut cents = frac.to_string();
        while cents.len() < 2 {
            cents.push('0');
        }
        if cents.len() > 2 {
            cents.truncate(2);
        }
        let total = format!("{}{}", int, cents);
        total.parse::<i64>().unwrap_or(0)
    } else {
        s.parse::<i64>().unwrap_or(0) * 100
    }
}

pub fn kind_to_str(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Expense => "expense",
        EntryKind::Income => "income",
    }
}

pub fn kind_from_str(value: String) -> Result<EntryKind, DomainError> {
    match value.as_str() {
        "expense" => Ok(EntryKind::Expense),
        "income" => Ok(EntryKind::Income),
        _ => Err(DomainError::InvalidData(format!(
            "unknown entry kind: {value}"
        ))),
    }
}
