//! PickBackground label ordering and guild drill row structure for the Guild Dialog.

use super::THEMES_UX_ORDER;
use crate::guilds::grouping::{
    MULTI_BACKGROUND_LABEL, guild_grouping, visible_indices_multi_drill,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildDrillSource {
    Thematic(usize),
    MultiOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildBrowseRow {
    Banner(&'static str),
    Toggle { definition_index: usize },
}

pub fn browse_labels() -> Vec<&'static str> {
    THEMES_UX_ORDER
        .iter()
        .map(|(_, ui_label)| *ui_label)
        .chain(std::iter::once(MULTI_BACKGROUND_LABEL))
        .collect()
}

pub fn drill_rows(source: GuildDrillSource, entry_count: usize) -> Vec<GuildBrowseRow> {
    match source {
        GuildDrillSource::Thematic(thematic_ix) => {
            let bucket = &guild_grouping().thematic[thematic_ix];
            let thematic_indices: Vec<usize> = bucket
                .playable_def_indices
                .iter()
                .copied()
                .filter(|definition_index| *definition_index < entry_count)
                .collect();

            let multi_filtered: Vec<usize> = visible_indices_multi_drill()
                .into_iter()
                .filter(|definition_index| *definition_index < entry_count)
                .collect();

            thematic_drill_rows(&thematic_indices, &multi_filtered)
        }
        GuildDrillSource::MultiOnly => {
            let multis: Vec<_> = visible_indices_multi_drill()
                .into_iter()
                .filter(|definition_index| *definition_index < entry_count)
                .collect();
            multi_only_drill_rows(&multis)
        }
    }
}

fn thematic_drill_rows(
    thematic_indices: &[usize],
    multi_filtered: &[usize],
) -> Vec<GuildBrowseRow> {
    let mut rows = Vec::new();

    if thematic_indices.is_empty() && multi_filtered.is_empty() {
        rows.push(GuildBrowseRow::Banner(
            "Nothing implemented yet for this thematic drill.",
        ));
    } else if thematic_indices.is_empty() {
        rows.push(GuildBrowseRow::Banner(
            "(No playable guild in this thematic group yet)",
        ));
        rows.push(GuildBrowseRow::Banner("Multi-background guilds"));
    }

    for &definition_index in thematic_indices {
        rows.push(GuildBrowseRow::Toggle { definition_index });
    }

    if !thematic_indices.is_empty() && !multi_filtered.is_empty() {
        rows.push(GuildBrowseRow::Banner("Multi-background guilds"));
    }

    for &definition_index in multi_filtered {
        rows.push(GuildBrowseRow::Toggle { definition_index });
    }

    rows
}

fn multi_only_drill_rows(multi_indices: &[usize]) -> Vec<GuildBrowseRow> {
    let mut rows = Vec::new();
    if multi_indices.is_empty() {
        rows.push(GuildBrowseRow::Banner(
            "No multi-background guilds implemented.",
        ));
    }
    for &definition_index in multi_indices {
        rows.push(GuildBrowseRow::Toggle { definition_index });
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guilds::grouping::guild_grouping;

    fn toggle_indices(rows: &[GuildBrowseRow]) -> Vec<usize> {
        rows.iter()
            .filter_map(|row| match row {
                GuildBrowseRow::Toggle { definition_index } => Some(*definition_index),
                GuildBrowseRow::Banner(_) => None,
            })
            .collect()
    }

    fn banner_texts(rows: &[GuildBrowseRow]) -> Vec<&'static str> {
        rows.iter()
            .filter_map(|row| match row {
                GuildBrowseRow::Banner(text) => Some(*text),
                GuildBrowseRow::Toggle { .. } => None,
            })
            .collect()
    }

    #[test]
    fn browse_labels_match_themes_plus_multi_entry() {
        let labels = browse_labels();
        assert_eq!(labels.len(), THEMES_UX_ORDER.len() + 1);
        assert_eq!(labels[0], "Civilized");
        assert_eq!(labels[1], "Magical");
        assert_eq!(labels[2], "Good Religious");
        assert_eq!(labels[3], "Evil Religious");
        assert_eq!(labels[4], "Nomad");
        assert_eq!(labels[5], MULTI_BACKGROUND_LABEL);
    }

    #[test]
    fn thematic_drill_empty_when_entry_count_zero() {
        let rows = drill_rows(GuildDrillSource::Thematic(0), 0);
        assert_eq!(
            banner_texts(&rows),
            vec!["Nothing implemented yet for this thematic drill."]
        );
        assert!(toggle_indices(&rows).is_empty());
    }

    #[test]
    fn thematic_drill_partial_empty_includes_multi_section() {
        let rows = thematic_drill_rows(&[], &[0]);
        assert_eq!(
            banner_texts(&rows),
            vec![
                "(No playable guild in this thematic group yet)",
                "Multi-background guilds",
            ]
        );
        assert_eq!(toggle_indices(&rows), vec![0]);
    }

    #[test]
    fn multi_only_drill_empty_when_entry_count_zero() {
        let rows = drill_rows(GuildDrillSource::MultiOnly, 0);
        assert_eq!(
            banner_texts(&rows),
            vec!["No multi-background guilds implemented."]
        );
        assert!(toggle_indices(&rows).is_empty());
    }

    #[test]
    fn drill_rows_filter_toggle_indices_below_entry_count() {
        let entry_count = guild_grouping()
            .thematic
            .iter()
            .flat_map(|bucket| bucket.playable_def_indices.iter().copied())
            .max()
            .map(|max| max + 1)
            .unwrap_or(0);

        for thematic_ix in 0..THEMES_UX_ORDER.len() {
            let rows = drill_rows(GuildDrillSource::Thematic(thematic_ix), entry_count);
            for index in toggle_indices(&rows) {
                assert!(
                    index < entry_count,
                    "thematic {thematic_ix}: toggle index {index} >= entry_count {entry_count}"
                );
            }
        }

        let rows = drill_rows(GuildDrillSource::MultiOnly, entry_count);
        for index in toggle_indices(&rows) {
            assert!(
                index < entry_count,
                "multi-only: toggle index {index} >= entry_count {entry_count}"
            );
        }
    }
}
