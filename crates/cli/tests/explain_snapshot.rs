use vz_adapter_vinglish::VinglishAdapter;

#[test]
fn explanation_matches_the_v1_snapshot() {
    let graph = VinglishAdapter
        .import_json(include_str!("../../../tests/fixtures/accumulate-v1.json"))
        .unwrap();
    let explanation = vz_semantic_engine::explanation::explain(&graph);

    assert_eq!(
        explanation.trim_end(),
        include_str!("../../../tests/fixtures/accumulate-v1.explanation.txt").trim_end()
    );
}
