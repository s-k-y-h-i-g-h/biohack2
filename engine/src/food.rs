//! Food nutrition tracking — shared types and pure aggregation (spec 007).
//!
//! Port of biohack1's diet feature to the v2 engine. This module is pure
//! computation (wasm-safe, no I/O): the HTTP clients for USDA FoodData
//! Central / OpenFoodFacts live in the server crate (`server/src/food_lookup.rs`)
//! because WASM builds must not embed API keys and the offline story requires
//! nutrients to be CACHED per entry at log time.
//!
//! ## Storage convention
//! A food log entry caches its consumed-quantity nutrients in
//! `LogEntry.custom_fields["nutrition"]` as JSON (`Vec<NutrientInfo>` —
//! same shape biohack1 persisted in `FoodLog.nutrients`, so old CLI data
//! imports unchanged). Riding `custom_fields` means BOTH persistence
//! backends (SQLite and LocalStorage) carry nutrition with zero schema
//! changes, and history/day-summaries keep working fully offline.
//!
//! Nutrients are stored for the CONSUMED quantity (already scaled from
//! per-100g at log time). Aggregation therefore never re-reads any DB.

use serde::{Deserialize, Serialize};

use crate::models::{ItemType, LogEntry};
use crate::nutrient_ref::{
    DailyNutrientStatus, NutrientStatus, calculate_nutrient_status, get_nutrient_references,
};

/// Which external food database a result came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FoodSource {
    /// OpenFoodFacts — best for UK branded products (barcode-keyed)
    OpenFoodFacts,
    /// USDA FoodData Central — best for generic ingredients (FDC-id-keyed)
    Usda,
}

impl FoodSource {
    pub fn label(&self) -> &'static str {
        match self {
            FoodSource::OpenFoodFacts => "OpenFoodFacts",
            FoodSource::Usda => "USDA",
        }
    }
}

/// One nutrient amount with its unit — the persisted atom (biohack1-compatible).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NutrientInfo {
    pub name: String,
    pub amount: f64,
    pub unit: String,
}

/// Unified search/lookup result shared between server proxy and web client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodSearchResult {
    /// Internal ID: FDC id for USDA, barcode for OpenFoodFacts
    pub id: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    pub source: FoodSource,
    /// Nutrients per 100 g when the source provides them
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nutrients_per_100g: Option<Vec<NutrientInfo>>,
}

/// Scale a per-100g nutrient list to a consumed quantity in grams.
///
/// Non-finite inputs and negative gram amounts yield an empty list — we
/// never invent or extrapolate nutrition data. Amounts that turn
/// non-finite after scaling are dropped.
pub fn scale_per_100g(per_100g: &[NutrientInfo], grams: f64) -> Vec<NutrientInfo> {
    if !grams.is_finite() || grams <= 0.0 {
        return Vec::new();
    }
    let factor = grams / 100.0;
    per_100g
        .iter()
        .filter_map(|n| {
            let amount = n.amount * factor;
            if amount.is_finite() {
                Some(NutrientInfo {
                    name: n.name.clone(),
                    amount,
                    unit: n.unit.clone(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Attach consumed-quantity nutrients to a log entry's `custom_fields`.
///
/// Preserves any other custom fields already present.
pub fn set_nutrition(entry: &mut LogEntry, nutrients: &[NutrientInfo]) {
    let mut obj = match entry.custom_fields.take() {
        Some(serde_json::Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };
    obj.insert(
        "nutrition".to_string(),
        serde_json::to_value(nutrients).unwrap_or(serde_json::Value::Null),
    );
    entry.custom_fields = Some(serde_json::Value::Object(obj));
}

/// Read cached nutrients from a food log entry.
///
/// Returns `None` for non-food entries, missing fields, or malformed JSON —
/// a corrupt payload must never poison a day total (tolerate, don't panic).
pub fn nutrition_of(entry: &LogEntry) -> Option<Vec<NutrientInfo>> {
    if entry.item_type != ItemType::Food {
        return None;
    }
    entry
        .custom_fields
        .as_ref()?
        .get("nutrition")
        .and_then(|v| serde_json::from_value::<Vec<NutrientInfo>>(v.clone()).ok())
}

/// Aggregated nutrition for one calendar day of food logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyNutrition {
    pub date: chrono::NaiveDate,
    /// Food entries that contributed (parsed nutrition present)
    pub entry_count: usize,
    /// Food entries seen for the day that carried NO parseable nutrition —
    /// surfaced so the UI can show "incomplete day" instead of a silently
    /// low total.
    pub entries_without_nutrition: usize,
    /// Sums keyed by (name, unit); sorted by name then unit for determinism.
    pub totals: Vec<NutrientInfo>,
}

impl DailyNutrition {
    /// Look up a total by nutrient name (first unit variant).
    pub fn total_for(&self, name: &str) -> Option<&NutrientInfo> {
        self.totals.iter().find(|n| n.name == name)
    }

    /// Compare totals against RDI/UL references. Nutrients without a
    /// reference entry stay in `totals` only (no invented status).
    pub fn status(&self) -> DailyNutrientStatus {
        let refs_ = get_nutrient_references();
        let nutrients: Vec<NutrientStatus> = self
            .totals
            .iter()
            .filter_map(|t| {
                refs_
                    .iter()
                    .find(|r| r.name == t.name && (r.unit == t.unit || t.unit.is_empty()))
                    .map(|r| calculate_nutrient_status(t.amount, r))
            })
            .collect();
        DailyNutrientStatus {
            date: self.date,
            nutrients,
        }
    }
}

/// Sum cached nutrients of all food entries logged on `date` (UTC).
pub fn aggregate_day(entries: &[LogEntry], date: chrono::NaiveDate) -> DailyNutrition {
    let mut sums: std::collections::BTreeMap<(String, String), f64> = Default::default();
    let mut entry_count = 0usize;
    let mut entries_without_nutrition = 0usize;

    for e in entries {
        if e.item_type != ItemType::Food || e.timestamp.date_naive() != date {
            continue;
        }
        match nutrition_of(e) {
            Some(nutrients) => {
                entry_count += 1;
                for n in nutrients {
                    if !n.amount.is_finite() {
                        continue;
                    }
                    *sums.entry((n.name, n.unit)).or_insert(0.0) += n.amount;
                }
            }
            None => entries_without_nutrition += 1,
        }
    }

    let totals = sums
        .into_iter()
        .map(|((name, unit), amount)| NutrientInfo { name, amount, unit })
        .collect();

    DailyNutrition {
        date,
        entry_count,
        entries_without_nutrition,
        totals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ItemType;
    use chrono::{TimeZone, Utc};

    fn food_entry(name: &str, ymd: &str, nutrition: Option<Vec<NutrientInfo>>) -> LogEntry {
        let mut e = LogEntry {
            id: uuid::Uuid::new_v4(),
            user_id: "u".into(),
            item_type: ItemType::Food,
            item_id: None,
            name: name.into(),
            quantity: None,
            unit: None,
            route: None,
            timestamp: Utc
                .with_ymd_and_hms(
                    ymd[..4].parse().unwrap(),
                    ymd[5..7].parse().unwrap(),
                    ymd[8..10].parse().unwrap(),
                    12,
                    0,
                    0,
                )
                .unwrap(),
            stack_id: None,
            notes: None,
            acknowledged_interaction: false,
            custom_fields: None,
        };
        if let Some(n) = nutrition {
            set_nutrition(&mut e, &n);
        }
        e
    }

    fn ni(name: &str, amount: f64, unit: &str) -> NutrientInfo {
        NutrientInfo {
            name: name.into(),
            amount,
            unit: unit.into(),
        }
    }

    fn date(s: &str) -> chrono::NaiveDate {
        chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn scale_is_linear_and_guards_garbage() {
        let per100 = vec![ni("Energy", 250.0, "kcal"), ni("Protein", 26.0, "g")];
        let s = scale_per_100g(&per100, 150.0);
        assert!((s[0].amount - 375.0).abs() < 1e-9);
        assert!((s[1].amount - 39.0).abs() < 1e-9);
        assert_eq!(scale_per_100g(&per100, 100.0).len(), 2);
        assert!(scale_per_100g(&per100, 0.0).is_empty());
        assert!(scale_per_100g(&per100, -5.0).is_empty());
        assert!(scale_per_100g(&per100, f64::NAN).is_empty());
    }

    #[test]
    fn set_then_read_roundtrips_and_preserves_other_fields() {
        let mut e = food_entry("Salmon", "2026-11-18", None);
        e.custom_fields = Some(serde_json::json!({"mood": 4}));
        set_nutrition(&mut e, &vec![ni("Energy", 208.0, "kcal")]);
        assert_eq!(
            e.custom_fields
                .as_ref()
                .unwrap()
                .get("mood")
                .unwrap()
                .as_i64(),
            Some(4)
        );
        let got = nutrition_of(&e).unwrap();
        assert_eq!(got.len(), 1);
        assert!((got[0].amount - 208.0).abs() < 1e-9);
    }

    #[test]
    fn malformed_or_missing_nutrition_is_none_not_panic() {
        let mut e = food_entry("Mystery", "2026-11-18", None);
        assert!(nutrition_of(&e).is_none());
        e.custom_fields = Some(serde_json::json!({"nutrition": "not-a-list"}));
        assert!(nutrition_of(&e).is_none());
        // Non-food entry with a nutrition payload is ignored by design.
        let mut s = food_entry(
            "Creatine",
            "2026-11-18",
            Some(vec![ni("Energy", 1.0, "kcal")]),
        );
        s.item_type = ItemType::Supplement;
        assert!(nutrition_of(&s).is_none());
    }

    #[test]
    fn aggregate_day_sums_names_and_isolates_days_and_types() {
        let entries = vec![
            food_entry(
                "Eggs",
                "2026-11-18",
                Some(vec![ni("Energy", 143.0, "kcal"), ni("Protein", 12.6, "g")]),
            ),
            food_entry(
                "Rice",
                "2026-11-18",
                Some(vec![
                    ni("Energy", 130.0, "kcal"),
                    ni("Carbohydrate", 28.0, "g"),
                ]),
            ),
            food_entry(
                "Snack",
                "2026-11-17",
                Some(vec![ni("Energy", 999.0, "kcal")]),
            ), // other day
            food_entry("NoData", "2026-11-18", None), // no nutrition
            food_entry("Whey", "2026-11-18", Some(vec![ni("Protein", 24.0, "g")])), // no time-of-day overlap issue
        ];
        let d = aggregate_day(&entries, date("2026-11-18"));
        assert_eq!(d.entry_count, 3);
        assert_eq!(d.entries_without_nutrition, 1);
        assert!((d.total_for("Energy").unwrap().amount - 273.0).abs() < 1e-9);
        assert!((d.total_for("Protein").unwrap().amount - 36.6).abs() < 1e-9);
        assert!(d.total_for("Carbohydrate").is_some());
        // Yesterday's snack must not leak.
        assert!(d.totals.iter().all(|t| (t.amount - 999.0).abs() > 1e-9));
        // Deterministic order.
        let names: Vec<&str> = d.totals.iter().map(|t| t.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn same_name_different_units_stay_separate() {
        let entries = vec![
            food_entry("A", "2026-11-18", Some(vec![ni("Sodium", 500.0, "mg")])),
            food_entry("B", "2026-11-18", Some(vec![ni("Sodium", 0.5, "g")])),
        ];
        let d = aggregate_day(&entries, date("2026-11-18"));
        assert_eq!(d.totals.iter().filter(|t| t.name == "Sodium").count(), 2);
    }

    #[test]
    fn day_status_maps_to_references_and_skips_unknowns() {
        let entries = vec![food_entry(
            "Big day",
            "2026-11-18",
            Some(vec![
                ni("Energy", 2500.0, "kcal"),
                ni("Mystery-X", 7.0, "zz"),
            ]),
        )];
        let st = aggregate_day(&entries, date("2026-11-18")).status();
        assert_eq!(
            st.nutrients.len(),
            1,
            "unknown nutrient must not gain a status"
        );
        assert_eq!(st.nutrients[0].name, "Energy");
        assert!(st.nutrients[0].percent_rdi > 100.0);
    }
}
