use crate::combat_damage::aggregate::{
    EventSortColumn, FilterParams, LandingSortColumn, MeleeFamilySection, SortDirection, TimeRange,
    UnattributedHpDetail, UnattributedHpSummary, VerbAggregate, category_aggregates,
    delete_reviewed_unattributed, delete_unattributed, get_unattributed, list_events, list_players,
    list_unattributed, mark_unattributed_reviewed, melee_family_aggregates,
};
use crate::combat_damage::storage::{open_readonly_db, open_validated_db};
use axum::{
    Form, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, Uri, uri::Authority},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;

pub const DEFAULT_PORT: u16 = 6464;
pub const BIND_ADDR: &str = "127.0.0.1";

const STYLE_CSS: &str = include_str!("../../assets/combat_damage/style.css");

fn format_percent(value: f64) -> String {
    format!("{:.0}%", value * 100.0)
}

#[derive(Clone)]
pub struct ViewerState {
    db_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ViewerQuery {
    pub range: Option<String>,
    pub player: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub family: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RemoveReviewedForm {
    range: Option<String>,
    player: Option<String>,
}

pub fn parse_port_from_args(args: impl IntoIterator<Item = String>) -> u16 {
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        if arg == "--port"
            && let Some(value) = iter.next()
            && let Ok(port) = value.parse::<u16>()
        {
            return port;
        }
    }
    DEFAULT_PORT
}

pub fn spawn_server(db_path: PathBuf, port: u16) -> Option<JoinHandle<()>> {
    let addr: SocketAddr = format!("{BIND_ADDR}:{port}").parse().ok()?;
    let state = ViewerState { db_path };
    std::thread::Builder::new()
        .name("damage-viewer".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("viewer runtime");
            runtime.block_on(async move {
                let app = router(Arc::new(state));
                let listener = match tokio::net::TcpListener::bind(addr).await {
                    Ok(listener) => listener,
                    Err(error) => {
                        log::warn!("damage viewer failed to bind {addr}: {error}");
                        return;
                    }
                };
                if let Err(error) = axum::serve(listener, app).await {
                    log::warn!("damage viewer stopped: {error}");
                }
            });
        })
        .ok()
}

pub fn router(state: Arc<ViewerState>) -> Router {
    Router::new()
        .route("/", get(landing))
        .route("/events/{category}/{verb}", get(drill_down))
        .route("/unattributed/{id}", get(unattributed_drill_down))
        .route("/unattributed/{id}/delete", post(delete_unattributed_event))
        .route(
            "/unattributed/remove-reviewed",
            post(remove_reviewed_unattributed),
        )
        .route("/style.css", get(style_css))
        .with_state(state)
}

async fn landing(
    State(state): State<Arc<ViewerState>>,
    Query(query): Query<ViewerQuery>,
) -> impl IntoResponse {
    let filters = FilterParams::from_query(query.range.as_deref(), query.player.as_deref());
    let sort_col = LandingSortColumn::parse(query.sort.as_deref());
    let sort_dir = SortDirection::parse(query.dir.as_deref());
    match open_readonly_db(&state.db_path) {
        Ok(conn) => match landing_data(&conn, &filters, sort_col, sort_dir, &query) {
            Ok(html) => Html(html).into_response(),
            Err(message) => service_unavailable(message),
        },
        Err(message) => service_unavailable(message),
    }
}

fn landing_data(
    conn: &rusqlite::Connection,
    filters: &FilterParams,
    sort_col: LandingSortColumn,
    sort_dir: SortDirection,
    query: &ViewerQuery,
) -> Result<String, String> {
    let players = list_players(conn, &FilterParams::from_query(None, None))?;
    let melee = melee_family_aggregates(conn, filters, sort_col.clone(), sort_dir)?;
    let skill = category_aggregates(conn, "skill", filters, sort_col.clone(), sort_dir)?;
    let spell = category_aggregates(conn, "spell", filters, sort_col, sort_dir)?;
    let unattributed = list_unattributed(conn, filters)?;
    Ok(render_landing(
        filters,
        &players,
        &melee,
        &skill,
        &spell,
        &unattributed,
        query,
    ))
}

async fn remove_reviewed_unattributed(
    State(state): State<Arc<ViewerState>>,
    Form(form): Form<RemoveReviewedForm>,
) -> impl IntoResponse {
    let filters = FilterParams::from_query(form.range.as_deref(), form.player.as_deref());
    match open_validated_db(&state.db_path) {
        Ok(conn) => {
            if let Err(err) = delete_reviewed_unattributed(&conn, &filters) {
                log::warn!("failed to remove reviewed unattributed triggers: {err}");
            }
        }
        Err(message) => {
            log::warn!("failed to open database for remove reviewed: {message}");
        }
    }
    Redirect::to(&build_query_string("/", &filters, None, None))
}

/// Deletes one ID and returns to the filtered landing section; failures return 503.
async fn delete_unattributed_event(
    State(state): State<Arc<ViewerState>>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Form(form): Form<RemoveReviewedForm>,
) -> axum::response::Response {
    if !is_local_submission(&headers) {
        return (
            StatusCode::FORBIDDEN,
            "Cross-origin deletion is not allowed",
        )
            .into_response();
    }
    let filters = FilterParams::from_query(form.range.as_deref(), form.player.as_deref());
    match open_validated_db(&state.db_path).and_then(|conn| delete_unattributed(&conn, id)) {
        Ok(_) => Redirect::to(&format!(
            "{}#unattributed",
            build_query_string("/", &filters, None, None)
        ))
        .into_response(),
        Err(message) => {
            log::warn!("failed to delete unattributed event {id}: {message}");
            service_unavailable(format!("Could not delete unattributed HP loss record {id}"))
        }
    }
}

/// Rejects foreign browser submissions and non-loopback hosts (including rebinding).
/// Headerless non-browser clients are allowed; browsers supply Origin or Fetch Metadata.
fn is_local_submission(headers: &HeaderMap) -> bool {
    if let Some(site) = headers.get("sec-fetch-site")
        && site != "same-origin"
        && site != "none"
    {
        return false;
    }
    let host = headers.get("host").and_then(|value| value.to_str().ok());
    if headers.contains_key("host")
        && !host
            .and_then(|value| value.parse::<Authority>().ok())
            .is_some_and(|authority| {
                authority.host() == BIND_ADDR || authority.host().eq_ignore_ascii_case("localhost")
            })
    {
        return false;
    }
    match headers.get("origin") {
        None => true,
        Some(origin) => origin
            .to_str()
            .ok()
            .and_then(|value| value.parse::<Uri>().ok())
            .is_some_and(|uri| {
                uri.scheme_str() == Some("http")
                    && uri.path() == "/"
                    && uri.query().is_none()
                    && uri.authority().zip(host).is_some_and(|(authority, host)| {
                        authority.as_str().eq_ignore_ascii_case(host)
                    })
            }),
    }
}

async fn unattributed_drill_down(
    State(state): State<Arc<ViewerState>>,
    Path(id): Path<i64>,
    Query(query): Query<ViewerQuery>,
) -> impl IntoResponse {
    let filters = FilterParams::from_query(query.range.as_deref(), query.player.as_deref());
    match open_validated_db(&state.db_path) {
        Ok(conn) => match unattributed_drill_down_data(&conn, id, &filters, &query) {
            Ok(html) => Html(html).into_response(),
            Err(message) => service_unavailable(message),
        },
        Err(message) => service_unavailable(message),
    }
}

fn unattributed_drill_down_data(
    conn: &rusqlite::Connection,
    id: i64,
    filters: &FilterParams,
    query: &ViewerQuery,
) -> Result<String, String> {
    let detail =
        get_unattributed(conn, id)?.ok_or_else(|| format!("unattributed event {id} not found"))?;
    if let Err(err) = mark_unattributed_reviewed(conn, id) {
        log::warn!("failed to mark unattributed event {id} reviewed: {err}");
    }
    let filtered = list_unattributed(conn, filters)?;
    let (prev_id, next_id) = unattributed_neighbor_ids(&filtered, id);
    Ok(render_unattributed_drill_down(
        &detail, filters, &filtered, query, prev_id, next_id,
    ))
}

fn unattributed_neighbor_ids(
    rows: &[UnattributedHpSummary],
    current_id: i64,
) -> (Option<i64>, Option<i64>) {
    let Some(index) = rows.iter().position(|row| row.id == current_id) else {
        return (None, None);
    };
    let prev = index.checked_sub(1).map(|idx| rows[idx].id);
    let next = rows.get(index + 1).map(|row| row.id);
    (prev, next)
}

async fn drill_down(
    State(state): State<Arc<ViewerState>>,
    Path((category, verb)): Path<(String, String)>,
    Query(query): Query<ViewerQuery>,
) -> impl IntoResponse {
    let filters = FilterParams::from_query(query.range.as_deref(), query.player.as_deref());
    let sort_col = EventSortColumn::parse(query.sort.as_deref());
    let sort_dir = if query.sort.is_some() {
        SortDirection::parse(query.dir.as_deref())
    } else {
        SortDirection::Desc
    };
    match open_readonly_db(&state.db_path) {
        Ok(conn) => match drill_down_data(
            &conn, &category, &verb, &filters, sort_col, sort_dir, &query,
        ) {
            Ok(html) => Html(html).into_response(),
            Err(message) => service_unavailable(message),
        },
        Err(message) => service_unavailable(message),
    }
}

fn drill_down_data(
    conn: &rusqlite::Connection,
    category: &str,
    verb: &str,
    filters: &FilterParams,
    sort_col: EventSortColumn,
    sort_dir: SortDirection,
    query: &ViewerQuery,
) -> Result<String, String> {
    let family = query.family.as_deref().filter(|value| !value.is_empty());
    let events = list_events(conn, category, verb, family, filters, sort_col, sort_dir)?;
    Ok(render_drill_down(
        category, verb, family, filters, &events, query,
    ))
}

async fn style_css() -> impl IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "text/css")], STYLE_CSS)
}

fn service_unavailable(message: String) -> axum::response::Response {
    (StatusCode::SERVICE_UNAVAILABLE, message).into_response()
}

fn render_landing(
    filters: &FilterParams,
    players: &[String],
    melee: &[MeleeFamilySection],
    skill: &[VerbAggregate],
    spell: &[VerbAggregate],
    unattributed: &[UnattributedHpSummary],
    query: &ViewerQuery,
) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str("<title>Combat damage</title><link rel=\"stylesheet\" href=\"/style.css\">");
    html.push_str("</head><body><h1>Combat damage</h1>");
    html.push_str(&render_filters(filters, players, "/"));
    html.push_str(&render_melee_section(melee, filters, query));
    html.push_str(&render_category_table(
        "Skill",
        "skill",
        "No skill damage recorded yet.",
        skill,
        filters,
        query,
        None,
    ));
    html.push_str(&render_category_table(
        "Spell",
        "spell",
        "No spell damage recorded yet.",
        spell,
        filters,
        query,
        None,
    ));
    html.push_str(&render_unattributed_section(unattributed, filters));
    html.push_str(render_sort_script());
    html.push_str("</body></html>");
    html
}

fn render_unattributed_section(rows: &[UnattributedHpSummary], filters: &FilterParams) -> String {
    let unreviewed_count = rows.iter().filter(|row| !row.reviewed).count();
    let reviewed_count = rows.iter().filter(|row| row.reviewed).count();
    let heading = if unreviewed_count > 0 {
        format!("Unattributed HP loss ({unreviewed_count} unreviewed)")
    } else {
        String::from("Unattributed HP loss")
    };
    let mut html = String::from("<div id=\"unattributed\" class=\"unattributed-header\"><h2>");
    html.push_str(&heading);
    html.push_str("</h2>");
    if reviewed_count > 0 {
        html.push_str(
            "<form method=\"post\" action=\"/unattributed/remove-reviewed\" class=\"remove-reviewed-form\">",
        );
        if filters.range != TimeRange::All {
            html.push_str(&format!(
                "<input type=\"hidden\" name=\"range\" value=\"{}\">",
                filters.range.as_str()
            ));
        }
        if let Some(player) = &filters.player {
            html.push_str(&format!(
                "<input type=\"hidden\" name=\"player\" value=\"{}\">",
                html_escape(player)
            ));
        }
        html.push_str("<button type=\"submit\">Remove reviewed</button></form>");
    }
    html.push_str("</div>");
    if rows.is_empty() {
        html.push_str(
            "<p class=\"empty-hint\">No unattributed HP loss recorded for the current filters.</p>",
        );
        return html;
    }
    html.push_str("<p class=\"empty-hint\">Delete permanently removes a record and its saved context. There is no undo.</p>");
    html.push_str("<table><thead><tr>");
    html.push_str(
        "<th>Recorded at</th><th>Player</th><th>HP delta</th><th>Lines</th><th>Reviewed</th><th>Actions</th>",
    );
    html.push_str("</tr></thead><tbody>");
    for row in rows {
        let href = format!("/unattributed/{}{}", row.id, build_filter_query(filters));
        let row_class = if row.reviewed {
            " class=\"reviewed\""
        } else {
            ""
        };
        html.push_str(&format!("<tr{row_class}>"));
        html.push_str(&format!(
            "<td><a href=\"{href}\">{}</a></td>",
            html_escape(&row.recorded_at)
        ));
        html.push_str(&format!("<td>{}</td>", html_escape(&row.player)));
        html.push_str(&format!("<td class=\"numeric\">{}</td>", row.hp_delta));
        html.push_str(&format!("<td class=\"numeric\">{}</td>", row.line_count));
        let reviewed_cell = if row.reviewed { "yes" } else { "" };
        html.push_str(&format!("<td>{reviewed_cell}</td><td>"));
        html.push_str(&render_delete_unattributed_form(row.id, filters));
        html.push_str("</td></tr>");
    }
    html.push_str("</tbody></table>");
    html
}

/// Renders the shared single-record POST action, preserving only return filters.
fn render_delete_unattributed_form(id: i64, filters: &FilterParams) -> String {
    let mut html = format!(
        "<form method=\"post\" action=\"/unattributed/{id}/delete\" class=\"delete-unattributed-form\">"
    );
    html.push_str(&format!(
        "<input type=\"hidden\" name=\"range\" value=\"{}\">",
        filters.range.as_str()
    ));
    if let Some(player) = &filters.player {
        html.push_str(&format!(
            "<input type=\"hidden\" name=\"player\" value=\"{}\">",
            html_escape(player)
        ));
    }
    html.push_str(&format!(
        "<button type=\"submit\" aria-label=\"Delete unattributed HP loss record {id}\">Delete</button></form>"
    ));
    html
}

fn render_unattributed_drill_down(
    detail: &UnattributedHpDetail,
    filters: &FilterParams,
    filtered: &[UnattributedHpSummary],
    query: &ViewerQuery,
    prev_id: Option<i64>,
    next_id: Option<i64>,
) -> String {
    let back_href = build_query_string("/", filters, query.sort.as_deref(), query.dir.as_deref());
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str(
        "<title>Unattributed HP loss</title><link rel=\"stylesheet\" href=\"/style.css\">",
    );
    html.push_str("</head><body>");
    html.push_str(&render_unattributed_nav(
        back_href.as_str(),
        filters,
        prev_id,
        next_id,
    ));
    html.push_str("<div class=\"unattributed-detail-header\"><h1>Unattributed HP loss</h1>");
    html.push_str(&render_delete_unattributed_form(detail.id, filters));
    html.push_str("</div><p class=\"empty-hint\">Delete permanently removes this record and its saved context. There is no undo.</p>");
    if !filtered.iter().any(|row| row.id == detail.id) {
        html.push_str("<p class=\"empty-hint\">This record is outside the active filters. Previous/Next navigation is unavailable; its saved context is shown in full.</p>");
    }
    html.push_str("<table><tbody>");
    html.push_str(&format!(
        "<tr><th>Recorded at</th><td>{}</td></tr>",
        html_escape(&detail.recorded_at)
    ));
    html.push_str(&format!(
        "<tr><th>Player</th><td>{}</td></tr>",
        html_escape(&detail.player)
    ));
    html.push_str(&format!(
        "<tr><th>HP delta</th><td class=\"numeric\">{}</td></tr>",
        detail.hp_delta
    ));
    html.push_str(&format!(
        "<tr><th>HP before</th><td class=\"numeric\">{}</td></tr>",
        detail.hp_before
    ));
    html.push_str(&format!(
        "<tr><th>HP after</th><td class=\"numeric\">{}</td></tr>",
        detail.hp_after
    ));
    html.push_str(&format!(
        "<tr><th>H line</th><td>{}</td></tr>",
        html_escape(&detail.h_line_text)
    ));
    html.push_str("</tbody></table>");
    html.push_str("<h2>Context lines</h2>");
    if detail.context_lines.is_empty() {
        html.push_str("<p class=\"empty-hint\">No context lines before this H line.</p>");
    }
    html.push_str("<ol class=\"context-lines\">");
    for line in detail
        .context_lines
        .iter()
        .chain(std::iter::once(&detail.h_line_text))
    {
        html.push_str(&format!("<li>{}</li>", html_escape(line)));
    }
    html.push_str("</ol>");
    html.push_str("</body></html>");
    html
}

fn render_unattributed_nav(
    back_href: &str,
    filters: &FilterParams,
    prev_id: Option<i64>,
    next_id: Option<i64>,
) -> String {
    let mut parts = vec![format!("<a href=\"{back_href}\">&larr; Back</a>")];
    if let Some(id) = prev_id {
        let href = format!("/unattributed/{id}{}", build_filter_query(filters));
        parts.push(format!("<a href=\"{href}\">&larr; Previous</a>"));
    }
    if let Some(id) = next_id {
        let href = format!("/unattributed/{id}{}", build_filter_query(filters));
        parts.push(format!("<a href=\"{href}\">Next &rarr;</a>"));
    }
    format!(
        "<div class=\"drill-navigation\"><p class=\"back-link\">{}</p>{}</div>",
        parts.join(" · "),
        render_filter_summary(filters, back_href)
    )
}

fn render_melee_section(
    sections: &[MeleeFamilySection],
    filters: &FilterParams,
    query: &ViewerQuery,
) -> String {
    let mut html = String::from("<h2>Melee</h2>");
    if sections.is_empty() {
        html.push_str("<p class=\"empty-hint\">No melee damage recorded yet.</p>");
        return html;
    }
    for section in sections {
        html.push_str(&format!(
            "<h3>{} ({})</h3>",
            html_escape(&section.family_title),
            html_escape(&section.family_id)
        ));
        html.push_str(&render_category_table(
            "",
            "melee",
            "",
            &section.aggregates,
            filters,
            query,
            Some(section.family_id.as_str()),
        ));
    }
    html
}

fn render_drill_down(
    category: &str,
    verb: &str,
    family: Option<&str>,
    filters: &FilterParams,
    events: &[crate::combat_damage::aggregate::DamageEvent],
    query: &ViewerQuery,
) -> String {
    let back_href = build_query_string("/", filters, query.sort.as_deref(), query.dir.as_deref());
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str(
        "<title>Combat damage events</title><link rel=\"stylesheet\" href=\"/style.css\">",
    );
    html.push_str("</head><body>");
    html.push_str(&format!(
        "<div class=\"drill-navigation\"><p class=\"back-link\"><a href=\"{back_href}\">&larr; Back</a></p>{}</div>",
        render_filter_summary(filters, &back_href)
    ));
    let heading = match family {
        Some(family) => format!(
            "{} / {} / {}",
            html_escape(category),
            html_escape(family),
            html_escape(verb)
        ),
        None => format!("{} / {}", html_escape(category), html_escape(verb)),
    };
    html.push_str(&format!("<h1>{heading}</h1>"));
    html.push_str("<table><thead><tr>");
    for (col, label) in [
        ("recorded_at", "Recorded at"),
        ("player", "Player"),
        ("hp_delta", "HP delta"),
        ("source_name", "Source"),
        ("confidence", "Confidence"),
        ("weight", "Weight"),
        ("candidate_count", "Candidates"),
    ] {
        let href = build_event_sort_href(category, verb, filters, col, query);
        html.push_str(&format!("<th><a href=\"{href}\">{label}</a></th>"));
    }
    html.push_str("<th>Verb</th>");
    let href = build_event_sort_href(category, verb, filters, "message_text", query);
    html.push_str(&format!("<th><a href=\"{href}\">Message</a></th>"));
    html.push_str("</tr></thead><tbody>");
    if events.is_empty() {
        html.push_str("</tbody></table><p class=\"empty-hint\">No events for this verb.</p>");
    } else {
        let mut batch_counts = HashMap::new();
        for event in events {
            *batch_counts.entry(event.batch_id).or_insert(0usize) += 1;
        }
        for event in events {
            let mut row_class = String::new();
            if event.candidate_count > 1 {
                row_class.push_str(" batch-group");
            }
            if event.row_role == crate::combat_damage::aggregate::EventRowRole::Sibling {
                row_class.push_str(" batch-sibling");
            }
            let batch_class = if row_class.is_empty() {
                String::new()
            } else {
                format!(" class=\"{}\"", row_class.trim())
            };
            let damage = if event.candidate_count > 1 {
                format!("{}–{}", event.damage_min, event.damage_max)
            } else {
                event.hp_delta.to_string()
            };
            let loose = event.candidate_count > 1;
            html.push_str(&format!("<tr{batch_class}>"));
            html.push_str(&format!("<td>{}</td>", html_escape(&event.recorded_at)));
            html.push_str(&format!("<td>{}</td>", html_escape(&event.player)));
            html.push_str(&format!(
                "<td class=\"numeric{}\">{damage}</td>",
                if loose { " loose" } else { "" }
            ));
            html.push_str(&format!("<td>{}</td>", html_escape(&event.source_name)));
            html.push_str(&format!(
                "<td class=\"numeric\">{}</td>",
                format_percent(event.confidence)
            ));
            html.push_str(&format!(
                "<td class=\"numeric\">{}</td>",
                format_percent(event.weight)
            ));
            html.push_str(&format!(
                "<td class=\"numeric\">{}</td>",
                event.candidate_count
            ));
            let verb_href = format!(
                "/events/{}/{}",
                percent_encode_path(&event.damage_category),
                percent_encode_path(&event.message_verb)
            );
            let verb_suffix =
                if event.row_role == crate::combat_damage::aggregate::EventRowRole::Sibling {
                    " <span class=\"batch-label\">sibling</span>"
                } else {
                    ""
                };
            html.push_str(&format!(
                "<td class=\"verb-cell\"><a href=\"{verb_href}{}\">{}{verb_suffix}</a></td>",
                build_filter_query(filters),
                html_escape(&event.message_verb)
            ));
            let batch_note = if batch_counts.get(&event.batch_id).copied().unwrap_or(0) > 1 {
                format!(
                    " <span class=\"batch-label\">(batch {})</span>",
                    event.batch_id
                )
            } else {
                String::new()
            };
            html.push_str(&format!(
                "<td>{}{}</td>",
                html_escape(&event.message_text),
                batch_note
            ));
            html.push_str("</tr>");
        }
        html.push_str("</tbody></table>");
    }
    html.push_str(render_sort_script());
    html.push_str("</body></html>");
    html
}

/// Shows inherited time/player scope and links to the main page for changes.
fn render_filter_summary(filters: &FilterParams, main_href: &str) -> String {
    let range = match filters.range {
        TimeRange::All => "All time",
        TimeRange::Hours24 => "Last 24h",
        TimeRange::Days7 => "Last 7d",
    };
    let player = match &filters.player {
        Some(player) => format!("Player: {}", html_escape(player)),
        None => "All players".to_string(),
    };
    format!(
        "<p class=\"filter-summary\">{range} · {player} · <a href=\"{}\">Change filters on main page</a></p>",
        html_escape(main_href)
    )
}

fn render_filters(filters: &FilterParams, players: &[String], action: &str) -> String {
    let mut html = String::from("<form class=\"filters\" method=\"get\" action=\"");
    html.push_str(action);
    html.push_str("\">");
    html.push_str("<label>Time range<select name=\"range\">");
    for (value, label) in [("all", "All time"), ("24h", "Last 24h"), ("7d", "Last 7d")] {
        let selected = if filters.range.as_str() == value {
            " selected"
        } else {
            ""
        };
        html.push_str(&format!(
            "<option value=\"{value}\"{selected}>{label}</option>"
        ));
    }
    html.push_str("</select></label><label>Player<select name=\"player\">");
    let all_selected = if filters.player.is_none() {
        " selected"
    } else {
        ""
    };
    html.push_str(&format!(
        "<option value=\"\"{all_selected}>All players</option>"
    ));
    for player in players {
        let selected = if filters.player.as_deref() == Some(player.as_str()) {
            " selected"
        } else {
            ""
        };
        html.push_str(&format!(
            "<option value=\"{}\"{selected}>{}</option>",
            html_escape(player),
            html_escape(player)
        ));
    }
    html.push_str("</select></label><button type=\"submit\">Apply</button></form>");
    html
}

fn render_category_table(
    title: &str,
    category: &str,
    empty_hint: &str,
    rows: &[VerbAggregate],
    filters: &FilterParams,
    query: &ViewerQuery,
    family: Option<&str>,
) -> String {
    let mut html = if title.is_empty() {
        String::new()
    } else {
        format!("<h2>{title}</h2>")
    };
    html.push_str("<table><thead><tr><th rowspan=\"2\">");
    let verb_href = build_landing_sort_href(filters, "verb", query);
    html.push_str(&format!("<a href=\"{verb_href}\">Verb</a></th>"));
    html.push_str("<th colspan=\"4\" class=\"confirmed\">Confirmed</th>");
    html.push_str("<th colspan=\"4\" class=\"estimated\">Estimated</th></tr><tr>");
    for (col, label) in [
        ("conf_obs", "Obs"),
        ("conf_min", "Min"),
        ("conf_max", "Max"),
        ("conf_avg", "Avg"),
        ("est_obs", "Obs"),
        ("est_min", "Min"),
        ("est_max", "Max"),
        ("est_avg", "Avg"),
    ] {
        let href = build_landing_sort_href(filters, col, query);
        let class = if col.starts_with("conf_") {
            "confirmed"
        } else {
            "estimated"
        };
        html.push_str(&format!(
            "<th class=\"{class}\"><a href=\"{href}\">{label}</a></th>"
        ));
    }
    html.push_str("</tr></thead><tbody>");
    if rows.is_empty() {
        if empty_hint.is_empty() {
            html.push_str("</tbody></table>");
        } else {
            html.push_str(&format!(
                "</tbody></table><p class=\"empty-hint\">{empty_hint}</p>"
            ));
        }
        return html;
    }
    for row in rows {
        let mut href = format!("/events/{category}/{}", percent_encode_path(&row.verb));
        let mut query_parts = build_filter_parts(filters);
        if let Some(family) = family.or(row.weapon_family.as_deref()) {
            query_parts.push(format!("family={}", percent_encode_path(family)));
        }
        if !query_parts.is_empty() {
            href.push('?');
            href.push_str(&query_parts.join("&"));
        }
        html.push_str("<tr>");
        html.push_str(&format!(
            "<td><a href=\"{href}\">{}</a></td>",
            html_escape(&row.verb)
        ));
        html.push_str(&format_numeric(row.confirmed_obs));
        html.push_str(&format_optional_i32(row.confirmed_min, false));
        html.push_str(&format_optional_i32(row.confirmed_max, false));
        html.push_str(&format_optional_f64(row.confirmed_avg, false));
        html.push_str(&format_numeric(row.estimated_obs));
        html.push_str(&format_optional_i32(row.estimated_min, row.estimated_loose));
        html.push_str(&format_optional_i32(row.estimated_max, row.estimated_loose));
        html.push_str(&format_optional_f64(row.estimated_avg, row.estimated_loose));
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table>");
    html
}

fn format_numeric(value: i64) -> String {
    format!("<td class=\"numeric\">{value}</td>")
}

fn format_optional_i32(value: Option<i32>, loose: bool) -> String {
    let class = if loose { "numeric loose" } else { "numeric" };
    match value {
        Some(number) => format!("<td class=\"{class}\">{number}</td>"),
        None => format!("<td class=\"{class}\">—</td>"),
    }
}

fn format_optional_f64(value: Option<f64>, loose: bool) -> String {
    let class = if loose { "numeric loose" } else { "numeric" };
    match value {
        Some(number) => format!("<td class=\"{class}\">{number:.1}</td>"),
        None => format!("<td class=\"{class}\">—</td>"),
    }
}

fn build_filter_parts(filters: &FilterParams) -> Vec<String> {
    let mut parts = Vec::new();
    if filters.range != TimeRange::All {
        parts.push(format!("range={}", filters.range.as_str()));
    }
    if let Some(player) = &filters.player {
        parts.push(format!("player={}", percent_encode_path(player)));
    }
    parts
}

fn build_filter_query(filters: &FilterParams) -> String {
    let parts = build_filter_parts(filters);
    if parts.is_empty() {
        String::new()
    } else {
        format!("?{}", parts.join("&"))
    }
}

fn build_query_string(
    path: &str,
    filters: &FilterParams,
    sort: Option<&str>,
    dir: Option<&str>,
) -> String {
    let mut parts = build_filter_parts(filters);
    if let Some(sort) = sort {
        parts.push(format!("sort={sort}"));
    }
    if let Some(dir) = dir {
        parts.push(format!("dir={dir}"));
    }
    if parts.is_empty() {
        path.to_string()
    } else {
        format!("{path}?{}", parts.join("&"))
    }
}

fn build_landing_sort_href(filters: &FilterParams, column: &str, query: &ViewerQuery) -> String {
    let current = query.sort.as_deref().unwrap_or("verb");
    let next_dir = if current == column {
        match query.dir.as_deref() {
            Some("desc") => "asc",
            _ => "desc",
        }
    } else if column == "verb" {
        "asc"
    } else {
        "desc"
    };
    let mut parts = build_filter_parts(filters);
    parts.push(format!("sort={column}"));
    parts.push(format!("dir={next_dir}"));
    format!("/?{}", parts.join("&"))
}

fn build_event_sort_href(
    category: &str,
    verb: &str,
    filters: &FilterParams,
    column: &str,
    query: &ViewerQuery,
) -> String {
    let current = query.sort.as_deref().unwrap_or("recorded_at");
    let next_dir = if current == column {
        match query.dir.as_deref() {
            Some("asc") => "desc",
            _ => "asc",
        }
    } else if column == "recorded_at" {
        "desc"
    } else {
        "asc"
    };
    let mut parts = build_filter_parts(filters);
    if let Some(family) = query.family.as_deref().filter(|value| !value.is_empty()) {
        parts.push(format!("family={}", percent_encode_path(family)));
    }
    parts.push(format!("sort={column}"));
    parts.push(format!("dir={next_dir}"));
    format!(
        "/events/{category}/{}?{}",
        percent_encode_path(verb),
        parts.join("&")
    )
}

fn percent_encode_path(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            b' ' => "%20".to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_sort_script() -> &'static str {
    "<script>document.querySelectorAll('th a').forEach((link)=>link.addEventListener('click',(event)=>{event.preventDefault();window.location=link.href;}));</script>"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_damage::storage::{CANNOT_OPEN_DATABASE, CURRENT_SCHEMA_VERSION, open_db};
    use crate::combat_damage::test_fixtures::{
        FixtureRow, open_fixture_db, remove_db_files, standard_fixture_rows,
    };
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use rusqlite::Connection;
    use tower::ServiceExt;

    fn app_with_rows(rows: &[FixtureRow]) -> (Router, PathBuf) {
        let path = open_fixture_db(rows);
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        (router(state), path)
    }

    async fn body_string(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn single_delete_forms_cover_reviewed_and_unreviewed_rows_and_detail() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[
                UnattributedFixtureRow::new("A<&\"", 10, "2020-01-01T00:00:00Z", "H:90 [-10]", &[]),
                UnattributedFixtureRow::new(
                    "A<&\"",
                    20,
                    "2020-01-01T00:00:00Z",
                    "H:80 [-20]",
                    &["context"],
                ),
            ],
        );
        mark_unattributed_reviewed_fixture(&path, &[1]);
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        let landing = body_string(
            app.clone()
                .oneshot(
                    Request::builder()
                        .uri("/?player=A%3C%26%22")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap(),
        )
        .await;
        assert!(landing.contains("<th>Actions</th>"));
        assert!(landing.contains("id=\"unattributed\""));
        assert!(landing.contains("Unattributed HP loss (1 unreviewed)"));
        assert!(landing.contains("Remove reviewed"));
        assert!(landing.contains("There is no undo."));
        for id in [1, 2] {
            assert!(landing.contains(&format!(
                "method=\"post\" action=\"/unattributed/{id}/delete\""
            )));
            assert!(landing.contains(&format!(
                "aria-label=\"Delete unattributed HP loss record {id}\""
            )));
        }
        assert!(landing.contains("name=\"player\" value=\"A&lt;&amp;&quot;\""));
        let detail = body_string(
            app.oneshot(
                Request::builder()
                    .uri("/unattributed/2?range=7d&player=Other")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
        )
        .await;
        assert!(detail.contains("method=\"post\" action=\"/unattributed/2/delete\""));
        assert!(detail.contains("name=\"range\" value=\"7d\""));
        assert!(detail.contains("name=\"player\" value=\"Other\""));
        assert!(detail.contains("There is no undo."));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn single_delete_is_id_scoped_idempotent_and_preserves_damage_and_review_state() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[FixtureRow::isolated(
                "skill",
                "bash",
                "A",
                30,
                "2020-01-01T00:00:00Z",
            )],
            &[
                UnattributedFixtureRow::new(
                    "A",
                    10,
                    "2020-01-01T00:00:00Z",
                    "H:90 [-10]",
                    &["one"],
                ),
                UnattributedFixtureRow::new(
                    "B",
                    20,
                    "2020-01-01T00:00:00Z",
                    "H:80 [-20]",
                    &["two"],
                ),
                UnattributedFixtureRow::new("A", 30, "2020-01-01T00:00:00Z", "H:70 [-30]", &[]),
            ],
        );
        mark_unattributed_reviewed_fixture(&path, &[2]);
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        // IDs 1 and 2 are outside these filters; the filters are return context only.
        for id in [1, 1, 2, 999] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/unattributed/{id}/delete"))
                        .header("content-type", "application/x-www-form-urlencoded")
                        .header("host", "localhost:8080")
                        .header("origin", "http://localhost:8080")
                        .header("sec-fetch-site", "same-origin")
                        .body(Body::from("range=24h&player=Other%20%26%20Player"))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::SEE_OTHER);
            assert_eq!(
                response.headers()["location"],
                "/?range=24h&player=Other%20%26%20Player#unattributed"
            );
        }
        let conn = open_readonly_db(&path).unwrap();
        let remaining = list_unattributed(&conn, &FilterParams::from_query(None, None)).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, 3);
        assert!(
            !remaining[0].reviewed,
            "redirect must not auto-review a neighbor"
        );
        assert!(get_unattributed(&conn, 1).unwrap().is_none());
        assert!(get_unattributed(&conn, 2).unwrap().is_none());
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM damage_events", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        drop(conn);
        let landing = body_string(
            app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
                .await
                .unwrap(),
        )
        .await;
        assert!(landing.contains("Unattributed HP loss (1 unreviewed)"));
        assert!(!landing.contains("Remove reviewed"));
        assert!(!landing.contains("action=\"/unattributed/1/delete\""));
        assert!(landing.contains("action=\"/unattributed/3/delete\""));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn single_delete_refreshes_filtered_previous_and_next_links() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[
                UnattributedFixtureRow::new("A", 10, "2020-01-01T00:00:00Z", "H:90 [-10]", &[]),
                UnattributedFixtureRow::new("A", 20, "2020-01-01T00:00:00Z", "H:80 [-20]", &[]),
                UnattributedFixtureRow::new("A", 30, "2020-01-01T00:00:00Z", "H:70 [-30]", &[]),
                UnattributedFixtureRow::new("B", 40, "2020-01-01T00:00:00Z", "H:60 [-40]", &[]),
            ],
        );
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/unattributed/2/delete")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("player=A"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        for (id, expected_link, absent_label) in [
            (
                3,
                "<a href=\"/unattributed/1?player=A\">Next &rarr;</a>",
                "Previous",
            ),
            (
                1,
                "<a href=\"/unattributed/3?player=A\">&larr; Previous</a>",
                "Next",
            ),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/unattributed/{id}?player=A"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let detail = body_string(response).await;
            assert!(detail.contains(expected_link));
            assert!(!detail.contains(absent_label));
            assert!(!detail.contains("href=\"/unattributed/2"));
            assert!(!detail.contains("href=\"/unattributed/4"));
        }
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn single_delete_rejects_foreign_submissions_get_and_malformed_ids() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[UnattributedFixtureRow::new(
                "A",
                10,
                "2020-01-01T00:00:00Z",
                "H:90 [-10]",
                &[],
            )],
        );
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        for (host, origin, site) in [
            ("127.0.0.1:6464", Some("https://evil.example"), None),
            ("127.0.0.1:6464", Some("http://127.0.0.1:9999"), None),
            ("127.0.0.1:6464", Some("null"), None),
            ("127.0.0.1:6464", Some("http://127.0.0.1:6464/path"), None),
            ("127.0.0.1:6464", None, Some("cross-site")),
            ("127.0.0.1:6464", None, Some("same-site")),
            (
                "evil.example:6464",
                Some("http://evil.example:6464"),
                Some("same-origin"),
            ),
            ("evil.example:6464", None, None),
        ] {
            let mut request = Request::builder()
                .method("POST")
                .uri("/unattributed/1/delete")
                .header("content-type", "application/x-www-form-urlencoded")
                .header("host", host);
            if let Some(origin) = origin {
                request = request.header("origin", origin);
            }
            if let Some(site) = site {
                request = request.header("sec-fetch-site", site);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::FORBIDDEN,
                "{host} {origin:?} {site:?}"
            );
        }
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/unattributed/1/delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/unattributed/not-an-id/delete")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let conn = open_readonly_db(&path).unwrap();
        assert!(get_unattributed(&conn, 1).unwrap().is_some());
        drop(conn);
        // Non-browser clients need no Origin/Fetch Metadata; browser forms need no JS.
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/unattributed/1/delete")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.headers()["location"], "/#unattributed");
        let conn = open_readonly_db(&path).unwrap();
        assert!(get_unattributed(&conn, 1).unwrap().is_none());
        drop(conn);
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn single_delete_database_failures_return_503_without_redirect() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed, temp_db_path,
        };
        let missing = temp_db_path("single-delete-missing");
        let path = open_fixture_db_with_unattributed(
            &[],
            &[UnattributedFixtureRow::new(
                "A",
                10,
                "2020-01-01T00:00:00Z",
                "H:90 [-10]",
                &[],
            )],
        );
        let conn = open_validated_db(&path).unwrap();
        conn.execute_batch("CREATE TRIGGER reject_delete BEFORE DELETE ON unattributed_hp_events BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
        drop(conn);
        for db_path in [&missing, &path] {
            let app = router(Arc::new(ViewerState {
                db_path: db_path.clone(),
            }));
            let response = app
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/unattributed/1/delete")
                        .header("content-type", "application/x-www-form-urlencoded")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
            assert!(response.headers().get("location").is_none());
            assert!(
                body_string(response)
                    .await
                    .contains("Could not delete unattributed HP loss record 1")
            );
        }
        let conn = open_readonly_db(&path).unwrap();
        assert!(get_unattributed(&conn, 1).unwrap().is_some());
        drop(conn);
        remove_db_files(&path);
        remove_db_files(&missing);
    }

    #[tokio::test]
    async fn missing_db_returns_503() {
        let path = crate::combat_damage::test_fixtures::temp_db_path("missing-db");
        remove_db_files(&path);
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        let app = router(state);
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        remove_db_files(&path);
    }

    #[test]
    fn default_port_is_6464() {
        assert_eq!(parse_port_from_args(["batrs".to_string()]), 6464);
    }

    #[test]
    fn port_flag_overrides_default() {
        assert_eq!(
            parse_port_from_args([
                "batrs".to_string(),
                "--port".to_string(),
                "8080".to_string()
            ]),
            8080
        );
    }

    #[test]
    fn bind_addr_is_localhost() {
        assert_eq!(BIND_ADDR, "127.0.0.1");
    }

    #[tokio::test]
    async fn empty_db_landing_returns_200_with_sections() {
        let path = crate::combat_damage::test_fixtures::temp_db_path("empty-landing");
        remove_db_files(&path);
        open_db(&path).unwrap();
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        let app = router(state);
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("Melee"));
        assert!(body.contains("Skill"));
        assert!(body.contains("Spell"));
        assert!(body.contains("Confirmed"));
        assert!(body.contains("Estimated"));
        assert!(body.contains("No melee damage recorded yet."));
        assert!(body.contains("Unattributed HP loss"));
        assert!(body.contains("No unattributed HP loss recorded for the current filters."));
        assert!(!body.contains("bitchslap"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn empty_db_player_dropdown_only_all_players() {
        let path = crate::combat_damage::test_fixtures::temp_db_path("empty-players");
        remove_db_files(&path);
        open_db(&path).unwrap();
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        let app = router(state);
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = body_string(response).await;
        assert!(body.contains("All players</option>"));
        assert!(!body.contains("Odefu</option>"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn empty_db_drill_down_returns_200() {
        let path = crate::combat_damage::test_fixtures::temp_db_path("empty-drill");
        remove_db_files(&path);
        open_db(&path).unwrap();
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        let app = router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/events/melee/bitchslaps")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("No events for this verb."));
        assert!(body.contains("href=\"/"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn empty_db_style_css_returns_css() {
        let path = crate::combat_damage::test_fixtures::temp_db_path("empty-css");
        remove_db_files(&path);
        open_db(&path).unwrap();
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        let app = router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/style.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("table"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn newer_schema_returns_503() {
        let path = crate::combat_damage::test_fixtures::temp_db_path("newer-schema");
        remove_db_files(&path);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_version (version INTEGER NOT NULL);
             INSERT INTO schema_version (version) VALUES (99);",
        )
        .unwrap();
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        let app = router(state);
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn fixture_landing_contains_verbs_and_headings() {
        let (app, path) = app_with_rows(&standard_fixture_rows());
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("Melee"));
        assert!(body.contains("Untrained unarmed attacks (unarmed)"));
        assert!(body.contains("bitchslap"));
        assert!(body.contains("family=unarmed"));
        assert!(body.contains("Confirmed"));
        assert!(body.contains("Estimated"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn drill_down_family_param_filters_events() {
        let rows = vec![
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
        ];
        let (app, path) = app_with_rows(&rows);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/events/melee/savagely%20strike?family=bash")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("melee / bash / savagely strike"));
        assert!(body.contains("family=bash&sort=hp_delta&dir=asc"));
        assert!(!body.contains("<select"));
        assert!(body.contains(">20<"));
        assert!(!body.contains(">35<"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn landing_preserves_filter_params_in_form() {
        let (app, path) = app_with_rows(&standard_fixture_rows());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?player=Odefu&range=7d")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = body_string(response).await;
        assert!(body.contains("value=\"7d\" selected"));
        assert!(body.contains("Odefu"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn drill_downs_inherit_main_filters_without_editable_controls() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let now = chrono::Utc::now().to_rfc3339();
        let player = "A<&\"";
        let path = open_fixture_db_with_unattributed(
            &[
                FixtureRow::isolated("melee", "bitchslap", player, 31, &now)
                    .with_rank(1, "unarmed"),
                FixtureRow::isolated("melee", "bitchslap", "Other", 97, &now)
                    .with_rank(1, "unarmed"),
            ],
            &[
                UnattributedFixtureRow::new(player, 10, &now, "H:90 [-10]", &["saved context"]),
                UnattributedFixtureRow::new(player, 20, &now, "H:80 [-20]", &[]),
                UnattributedFixtureRow::new("Other", 30, &now, "H:70 [-30]", &[]),
            ],
        );
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        for (query, summary, main_href) in [
            ("", "All time · All players", "/"),
            (
                "?range=24h&player=A%3C%26%22",
                "Last 24h · Player: A&lt;&amp;&quot;",
                "/?range=24h&amp;player=A%3C%26%22",
            ),
            (
                "?range=7d&player=A%3C%26%22",
                "Last 7d · Player: A&lt;&amp;&quot;",
                "/?range=7d&amp;player=A%3C%26%22",
            ),
        ] {
            for route in ["/unattributed/1", "/events/melee/bitchslap"] {
                let response = app
                    .clone()
                    .oneshot(
                        Request::builder()
                            .uri(format!("{route}{query}"))
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                let body = body_string(response).await;
                assert!(body.contains(summary));
                let navigation = body
                    .split_once("<div class=\"drill-navigation\">")
                    .unwrap()
                    .1
                    .split_once("</div>")
                    .unwrap()
                    .0;
                assert!(navigation.contains("class=\"back-link\""));
                assert!(navigation.contains("class=\"filter-summary\""));
                assert!(navigation.contains(summary));
                assert_eq!(body.matches("class=\"filter-summary\"").count(), 1);
                assert!(body.contains(&format!(
                    "href=\"{main_href}\">Change filters on main page</a>"
                )));
                assert!(!body.contains("<select"));
                assert!(!body.contains("method=\"get\""));
                if route.starts_with("/unattributed") {
                    assert!(body.contains("saved context"));
                    assert!(!body.contains("outside the active filters"));
                    assert!(
                        body.contains(&format!("/unattributed/2{query}\">&larr; Previous</a>"))
                    );
                } else if !query.is_empty() {
                    assert!(body.contains(">31<"));
                    assert!(!body.contains(">97<"));
                }
            }
        }
        let landing = body_string(
            app.oneshot(
                Request::builder()
                    .uri("/?range=7d&player=A%3C%26%22")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
        )
        .await;
        assert!(landing.contains("<form class=\"filters\" method=\"get\" action=\"/\">"));
        assert!(landing.contains("value=\"7d\" selected"));
        assert!(landing.contains("/unattributed/1?range=7d&player=A%3C%26%22"));
        assert!(
            landing.contains("/events/melee/bitchslap?range=7d&player=A%3C%26%22&family=unarmed")
        );
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn outside_filter_record_keeps_context_and_explains_missing_navigation() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[UnattributedFixtureRow::new(
                "A",
                10,
                "2020-01-01T00:00:00Z",
                "H:90 [-10]",
                &["saved context"],
            )],
        );
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        for query in ["?range=24h", "?range=7d", "?player=Other"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(format!("/unattributed/1{query}"))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = body_string(response).await;
            assert!(body.contains("This record is outside the active filters."));
            assert!(!body.contains("&larr; Previous</a>"));
            assert!(!body.contains("Next &rarr;</a>"));
            assert!(body.contains("<li>saved context</li><li>H:90 [-10]</li>"));
        }
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn drill_down_contains_fixture_message_text() {
        let (app, path) = app_with_rows(&standard_fixture_rows());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/events/melee/bitchslap")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("Holy man bitchslaps you."));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn drill_down_skill_and_spell_routes_work() {
        let (app, path) = app_with_rows(&standard_fixture_rows());
        for uri in ["/events/skill/bash", "/events/spell/magic%20missile"] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn ambiguous_batch_siblings_appear_on_drill_down() {
        let (app, path) = app_with_rows(&standard_fixture_rows());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/events/melee/bitchslap?player=Odefu")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = body_string(response).await;
        assert!(body.contains("batch-group"));
        assert!(body.contains("0–22"));
        assert!(body.contains("Holy man boots you."));
        assert!(body.contains("href=\"/events/melee/boot?player=Odefu\""));
        assert!(body.contains("player=Odefu&sort=hp_delta&dir=asc"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn sort_links_preserve_filter_params() {
        let (app, path) = app_with_rows(&standard_fixture_rows());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?player=Odefu&range=7d")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = body_string(response).await;
        assert!(body.contains("player=Odefu"));
        assert!(body.contains("range=7d"));
        remove_db_files(&path);
    }

    #[test]
    fn unreadable_db_message_is_documented() {
        assert_eq!(CANNOT_OPEN_DATABASE, "Cannot open combat damage database.");
        assert_eq!(CURRENT_SCHEMA_VERSION, 5);
    }

    #[test]
    fn unattributed_transcript_ends_with_escaped_hp_loss_line() {
        for context_lines in [vec![], vec!["Knight misses.".to_string()]] {
            let detail = UnattributedHpDetail {
                id: 997,
                recorded_at: "2026-10-28T07:29:02Z".to_string(),
                player: "Fueryon".to_string(),
                hp_delta: 4,
                hp_before: 549,
                hp_after: 545,
                h_line_text: "H:545/938 [-4] <test>&".to_string(),
                context_lines,
            };
            let body = render_unattributed_drill_down(
                &detail,
                &FilterParams::from_query(None, None),
                &[],
                &ViewerQuery {
                    range: None,
                    player: None,
                    sort: None,
                    dir: None,
                    family: None,
                },
                None,
                None,
            );
            assert!(body.contains("<li>H:545/938 [-4] &lt;test&gt;&amp;</li></ol>"));
            if !detail.context_lines.is_empty() {
                assert!(
                    body.contains(
                        "<li>Knight misses.</li><li>H:545/938 [-4] &lt;test&gt;&amp;</li>"
                    )
                );
            }
        }
    }

    #[tokio::test]
    async fn unattributed_fixture_landing_and_drill_down() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[UnattributedFixtureRow::new(
                "Odefu",
                22,
                "2026-08-06T15:00:00Z",
                "H:760/782 [-22] S:100/100 [] E:100/100 [] $:100 [] exp:100 []",
                &["Holy man misses.", "You miss."],
            )],
        );
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("Unattributed HP loss"));
        assert!(body.contains("2026-08-06T15:00:00Z"));
        assert!(body.contains("Odefu"));
        assert!(body.contains(">22<"));
        assert!(body.contains(">2<"));

        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/unattributed/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = body_string(response).await;
        assert!(body.contains("Holy man misses."));
        assert!(body.contains("You miss."));
        assert!(body.contains(
            "<li>H:760/782 [-22] S:100/100 [] E:100/100 [] $:100 [] exp:100 []</li></ol>"
        ));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn unattributed_drill_down_marks_reviewed_and_landing_reflects_it() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[
                UnattributedFixtureRow::new(
                    "Odefu",
                    22,
                    "2026-08-06T15:00:00Z",
                    "H:760/782 [-22] S:100/100 []",
                    &["Holy man misses."],
                ),
                UnattributedFixtureRow::new(
                    "Odefu",
                    10,
                    "2026-08-06T16:00:00Z",
                    "H:90/100 [-10] S:100/100 []",
                    &[],
                ),
            ],
        );
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        let app = router(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/unattributed/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let app = router(state);
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = body_string(response).await;
        assert!(body.contains("(1 unreviewed)"));
        assert!(body.contains("class=\"reviewed\""));
        assert!(body.contains(">yes<"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn unattributed_reopen_is_idempotent() {
        use crate::combat_damage::storage::open_db;
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[UnattributedFixtureRow::new(
                "Odefu",
                22,
                "2026-08-06T15:00:00Z",
                "H:760/782 [-22] S:100/100 []",
                &["Holy man misses."],
            )],
        );
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        for _ in 0..2 {
            let app = router(state.clone());
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/unattributed/1")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let conn = open_db(&path).unwrap();
        let reviewed_at: String = conn
            .query_row(
                "SELECT reviewed_at FROM unattributed_hp_events WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!reviewed_at.is_empty());
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn unattributed_player_filter_applies() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[
                UnattributedFixtureRow::new(
                    "Odefu",
                    22,
                    "2026-08-06T15:00:00Z",
                    "H:760/782 [-22] S:100/100 []",
                    &["Holy man misses."],
                ),
                UnattributedFixtureRow::new(
                    "Beta",
                    10,
                    "2026-08-06T16:00:00Z",
                    "H:90/100 [-10] S:100/100 []",
                    &[],
                ),
            ],
        );
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });
        let app = router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?player=Odefu")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = body_string(response).await;
        assert!(body.contains("2026-08-06T15:00:00Z"));
        assert!(!body.contains("2026-08-06T16:00:00Z"));
        remove_db_files(&path);
    }

    #[test]
    fn unattributed_neighbor_ids_uses_landing_list_position() {
        use crate::combat_damage::aggregate::UnattributedHpSummary;
        let rows = vec![
            UnattributedHpSummary {
                id: 3,
                recorded_at: String::new(),
                player: String::new(),
                hp_delta: 0,
                line_count: 0,
                reviewed: false,
            },
            UnattributedHpSummary {
                id: 2,
                recorded_at: String::new(),
                player: String::new(),
                hp_delta: 0,
                line_count: 0,
                reviewed: false,
            },
            UnattributedHpSummary {
                id: 1,
                recorded_at: String::new(),
                player: String::new(),
                hp_delta: 0,
                line_count: 0,
                reviewed: false,
            },
        ];
        assert_eq!(unattributed_neighbor_ids(&rows, 3), (None, Some(2)));
        assert_eq!(unattributed_neighbor_ids(&rows, 2), (Some(3), Some(1)));
        assert_eq!(unattributed_neighbor_ids(&rows, 1), (Some(2), None));
        assert_eq!(unattributed_neighbor_ids(&rows, 99), (None, None));
    }

    #[tokio::test]
    async fn unattributed_drill_down_prev_next_within_filtered_list() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[
                UnattributedFixtureRow::new(
                    "Odefu",
                    5,
                    "2026-08-06T14:00:00Z",
                    "H:95/100 [-5] S:100/100 []",
                    &[],
                ),
                UnattributedFixtureRow::new(
                    "Odefu",
                    10,
                    "2026-08-06T15:00:00Z",
                    "H:90/100 [-10] S:100/100 []",
                    &[],
                ),
                UnattributedFixtureRow::new(
                    "Odefu",
                    22,
                    "2026-08-06T16:00:00Z",
                    "H:760/782 [-22] S:100/100 []",
                    &[],
                ),
            ],
        );
        let state = Arc::new(ViewerState {
            db_path: path.clone(),
        });

        let app = router(state.clone());
        let body = body_string(
            app.oneshot(
                Request::builder()
                    .uri("/unattributed/3")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
        )
        .await;
        assert!(!body.contains("&larr; Previous</a>"));
        assert!(body.contains("/unattributed/2\">Next &rarr;</a>"));

        let app = router(state.clone());
        let body = body_string(
            app.oneshot(
                Request::builder()
                    .uri("/unattributed/2")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
        )
        .await;
        assert!(body.contains("/unattributed/3\">&larr; Previous</a>"));
        assert!(body.contains("/unattributed/1\">Next &rarr;</a>"));

        let app = router(state.clone());
        let body = body_string(
            app.oneshot(
                Request::builder()
                    .uri("/unattributed/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
        )
        .await;
        assert!(body.contains("/unattributed/2\">&larr; Previous</a>"));
        assert!(!body.contains("Next &rarr;</a>"));

        let single_path = open_fixture_db_with_unattributed(
            &[],
            &[UnattributedFixtureRow::new(
                "Odefu",
                22,
                "2026-08-06T15:00:00Z",
                "H:760/782 [-22] S:100/100 []",
                &[],
            )],
        );
        let app = router(Arc::new(ViewerState {
            db_path: single_path.clone(),
        }));
        let body = body_string(
            app.oneshot(
                Request::builder()
                    .uri("/unattributed/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
        )
        .await;
        assert!(!body.contains("&larr; Previous</a>"));
        assert!(!body.contains("Next &rarr;</a>"));
        remove_db_files(&single_path);

        let app = router(state);
        let body = body_string(
            app.oneshot(
                Request::builder()
                    .uri("/unattributed/1?player=Beta")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap(),
        )
        .await;
        assert!(!body.contains("&larr; Previous</a>"));
        assert!(!body.contains("Next &rarr;</a>"));
        remove_db_files(&path);
    }

    fn mark_unattributed_reviewed_fixture(path: &std::path::Path, ids: &[i64]) {
        let conn = open_db(path).unwrap();
        for id in ids {
            conn.execute(
                "UPDATE unattributed_hp_events SET reviewed_at = '2026-08-06T12:00:00Z' WHERE id = ?1",
                [*id],
            )
            .unwrap();
        }
    }

    #[tokio::test]
    async fn remove_reviewed_button_absent_when_none_reviewed() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[UnattributedFixtureRow::new(
                "Odefu",
                22,
                "2026-08-06T15:00:00Z",
                "H:760/782 [-22] S:100/100 []",
                &[],
            )],
        );
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        let body = body_string(
            app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
                .await
                .unwrap(),
        )
        .await;
        assert!(!body.contains("Remove reviewed"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn remove_reviewed_post_deletes_only_reviewed_in_filter() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[
                UnattributedFixtureRow::new(
                    "Odefu",
                    22,
                    "2026-08-06T15:00:00Z",
                    "H:760/782 [-22] S:100/100 []",
                    &[],
                ),
                UnattributedFixtureRow::new(
                    "Odefu",
                    10,
                    "2026-08-06T14:00:00Z",
                    "H:90/100 [-10] S:100/100 []",
                    &[],
                ),
                UnattributedFixtureRow::new(
                    "Odefu",
                    5,
                    "2026-08-06T13:00:00Z",
                    "H:95/100 [-5] S:100/100 []",
                    &[],
                ),
            ],
        );
        mark_unattributed_reviewed_fixture(&path, &[1, 2]);
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/unattributed/remove-reviewed")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from(""))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get("location").unwrap(),
            "/",
            "redirect preserves landing without filters"
        );

        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        let body = body_string(
            app.oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
                .await
                .unwrap(),
        )
        .await;
        assert!(!body.contains("class=\"reviewed\""));
        assert!(!body.contains("Remove reviewed"));
        assert!(body.contains(">5<"));
        assert!(!body.contains(">22<"));
        assert!(!body.contains(">10<"));
        remove_db_files(&path);
    }

    #[tokio::test]
    async fn remove_reviewed_post_respects_player_filter() {
        use crate::combat_damage::test_fixtures::{
            UnattributedFixtureRow, open_fixture_db_with_unattributed,
        };
        let path = open_fixture_db_with_unattributed(
            &[],
            &[
                UnattributedFixtureRow::new(
                    "Odefu",
                    22,
                    "2026-08-06T15:00:00Z",
                    "H:760/782 [-22] S:100/100 []",
                    &[],
                ),
                UnattributedFixtureRow::new(
                    "Beta",
                    10,
                    "2026-08-06T14:00:00Z",
                    "H:90/100 [-10] S:100/100 []",
                    &[],
                ),
            ],
        );
        mark_unattributed_reviewed_fixture(&path, &[1, 2]);
        let app = router(Arc::new(ViewerState {
            db_path: path.clone(),
        }));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/unattributed/remove-reviewed")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("player=Odefu"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);

        let conn = open_db(&path).unwrap();
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM unattributed_hp_events", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(remaining, 1);
        let beta_player: String = conn
            .query_row(
                "SELECT player FROM unattributed_hp_events WHERE id = 2",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(beta_player, "Beta");
        remove_db_files(&path);
    }
}
