//! Insights surface (spec 006, US6 family: T053 feed, T055 confidence card,
//! T057 insufficient-data empty state; D2 candidate queue and D4 ledger).
//!
//! Honest increment: this page shows the drift feed (engine D3), the
//! note-grounded candidate queue (D2) and the test ledger (D4). Candidates
//! carry their origin note and available data and are explicitly *hypotheses*
//! — the `Candidate` type cannot express a verdict, and the ledger only
//! changes through an actual review outcome, so no UI path lets an
//! un-earned insight render as a finding. Running a review from a candidate
//! uses the engine's fixed default windows (14 days each side, floor of 7
//! points): the note's timestamp anchors the intervention, so the user
//! cannot shop for a flattering window (FR-096).

use crate::state::db::{
    get_catalog_items, get_insight_ledger, get_log_entries, get_notes, get_vitals_entries,
    set_insight_ledger,
};
use crate::state::store::AppContext;
use engine::candidates::{
    Candidate, LedgerStatus, generate_candidates, latest_mention, record_outcome,
};
use engine::insights::{
    ConfounderKind, DriftConfig, DriftDirection, DriftInsight, Intervention, Metric, ReviewResult,
    ReviewSpec, Side, Verdict, detect_drift, run_review,
};
use leptos::ev::MouseEvent;
use leptos::prelude::*;

/// A review started from a candidate: the pair under test, the anchor the
/// note provided, and the engine's answer. Stored in a page signal so the
/// panel (verdict + limitations) and the record action see one object —
/// the verdict can never travel without its limitations text (FR-094).
#[derive(Clone, PartialEq)]
struct OpenReview {
    substance: String,
    metric: Metric,
    result: ReviewResult,
}

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

    // D1 review, started from a candidate or a ledger re-test. One review
    // is open at a time; recording its outcome is the only way into the
    // ledger.
    let review = RwSignal::<Option<OpenReview>>::new(None);
    let open_review = Callback::new(
        move |(substance, metric, anchor): (String, Metric, chrono::DateTime<chrono::Utc>)| {
            let vitals = get_vitals_entries(&Default::default()).unwrap_or_default();
            let logs = get_log_entries().unwrap_or_default();
            // The anchor is decided by the caller's fixed rule (origin note
            // for candidates, newest mention for re-tests) — the user picks
            // no window, so there is nothing to shop for (FR-096).
            let intervention = Intervention {
                name: substance.clone(),
                started_at: anchor,
            };
            let spec = ReviewSpec::default_for(metric);
            let result = run_review(&vitals, &logs, &intervention, &spec, chrono::Utc::now());
            review.set(Some(OpenReview {
                substance,
                metric,
                result,
            }));
        },
    );
    let on_review = Callback::new(move |c: Candidate| {
        let Some(metric) = c.metric else { return };
        open_review.run((c.substance, metric, c.origin_note_at));
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
                                .map(|c| view! { <CandidateRow candidate=c on_review=on_review /> })
                                .collect::<Vec<_>>()
                        }}
                    </ul>
                </Show>
            </section>

            <Show when=move || review.get().is_some()>
                {move || {
                    review.get().map(|r| view! { <ReviewPanel review=r close=review /> })
                }}
            </Show>

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
                                <th>"Re-test"</th>
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
                                        // A re-test anchors on the newest note naming
                                        // the substance (fixed engine rule); no such
                                        // note means nothing to re-anchor to, and the
                                        // button says so instead of pretending.
                                        let substance = e.substance.clone();
                                        let metric = e.metric;
                                        let has_anchor = latest_mention(
                                            &get_notes().unwrap_or_default(),
                                            &substance,
                                        )
                                        .is_some();
                                        let retest_substance = substance.clone();
                                        view! {
                                            <tr>
                                                <td>{pair}</td>
                                                <td><span class=format!("ledger-status {}", status.clone())>{status.clone()}</span></td>
                                                <td class="ledger-evidence">{evidence}</td>
                                                <td>{outcome}</td>
                                                <td>
                                                    <button
                                                        class="candidate-review"
                                                        disabled=!has_anchor
                                                        title=if has_anchor {
                                                            "Re-run the review anchored on the newest note naming this substance"
                                                        } else {
                                                            "No note names this substance — nothing to anchor a re-test to"
                                                        }
                                                        on:click=move |_| {
                                                            let notes = get_notes().unwrap_or_default();
                                                            if let Some(anchor) =
                                                                latest_mention(&notes, &retest_substance)
                                                            {
                                                                open_review.run((
                                                                    retest_substance.clone(),
                                                                    metric,
                                                                    anchor.timestamp,
                                                                ));
                                                            }
                                                        }
                                                    >
                                                        "Re-test"
                                                    </button>
                                                </td>
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
/// The review button is the only action a candidate offers — and it exists
/// only where a review is meaningful (testable, not yet tested).
#[component]
fn CandidateRow(
    candidate: Candidate,
    #[prop(into)] on_review: Callback<Candidate>,
) -> impl IntoView {
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
    let for_review = candidate.clone();
    let can_review = candidate.testable && !candidate.tested;

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
            // Static per row (rows rebuild when the queue signal changes),
            // so a plain Option — no reactive closure owning the candidate.
            {can_review.then(|| {
                let target = for_review.clone();
                view! {
                    <button
                        class="candidate-review"
                        on:click=move |_| on_review.run(target.clone())
                    >
                        "Review 14d vs 14d"
                    </button>
                }
            })}
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
    let gaps = if insight.gaps.is_empty() {
        "none".to_string()
    } else {
        insight
            .gaps
            .iter()
            .map(|g| format!("{} → {} ({} days)", g.start_day, g.end_day, g.days))
            .collect::<Vec<_>>()
            .join("; ")
    };
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

/// The review the engine produced for one candidate, shown before it can be
/// recorded: verdict, effect, both windows, n per side, confounders, gaps,
/// and the limitations sentence — the full disclosure contract of the drift
/// card applied to a review (FR-091/094/095). Recording is offered only when
/// the floors were met; `InsufficientData` explains what is missing instead
/// (FR-092), because writing a status from thin data would fake a test.
#[component]
fn ReviewPanel(review: OpenReview, close: RwSignal<Option<OpenReview>>) -> impl IntoView {
    let ctx = expect_context::<AppContext>();
    let pair = format!("{} × {}", review.substance, review.metric.name());
    let r = review.result.clone();
    let verdict = match r.verdict {
        Verdict::ChangeObserved => "a change beyond the noise band",
        Verdict::NoChangeObserved => "no change beyond the noise band",
        Verdict::Inconclusive => "an effect straddling the threshold",
        Verdict::InsufficientData => "not enough data to test",
    };
    let windows = format!(
        "pre {} → {} · post {} → {}",
        r.pre_window.0.date_naive(),
        r.pre_window.1.date_naive(),
        r.post_window.0.date_naive(),
        r.post_window.1.date_naive(),
    );
    let counts = format!("{} before · {} after", r.n_pre, r.n_post);
    let effect = r
        .effect_size
        .map(|e| format!("{:+.3}", e))
        .unwrap_or_else(|| "—".to_string());
    let shift = r
        .median_shift
        .map(|m| format!("{:+.1}", m))
        .unwrap_or_else(|| "—".to_string());
    let density = format!(
        "{} of {} pre days · {} of {} post days",
        r.n_pre,
        (r.pre_window.1 - r.pre_window.0).num_days(),
        r.n_post,
        (r.post_window.1 - r.post_window.0).num_days(),
    );
    let definition = r.metric.definition().to_string();
    // Names and dates, never bare counts: "3 confounders" hides which
    // three, and a review that hides them is a lie (FR-093, FR-101).
    let confounders = if r.confounders.is_empty() {
        "none found".to_string()
    } else {
        r.confounders
            .iter()
            .map(|c| match c.kind {
                ConfounderKind::StartedInWindow => {
                    format!(
                        "{} (new — first logged {})",
                        c.name,
                        c.first_seen.date_naive()
                    )
                }
                ConfounderKind::LogVolumeChanged => {
                    format!(
                        "{} (log volume changed — seen since {})",
                        c.name,
                        c.first_seen.date_naive()
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("; ")
    };
    let gaps = if r.gaps.is_empty() {
        "none found".to_string()
    } else {
        r.gaps
            .iter()
            .map(|g| {
                let side = match g.window {
                    Side::Pre => "pre",
                    Side::Post => "post",
                };
                format!(
                    "{} window {} → {} ({} days)",
                    side, g.start_day, g.end_day, g.days
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    };
    let missing = if r.missing.is_empty() {
        None
    } else {
        Some(r.missing.join("; "))
    };
    let limitations = r.limitations.clone();
    let recordable = r.verdict != Verdict::InsufficientData;

    // Descriptive label first ("a change beyond the noise band"), ledger
    // status second — the ledger word is an administrative state, the
    // sentence is what was actually observed (FR-095).
    let on_record = Callback::new(move |_: MouseEvent| {
        let Some(open) = close.get() else { return };
        let status = match open.result.verdict {
            Verdict::ChangeObserved => LedgerStatus::Supported,
            Verdict::NoChangeObserved => LedgerStatus::Unsupported,
            Verdict::Inconclusive => LedgerStatus::Inconclusive,
            Verdict::InsufficientData => return,
        };
        let evidence = format!(
            "review {}→{} vs {}→{}, n={}/{}",
            open.result.pre_window.0.date_naive(),
            open.result.pre_window.1.date_naive(),
            open.result.post_window.0.date_naive(),
            open.result.post_window.1.date_naive(),
            open.result.n_pre,
            open.result.n_post,
        );
        let updated = record_outcome(
            &get_insight_ledger(),
            &open.substance,
            open.metric,
            status,
            &evidence,
            None,
            chrono::Utc::now(),
        );
        if set_insight_ledger(&updated).is_ok() {
            close.set(None);
            ctx.data_version.update(|v| *v += 1);
        }
    });

    view! {
        <article class="review-panel" aria-label=format!("Review: {}", pair.clone())>
            <header class="insight-card-header">
                <h3>{pair.clone()}</h3>
                <span class="review-verdict">{verdict.to_string()}</span>
            </header>
            <dl class="insight-meta">
                <dt>"Windows"</dt>
                <dd>{windows.clone()}</dd>
                <dt>"Points"</dt>
                <dd>{counts.clone()}</dd>
                <dt>"Density"</dt>
                <dd>{density.clone()}</dd>
                <dt>"Definition"</dt>
                <dd>{definition.clone()}</dd>
                <dt>"Effect size"</dt>
                <dd>{effect.clone()}</dd>
                <dt>"Median shift"</dt>
                <dd>{shift.clone()}</dd>
                <dt>"Confounders"</dt>
                <dd>{confounders.clone()}</dd>
                <dt>"Gaps ≥2 days"</dt>
                <dd>{gaps.clone()}</dd>
            </dl>
            {missing.map(|m| view! { <p class="review-missing">{format!("Cannot test yet: {}.", m)}</p> })}
            <p class="insight-limitations">{limitations.clone()}</p>
            <div class="review-actions">
                <button type="button" class="cancel-btn" on:click=move |_| close.set(None)>"Close"</button>
                {recordable.then(|| {
                    view! {
                        <button type="button" class="save-button" on:click=move |ev| on_record.run(ev)>"Record in ledger"</button>
                    }
                })}
            </div>
        </article>
    }
}
