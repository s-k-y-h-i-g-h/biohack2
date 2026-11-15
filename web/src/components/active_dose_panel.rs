use engine::dashboard::PanelView;
use leptos::prelude::*;

/// spec-003 T241–T244: the active-dose panel. Draws a PanelView and
/// computes nothing semantic — all derivation lives in `engine::dashboard`
/// so the future terminal renderer shares it (spec 005 FR-081).
///
/// Band, not line (FR-033): the shaded region between low and high is the
/// primary mark; the expected line rides inside it. The "now" marker is a
/// vertical rule at the 24 h point of a 24-back/12-forward window (T243),
/// and target edges are dashed rules on the same axis (T244).
#[component]
pub fn ActiveDosePanel(view: PanelView) -> impl IntoView {
    let class_label = format!("{:?}", view.model_class);
    let badge_class = match view.model_class {
        engine::models::ModelClass::None => "badge badge-none",
        _ if view.below_target => "badge badge-low",
        _ => "badge",
    };
    let substance = view.substance.clone();
    let note = view.note.clone();
    let below = view.below_target;
    let assumption = view.band_is_assumption;

    // ── SVG geometry (viewBox 360×80; 37 hourly points at 10 px apart) ──
    const W: f64 = 360.0;
    const H: f64 = 80.0;
    const PAD: f64 = 6.0;
    let step = W / 36.0;
    let max = view
        .series
        .iter()
        .map(|p| p.high)
        .chain(view.target.iter().map(|t| t.high))
        .fold(0.0f64, f64::max)
        .max(f64::EPSILON);
    let y = |v: f64| H - PAD - (v / max) * (H - 2.0 * PAD);
    let x = |i: usize| (i as f64 * step).min(W);

    let expected_line: String = view
        .series
        .iter()
        .enumerate()
        .map(|(i, p)| format!("{:.1},{:.1}", x(i), y(p.expected)))
        .collect::<Vec<_>>()
        .join(" ");
    // Band polygon: high edge forward, low edge backward, closed.
    let band_shape: String = {
        let mut pts: Vec<String> = view
            .series
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{:.1},{:.1}", x(i), y(p.high)))
            .collect();
        pts.extend(
            view.series
                .iter()
                .enumerate()
                .rev()
                .map(|(i, p)| format!("{:.1},{:.1}", x(i), y(p.low))),
        );
        pts.join(" ")
    };
    let now_x = x(view.now_index).to_string();
    // Target edges precomputed as plain strings — reactive closures must
    // not capture the geometry helpers.
    let target_lines: Option<(String, String)> = view
        .target
        .as_ref()
        .map(|t| (y(t.low).to_string(), y(t.high).to_string()));

    let gauge = view.now_reading.clone();
    let note_for_provenance = note.clone();

    view! {
        <div class="pk-panel">
            <div class="pk-panel-head">
                <h4>{substance.clone()}</h4>
                <span class=badge_class>{class_label}</span>
            </div>

            {move || match &gauge {
                Some(r) => view! {
                    <>
                        <div class=format!("pk-gauge{}", if below { " below" } else { "" })>
                            <span class="pk-gauge-value">{format!("{:.0}", r.expected)}</span>
                            <span class="pk-gauge-unit">{r.unit.clone()}</span>
                            <span class="pk-gauge-band">
                                {format!("({:.0}–{:.0} {})", r.low, r.high, r.unit)}
                            </span>
                            {below.then(|| view! { <span class="pk-gauge-flag">"below target"</span> })}
                        </div>
                        <div class="pk-spark" title="24 h back · 12 h forward, hourly">
                            <svg viewBox="0 0 360 80" preserveAspectRatio="none" aria-hidden="true">
                                // band first: the primary mark (FR-033)
                                <polygon points=band_shape.clone() class="pk-band"/>
                                <polyline points=expected_line.clone() class="pk-expected" fill="none"/>
                                // target edges on the same axis (T244)
                                {target_lines.clone().map(|(low, high)| view! {
                                    <>
                                        <line x1="0" x2="360" y1=low.clone() y2=low class="pk-target-edge"/>
                                        <line x1="0" x2="360" y1=high.clone() y2=high class="pk-target-edge"/>
                                    </>
                                })}
                                // now marker (T243)
                                <line x1=now_x.clone() x2=now_x.clone() y1="0" y2="80" class="pk-now"/>
                            </svg>
                            {assumption.then(|| view! {
                                <div class="pk-assumption-note">
                                    "band width assumed ±25% — no catalog range"
                                </div>
                            })}
                        </div>
                    </>
                }
                .into_any(),
                // T239 surface: no model ⇒ say why, never draw zero.
                None => view! { <div class="pk-nomodel">{note.clone()}</div> }.into_any(),
            }}

            <details class="pk-provenance">
                <summary>"Model & limitations"</summary>
                <p>{note_for_provenance}</p>
            </details>
        </div>
    }
}
