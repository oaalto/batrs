use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, OptionalExtension};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    Hours24,
    Days7,
    All,
}

impl TimeRange {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("24h") => Self::Hours24,
            Some("7d") => Self::Days7,
            _ => Self::All,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hours24 => "24h",
            Self::Days7 => "7d",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterParams {
    pub range: TimeRange,
    pub player: Option<String>,
}

impl FilterParams {
    pub fn from_query(range: Option<&str>, player: Option<&str>) -> Self {
        Self {
            range: TimeRange::parse(range),
            player: player.filter(|name| !name.is_empty()).map(str::to_string),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("desc") => Self::Desc,
            _ => Self::Asc,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LandingSortColumn {
    Verb,
    ConfObs,
    ConfMin,
    ConfMax,
    ConfAvg,
    EstObs,
    EstMin,
    EstMax,
    EstAvg,
}

impl LandingSortColumn {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("conf_obs") => Self::ConfObs,
            Some("conf_min") => Self::ConfMin,
            Some("conf_max") => Self::ConfMax,
            Some("conf_avg") => Self::ConfAvg,
            Some("est_obs") => Self::EstObs,
            Some("est_min") => Self::EstMin,
            Some("est_max") => Self::EstMax,
            Some("est_avg") => Self::EstAvg,
            _ => Self::Verb,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventSortColumn {
    RecordedAt,
    Player,
    HpDelta,
    SourceName,
    Confidence,
    Weight,
    CandidateCount,
}

impl EventSortColumn {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("player") => Self::Player,
            Some("hp_delta") => Self::HpDelta,
            Some("source_name") => Self::SourceName,
            Some("confidence") => Self::Confidence,
            Some("weight") => Self::Weight,
            Some("candidate_count") => Self::CandidateCount,
            _ => Self::RecordedAt,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerbAggregate {
    pub verb: String,
    pub weapon_family: Option<String>,
    pub catalog_rank: Option<i32>,
    pub confirmed_obs: i64,
    pub confirmed_min: Option<i32>,
    pub confirmed_max: Option<i32>,
    pub confirmed_avg: Option<f64>,
    pub estimated_obs: i64,
    pub estimated_min: Option<i32>,
    pub estimated_max: Option<i32>,
    pub estimated_avg: Option<f64>,
    pub estimated_loose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventRowRole {
    Focal,
    Sibling,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamageEvent {
    pub id: i64,
    pub batch_id: i64,
    pub recorded_at: String,
    pub player: String,
    pub hp_delta: i32,
    pub damage_min: i32,
    pub damage_max: i32,
    pub source_name: String,
    pub confidence: f64,
    pub weight: f64,
    pub candidate_count: i32,
    pub message_verb: String,
    pub damage_category: String,
    pub message_text: String,
    pub row_role: EventRowRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchCandidate {
    pub category: String,
    pub family: Option<String>,
    pub verb: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AggKey {
    category: String,
    family: Option<String>,
    verb: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeleeFamilySection {
    pub family_id: String,
    pub family_title: String,
    pub aggregates: Vec<VerbAggregate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectiveBounds {
    pub min: i32,
    pub max: i32,
    pub loose: bool,
}

pub fn extrapolate_batch(
    hp_delta: i32,
    candidates: &[BatchCandidate],
    known_mins: &HashMap<AggKey, i32>,
) -> Vec<EffectiveBounds> {
    let loose = |loose: bool| EffectiveBounds {
        min: 0,
        max: hp_delta,
        loose,
    };

    if candidates.is_empty() {
        return Vec::new();
    }

    for candidate in candidates {
        let key = agg_key(
            &candidate.category,
            candidate.family.as_deref(),
            &candidate.verb,
        );
        if known_mins.get(&key).copied() == Some(hp_delta) {
            return candidates
                .iter()
                .map(|row| {
                    if row.verb == candidate.verb
                        && row.category == candidate.category
                        && row.family == candidate.family
                    {
                        EffectiveBounds {
                            min: hp_delta,
                            max: hp_delta,
                            loose: false,
                        }
                    } else {
                        EffectiveBounds {
                            min: 0,
                            max: 0,
                            loose: false,
                        }
                    }
                })
                .collect();
        }
    }

    let sum_known_min: i32 = candidates
        .iter()
        .map(|candidate| {
            known_mins
                .get(&agg_key(
                    &candidate.category,
                    candidate.family.as_deref(),
                    &candidate.verb,
                ))
                .copied()
                .unwrap_or(0)
        })
        .sum();

    if sum_known_min > hp_delta {
        return vec![loose(true); candidates.len()];
    }

    vec![loose(true); candidates.len()]
}

fn range_cutoff(range: TimeRange) -> Option<DateTime<Utc>> {
    let now = Utc::now();
    match range {
        TimeRange::Hours24 => Some(now - Duration::hours(24)),
        TimeRange::Days7 => Some(now - Duration::days(7)),
        TimeRange::All => None,
    }
}

fn filter_clause(filters: &FilterParams) -> (String, Vec<String>) {
    let mut clauses = Vec::new();
    let mut params = Vec::new();
    if let Some(cutoff) = range_cutoff(filters.range) {
        clauses.push("recorded_at >= ?".to_string());
        params.push(cutoff.to_rfc3339());
    }
    if let Some(player) = &filters.player {
        clauses.push("player = ?".to_string());
        params.push(player.clone());
    }
    let sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" AND {}", clauses.join(" AND "))
    };
    (sql, params)
}

pub fn list_players(conn: &Connection, filters: &FilterParams) -> Result<Vec<String>, String> {
    let (extra, params) = filter_clause(filters);
    let sql = format!(
        "SELECT DISTINCT player FROM (
            SELECT player FROM damage_events WHERE 1=1{extra}
            UNION
            SELECT player FROM unattributed_hp_events WHERE 1=1{extra}
        ) ORDER BY player"
    );
    let mut all_params = params.clone();
    all_params.extend(params);
    let mut statement = conn.prepare(&sql).map_err(|err| err.to_string())?;
    let rows = statement
        .query_map(rusqlite::params_from_iter(all_params.drain(..)), |row| {
            row.get::<_, String>(0)
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())
}

struct RawEventRow {
    id: i64,
    batch_id: i64,
    recorded_at: String,
    player: String,
    hp_delta: i32,
    damage_category: String,
    source_name: String,
    message_verb: String,
    message_text: String,
    candidate_count: i32,
    confidence: f64,
    damage_min: i32,
    damage_max: i32,
    catalog_rank: Option<i32>,
    weapon_family: Option<String>,
    weight: f64,
}

fn agg_key(category: &str, family: Option<&str>, verb: &str) -> AggKey {
    AggKey {
        category: category.to_string(),
        family: family.map(str::to_string),
        verb: verb.to_string(),
    }
}

fn load_filtered_events(
    conn: &Connection,
    filters: &FilterParams,
    category: Option<&str>,
    verb: Option<&str>,
    family: Option<&str>,
) -> Result<Vec<RawEventRow>, String> {
    let (extra, mut params) = filter_clause(filters);
    let mut sql = format!(
        "SELECT id, batch_id, recorded_at, player, hp_delta, damage_category, source_name,
                message_verb, message_text, candidate_count, confidence, damage_min, damage_max,
                catalog_rank, weapon_family, weight
         FROM damage_events
         WHERE 1=1{extra}"
    );
    if let Some(category) = category {
        sql.push_str(" AND damage_category = ?");
        params.push(category.to_string());
    }
    if let Some(verb) = verb {
        sql.push_str(" AND message_verb = ?");
        params.push(verb.to_string());
    }
    if let Some(family) = family {
        sql.push_str(" AND weapon_family = ?");
        params.push(family.to_string());
    }
    let mut statement = conn.prepare(&sql).map_err(|err| err.to_string())?;
    let rows = statement
        .query_map(rusqlite::params_from_iter(params.drain(..)), |row| {
            Ok(RawEventRow {
                id: row.get(0)?,
                batch_id: row.get(1)?,
                recorded_at: row.get(2)?,
                player: row.get(3)?,
                hp_delta: row.get(4)?,
                damage_category: row.get(5)?,
                source_name: row.get(6)?,
                message_verb: row.get(7)?,
                message_text: row.get(8)?,
                candidate_count: row.get(9)?,
                confidence: row.get(10)?,
                damage_min: row.get(11)?,
                damage_max: row.get(12)?,
                catalog_rank: row.get(13)?,
                weapon_family: row.get(14)?,
                weight: row.get(15)?,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())
}

fn load_batch_siblings(
    conn: &Connection,
    filters: &FilterParams,
    batch_ids: &[i64],
    exclude_ids: &[i64],
) -> Result<Vec<RawEventRow>, String> {
    if batch_ids.is_empty() {
        return Ok(Vec::new());
    }
    let (extra, filter_params) = filter_clause(filters);
    let batch_placeholders = batch_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let exclude_sql = if exclude_ids.is_empty() {
        String::new()
    } else {
        let placeholders = exclude_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        format!(" AND id NOT IN ({placeholders})")
    };
    let sql = format!(
        "SELECT id, batch_id, recorded_at, player, hp_delta, damage_category, source_name,
                message_verb, message_text, candidate_count, confidence, damage_min, damage_max,
                catalog_rank, weapon_family, weight
         FROM damage_events
         WHERE batch_id IN ({batch_placeholders}){exclude_sql}{extra}"
    );
    let mut params = batch_ids
        .iter()
        .chain(exclude_ids.iter())
        .map(|id| id.to_string())
        .collect::<Vec<_>>();
    params.extend(filter_params);
    let mut statement = conn.prepare(&sql).map_err(|err| err.to_string())?;
    let rows = statement
        .query_map(rusqlite::params_from_iter(params.drain(..)), |row| {
            Ok(RawEventRow {
                id: row.get(0)?,
                batch_id: row.get(1)?,
                recorded_at: row.get(2)?,
                player: row.get(3)?,
                hp_delta: row.get(4)?,
                damage_category: row.get(5)?,
                source_name: row.get(6)?,
                message_verb: row.get(7)?,
                message_text: row.get(8)?,
                candidate_count: row.get(9)?,
                confidence: row.get(10)?,
                damage_min: row.get(11)?,
                damage_max: row.get(12)?,
                catalog_rank: row.get(13)?,
                weapon_family: row.get(14)?,
                weight: row.get(15)?,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())
}

fn raw_to_damage_event(row: RawEventRow, row_role: EventRowRole) -> DamageEvent {
    DamageEvent {
        id: row.id,
        batch_id: row.batch_id,
        recorded_at: row.recorded_at,
        player: row.player,
        hp_delta: row.hp_delta,
        damage_min: row.damage_min,
        damage_max: row.damage_max,
        source_name: row.source_name,
        confidence: row.confidence,
        weight: row.weight,
        candidate_count: row.candidate_count,
        message_verb: row.message_verb,
        damage_category: row.damage_category,
        message_text: row.message_text,
        row_role,
    }
}

fn known_mins_from_isolated(rows: &[RawEventRow]) -> HashMap<AggKey, i32> {
    let mut mins = HashMap::new();
    for row in rows.iter().filter(|row| row.candidate_count == 1) {
        let key = agg_key(
            &row.damage_category,
            row.weapon_family.as_deref(),
            &row.message_verb,
        );
        mins.entry(key)
            .and_modify(|current: &mut i32| *current = (*current).min(row.hp_delta))
            .or_insert(row.hp_delta);
    }
    mins
}

struct EstimatedContribution {
    key: AggKey,
    min: i32,
    max: i32,
    loose: bool,
    point_estimate: f64,
}

fn rank_point_estimates(batch_rows: &[&RawEventRow], hp_delta: i32) -> Vec<f64> {
    let sum_ranks: i32 = batch_rows.iter().filter_map(|row| row.catalog_rank).sum();
    if sum_ranks > 0 {
        batch_rows
            .iter()
            .map(|row| {
                row.catalog_rank
                    .map(|rank| f64::from(hp_delta) * f64::from(rank) / f64::from(sum_ranks))
                    .unwrap_or(0.0)
            })
            .collect()
    } else {
        let share = f64::from(hp_delta) / batch_rows.len().max(1) as f64;
        vec![share; batch_rows.len()]
    }
}

fn estimated_contributions(rows: &[RawEventRow]) -> Vec<EstimatedContribution> {
    let known_mins = known_mins_from_isolated(rows);
    let mut contributions = Vec::new();

    let mut batches: HashMap<i64, Vec<&RawEventRow>> = HashMap::new();
    for row in rows.iter().filter(|row| row.candidate_count > 1) {
        batches.entry(row.batch_id).or_default().push(row);
    }

    for batch_rows in batches.values() {
        let hp_delta = batch_rows[0].hp_delta;
        let candidates = batch_rows
            .iter()
            .map(|row| BatchCandidate {
                category: row.damage_category.clone(),
                family: row.weapon_family.clone(),
                verb: row.message_verb.clone(),
            })
            .collect::<Vec<_>>();
        let bounds = extrapolate_batch(hp_delta, &candidates, &known_mins);
        let batch_loose = bounds.iter().all(|bound| bound.loose);
        let point_estimates = if batch_loose {
            rank_point_estimates(batch_rows, hp_delta)
        } else {
            bounds
                .iter()
                .map(|bound| (f64::from(bound.min) + f64::from(bound.max)) / 2.0)
                .collect()
        };
        for ((row, bound), point_estimate) in batch_rows.iter().zip(bounds).zip(point_estimates) {
            contributions.push(EstimatedContribution {
                key: agg_key(
                    &row.damage_category,
                    row.weapon_family.as_deref(),
                    &row.message_verb,
                ),
                min: bound.min,
                max: bound.max,
                loose: bound.loose,
                point_estimate,
            });
        }
    }

    contributions
}

fn rollup_confirmed(rows: &[RawEventRow]) -> HashMap<AggKey, VerbAggregate> {
    let mut by_verb = HashMap::new();
    for row in rows.iter().filter(|row| row.candidate_count == 1) {
        let key = agg_key(
            &row.damage_category,
            row.weapon_family.as_deref(),
            &row.message_verb,
        );
        let entry = by_verb.entry(key).or_insert(VerbAggregate {
            verb: row.message_verb.clone(),
            weapon_family: row.weapon_family.clone(),
            catalog_rank: row.catalog_rank,
            confirmed_obs: 0,
            confirmed_min: None,
            confirmed_max: None,
            confirmed_avg: None,
            estimated_obs: 0,
            estimated_min: None,
            estimated_max: None,
            estimated_avg: None,
            estimated_loose: false,
        });
        entry.confirmed_obs += 1;
        if let Some(rank) = row.catalog_rank {
            entry.catalog_rank = Some(entry.catalog_rank.map_or(rank, |current| current.min(rank)));
        }
        entry.confirmed_min = Some(
            entry
                .confirmed_min
                .map_or(row.hp_delta, |current| current.min(row.hp_delta)),
        );
        entry.confirmed_max = Some(
            entry
                .confirmed_max
                .map_or(row.hp_delta, |current| current.max(row.hp_delta)),
        );
    }
    for (key, aggregate) in by_verb.iter_mut() {
        if aggregate.confirmed_obs > 0 {
            let sum: i64 = rows
                .iter()
                .filter(|row| {
                    row.candidate_count == 1
                        && agg_key(
                            &row.damage_category,
                            row.weapon_family.as_deref(),
                            &row.message_verb,
                        ) == *key
                })
                .map(|row| i64::from(row.hp_delta))
                .sum();
            aggregate.confirmed_avg = Some(sum as f64 / aggregate.confirmed_obs as f64);
        }
    }
    by_verb
}

fn rollup_estimated(
    confirmed: &mut HashMap<AggKey, VerbAggregate>,
    contributions: &[EstimatedContribution],
) {
    for contribution in contributions {
        let entry = confirmed
            .entry(contribution.key.clone())
            .or_insert_with(|| VerbAggregate {
                verb: contribution.key.verb.clone(),
                weapon_family: contribution.key.family.clone(),
                catalog_rank: None,
                confirmed_obs: 0,
                confirmed_min: None,
                confirmed_max: None,
                confirmed_avg: None,
                estimated_obs: 0,
                estimated_min: None,
                estimated_max: None,
                estimated_avg: None,
                estimated_loose: false,
            });
        entry.estimated_obs += 1;
        entry.estimated_min = Some(
            entry
                .estimated_min
                .map_or(contribution.min, |current| current.min(contribution.min)),
        );
        entry.estimated_max = Some(
            entry
                .estimated_max
                .map_or(contribution.max, |current| current.max(contribution.max)),
        );
        entry.estimated_loose |= contribution.loose;
    }
    for (key, aggregate) in confirmed.iter_mut() {
        if aggregate.estimated_obs > 0 {
            let points: Vec<f64> = contributions
                .iter()
                .filter(|row| row.key == *key)
                .map(|row| row.point_estimate)
                .collect();
            aggregate.estimated_avg = Some(points.iter().sum::<f64>() / points.len().max(1) as f64);
        }
    }
}

fn category_aggregates_from_rows(
    rows: &[RawEventRow],
    sort_col: LandingSortColumn,
    sort_dir: SortDirection,
) -> Vec<VerbAggregate> {
    let contributions = estimated_contributions(rows);
    let mut aggregates = rollup_confirmed(rows);
    rollup_estimated(&mut aggregates, &contributions);
    let mut list = aggregates.into_values().collect::<Vec<_>>();
    sort_verb_aggregates(&mut list, sort_col, sort_dir);
    list
}

pub fn category_aggregates(
    conn: &Connection,
    category: &str,
    filters: &FilterParams,
    sort_col: LandingSortColumn,
    sort_dir: SortDirection,
) -> Result<Vec<VerbAggregate>, String> {
    let rows = load_filtered_events(conn, filters, Some(category), None, None)?;
    Ok(category_aggregates_from_rows(&rows, sort_col, sort_dir))
}

pub fn melee_family_aggregates(
    conn: &Connection,
    filters: &FilterParams,
    sort_col: LandingSortColumn,
    sort_dir: SortDirection,
) -> Result<Vec<MeleeFamilySection>, String> {
    use crate::combat_damage::catalog::{FAMILY_IDS, FAMILY_TITLES};

    let rows = load_filtered_events(conn, filters, Some("melee"), None, None)?;
    let aggregates = category_aggregates_from_rows(&rows, sort_col, sort_dir);

    let mut by_family: HashMap<String, Vec<VerbAggregate>> = HashMap::new();
    let mut unknown = Vec::new();
    for aggregate in aggregates {
        match aggregate.weapon_family.as_deref() {
            Some(family) => by_family
                .entry(family.to_string())
                .or_default()
                .push(aggregate),
            None => unknown.push(aggregate),
        }
    }

    let mut sections = Vec::new();
    for (index, family_id) in FAMILY_IDS.iter().enumerate() {
        let Some(family_rows) = by_family.remove(*family_id) else {
            continue;
        };
        sections.push(MeleeFamilySection {
            family_id: (*family_id).to_string(),
            family_title: FAMILY_TITLES[index].to_string(),
            aggregates: family_rows,
        });
    }
    if !unknown.is_empty() {
        sections.push(MeleeFamilySection {
            family_id: "unknown".to_string(),
            family_title: "Unknown family".to_string(),
            aggregates: unknown,
        });
    }
    Ok(sections)
}

fn sort_verb_aggregates(
    aggregates: &mut [VerbAggregate],
    sort_col: LandingSortColumn,
    sort_dir: SortDirection,
) {
    aggregates.sort_by(|left, right| {
        let ordering = match sort_col {
            LandingSortColumn::Verb => left
                .catalog_rank
                .cmp(&right.catalog_rank)
                .then_with(|| left.verb.cmp(&right.verb)),
            LandingSortColumn::ConfObs => left.confirmed_obs.cmp(&right.confirmed_obs),
            LandingSortColumn::ConfMin => cmp_option_i32(left.confirmed_min, right.confirmed_min),
            LandingSortColumn::ConfMax => cmp_option_i32(left.confirmed_max, right.confirmed_max),
            LandingSortColumn::ConfAvg => cmp_option_f64(left.confirmed_avg, right.confirmed_avg),
            LandingSortColumn::EstObs => left.estimated_obs.cmp(&right.estimated_obs),
            LandingSortColumn::EstMin => cmp_option_i32(left.estimated_min, right.estimated_min),
            LandingSortColumn::EstMax => cmp_option_i32(left.estimated_max, right.estimated_max),
            LandingSortColumn::EstAvg => cmp_option_f64(left.estimated_avg, right.estimated_avg),
        };
        match sort_dir {
            SortDirection::Asc => ordering,
            SortDirection::Desc => ordering.reverse(),
        }
    });
}

fn cmp_option_i32(left: Option<i32>, right: Option<i32>) -> std::cmp::Ordering {
    match (left, right) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(left), Some(right)) => left.cmp(&right),
    }
}

fn cmp_option_f64(left: Option<f64>, right: Option<f64>) -> std::cmp::Ordering {
    match (left, right) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(left), Some(right)) => left
            .partial_cmp(&right)
            .unwrap_or(std::cmp::Ordering::Equal),
    }
}

pub fn list_events(
    conn: &Connection,
    category: &str,
    verb: &str,
    family: Option<&str>,
    filters: &FilterParams,
    sort_col: EventSortColumn,
    sort_dir: SortDirection,
) -> Result<Vec<DamageEvent>, String> {
    let focal_rows = load_filtered_events(conn, filters, Some(category), Some(verb), family)?;
    let focal_ids = focal_rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let ambiguous_batch_ids = focal_rows
        .iter()
        .filter(|row| row.candidate_count > 1)
        .map(|row| row.batch_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let sibling_rows = load_batch_siblings(conn, filters, &ambiguous_batch_ids, &focal_ids)?;

    let mut siblings_by_batch: HashMap<i64, Vec<DamageEvent>> = HashMap::new();
    for row in sibling_rows {
        let event = raw_to_damage_event(row, EventRowRole::Sibling);
        siblings_by_batch
            .entry(event.batch_id)
            .or_default()
            .push(event);
    }
    for siblings in siblings_by_batch.values_mut() {
        siblings.sort_by(|left, right| {
            left.damage_category
                .cmp(&right.damage_category)
                .then_with(|| left.message_verb.cmp(&right.message_verb))
        });
    }

    let mut focal_events = focal_rows
        .into_iter()
        .map(|row| raw_to_damage_event(row, EventRowRole::Focal))
        .collect::<Vec<_>>();
    sort_events(&mut focal_events, sort_col, sort_dir);

    let mut events = Vec::new();
    for focal in focal_events {
        let batch_id = focal.batch_id;
        let show_siblings = focal.candidate_count > 1;
        events.push(focal);
        if show_siblings && let Some(siblings) = siblings_by_batch.get(&batch_id) {
            events.extend(siblings.iter().cloned());
        }
    }
    Ok(events)
}

fn sort_events(events: &mut [DamageEvent], sort_col: EventSortColumn, sort_dir: SortDirection) {
    events.sort_by(|left, right| {
        let ordering = match sort_col {
            EventSortColumn::RecordedAt => left.recorded_at.cmp(&right.recorded_at),
            EventSortColumn::Player => left.player.cmp(&right.player),
            EventSortColumn::HpDelta => left.hp_delta.cmp(&right.hp_delta),
            EventSortColumn::SourceName => left.source_name.cmp(&right.source_name),
            EventSortColumn::Confidence => left
                .confidence
                .partial_cmp(&right.confidence)
                .unwrap_or(std::cmp::Ordering::Equal),
            EventSortColumn::Weight => left
                .weight
                .partial_cmp(&right.weight)
                .unwrap_or(std::cmp::Ordering::Equal),
            EventSortColumn::CandidateCount => left.candidate_count.cmp(&right.candidate_count),
        };
        match sort_dir {
            SortDirection::Asc => ordering,
            SortDirection::Desc => ordering.reverse(),
        }
    });
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnattributedHpSummary {
    pub id: i64,
    pub recorded_at: String,
    pub player: String,
    pub hp_delta: i32,
    pub line_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnattributedHpDetail {
    pub id: i64,
    pub recorded_at: String,
    pub player: String,
    pub hp_delta: i32,
    pub hp_before: i32,
    pub hp_after: i32,
    pub h_line_text: String,
    pub context_lines: Vec<String>,
}

pub fn list_unattributed(
    conn: &Connection,
    filters: &FilterParams,
) -> Result<Vec<UnattributedHpSummary>, String> {
    let (extra, mut params) = filter_clause(filters);
    let sql = format!(
        "SELECT id, recorded_at, player, hp_delta, json_array_length(context_lines)
         FROM unattributed_hp_events
         WHERE 1=1{extra}
         ORDER BY recorded_at DESC, id DESC"
    );
    let mut statement = conn.prepare(&sql).map_err(|err| err.to_string())?;
    let rows = statement
        .query_map(rusqlite::params_from_iter(params.drain(..)), |row| {
            Ok(UnattributedHpSummary {
                id: row.get(0)?,
                recorded_at: row.get(1)?,
                player: row.get(2)?,
                hp_delta: row.get(3)?,
                line_count: row.get::<_, i64>(4)? as usize,
            })
        })
        .map_err(|err| err.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())
}

pub fn get_unattributed(
    conn: &Connection,
    id: i64,
) -> Result<Option<UnattributedHpDetail>, String> {
    let mut statement = conn
        .prepare(
            "
            SELECT id, recorded_at, player, hp_delta, hp_before, hp_after,
                   h_line_text, context_lines
            FROM unattributed_hp_events
            WHERE id = ?1
            ",
        )
        .map_err(|err| err.to_string())?;
    let row = statement
        .query_row([id], |row| {
            let context_json: String = row.get(7)?;
            Ok(UnattributedHpDetail {
                id: row.get(0)?,
                recorded_at: row.get(1)?,
                player: row.get(2)?,
                hp_delta: row.get(3)?,
                hp_before: row.get(4)?,
                hp_after: row.get(5)?,
                h_line_text: row.get(6)?,
                context_lines: crate::combat_damage::collector::parse_json_string_array(
                    &context_json,
                ),
            })
        })
        .optional()
        .map_err(|err| err.to_string())?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_damage::test_fixtures::{FixtureRow, open_fixture_db, remove_db_files};

    #[test]
    fn isolated_only_verb_does_not_double_count_estimated_obs() {
        let path = open_fixture_db(&[FixtureRow::isolated(
            "melee",
            "perforate",
            "Odefu",
            68,
            "2026-08-06T08:06:57Z",
        )]);
        let conn = Connection::open(&path).unwrap();
        let aggregates = category_aggregates(
            &conn,
            "melee",
            &FilterParams::from_query(None, None),
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        let perforate = aggregates
            .iter()
            .find(|row| row.verb == "perforate")
            .unwrap();
        assert_eq!(perforate.confirmed_obs, 1);
        assert_eq!(perforate.estimated_obs, 0);
        assert_eq!(perforate.estimated_min, None);
        remove_db_files(&path);
    }

    #[test]
    fn confirmed_rollup_uses_isolated_rows_only() {
        let path = open_fixture_db(&[
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 20, "2026-08-06T10:00:00Z"),
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 24, "2026-08-06T11:00:00Z"),
            FixtureRow::ambiguous("melee", "boot", "Odefu", 30, "2026-08-06T12:00:00Z", 1, 2),
        ]);
        let conn = Connection::open(&path).unwrap();
        let aggregates = category_aggregates(
            &conn,
            "melee",
            &FilterParams::from_query(None, None),
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        let bitchslap = aggregates
            .iter()
            .find(|row| row.verb == "bitchslap")
            .unwrap();
        assert_eq!(bitchslap.confirmed_obs, 2);
        assert_eq!(bitchslap.confirmed_min, Some(20));
        assert_eq!(bitchslap.confirmed_max, Some(24));
        assert_eq!(bitchslap.confirmed_avg, Some(22.0));
        let boot = aggregates.iter().find(|row| row.verb == "boot").unwrap();
        assert_eq!(boot.confirmed_obs, 0);
        remove_db_files(&path);
    }

    #[test]
    fn rank_weighted_estimated_avg_skews_toward_high_rank_verb() {
        let path = open_fixture_db(&[
            FixtureRow::ambiguous("melee", "pat", "Odefu", 21, "2026-08-06T12:00:00Z", 1, 2)
                .with_rank(1, "unarmed"),
            FixtureRow::ambiguous(
                "melee",
                "savagely triple-kick",
                "Odefu",
                21,
                "2026-08-06T12:00:00Z",
                1,
                2,
            )
            .with_rank(20, "unarmed"),
        ]);
        let conn = Connection::open(&path).unwrap();
        let aggregates = category_aggregates(
            &conn,
            "melee",
            &FilterParams::from_query(None, None),
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        let pat = aggregates.iter().find(|row| row.verb == "pat").unwrap();
        let high = aggregates
            .iter()
            .find(|row| row.verb == "savagely triple-kick")
            .unwrap();
        assert_eq!(pat.estimated_avg, Some(1.0));
        assert_eq!(high.estimated_avg, Some(20.0));
        assert_eq!(pat.confirmed_obs, 0);
        remove_db_files(&path);
    }

    #[test]
    fn estimated_rollup_includes_ambiguous_bounds() {
        let path = open_fixture_db(&[FixtureRow::ambiguous(
            "melee",
            "boot",
            "Odefu",
            30,
            "2026-08-06T12:00:00Z",
            1,
            2,
        )]);
        let conn = Connection::open(&path).unwrap();
        let aggregates = category_aggregates(
            &conn,
            "melee",
            &FilterParams::from_query(None, None),
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        let boot = aggregates.iter().find(|row| row.verb == "boot").unwrap();
        assert_eq!(boot.estimated_obs, 1);
        assert_eq!(boot.estimated_min, Some(0));
        assert_eq!(boot.estimated_max, Some(30));
        assert!(boot.estimated_loose);
        remove_db_files(&path);
    }

    #[test]
    fn extrapolation_assigns_full_delta_when_known_min_matches() {
        let candidates = vec![
            BatchCandidate {
                category: "melee".to_string(),
                family: Some("unarmed".to_string()),
                verb: "bitchslap".to_string(),
            },
            BatchCandidate {
                category: "melee".to_string(),
                family: Some("unarmed".to_string()),
                verb: "boot".to_string(),
            },
        ];
        let mut known_mins = HashMap::new();
        known_mins.insert(agg_key("melee", Some("unarmed"), "bitchslap"), 22);
        let bounds = extrapolate_batch(22, &candidates, &known_mins);
        assert_eq!(
            bounds,
            vec![
                EffectiveBounds {
                    min: 22,
                    max: 22,
                    loose: false
                },
                EffectiveBounds {
                    min: 0,
                    max: 0,
                    loose: false
                },
            ]
        );
    }

    #[test]
    fn extrapolation_keeps_loose_bounds_when_sum_known_min_exceeds_delta() {
        let candidates = vec![
            BatchCandidate {
                category: "melee".to_string(),
                family: Some("unarmed".to_string()),
                verb: "bitchslap".to_string(),
            },
            BatchCandidate {
                category: "melee".to_string(),
                family: Some("unarmed".to_string()),
                verb: "boot".to_string(),
            },
        ];
        let mut known_mins = HashMap::new();
        known_mins.insert(agg_key("melee", Some("unarmed"), "bitchslap"), 20);
        known_mins.insert(agg_key("melee", Some("unarmed"), "boot"), 18);
        let bounds = extrapolate_batch(22, &candidates, &known_mins);
        assert!(
            bounds
                .iter()
                .all(|bound| bound.loose && bound.min == 0 && bound.max == 22)
        );
    }

    #[test]
    fn extrapolation_keeps_loose_bounds_when_unresolved() {
        let candidates = vec![BatchCandidate {
            category: "melee".to_string(),
            family: Some("unarmed".to_string()),
            verb: "boot".to_string(),
        }];
        let bounds = extrapolate_batch(30, &candidates, &HashMap::new());
        assert_eq!(
            bounds,
            vec![EffectiveBounds {
                min: 0,
                max: 30,
                loose: true
            }]
        );
    }

    #[test]
    fn filter_range_24h_excludes_old_rows() {
        let old = (Utc::now() - Duration::hours(30)).to_rfc3339();
        let recent = (Utc::now() - Duration::hours(1)).to_rfc3339();
        let path = open_fixture_db(&[
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 20, &old),
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 24, &recent),
        ]);
        let conn = Connection::open(&path).unwrap();
        let aggregates = category_aggregates(
            &conn,
            "melee",
            &FilterParams::from_query(Some("24h"), None),
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        let bitchslap = aggregates
            .iter()
            .find(|row| row.verb == "bitchslap")
            .unwrap();
        assert_eq!(bitchslap.confirmed_obs, 1);
        assert_eq!(bitchslap.confirmed_min, Some(24));
        remove_db_files(&path);
    }

    #[test]
    fn filter_range_7d_excludes_older_rows() {
        let old = (Utc::now() - Duration::days(8)).to_rfc3339();
        let recent = (Utc::now() - Duration::days(2)).to_rfc3339();
        let path = open_fixture_db(&[
            FixtureRow::isolated("melee", "bash", "Odefu", 10, &old),
            FixtureRow::isolated("skill", "bash", "Odefu", 12, &recent),
        ]);
        let conn = Connection::open(&path).unwrap();
        let melee = category_aggregates(
            &conn,
            "melee",
            &FilterParams::from_query(Some("7d"), None),
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        assert!(melee.is_empty());
        let skill = category_aggregates(
            &conn,
            "skill",
            &FilterParams::from_query(Some("7d"), None),
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        assert_eq!(skill.len(), 1);
        remove_db_files(&path);
    }

    #[test]
    fn filter_player_limits_rollups_and_events() {
        let path = open_fixture_db(&[
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 20, "2026-08-06T10:00:00Z"),
            FixtureRow::isolated("melee", "bitchslap", "Beta", 15, "2026-08-06T11:00:00Z"),
        ]);
        let conn = Connection::open(&path).unwrap();
        let filters = FilterParams::from_query(None, Some("Odefu"));
        let aggregates = category_aggregates(
            &conn,
            "melee",
            &filters,
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        assert_eq!(aggregates[0].confirmed_obs, 1);
        let events = list_events(
            &conn,
            "melee",
            "bitchslap",
            None,
            &filters,
            EventSortColumn::RecordedAt,
            SortDirection::Desc,
        )
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].player, "Odefu");
        remove_db_files(&path);
    }

    #[test]
    fn default_filters_include_all_rows() {
        let path = open_fixture_db(&[
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 20, "2026-08-06T10:00:00Z"),
            FixtureRow::isolated("melee", "bitchslap", "Beta", 15, "2026-08-06T11:00:00Z"),
        ]);
        let conn = Connection::open(&path).unwrap();
        let aggregates = category_aggregates(
            &conn,
            "melee",
            &FilterParams::from_query(None, None),
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        assert_eq!(aggregates[0].confirmed_obs, 2);
        remove_db_files(&path);
    }

    #[test]
    fn landing_sort_by_verb_desc_changes_order() {
        let path = open_fixture_db(&[
            FixtureRow::isolated("melee", "boot", "Odefu", 20, "2026-08-06T10:00:00Z"),
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 24, "2026-08-06T11:00:00Z"),
        ]);
        let conn = Connection::open(&path).unwrap();
        let aggregates = category_aggregates(
            &conn,
            "melee",
            &FilterParams::from_query(None, None),
            LandingSortColumn::Verb,
            SortDirection::Desc,
        )
        .unwrap();
        assert_eq!(aggregates[0].verb, "boot");
        remove_db_files(&path);
    }

    #[test]
    fn drill_down_includes_batch_siblings_for_ambiguous_batches() {
        let path = open_fixture_db(&[
            FixtureRow::ambiguous(
                "melee",
                "bitchslap",
                "Odefu",
                22,
                "2026-08-06T14:35:00Z",
                2,
                2,
            )
            .with_text("Holy man bitchslaps you."),
            FixtureRow::ambiguous("melee", "boot", "Odefu", 22, "2026-08-06T14:35:00Z", 2, 2)
                .with_text("Holy man boots you."),
        ]);
        let conn = Connection::open(&path).unwrap();
        let events = list_events(
            &conn,
            "melee",
            "bitchslap",
            None,
            &FilterParams::from_query(None, None),
            EventSortColumn::RecordedAt,
            SortDirection::Desc,
        )
        .unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].row_role, EventRowRole::Focal);
        assert_eq!(events[0].message_verb, "bitchslap");
        assert_eq!(events[1].row_role, EventRowRole::Sibling);
        assert_eq!(events[1].message_verb, "boot");
        assert_eq!(events[1].message_text, "Holy man boots you.");
        remove_db_files(&path);
    }

    #[test]
    fn drill_down_sort_by_recorded_at_changes_order() {
        let path = open_fixture_db(&[
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 20, "2026-08-06T10:00:00Z"),
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 24, "2026-08-06T11:00:00Z"),
        ]);
        let conn = Connection::open(&path).unwrap();
        let events = list_events(
            &conn,
            "melee",
            "bitchslap",
            None,
            &FilterParams::from_query(None, None),
            EventSortColumn::RecordedAt,
            SortDirection::Asc,
        )
        .unwrap();
        assert_eq!(events[0].hp_delta, 20);
        remove_db_files(&path);
    }

    #[test]
    fn categories_stay_in_their_rollups() {
        let path = open_fixture_db(&[
            FixtureRow::isolated("melee", "bitchslap", "Odefu", 20, "2026-08-06T10:00:00Z"),
            FixtureRow::isolated("skill", "bash", "Odefu", 10, "2026-08-06T10:00:00Z"),
            FixtureRow::isolated("spell", "magic missile", "Odefu", 5, "2026-08-06T10:00:00Z"),
        ]);
        let conn = Connection::open(&path).unwrap();
        let filters = FilterParams::from_query(None, None);
        let melee = category_aggregates(
            &conn,
            "melee",
            &filters,
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        let skill = category_aggregates(
            &conn,
            "skill",
            &filters,
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        let spell = category_aggregates(
            &conn,
            "spell",
            &filters,
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        assert_eq!(melee[0].verb, "bitchslap");
        assert_eq!(skill[0].verb, "bash");
        assert_eq!(spell[0].verb, "magic missile");
        remove_db_files(&path);
    }

    #[test]
    fn savagely_strike_stays_separate_per_weapon_family() {
        let path = open_fixture_db(&[
            FixtureRow::isolated(
                "melee",
                "savagely strike",
                "Odefu",
                20,
                "2026-08-06T10:00:00Z",
            )
            .with_rank(22, "bash"),
            FixtureRow::isolated(
                "melee",
                "savagely strike",
                "Odefu",
                35,
                "2026-08-06T11:00:00Z",
            )
            .with_rank(22, "claw"),
        ]);
        let conn = Connection::open(&path).unwrap();
        let sections = melee_family_aggregates(
            &conn,
            &FilterParams::from_query(None, None),
            LandingSortColumn::Verb,
            SortDirection::Asc,
        )
        .unwrap();
        assert_eq!(sections.len(), 2);
        let bash = sections
            .iter()
            .find(|section| section.family_id == "bash")
            .unwrap();
        let claw = sections
            .iter()
            .find(|section| section.family_id == "claw")
            .unwrap();
        assert_eq!(bash.aggregates[0].confirmed_min, Some(20));
        assert_eq!(claw.aggregates[0].confirmed_min, Some(35));
        remove_db_files(&path);
    }

    #[test]
    fn drill_down_family_filter_limits_focal_rows() {
        let path = open_fixture_db(&[
            FixtureRow::isolated(
                "melee",
                "savagely strike",
                "Odefu",
                20,
                "2026-08-06T10:00:00Z",
            )
            .with_rank(22, "bash"),
            FixtureRow::isolated(
                "melee",
                "savagely strike",
                "Odefu",
                35,
                "2026-08-06T11:00:00Z",
            )
            .with_rank(22, "claw"),
        ]);
        let conn = Connection::open(&path).unwrap();
        let events = list_events(
            &conn,
            "melee",
            "savagely strike",
            Some("bash"),
            &FilterParams::from_query(None, None),
            EventSortColumn::RecordedAt,
            SortDirection::Desc,
        )
        .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].hp_delta, 20);
        remove_db_files(&path);
    }
}
