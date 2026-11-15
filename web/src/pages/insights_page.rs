//! Insights surface (spec 006, US6 family: T053 feed, T055 confidence card,
//! T057 insufficient-data empty state; D2 candidate queue and D4 ledger).
//!
//! Honest increment: this page shows the drift feed (engine D3), the
//! note-grounded candidate queue (D2) and the test ledger (D4). Candidates
//! carry their origin note and available data and are explicitly *hypotheses*
//! — the `Candidate` type cannot express a verdict, and the ledger only
//! changes through an actual review outcome, so no UI path lets an
//! un-earned insight render as a finding. The intervention-review window
//! picker (starting a D1 review from a candidate) is the next increment;
//! until it exists the candidate states what running one would use.

use crate::state::db::{get_catalog_items, get_insight_ledger, get_notes, get_vitals_entries};
use crate::state::store::AppContext;
use engine::candidates::{Candidate, generate_candidates};
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

    // D2 queue: substance × metric hypotheses grounded in the user's own
    // notes, each carrying its origin and the data a review could use.
    let candidates = Signal::derive(move || {
        version.get(); // track
        generate_candidates(
            &get_notes().unwrap_or_default(),
            &get_catalog_items(),
            &get_vitals_entries(&Default::default()).unwrap_or_default(),
            &get_insight_ledger(),
        )
    });

    // D4 ledger: what has already been tested, so it shows as tested and
    // never resurfaces as new.
    let ledger = Signal::derive(move || {
        version.get(); // track
        get_insight_ledger()
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

            <section class="candidate-queue" aria-label="Correlation candidates">
                <h3>"Correlation candidates"</h3>
                <p class="insights-subtitle">
                    "Hypotheses from your notes — a note names something you wanted to see;
                     the queue shows what data a review could use. Nothing here is a finding."
                </p>
                <Show
                    when=move || !candidates.get().is_empty()
                    fallback=|| view! {
                        <p class="empty-state" role="status">
                            "No candidates yet. A note that names a catalog substance
                             (\"anxiety after Ashwagandha\") becomes a candidate here."
                        </p>
                    }
                >
                    <ul class="candidate-list">
                        {move || {
                            candidates
                                .get()
                                .into_iter()
                                .map(|c| view! { <CandidateRow candidate=c /> })
                                .collect::<Vec<_>>()
                        }}
                    </ul>
                </Show>
            </section>

            <Show when=move || !ledger.get().is_empty()>
                <section class="insight-ledger" aria-label="Test ledger">
                    <h3>"Test ledger"</h3>
                    <table class="ledger-table">
                        <thead>
                            <tr>
                                <th>"Substance × metric"</th>
                                <th>"Status"</th>
                                <th>"Evidence"</th>
                                <th>"Outcome"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                ledger
                                    .get()
                                    .into_iter()
                                    .map(|e| {
                                        let pair = format!(
                                            "{} × {}",
                                            e.substance,
                                            e.metric.name()
                                        );
                                        let status = e.status.label().to_string();
                                        let evidence = e.evidence.clone();
                                        let outcome = e
                                            .outcome_at
                                            .map(|t| t.date_naive().to_string())
                                            .unwrap_or_default();
                                        view! {
                                            <tr>
                                                <td>{pair}</td>
                                                <td><span class=format!("ledger-status {}", status.clone())>{status.clone()}</span></td>
                                                <td class="ledger-evidence">{evidence}</td>
                                                <td>{outcome}</td>
                                            </tr>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            }}
                        </tbody>
                    </table>
                </section>
            </Show>
        </div>
    }
}

/// One queue row. Deliberately a row, not a card: a candidate is cheaper
/// than an insight and must not visually compete with one (FR-097).
#[component]
fn CandidateRow(candidate: Candidate) -> impl IntoView {
    let substance = candidate.substance.clone();
    let metric = candidate
        .metric
        .map(|m| m.name().to_string())
        .unwrap_or_else(|| "no measurable outcome".to_string());
    let origin = format!("from note {}", candidate.origin_note_at.date_naive());
    let reason = candidate.reason.clone();
    let tested = candidate.tested;
    let testable = candidate.testable;
    let points = if candidate.data_points > 0 {
        format!("{} readings", candidate.data_points)
    } else {
        "0 readings".to_string()
    };

    view! {
        <li class="candidate-row" aria-label=format!("Candidate: {} × {}", substance.clone(), metric.clone())>
            <span class="candidate-pair">
                {substance.clone()}
                <em>{" × "}</em>
                {metric.clone()}
            </span>
            <span class="candidate-origin">{origin}</span>
            <span class="candidate-data">{points}</span>
            <span class="candidate-status">
                {if tested {
                    "tested"
                } else if testable {
                    "candidate"
                } else {
                    "untestable"
                }}
            </span>
            <span class="candidate-reason" title=reason.clone()>{reason.clone()}</span>
        </li>
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
