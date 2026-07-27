use crate::ansi::StyledLine;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum CoinType {
    Anipium,
    Batium,
    Mithril,
    Platinum,
}

impl CoinType {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "anipium" => Some(Self::Anipium),
            "batium" => Some(Self::Batium),
            "mithril" => Some(Self::Mithril),
            "platinum" => Some(Self::Platinum),
            _ => None,
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Anipium => "Anipium",
            Self::Batium => "Batium",
            Self::Mithril => "Mithril",
            Self::Platinum => "Platinum",
        }
    }

    fn multiplier(self) -> u64 {
        match self {
            Self::Anipium => 50,
            Self::Batium => 100,
            Self::Mithril => 500,
            Self::Platinum => 10,
        }
    }

    fn order_index(self) -> usize {
        match self {
            Self::Anipium => 0,
            Self::Batium => 1,
            Self::Mithril => 2,
            Self::Platinum => 3,
        }
    }
}

pub(crate) fn push_money_summary(list_text: &str, output_lines: &mut Vec<StyledLine>) {
    let normalized = list_text.trim().replace(" and ", ", ");
    let mut counts = [None; 4];
    let mut last_index = None;

    for entry in normalized.split(", ") {
        let mut parts = entry.splitn(2, ' ');
        let amount = parts.next().and_then(|value| value.parse::<u64>().ok());
        let coin = parts.next().and_then(CoinType::from_str);

        let (Some(amount), Some(coin)) = (amount, coin) else {
            return;
        };

        let idx = coin.order_index();
        if counts[idx].is_some() {
            return;
        }
        if let Some(last_idx) = last_index
            && idx <= last_idx
        {
            return;
        }

        counts[idx] = Some(amount);
        last_index = Some(idx);
    }

    if counts.iter().all(|value| value.is_none()) {
        return;
    }

    let mut total = 0u64;
    for coin in [
        CoinType::Platinum,
        CoinType::Anipium,
        CoinType::Batium,
        CoinType::Mithril,
    ] {
        if let Some(amount) = counts[coin.order_index()] {
            let value = amount * coin.multiplier();
            total += value;
            output_lines.push(StyledLine::new(&format!(
                "{} {} = {}",
                coin.display_name(),
                amount,
                value
            )));
        }
    }

    output_lines.push(StyledLine::new(&format!("Total = {}", total)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_summary_allows_missing_coin_types() {
        let mut lines = Vec::new();
        push_money_summary("2 anipium and 1 platinum", &mut lines);

        let plain: Vec<&str> = lines.iter().map(|line| line.plain_line.as_str()).collect();
        assert_eq!(
            plain,
            vec!["Platinum 1 = 10", "Anipium 2 = 100", "Total = 110"]
        );
    }

    #[test]
    fn money_summary_rejects_invalid_coin_order() {
        let mut lines = Vec::new();
        push_money_summary("1 platinum, 2 anipium", &mut lines);
        assert!(lines.is_empty());
    }
}
