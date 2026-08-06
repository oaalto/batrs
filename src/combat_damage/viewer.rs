use crate::combat_damage::aggregate::{
    EventSortColumn, FilterParams, LandingSortColumn, SortDirection, TimeRange, VerbAggregate,
    category_aggregates, list_events, list_players,
};
use crate::combat_damage::storage::open_readonly_db;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
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
    let melee = category_aggregates(conn, "melee", filters, sort_col.clone(), sort_dir)?;
    let skill = category_aggregates(conn, "skill", filters, sort_col.clone(), sort_dir)?;
    let spell = category_aggregates(conn, "spell", filters, sort_col, sort_dir)?;
    Ok(render_landing(
        filters, &players, &melee, &skill, &spell, query,
    ))
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
    let players = list_players(conn, &FilterParams::from_query(None, None))?;
    let events = list_events(conn, category, verb, filters, sort_col, sort_dir)?;
    Ok(render_drill_down(
        category, verb, filters, &players, &events, query,
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
    melee: &[VerbAggregate],
    skill: &[VerbAggregate],
    spell: &[VerbAggregate],
    query: &ViewerQuery,
) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\">");
    html.push_str("<title>Combat damage</title><link rel=\"stylesheet\" href=\"/style.css\">");
    html.push_str("</head><body><h1>Combat damage</h1>");
    html.push_str(&render_filters(filters, players, "/"));
    html.push_str(&render_category_table(
        "Melee",
        "melee",
        "No melee damage recorded yet.",
        melee,
        filters,
        query,
    ));
    html.push_str(&render_category_table(
        "Skill",
        "skill",
        "No skill damage recorded yet.",
        skill,
        filters,
        query,
    ));
    html.push_str(&render_category_table(
        "Spell",
        "spell",
        "No spell damage recorded yet.",
        spell,
        filters,
        query,
    ));
    html.push_str(render_sort_script());
    html.push_str("</body></html>");
    html
}

fn render_drill_down(
    category: &str,
    verb: &str,
    filters: &FilterParams,
    players: &[String],
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
        "<p class=\"back-link\"><a href=\"{back_href}\">&larr; Back</a></p>"
    ));
    html.push_str(&format!(
        "<h1>{} / {}</h1>",
        html_escape(category),
        html_escape(verb)
    ));
    html.push_str(&render_filters(
        filters,
        players,
        &format!("/events/{category}/{verb}"),
    ));
    html.push_str("<table><thead><tr>");
    for (col, label) in [
        ("recorded_at", "Recorded at"),
        ("player", "Player"),
        ("hp_delta", "HP delta"),
        ("source_name", "Source"),
        ("weight", "Weight"),
        ("candidate_count", "Candidates"),
        ("message_text", "Message"),
    ] {
        let href = build_event_sort_href(category, verb, filters, col, query);
        html.push_str(&format!("<th><a href=\"{href}\">{label}</a></th>"));
    }
    html.push_str("</tr></thead><tbody>");
    if events.is_empty() {
        html.push_str("</tbody></table><p class=\"empty-hint\">No events for this verb.</p>");
    } else {
        let mut batch_counts = HashMap::new();
        for event in events {
            *batch_counts.entry(event.batch_id).or_insert(0usize) += 1;
        }
        for event in events {
            let batch_class = if event.candidate_count > 1 {
                " class=\"batch-group\""
            } else {
                ""
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
            html.push_str(&format!("<td class=\"numeric\">{:.2}</td>", event.weight));
            html.push_str(&format!(
                "<td class=\"numeric\">{}</td>",
                event.candidate_count
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
) -> String {
    let mut html = format!("<h2>{title}</h2><table><thead><tr><th rowspan=\"2\">");
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
        html.push_str(&format!(
            "</tbody></table><p class=\"empty-hint\">{empty_hint}</p>"
        ));
        return html;
    }
    for row in rows {
        let href = format!("/events/{category}/{}", percent_encode_path(&row.verb));
        let query_suffix = build_filter_query(filters);
        html.push_str("<tr>");
        html.push_str(&format!(
            "<td><a href=\"{href}{query_suffix}\">{}</a></td>",
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

fn build_filter_query(filters: &FilterParams) -> String {
    let mut parts = Vec::new();
    if filters.range != TimeRange::All {
        parts.push(format!("range={}", filters.range.as_str()));
    }
    if let Some(player) = &filters.player {
        parts.push(format!("player={}", percent_encode_path(player)));
    }
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
    let mut parts = Vec::new();
    if filters.range != TimeRange::All {
        parts.push(format!("range={}", filters.range.as_str()));
    }
    if let Some(player) = &filters.player {
        parts.push(format!("player={}", percent_encode_path(player)));
    }
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
    format!(
        "/?{}",
        [
            build_filter_query(filters)
                .trim_start_matches('?')
                .to_string(),
            format!("sort={column}"),
            format!("dir={next_dir}"),
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("&")
    )
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
    format!(
        "/events/{category}/{}?{}",
        percent_encode_path(verb),
        [
            build_filter_query(filters)
                .trim_start_matches('?')
                .to_string(),
            format!("sort={column}"),
            format!("dir={next_dir}"),
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("&")
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
        assert!(body.contains("bitchslap"));
        assert!(body.contains("Confirmed"));
        assert!(body.contains("Estimated"));
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
                    .uri("/events/melee/bitchslap")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = body_string(response).await;
        assert!(body.contains("batch-group"));
        assert!(body.contains("0–22"));
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
        assert_eq!(CURRENT_SCHEMA_VERSION, 2);
    }
}
