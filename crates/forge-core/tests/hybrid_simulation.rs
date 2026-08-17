use forge_core::{
    pareto_frontier, simulate_hybrid_topologies, strongest_candidate, HybridSimulationConfig,
    HybridSimulationSummary, HybridTopology, SimulationWeights,
};

fn summary(
    topology: HybridTopology,
    success_bps: u16,
    cost_milliunits: u32,
    latency_ms: u32,
    security_violations: u32,
    complexity: u8,
) -> HybridSimulationSummary {
    HybridSimulationSummary::synthetic(
        topology,
        success_bps,
        cost_milliunits,
        latency_ms,
        security_violations,
        complexity,
    )
}

#[test]
fn paired_seed_simulation_is_reproducible() {
    let config = HybridSimulationConfig::default().with_seeds(30);

    let first = simulate_hybrid_topologies(&config);
    let second = simulate_hybrid_topologies(&config);

    assert_eq!(first, second);
    assert_eq!(first.len(), HybridTopology::ALL.len());
    assert!(first.iter().all(|result| result.trace_count == 30));
}

#[test]
fn pareto_frontier_excludes_strictly_dominated_topologies() {
    let safe_fast = summary(HybridTopology::RustKmp, 8000, 1000, 100, 0, 2);
    let dominated = summary(HybridTopology::RustGoGateway, 7900, 1200, 120, 0, 4);
    let higher_success = summary(HybridTopology::RustBoundedGoWorker, 8300, 1400, 130, 0, 4);

    let frontier = pareto_frontier(&[safe_fast.clone(), dominated, higher_success.clone()]);

    assert_eq!(frontier, vec![higher_success, safe_fast]);
}

#[test]
fn security_violations_are_a_hard_disqualification() {
    let unsafe_high_score = summary(HybridTopology::RustGoGateway, 9900, 500, 50, 1, 1);
    let safe_candidate = summary(HybridTopology::RustKmp, 7800, 1000, 100, 0, 2);

    let selected = strongest_candidate(
        &[unsafe_high_score, safe_candidate.clone()],
        &SimulationWeights::default(),
    )
    .expect("a safe candidate exists");

    assert_eq!(selected, safe_candidate);
}

#[test]
fn default_model_runs_every_locked_hybrid_candidate() {
    let config = HybridSimulationConfig::default().with_seeds(30);
    let results = simulate_hybrid_topologies(&config);
    let topologies: Vec<HybridTopology> = results.iter().map(|result| result.topology).collect();

    assert_eq!(topologies, HybridTopology::ALL.to_vec());
    assert!(results.iter().all(|result| result.security_violations == 0));
    assert!(strongest_candidate(&results, &SimulationWeights::default()).is_some());
}

#[test]
fn default_30_seed_model_selects_rust_kmp() {
    let results = simulate_hybrid_topologies(&HybridSimulationConfig::default().with_seeds(30));
    let selected = strongest_candidate(&results, &SimulationWeights::default())
        .expect("default model must leave an eligible candidate");

    assert_eq!(selected.topology, HybridTopology::RustKmp);
    assert_eq!(selected.success_bps, 8188);
    assert_eq!(selected.cost_milliunits, 1017);
    assert_eq!(selected.latency_ms, 104);
    assert_eq!(selected.security_violations, 0);
    assert_eq!(selected.complexity, 2);
}
