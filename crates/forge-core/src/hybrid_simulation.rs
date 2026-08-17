//! Deterministic, transparent simulation for AutoDev polyglot topology choices.
//!
//! This is an experiment model, not production performance evidence. Parameters
//! are explicit and traces are paired by seed so architecture candidates can be
//! compared reproducibly before adding another runtime to production.

use serde::{Deserialize, Serialize};

/// Candidate topologies selected by the locked polyglot reconciliation design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridTopology {
    /// Rust authority/server with Kotlin Multiplatform and Android clients.
    RustKmp,
    /// Rust authority with a Go stateless MCP/network gateway.
    RustGoGateway,
    /// Kotlin edge/control-plane adapter with Rust trusted authority.
    KotlinEdgeRust,
    /// Rust authority with a bounded Go networking/worker specialization.
    RustBoundedGoWorker,
    /// Rust authority with a future Flutter presentation client.
    RustFutureFlutterClient,
}

impl HybridTopology {
    pub const ALL: [HybridTopology; 5] = [
        HybridTopology::RustKmp,
        HybridTopology::RustGoGateway,
        HybridTopology::KotlinEdgeRust,
        HybridTopology::RustBoundedGoWorker,
        HybridTopology::RustFutureFlutterClient,
    ];
}

/// One paired-seed observation from the offline model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridSimulationTrace {
    pub topology: HybridTopology,
    pub seed: u32,
    pub success_bps: u16,
    pub cost_milliunits: u32,
    pub latency_ms: u32,
    pub security_violations: u32,
    pub complexity: u8,
}

/// Aggregate metrics for one topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridSimulationSummary {
    pub topology: HybridTopology,
    pub success_bps: u16,
    pub cost_milliunits: u32,
    pub latency_ms: u32,
    pub security_violations: u32,
    pub complexity: u8,
    pub trace_count: u32,
}

impl HybridSimulationSummary {
    /// Construct an explicit fixture for architecture-policy tests.
    pub fn synthetic(
        topology: HybridTopology,
        success_bps: u16,
        cost_milliunits: u32,
        latency_ms: u32,
        security_violations: u32,
        complexity: u8,
    ) -> Self {
        Self {
            topology,
            success_bps,
            cost_milliunits,
            latency_ms,
            security_violations,
            complexity,
            trace_count: 1,
        }
    }
}

/// Reproducible simulator configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HybridSimulationConfig {
    pub seeds: u32,
}

impl Default for HybridSimulationConfig {
    fn default() -> Self {
        Self { seeds: 30 }
    }
}

impl HybridSimulationConfig {
    pub fn with_seeds(mut self, seeds: u32) -> Self {
        self.seeds = seeds.max(1);
        self
    }
}

/// Secondary utility weights used only after hard eligibility and Pareto gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationWeights {
    pub success_weight: i64,
    pub cost_penalty: i64,
    pub latency_penalty: i64,
    pub complexity_penalty: i64,
}

impl Default for SimulationWeights {
    fn default() -> Self {
        Self {
            success_weight: 100,
            cost_penalty: 100,
            latency_penalty: 10,
            complexity_penalty: 1_000,
        }
    }
}

/// Run all locked topology candidates using the same seed schedule.
pub fn simulate_hybrid_topologies(
    config: &HybridSimulationConfig,
) -> Vec<HybridSimulationSummary> {
    HybridTopology::ALL
        .iter()
        .copied()
        .map(|topology| summarize(&simulate_hybrid_traces(topology, config)))
        .collect()
}

/// Export the paired traces for one topology so simulator assumptions can be
/// inspected rather than hidden behind an aggregate score.
pub fn simulate_hybrid_traces(
    topology: HybridTopology,
    config: &HybridSimulationConfig,
) -> Vec<HybridSimulationTrace> {
    let base = parameters(topology);
    (0..config.seeds.max(1))
        .map(|seed| {
            let success_noise = signed_noise(seed, topology, 101);
            let cost_noise = unsigned_noise(seed, topology, 37);
            let latency_noise = unsigned_noise(seed, topology, 11);
            HybridSimulationTrace {
                topology,
                seed,
                success_bps: clamp_bps(base.success_bps as i32 + success_noise),
                cost_milliunits: base.cost_milliunits + cost_noise,
                latency_ms: base.latency_ms + latency_noise,
                security_violations: 0,
                complexity: base.complexity,
            }
        })
        .collect()
}

/// Return non-dominated eligible summaries ordered by success descending and
/// then cost ascending for deterministic presentation.
pub fn pareto_frontier(
    summaries: &[HybridSimulationSummary],
) -> Vec<HybridSimulationSummary> {
    let mut frontier: Vec<HybridSimulationSummary> = summaries
        .iter()
        .filter(|candidate| candidate.security_violations == 0)
        .filter(|candidate| {
            !summaries.iter().any(|other| {
                other.topology != candidate.topology
                    && other.security_violations == 0
                    && dominates(other, candidate)
            })
        })
        .cloned()
        .collect();

    frontier.sort_by(|left, right| {
        right
            .success_bps
            .cmp(&left.success_bps)
            .then_with(|| left.cost_milliunits.cmp(&right.cost_milliunits))
            .then_with(|| left.latency_ms.cmp(&right.latency_ms))
            .then_with(|| left.complexity.cmp(&right.complexity))
            .then_with(|| left.topology.cmp(&right.topology))
    });
    frontier
}

/// Select a candidate only after security and Pareto gates.
///
/// The weighted utility is intentionally secondary; no unsafe or strictly
/// dominated topology can win by manipulating weights.
pub fn strongest_candidate(
    summaries: &[HybridSimulationSummary],
    weights: &SimulationWeights,
) -> Option<HybridSimulationSummary> {
    pareto_frontier(summaries).into_iter().max_by(|left, right| {
        utility(left, weights)
            .cmp(&utility(right, weights))
            .then_with(|| left.success_bps.cmp(&right.success_bps))
            .then_with(|| right.cost_milliunits.cmp(&left.cost_milliunits))
            .then_with(|| right.topology.cmp(&left.topology))
    })
}

#[derive(Debug, Clone, Copy)]
struct TopologyParameters {
    success_bps: u16,
    cost_milliunits: u32,
    latency_ms: u32,
    complexity: u8,
}

fn parameters(topology: HybridTopology) -> TopologyParameters {
    match topology {
        HybridTopology::RustKmp => TopologyParameters {
            success_bps: 8_200,
            cost_milliunits: 1_000,
            latency_ms: 100,
            complexity: 2,
        },
        HybridTopology::RustGoGateway => TopologyParameters {
            success_bps: 8_250,
            cost_milliunits: 1_250,
            latency_ms: 115,
            complexity: 4,
        },
        HybridTopology::KotlinEdgeRust => TopologyParameters {
            success_bps: 8_000,
            cost_milliunits: 1_150,
            latency_ms: 110,
            complexity: 4,
        },
        HybridTopology::RustBoundedGoWorker => TopologyParameters {
            success_bps: 8_400,
            cost_milliunits: 1_350,
            latency_ms: 120,
            complexity: 4,
        },
        HybridTopology::RustFutureFlutterClient => TopologyParameters {
            success_bps: 8_100,
            cost_milliunits: 1_300,
            latency_ms: 125,
            complexity: 5,
        },
    }
}

fn summarize(traces: &[HybridSimulationTrace]) -> HybridSimulationSummary {
    let count = traces.len().max(1) as u64;
    let first = traces
        .first()
        .expect("simulation always emits at least one trace");
    HybridSimulationSummary {
        topology: first.topology,
        success_bps: (traces.iter().map(|trace| trace.success_bps as u64).sum::<u64>() / count)
            as u16,
        cost_milliunits: (traces
            .iter()
            .map(|trace| trace.cost_milliunits as u64)
            .sum::<u64>()
            / count) as u32,
        latency_ms: (traces
            .iter()
            .map(|trace| trace.latency_ms as u64)
            .sum::<u64>()
            / count) as u32,
        security_violations: traces
            .iter()
            .map(|trace| trace.security_violations)
            .sum(),
        complexity: first.complexity,
        trace_count: traces.len() as u32,
    }
}

fn dominates(left: &HybridSimulationSummary, right: &HybridSimulationSummary) -> bool {
    let no_worse = left.success_bps >= right.success_bps
        && left.cost_milliunits <= right.cost_milliunits
        && left.latency_ms <= right.latency_ms
        && left.security_violations <= right.security_violations
        && left.complexity <= right.complexity;
    let strictly_better = left.success_bps > right.success_bps
        || left.cost_milliunits < right.cost_milliunits
        || left.latency_ms < right.latency_ms
        || left.security_violations < right.security_violations
        || left.complexity < right.complexity;
    no_worse && strictly_better
}

fn utility(summary: &HybridSimulationSummary, weights: &SimulationWeights) -> i64 {
    summary.success_bps as i64 * weights.success_weight
        - summary.cost_milliunits as i64 * weights.cost_penalty
        - summary.latency_ms as i64 * weights.latency_penalty
        - summary.complexity as i64 * weights.complexity_penalty
}

fn clamp_bps(value: i32) -> u16 {
    value.clamp(0, 10_000) as u16
}

fn signed_noise(seed: u32, topology: HybridTopology, amplitude: i32) -> i32 {
    let span = amplitude * 2 + 1;
    (mixed(seed, topology) % span as u64) as i32 - amplitude
}

fn unsigned_noise(seed: u32, topology: HybridTopology, amplitude: u32) -> u32 {
    (mixed(seed.wrapping_add(amplitude), topology) % (amplitude as u64 + 1)) as u32
}

fn mixed(seed: u32, topology: HybridTopology) -> u64 {
    let topology_tag = match topology {
        HybridTopology::RustKmp => 0x9e37_79b9_u64,
        HybridTopology::RustGoGateway => 0x85eb_ca6b_u64,
        HybridTopology::KotlinEdgeRust => 0xc2b2_ae35_u64,
        HybridTopology::RustBoundedGoWorker => 0x27d4_eb2f_u64,
        HybridTopology::RustFutureFlutterClient => 0x1656_67b1_u64,
    };
    let mut value = seed as u64 ^ topology_tag;
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}
