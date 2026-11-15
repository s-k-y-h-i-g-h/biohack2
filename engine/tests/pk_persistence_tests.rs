//! SQLite persistence for spec 003 Phase 1 (T206/T207): the
//! `substance_models` table plus numeric half-life columns on
//! `catalog_items`, their seeding, idempotent migration, and read paths.

#![cfg(feature = "db")]

use engine::db;
use engine::db::DbPool;
use engine::models::ModelClass;
use engine::pharmacokinetics::half_life_fields;

#[sqlx::test]
async fn seed_writes_numeric_half_lives(pool: DbPool) -> anyhow::Result<()> {
    db::migrate(&pool).await?;
    let seed = engine::catalog::seed_catalog();
    db::seed_catalog(&pool, &seed).await?;
    let items = db::get_catalog_items(&pool).await?;
    assert_eq!(items.len(), seed.len());
    // Stored numerics must equal the canonical derivation of the seed
    // string — one parser, so a round trip can never drift.
    for item in &items {
        let derived = half_life_fields(item.half_life.as_deref());
        assert_eq!(item.half_life_hours, derived.0, "{} hours", item.name);
        assert_eq!(item.half_life_range, derived.1, "{} range", item.name);
    }
    let caffeine = items
        .iter()
        .find(|i| i.name == "Caffeine")
        .expect("caffeine seeded");
    assert!(caffeine.half_life_hours.unwrap() > 0.0);
    Ok(())
}

#[sqlx::test]
async fn migrate_is_idempotent_and_reader_falls_back_for_legacy_rows(
    pool: DbPool,
) -> anyhow::Result<()> {
    db::migrate(&pool).await?; // setup
    // Second migrate must not fail even though the T207 columns exist.
    db::migrate(&pool).await?;
    // Pre-T207 row: half-life string only, numeric columns NULL — the
    // reader must fall back to the canonical derivation.
    sqlx::query(
        "INSERT INTO catalog_items (id, name, category, dosage_range, half_life, contraindications, warnings, mechanism_tags, is_custom, source, version)
         VALUES ('x1', 'TestVital', 'supplement', 'null', '36h', '[]', '[]', '[]', 0, NULL, 1)",
    )
    .execute(&pool)
    .await?;
    let items = db::get_catalog_items(&pool).await?;
    let row = items.iter().find(|i| i.name == "TestVital").unwrap();
    assert_eq!(row.half_life_hours, Some(36.0));
    Ok(())
}

#[sqlx::test]
async fn substance_models_round_trip_and_are_idempotent(pool: DbPool) -> anyhow::Result<()> {
    db::migrate(&pool).await?;
    let seed = engine::catalog::seed_catalog();
    db::seed_catalog(&pool, &seed).await?;
    let n = db::ensure_substance_models(&pool).await?;
    assert_eq!(n, seed.len(), "every catalog substance gets a model row");
    // Idempotent: re-running must neither duplicate nor drop rows.
    db::ensure_substance_models(&pool).await?;
    let models = db::get_substance_models(&pool).await?;
    assert_eq!(models.len(), n);
    for m in &models {
        assert!(
            !m.provenance.trim().is_empty(),
            "{} provenance",
            m.substance
        );
        if m.model_class != ModelClass::Decay {
            assert!(
                !m.rationale.as_deref().unwrap_or("").trim().is_empty(),
                "{} needs a rationale",
                m.substance
            );
        }
    }
    let find = |name: &str| models.iter().find(|m| m.substance == name).unwrap();
    assert_eq!(find("Caffeine").model_class, ModelClass::Decay);
    assert_eq!(find("Alcohol").model_class, ModelClass::None);
    // Every assignment class is exercised by the seed catalog.
    let classes: Vec<ModelClass> = models.iter().map(|m| m.model_class).collect();
    for c in [
        ModelClass::Decay,
        ModelClass::Plateau,
        ModelClass::Accumulation,
        ModelClass::None,
    ] {
        assert!(classes.contains(&c), "seed catalog misses class {c:?}");
    }
    // The search path reads the same numeric columns.
    let hits = db::search_catalog(&pool, "Caffeine").await?;
    assert!(!hits.is_empty());
    assert!(hits[0].half_life_hours.is_some());
    Ok(())
}
