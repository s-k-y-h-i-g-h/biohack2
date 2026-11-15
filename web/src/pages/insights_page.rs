//! Insights surface (spec 006, US6 family: T053 feed, T055 confidence card,
//! T057 insufficient-data empty state).
//!
//! Honest increment: this page shows the drift feed (engine D3) and nothing
//! else. The intervention-review list and pending-candidate queue depend on
//! the D2/D4 ledger (T058–T060), which is not implemented — the page omits
//! those sections rather than rendering placeholder pseudo-insights. The
//! whole point of spec 006 is that an insight you cannot earn is worse than
//! no insight, and that applies to our own UI scaffolding too.

use crate::state::db::get_vitals_entries;
use crate::state::store::AppContext;
use engine::insights::{DriftConfig, DriftDirection, DriftInsight, Metric, detect_drift};
use leptos::prelude::*;

/// Metrics the feed sweeps. An insight appears per metric that has enough
/// history; metrics without data simply contribute nothing (FR-092).
const FEED_METRICS: [Metric; 7] = [
    Metric::HeartRate,
    Metric::Hrv,
    Metric::Weight,
    Metric::Spo2,
    Metric::BloodGlucose,
    Metric::BpSystolic,
    Metric::BpDiastolic,
];

#[component]
pub fn InsightsPage() -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let version = ctx.data_version;

    // Pure engine calls over freshly-read storage; re-computes whenever any
    // data changes (same reactive contract as the other pages).
    let insights = Signal::derive(move || {
        version.get(); // track
        let vitals = get_vitals_entries(&Default::default()).unwrap_or_default();
        let now = chrono::Utc::now();
        FEED_METRICS
            .iter()
            .filter_map(|m| detect_drift(&vitals, *m, now, &DriftConfig::default()))
            .collect::<Vec<DriftInsight>>()
    });

    view! {
        <div class="page insights-page">
            <h2>"Insights"</h2>
            <p class="insights-subtitle">
                "Sustained movements in what you recorded — descriptive, over your own data,
                 never a clinical conclusion."
            </p>
            <Show
                when=move || !insights.get().is_empty()
                fallback=|| view! {
                    // T057: the empty state states what an insight requires.
                    // Refusing to speculate on thin data is the feature.
                    <div class="empty-state" role="status">
                        <p>"No insights yet."</p>
                        <p>
                            "An insight needs at least 14 readings of one metric spanning at
                             least 4 weeks. Keep logging — the feed fills as your history
                             accumulates."
                        </p>
                    </div>
                }
            >
                <div class="insights-feed">
                    {move || {
                        insights
                            .get()
                            .into_iter()
                            .map(|d| view! { <DriftCard insight=d /> })
                            .collect::<Vec<_>>()
                    }}
                </div>
            </Show>
        </div>
    }
}

/// T055: confidence card. The window, n, density and the limitations
/// statement render in the same view as the number, always — there is no
/// "details" link that would let a bare effect size travel alone
/// (FR-091, FR-094, AC D1-3).
#[component]
fn DriftCard(insight: DriftInsight) -> impl IntoView {
    let metric = insight.metric.name().to_string();
    let definition = insight.metric.definition().to_string();
    let direction = match insight.direction {
        DriftDirection::Rising => "rising",
        DriftDirection::Falling => "falling",
    }
    .to_string();
    let window = format!(
        "{} → {}",
        insight.window.0.date_naive(),
        insight.window.1.date_naive()
    );
    let points = insight.n.to_string();
    let density = format!("{:.1}/day", insight.density);
    let movement = format!(
        "{:+.1} over the window ({:+.3}/day)",
        insight.total_move, insight.slope_per_day
    );
    let consistency = format!("{:.0}% of movements agree", insight.consistency * 100.0);
    let gaps = format!("{}", insight.gaps.len());
    let limitations = insight.limitations.clone();

    view! {
        <article class="insight-card" aria-label=format!("Drift insight: {}", metric.clone())>
            <header class="insight-card-header">
                <h3>{metric.clone()}</h3>
                <span class="drift-direction">{direction.clone()}</span>
            </header>
            <p class="insight-movement">{movement.clone()}</p>
            <dl class="insight-meta">
                <dt>"Window"</dt>
                <dd>{window.clone()}</dd>
                <dt>"Points"</dt>
                <dd>{points.clone()}</dd>
                <dt>"Density"</dt>
                <dd>{density.clone()}</dd>
                <dt>"Consistency"</dt>
                <dd>{consistency.clone()}</dd>
                <dt>"Gaps ≥2 days"</dt>
                <dd>{gaps.clone()}</dd>
                <dt>"Definition"</dt>
                <dd>{definition.clone()}</dd>
            </dl>
            <p class="insight-limitations">{limitations.clone()}</p>
        </article>
    }
}
