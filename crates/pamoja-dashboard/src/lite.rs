//! The no-JavaScript floor: a server-rendered, meta-refreshing status table.
//!
//! The smallest tier serves the embedded [`lite.html`](../web/lite.html), which renders the
//! fleet with a tiny script. When scripting is off entirely, that page bounces to `GET
//! /lite`, served from here: the same readable status table built once on the device and
//! refreshed by a `<meta http-equiv="refresh">`, with no script at all. It is plain, but it
//! is legible and it works on any browser. This is the only place the device formats HTML at
//! runtime, kept to a single small table on purpose.

use crate::state::{Reading, State, Status};

/// How often the no-script page reloads itself, in seconds.
const REFRESH_SECS: u32 = 15;

/// Renders the fleet snapshot as a complete, self-contained, no-script HTML page.
///
/// The page carries its own styles inline and a `<meta http-equiv="refresh">`, so it needs
/// no other asset and stays current without any client code.
///
/// # Arguments
///
/// * `state` - the fleet snapshot to render.
///
/// # Returns
///
/// A full HTML document as a string.
pub(crate) fn render_lite(state: &State) -> String {
    let (word, sym, class) = status_bits(state.status);
    let mut out = String::with_capacity(2048);
    out.push_str(&page_head());
    out.push_str(&format!(
        "<header><h1>pamoja</h1><p class=\"status {class}\">{sym} {word}</p></header>\n"
    ));

    let mut any = false;
    for org in &state.orgs {
        for group in &org.groups {
            any = true;
            let (gword, gsym, gclass) = status_bits(group.status);
            let online = if group.link.online {
                "online"
            } else {
                "offline"
            };
            out.push_str(&format!(
                "<section><h2>{name} <span class=\"badge {gclass}\">{gsym} {gword}</span></h2>\n\
                 <p class=\"muted\">{kind} {strength}/4 {online}</p>\n",
                name = esc(&group.name),
                kind = esc(link_kind(group.link.kind)),
                strength = group.link.strength,
            ));
            out.push_str(
                "<table><thead><tr><th>Sensor</th><th>Reading</th><th>Status</th></tr></thead><tbody>\n",
            );
            for sensor in &group.sensors {
                let (sword, ssym, sclass) = status_bits(sensor.reading.status);
                out.push_str(&format!(
                    "<tr><th scope=\"row\">{id}</th><td>{reading}</td><td class=\"{sclass}\">{ssym} {sword}</td></tr>\n",
                    id = esc(&sensor.id),
                    reading = reading_text(&sensor.reading),
                ));
            }
            out.push_str("</tbody></table></section>\n");
        }
    }
    if !any {
        out.push_str("<p class=\"muted\">No sensors are reporting yet.</p>\n");
    }

    out.push_str("<footer><a href=\"./\">Full dashboard</a></footer>\n</body></html>\n");
    out
}

/// Renders a minimal page for the rare case the snapshot cannot be read.
///
/// # Returns
///
/// A full HTML document reporting that the status is briefly unavailable.
pub(crate) fn render_unavailable() -> String {
    format!(
        "{}<header><h1>pamoja</h1><p class=\"status warn\">\u{25B2} Status unavailable</p></header>\n</body></html>\n",
        page_head()
    )
}

// The shared document head: doctype, meta-refresh, and the inline styles. The styles mirror
// lite.html so the scripted and no-script floors read the same.
fn page_head() -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\" dir=\"ltr\">\n<head>\n\
         <meta charset=\"utf-8\" />\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n\
         <meta name=\"color-scheme\" content=\"light dark\" />\n\
         <meta http-equiv=\"refresh\" content=\"{REFRESH_SECS}\" />\n\
         <title>pamoja status</title>\n<style>{STYLE}</style>\n</head>\n<body>\n",
    )
}

// Word, shape, and color class for a status: redundant encoding so it never relies on color.
fn status_bits(status: Status) -> (&'static str, &'static str, &'static str) {
    match status {
        Status::Alarm => ("ALARM", "\u{2715}", "alarm"),
        Status::Warn => ("WARN", "\u{25B2}", "warn"),
        Status::Ok => ("OK", "\u{2713}", "ok"),
    }
}

// The reading as legible text: the leaf of a discrete state code, or a rounded value and unit.
fn reading_text(reading: &Reading) -> String {
    if let Some(state) = &reading.state {
        let leaf = state.rsplit('.').next().unwrap_or(state);
        return esc(leaf);
    }
    let rounded = (reading.value as f64 * 100.0).round() / 100.0;
    if reading.unit.is_empty() {
        format!("{rounded}")
    } else {
        format!("{rounded} {}", esc(&reading.unit))
    }
}

// A short, stable English label for a link kind. The floor page is single-locale by design.
fn link_kind(kind: crate::state::LinkKind) -> &'static str {
    use crate::state::LinkKind::*;
    match kind {
        Lora => "LoRa",
        Wifi => "Wi-Fi",
        Cellular => "Cellular",
        NbIot => "NB-IoT",
        Satellite => "Satellite",
        Ethernet => "Ethernet",
        Mesh => "Mesh",
    }
}

// Escapes the characters that would otherwise break out of HTML text content.
fn esc(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const STYLE: &str = "\
:root{--bg:#f6f7fb;--card:#fff;--line:#c8cedd;--text:#11151f;--muted:#5a6478;--ok:#137a4b;--warn:#8a5a00;--alarm:#b3261e}\
@media(prefers-color-scheme:dark){:root{--bg:#0b0f1a;--card:#131a2a;--line:#2a3552;--text:#eef2fb;--muted:#98a3bd;--ok:#34d399;--warn:#fbbf24;--alarm:#f87171}}\
*{box-sizing:border-box}\
body{margin:0;padding:1rem;background:var(--bg);color:var(--text);font:16px/1.5 system-ui,-apple-system,Segoe UI,Roboto,sans-serif}\
header{display:flex;align-items:baseline;justify-content:space-between;gap:1rem;flex-wrap:wrap;margin-bottom:1rem}\
h1{margin:0;font-size:1.3rem;letter-spacing:.02em}\
h2{margin:1.25rem 0 .4rem;font-size:1.05rem;display:flex;align-items:center;gap:.6rem;flex-wrap:wrap}\
.status{font-weight:700;font-size:1.05rem}\
.badge{font-size:.78rem;font-weight:700;padding:.15rem .5rem;border-radius:999px;border:1px solid currentColor}\
.ok{color:var(--ok)}.warn{color:var(--warn)}.alarm{color:var(--alarm)}\
.muted{color:var(--muted);font-size:.85rem;margin:.1rem 0}\
table{width:100%;border-collapse:collapse;background:var(--card);border:1px solid var(--line);border-radius:10px;overflow:hidden}\
th,td{text-align:start;padding:.55rem .7rem;border-bottom:1px solid var(--line)}\
thead th{font-size:.75rem;text-transform:uppercase;letter-spacing:.04em;color:var(--muted)}\
tbody tr:last-child th,tbody tr:last-child td{border-bottom:0}\
tbody th{font-weight:600}\
td.ok,td.warn,td.alarm{font-weight:700;white-space:nowrap}\
footer{margin-top:1.5rem}a{color:inherit}";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Mock, Scenario, StateSource};

    #[test]
    fn renders_a_complete_document_with_meta_refresh() {
        let html = render_lite(&Mock::new(Scenario::Normal).snapshot());
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("http-equiv=\"refresh\""));
        assert!(html.trim_end().ends_with("</html>"));
        // No script tag at all: this is the no-JavaScript floor.
        assert!(!html.contains("<script"));
    }

    #[test]
    fn an_alarm_fleet_shows_the_alarm_word_not_just_a_color() {
        let html = render_lite(&Mock::new(Scenario::Alarm).snapshot());
        assert!(
            html.contains("ALARM"),
            "the status word must be present, not color alone"
        );
    }

    #[test]
    fn a_reading_shows_its_rounded_value_and_unit() {
        let mut state = State {
            orgs: vec![crate::Org {
                id: "o".into(),
                name: "Clinic".into(),
                groups: vec![crate::Group {
                    id: "g".into(),
                    name: "Cold chain".into(),
                    link: crate::Link {
                        kind: crate::LinkKind::Lora,
                        strength: 3,
                        online: true,
                    },
                    status: Status::Ok,
                    sensors: vec![crate::Sensor::new(
                        "fridge-1",
                        Reading::new("temperature", 6.789, "celsius"),
                    )],
                    lat: None,
                    lon: None,
                }],
            }],
            status: Status::Ok,
            uptime_secs: None,
            demo: false,
        };
        state.recompute_status();
        let html = render_lite(&state);
        assert!(html.contains("fridge-1"));
        assert!(
            html.contains("6.79 celsius"),
            "value should round to two places: {html}"
        );
        assert!(html.contains("LoRa 3/4 online"));
    }

    #[test]
    fn names_are_html_escaped() {
        let html = render_lite(&State {
            orgs: vec![crate::Org {
                id: "o".into(),
                name: "Org".into(),
                groups: vec![crate::Group {
                    id: "g".into(),
                    name: "Acme & Co <x>".into(),
                    link: crate::Link {
                        kind: crate::LinkKind::Wifi,
                        strength: 4,
                        online: true,
                    },
                    status: Status::Ok,
                    sensors: Vec::new(),
                    lat: None,
                    lon: None,
                }],
            }],
            status: Status::Ok,
            uptime_secs: None,
            demo: false,
        });
        assert!(!html.contains("Acme & Co <x>"));
        assert!(html.contains("Acme &amp; Co &lt;x&gt;"));
    }

    #[test]
    fn a_discrete_state_shows_its_leaf() {
        let reading = Reading::new("valve", 0.0, "").with_state("state.open");
        assert_eq!(reading_text(&reading), "open");
    }
}
