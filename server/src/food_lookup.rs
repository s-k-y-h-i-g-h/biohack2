//! Food database lookup proxy (spec 007) — port of biohack1 `food_db.rs`.
//!
//! Lives in the SERVER crate on purpose:
//! - the USDA API key must never ship inside the WASM bundle,
//! - OpenFoodFacts/USDA responses are normalized here into
//!   `engine::food::FoodSearchResult` so the web client and the engine
//!   aggregation share one shape,
//! - a process-local TTL cache absorbs repeat searches.
//!
//! Priority (unchanged from biohack1): OpenFoodFacts first (best for UK
//! branded items, keyless), USDA FoodData Central second (needs
//! `USDA_API_KEY`; without it USDA lookups are silently skipped).
//!
//! KNOWN LIMITATION (inherited, not hidden): USDA `foods/search` nutrient
//! amounts are documented as per-100g for Foundation foods but individual
//! records vary; OFF's `*_100g` fields are exact. Treat USDA per-100g data
//! as approximate. Gram-quantities only — no cup/spoon conversion (biohack1's
//! conversion table was a guess-list; dropped deliberately).

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use engine::food::{FoodSearchResult, FoodSource, NutrientInfo};
use engine::nutrient_ref::build_reference_map;
use tokio::sync::RwLock;

const OFF_API_BASE: &str = "https://world.openfoodfacts.org";
const FDC_API_BASE: &str = "https://api.nal.usda.gov/fdc/v1";
const MAX_SEARCH_RESULTS: usize = 10;
const CACHE_TTL: Duration = Duration::from_secs(300);

/// OpenFoodFacts nutriments field → USDA nutrient id + display unit.
/// Ported verbatim from biohack1 `parse_off_nutrients` nutrient_map.
const OFF_FIELD_MAP: &[(&str, i64, &str)] = &[
    ("energy-kcal_100g", 1008, "kcal"),
    ("proteins_100g", 1003, "g"),
    ("fat_100g", 1004, "g"),
    ("carbohydrates_100g", 1005, "g"),
    ("fiber_100g", 1079, "g"),
    ("sugars_100g", 2000, "g"),
    ("calcium_100g", 1087, "mg"),
    ("iron_100g", 1089, "mg"),
    ("magnesium_100g", 1090, "mg"),
    ("phosphorus_100g", 1091, "mg"),
    ("potassium_100g", 1092, "mg"),
    ("sodium_100g", 1093, "mg"),
    ("zinc_100g", 1095, "mg"),
    ("copper_100g", 1098, "mg"),
    ("manganese_100g", 1101, "mg"),
    ("selenium_100g", 1103, "µg"),
    ("vitamin-c_100g", 1162, "mg"),
    ("vitamin-b1_100g", 1165, "mg"),
    ("vitamin-b2_100g", 1166, "mg"),
    ("vitamin-pp_100g", 1167, "mg"),
    ("vitamin-b6_100g", 1175, "mg"),
    ("vitamin-b9_100g", 1177, "µg"),
    ("vitamin-b12_100g", 1178, "µg"),
    ("vitamin-a_100g", 1106, "µg"),
    ("vitamin-e_100g", 1109, "mg"),
    ("vitamin-d_100g", 1114, "µg"),
    ("vitamin-k_100g", 1185, "µg"),
    ("cholesterol_100g", 1253, "mg"),
    ("saturated-fat_100g", 1258, "g"),
    ("monounsaturated-fat_100g", 1292, "g"),
    ("polyunsaturated-fat_100g", 1293, "g"),
    ("omega-3-fat_100g", 1402, "g"), // DHA
];

/// Display name for a USDA nutrient id — canonical reference-table name when
/// known, raw upstream name otherwise. Shared by both sources so day-total
/// matching (by name) converges regardless of where a food came from. This
/// fixes a real biohack1 bug where USDA raw names ("Total lipid (fat)")
/// silently missed the reference table ("Total Fat").
pub fn canonical_name(usda_id: i64, fallback: &str) -> String {
    static NAMES: OnceLock<HashMap<i64, &'static str>> = OnceLock::new();
    NAMES
        .get_or_init(|| {
            build_reference_map()
                .into_iter()
                .map(|(id, r)| (id, r.name))
                .collect()
        })
        .get(&usda_id)
        .map(|s| s.to_string())
        .unwrap_or_else(|| fallback.to_string())
}

/// Parse an OpenFoodFacts `nutriments` object into per-100g nutrients.
/// Zero/negative and non-numeric entries are dropped (never stored as 0 —
/// "not reported" ≠ "zero" and aggregation must not learn to distrust data).
pub fn off_nutriments_to_infos(nutriments: Option<&serde_json::Value>) -> Vec<NutrientInfo> {
    let Some(obj) = nutriments.and_then(|v| v.as_object()) else {
        return Vec::new();
    };
    OFF_FIELD_MAP
        .iter()
        .filter_map(|(field, id, unit)| {
            let amount = obj.get(*field)?.as_f64()?;
            if amount > 0.0 && amount.is_finite() {
                Some(NutrientInfo {
                    name: canonical_name(*id, field),
                    amount,
                    unit: (*unit).to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

/// Map a USDA foodNutrients entry `{nutrientId, nutrientName, unitName, amount}`.
pub fn usda_nutrients_to_infos(food_nutrients: Option<&serde_json::Value>) -> Vec<NutrientInfo> {
    let Some(arr) = food_nutrients.and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|n| {
            let id = n.get("nutrientId")?.as_i64()?;
            let raw_name = n
                .get("nutrientName")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let unit = n
                .get("unitName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let amount = n
                .get("amount")
                .or_else(|| n.get("value"))
                .and_then(|v| v.as_f64())?;
            if amount > 0.0 && amount.is_finite() {
                Some(NutrientInfo {
                    name: canonical_name(id, raw_name),
                    amount,
                    unit,
                })
            } else {
                None
            }
        })
        .collect()
}

struct CachedSearch {
    at: Instant,
    results: Vec<FoodSearchResult>,
}

/// Shared food-DB client held in server state.
#[derive(Clone)]
pub struct FoodLookup {
    http: reqwest::Client,
    usda_api_key: Option<String>,
    cache: std::sync::Arc<RwLock<HashMap<String, CachedSearch>>>,
}

impl FoodLookup {
    /// `USDA_API_KEY` env var gates the USDA source; OFF works keyless.
    pub fn from_env() -> Self {
        let http = reqwest::Client::builder()
            .user_agent(concat!(
                "biohack2/",
                env!("CARGO_PKG_VERSION"),
                " (tracking app)"
            ))
            .timeout(Duration::from_secs(15))
            .build()
            .expect("reqwest client");
        Self {
            http,
            usda_api_key: std::env::var("USDA_API_KEY").ok().filter(|k| !k.is_empty()),
            cache: Default::default(),
        }
    }

    pub fn has_usda_key(&self) -> bool {
        self.usda_api_key.is_some()
    }

    /// Search OFF (always) + USDA (if keyed), OFF results first.
    /// One source failing must not fail the search — returns whatever the
    /// other source gave; Err only if BOTH failed.
    pub async fn search(&self, query: &str) -> Result<Vec<FoodSearchResult>> {
        let key = query.trim().to_lowercase();
        if key.is_empty() {
            return Err(anyhow!("empty query"));
        }
        if let Some(hit) = self.cache.read().await.get(&key) {
            if hit.at.elapsed() < CACHE_TTL {
                return Ok(hit.results.clone());
            }
        }

        let off = self.search_off(query);
        let usda = match &self.usda_api_key {
            Some(k) => futures::future::Either::Left(self.search_usda(query, k)),
            None => futures::future::Either::Right(futures::future::ready(Ok(Vec::new()))),
        };
        let (off_res, usda_res) = tokio::join!(off, usda);

        let mut results = Vec::new();
        let mut errors = Vec::new();
        match off_res {
            Ok(v) => results.extend(v),
            Err(e) => errors.push(format!("OpenFoodFacts: {e:#}")),
        }
        match usda_res {
            Ok(v) => results.extend(v),
            Err(e) => errors.push(format!("USDA: {e:#}")),
        }
        if results.is_empty() && !errors.is_empty() {
            return Err(anyhow!(errors.join("; ")));
        }

        // Opportunistic GC of expired entries while we hold the write lock.
        let mut cache = self.cache.write().await;
        cache.retain(|_, c| c.at.elapsed() < CACHE_TTL);
        cache.insert(
            key,
            CachedSearch {
                at: Instant::now(),
                results: results.clone(),
            },
        );
        Ok(results)
    }

    async fn search_off(&self, query: &str) -> Result<Vec<FoodSearchResult>> {
        // OFF's search.pl 503s intermittently (~50% under light load; verified
        // 2026-11-18 with plain curl, client-independent). Bounded retry with
        // small backoff is the difference between a working and a coin-flip
        // food search.
        const OFF_ATTEMPTS: usize = 3;
        let mut last_err = anyhow!("OpenFoodFacts: not attempted");
        for attempt in 0..OFF_ATTEMPTS {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(400 * attempt as u64)).await;
            }
            match self
                .http
                .get(format!("{OFF_API_BASE}/cgi/search.pl",))
                .query(&[
                    ("search_terms", query),
                    ("search_simple", "1"),
                    ("action", "process"),
                    ("json", "1"),
                    ("page_size", &MAX_SEARCH_RESULTS.to_string()),
                ])
                .send()
                .await
            {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        last_err = anyhow!("OpenFoodFacts HTTP {}", resp.status());
                        continue;
                    }
                    let body: serde_json::Value = resp.json().await.context("OFF parse")?;
                    return Ok(parse_off_search(&body));
                }
                Err(e) => last_err = anyhow!("OpenFoodFacts request failed: {e}"),
            }
        }
        Err(last_err)
    }

    async fn search_usda(&self, query: &str, api_key: &str) -> Result<Vec<FoodSearchResult>> {
        let resp = self
            .http
            .post(format!("{FDC_API_BASE}/foods/search"))
            .query(&[("api_key", api_key)])
            .json(&serde_json::json!({
                "query": query,
                "pageSize": MAX_SEARCH_RESULTS,
                "dataType": ["Foundation", "Survey (FNDDS)"],
                "sortBy": "dataType.keyword",
                "sortOrder": "asc"
            }))
            .send()
            .await
            .context("USDA request failed")?;
        if !resp.status().is_success() {
            return Err(anyhow!("USDA HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().await.context("USDA parse")?;
        Ok(parse_usda_search(&body))
    }

    /// Details by id: 8-14 digit numeric → OFF barcode, else USDA FDC id.
    pub async fn details(&self, id: &str) -> Result<FoodSearchResult> {
        let id = id.trim();
        let is_barcode = id.len() >= 8 && id.len() <= 14 && id.chars().all(|c| c.is_ascii_digit());
        if is_barcode {
            let resp = self
                .http
                .get(format!("{OFF_API_BASE}/api/v2/product/{id}"))
                .send()
                .await
                .context("OFF details request failed")?;
            if !resp.status().is_success() {
                return Err(anyhow!("OpenFoodFacts HTTP {}", resp.status()));
            }
            let body: serde_json::Value = resp.json().await.context("OFF parse")?;
            let product = body.get("product").unwrap_or(&body);
            let code = product
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or(id)
                .to_string();
            let description = product
                .get("product_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown product")
                .to_string();
            Ok(FoodSearchResult {
                id: code,
                description,
                brand: product
                    .get("brands")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                source: FoodSource::OpenFoodFacts,
                nutrients_per_100g: (!off_nutriments_to_infos(product.get("nutriments"))
                    .is_empty())
                .then(|| off_nutriments_to_infos(product.get("nutriments"))),
            })
        } else {
            let api_key = self
                .usda_api_key
                .as_deref()
                .ok_or_else(|| anyhow!("USDA_API_KEY not configured; USDA details unavailable"))?;
            let fdc_id: i64 = id.parse().context("USDA id must be numeric")?;
            let resp = self
                .http
                .get(format!("{FDC_API_BASE}/food/{fdc_id}"))
                .query(&[("api_key", api_key)])
                .send()
                .await
                .context("USDA details request failed")?;
            if !resp.status().is_success() {
                return Err(anyhow!("USDA HTTP {}", resp.status()));
            }
            let body: serde_json::Value = resp.json().await.context("USDA parse")?;
            let infos = usda_nutrients_to_infos(body.get("foodNutrients"));
            Ok(FoodSearchResult {
                id: fdc_id.to_string(),
                description: body
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown food")
                    .to_string(),
                brand: None,
                source: FoodSource::Usda,
                nutrients_per_100g: (!infos.is_empty()).then_some(infos),
            })
        }
    }
}

/// OFF search payload → results. Split out for fixture testing (no network).
pub fn parse_off_search(body: &serde_json::Value) -> Vec<FoodSearchResult> {
    body.get("products")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let name = p.get("product_name")?.as_str()?;
                    if name.trim().is_empty() {
                        return None;
                    }
                    let infos = off_nutriments_to_infos(p.get("nutriments"));
                    Some(FoodSearchResult {
                        id: p
                            .get("code")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        description: name.to_string(),
                        brand: p
                            .get("brands")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        source: FoodSource::OpenFoodFacts,
                        nutrients_per_100g: (!infos.is_empty()).then_some(infos),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// USDA search payload → results. Split out for fixture testing.
pub fn parse_usda_search(body: &serde_json::Value) -> Vec<FoodSearchResult> {
    body.get("foods")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    let description = f.get("description")?.as_str()?.to_string();
                    let infos = usda_nutrients_to_infos(f.get("foodNutrients"));
                    Some(FoodSearchResult {
                        id: f.get("fdcId").map(|v| v.to_string()).unwrap_or_default(),
                        description,
                        brand: None,
                        source: FoodSource::Usda,
                        nutrients_per_100g: (!infos.is_empty()).then_some(infos),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_fixture_maps_canonical_names() {
        let body = serde_json::json!({
            "products": [{
                "code": "5000159042668",
                "product_name": "Test Granola",
                "brands": "Testco",
                "nutriments": {
                    "energy-kcal_100g": 450.0,
                    "proteins_100g": 9.0,
                    "fat_100g": 18.0,
                    "carbohydrates_100g": 63.0,
                    "salt_100g": 0.5,          // unmapped field → dropped
                    "sugars_100g": 0.0,        // zero → dropped (not reported)
                    "fiber_100g": "nonsense",  // non-numeric → dropped
                    "vitamin-b9_100g": 40.0
                }
            }, {
                "code": "no_name",             // no product_name → skipped
                "nutriments": {"energy-kcal_100g": 1.0}
            }]
        });
        let rs = parse_off_search(&body);
        assert_eq!(rs.len(), 1);
        let r = &rs[0];
        assert_eq!(r.brand.as_deref(), Some("Testco"));
        let n = r.nutrients_per_100g.clone().unwrap();
        assert_eq!(n.len(), 5);
        let energy = n.iter().find(|x| x.name == "Energy").unwrap();
        assert_eq!(energy.unit, "kcal");
        assert!((energy.amount - 450.0).abs() < 1e-9);
        assert!(n.iter().any(|x| x.name == "Folate")); // 1177 canonical name
        assert!(!n.iter().any(|x| x.name.contains("salt")));
    }

    #[test]
    fn usda_fixture_normalizes_biohack1_name_bug() {
        let body = serde_json::json!({
            "foods": [{
                "fdcId": 170346,
                "description": "Turkey, whole",
                "foodNutrients": [
                    {"nutrientId": 1004, "nutrientName": "Total lipid (fat)", "unitName": "g", "amount": 5.0},
                    {"nutrientId": 1008, "nutrientName": "Energy", "unitName": "kcal", "value": 143.0},
                    {"nutrientId": 9999, "nutrientName": "Oddball X", "unitName": "zz", "amount": -1.0}
                ]
            }]
        });
        let rs = parse_usda_search(&body);
        assert_eq!(rs.len(), 1);
        let n = rs[0].nutrients_per_100g.clone().unwrap();
        // 1004 must surface as "Total Fat" (the fix), not "Total lipid (fat)".
        assert!(n.iter().any(|x| x.name == "Total Fat"));
        assert!(!n.iter().any(|x| x.name == "Total lipid (fat)"));
        // `value` alias honored; negative dropped; unknown id kept under raw name.
        assert!(
            n.iter()
                .any(|x| x.name == "Energy" && (x.amount - 143.0).abs() < 1e-9)
        );
        assert_eq!(n.len(), 2);
    }

    #[test]
    fn unknown_id_falls_back_to_raw_name() {
        assert_eq!(canonical_name(987654, "Weird Thing"), "Weird Thing");
        assert_eq!(canonical_name(1003, "Protein-ish"), "Protein");
    }

    #[test]
    fn empty_payloads_parse_to_empty_not_panic() {
        assert!(parse_off_search(&serde_json::json!({})).is_empty());
        assert!(parse_usda_search(&serde_json::json!({"foods": null})).is_empty());
        assert!(off_nutriments_to_infos(None).is_empty());
        assert!(usda_nutrients_to_infos(Some(&serde_json::json!("nope"))).is_empty());
    }
}
