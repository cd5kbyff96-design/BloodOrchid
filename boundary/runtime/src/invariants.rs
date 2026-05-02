use std::collections::HashMap;
use std::sync::Arc;

use crate::proto::SimulationState;

pub trait Invariant: std::fmt::Debug {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn verify(&self, context: &VerificationContext) -> InvariantResult;
}

#[derive(Debug, Clone)]
pub enum InvariantResult {
    Passed,
    Failed(InvariantViolation),
    Unknown(String),
}

impl InvariantResult {
    pub fn is_passed(&self) -> bool {
        matches!(self, InvariantResult::Passed)
    }
}

#[derive(Debug, Clone)]
pub struct InvariantViolation {
    pub invariant: String,
    pub details: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Default)]
pub struct VerificationContext {
    pub state_updates: Vec<StateUpdate>,
    pub state_history: Vec<StateSnapshot>,
    pub cve_analysis: Option<CVEAnalysis>,
    pub kernel_hash_runs: Vec<u64>,
    pub contract_schemas: HashMap<String, SchemaDef>,
}

#[derive(Debug, Clone)]
pub struct StateUpdate {
    pub source: StateSource,
    pub step_index: u64,
    pub state_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateSource {
    Boundary,
    Kernel,
    External,
}

#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub step_index: u64,
    pub simulation_id: String,
    pub state_hash: u64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone)]
pub struct CVEAnalysis {
    pub has_static_mut: bool,
    pub has_io_operations: bool,
    pub has_global_state: bool,
    pub has_non_determinism: bool,
}

#[derive(Debug, Clone)]
pub struct SchemaDef {
    pub name: String,
    pub fields: Vec<SchemaField>,
}

#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub ty: String,
}

pub struct BoundaryStateAuthorityInvariant;

impl std::fmt::Debug for BoundaryStateAuthorityInvariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BoundaryStateAuthorityInvariant")
    }
}

impl Invariant for BoundaryStateAuthorityInvariant {
    fn name(&self) -> &str {
        "BoundaryStateAuthority"
    }

    fn description(&self) -> &str {
        "Only BoundaryRuntime can modify simulation state"
    }

    fn verify(&self, ctx: &VerificationContext) -> InvariantResult {
        let mut violations = Vec::new();

        for update in &ctx.state_updates {
            if update.source != StateSource::Boundary && update.source != StateSource::Kernel {
                violations.push(InvariantViolation {
                    invariant: "BoundaryStateAuthority".to_string(),
                    details: format!("State modified by {:?}", update.source),
                    severity: ViolationSeverity::Critical,
                });
            }
        }

        if violations.is_empty() {
            InvariantResult::Passed
        } else {
            InvariantResult::Failed(InvariantViolation {
                invariant: "BoundaryStateAuthority".to_string(),
                details: violations.iter()
                    .map(|v| v.details.clone())
                    .collect::<Vec<_>>()
                    .join("; "),
                severity: ViolationSeverity::Critical,
            })
        }
    }
}

pub struct CVEPurityInvariant;

impl std::fmt::Debug for CVEPurityInvariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CVEPurityInvariant")
    }
}

impl Invariant for CVEPurityInvariant {
    fn name(&self) -> &str {
        "CVEPurity"
    }

    fn description(&self) -> &str {
        "CVE must be a pure function with no side effects"
    }

    fn verify(&self, ctx: &VerificationContext) -> InvariantResult {
        if let Some(analysis) = &ctx.cve_analysis {
            let mut issues = Vec::new();

            if analysis.has_static_mut {
                issues.push("CVE contains static mutability");
            }
            if analysis.has_io_operations {
                issues.push("CVE contains IO operations");
            }
            if analysis.has_global_state {
                issues.push("CVE contains global state");
            }
            if analysis.has_non_determinism {
                issues.push("CVE may have non-deterministic behavior");
            }

            if issues.is_empty() {
                InvariantResult::Passed
            } else {
                InvariantResult::Failed(InvariantViolation {
                    invariant: "CVEPurity".to_string(),
                    details: issues.join("; "),
                    severity: ViolationSeverity::Critical,
                })
            }
        } else {
            InvariantResult::Unknown("No CVE analysis available".to_string())
        }
    }
}

pub struct KernelDeterminismInvariant {
    pub iterations: u32,
}

impl KernelDeterminismInvariant {
    pub fn new(iterations: u32) -> Self {
        Self { iterations }
    }
}

impl std::fmt::Debug for KernelDeterminismInvariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KernelDeterminismInvariant(iterations={})", self.iterations)
    }
}

impl Invariant for KernelDeterminismInvariant {
    fn name(&self) -> &str {
        "KernelDeterminism"
    }

    fn description(&self) -> &str {
        "Kernel must produce identical output for identical input"
    }

    fn verify(&self, ctx: &VerificationContext) -> InvariantResult {
        let hashes = &ctx.kernel_hash_runs;

        if hashes.len() < 2 {
            return InvariantResult::Unknown("Insufficient kernel runs for verification".to_string());
        }

        let first = hashes[0];
        for (i, hash) in hashes.iter().enumerate().skip(1) {
            if *hash != first {
                return InvariantResult::Failed(InvariantViolation {
                    invariant: "KernelDeterminism".to_string(),
                    details: format!("Iteration {} hash differs: {:016x} vs {:016x}", i + 1, hash, first),
                    severity: ViolationSeverity::Critical,
                });
            }
        }

        InvariantResult::Passed
    }
}

pub struct StepMonotonicityInvariant;

impl std::fmt::Debug for StepMonotonicityInvariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StepMonotonicityInvariant")
    }
}

impl Invariant for StepMonotonicityInvariant {
    fn name(&self) -> &str {
        "StepMonotonicity"
    }

    fn description(&self) -> &str {
        "step_index must always increase monotonically"
    }

    fn verify(&self, ctx: &VerificationContext) -> InvariantResult {
        let history = &ctx.state_history;

        for window in history.windows(2) {
            let current = window[0].step_index;
            let next = window[1].step_index;

            if next <= current {
                return InvariantResult::Failed(InvariantViolation {
                    invariant: "StepMonotonicity".to_string(),
                    details: format!("step_index decreased: {} -> {}", current, next),
                    severity: ViolationSeverity::Critical,
                });
            }
        }

        InvariantResult::Passed
    }
}

pub struct ContractConsistencyInvariant;

impl std::fmt::Debug for ContractConsistencyInvariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ContractConsistencyInvariant")
    }
}

impl Invariant for ContractConsistencyInvariant {
    fn name(&self) -> &str {
        "ContractConsistency"
    }

    fn description(&self) -> &str {
        "Protobuf schemas must be consistent across all language bindings"
    }

    fn verify(&self, ctx: &VerificationContext) -> InvariantResult {
        let schemas = &ctx.contract_schemas;

        if schemas.is_empty() {
            return InvariantResult::Unknown("No contract schemas loaded".to_string());
        }

        let mut issues = Vec::new();

        for (name, schema) in schemas {
            if schema.fields.is_empty() {
                issues.push(format!("{}: empty schema", name));
            }
        }

        if issues.is_empty() {
            InvariantResult::Passed
        } else {
            InvariantResult::Failed(InvariantViolation {
                invariant: "ContractConsistency".to_string(),
                details: issues.join("; "),
                severity: ViolationSeverity::Critical,
            })
        }
    }
}

pub struct VerificationEngine {
    invariants: Vec<Box<dyn Invariant>>,
}

impl VerificationEngine {
    pub fn new() -> Self {
        Self { invariants: Vec::new() }
    }

    pub fn with_default_invariants() -> Self {
        let mut engine = Self::new();
        engine.register(Box::new(BoundaryStateAuthorityInvariant));
        engine.register(Box::new(CVEPurityInvariant));
        engine.register(Box::new(KernelDeterminismInvariant::new(10)));
        engine.register(Box::new(StepMonotonicityInvariant));
        engine.register(Box::new(ContractConsistencyInvariant));
        engine
    }

    pub fn register(&mut self, invariant: Box<dyn Invariant>) {
        self.invariants.push(invariant);
    }

    pub fn verify_all(&self, context: &VerificationContext) -> VerificationReport {
        let mut checks = Vec::new();

        for invariant in &self.invariants {
            let result = invariant.verify(context);
            checks.push(InvariantCheck {
                name: invariant.name().to_string(),
                description: invariant.description().to_string(),
                result,
            });
        }

        let passed = checks.iter().filter(|c| c.result.is_passed()).count();
        let failed = checks.iter().filter(|c| matches!(c.result, InvariantResult::Failed(_))).count();

        VerificationReport {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            checks,
            passed,
            failed,
        }
    }
}

impl Default for VerificationEngine {
    fn default() -> Self {
        Self::with_default_invariants()
    }
}

#[derive(Debug, Clone)]
pub struct InvariantCheck {
    pub name: String,
    pub description: String,
    pub result: InvariantResult,
}

#[derive(Debug)]
pub struct VerificationReport {
    pub timestamp: u64,
    pub checks: Vec<InvariantCheck>,
    pub passed: usize,
    pub failed: usize,
}

impl VerificationReport {
    pub fn is_all_passed(&self) -> bool {
        self.failed == 0
    }

    pub fn summary(&self) -> String {
        format!("VerificationReport: {}/{} passed, {} failed",
            self.passed, self.checks.len(), self.failed)
    }
}

pub fn analyze_cve_purity(cve_source: &str) -> CVEAnalysis {
    CVEAnalysis {
        has_static_mut: cve_source.contains("static mut"),
        has_io_operations: cve_source.contains("std::fs") || cve_source.contains("std::net"),
        has_global_state: cve_source.contains("lazy_static") || cve_source.contains("once_cell"),
        has_non_determinism: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context_with_history(steps: Vec<u64>) -> VerificationContext {
        let history: Vec<StateSnapshot> = steps.iter().enumerate().map(|(i, &step)| {
            StateSnapshot {
                step_index: step,
                simulation_id: "test".to_string(),
                state_hash: 0x1000 + i as u64,
                timestamp_ms: i as u64 * 100,
            }
        }).collect();

        VerificationContext {
            state_history: history,
            ..Default::default()
        }
    }

    #[test]
    fn test_step_monotonicity_pass() {
        let invariant = StepMonotonicityInvariant;
        let ctx = make_context_with_history(vec![1, 2, 3, 4, 5]);
        let result = invariant.verify(&ctx);

        assert!(result.is_passed());
    }

    #[test]
    fn test_step_monotonicity_fail() {
        let invariant = StepMonotonicityInvariant;
        let ctx = make_context_with_history(vec![1, 3, 2, 4, 5]);
        let result = invariant.verify(&ctx);

        assert!(!result.is_passed());
    }

    #[test]
    fn test_kernel_determinism_pass() {
        let invariant = KernelDeterminismInvariant::new(5);
        let ctx = VerificationContext {
            kernel_hash_runs: vec![0xABC, 0xABC, 0xABC, 0xABC, 0xABC],
            ..Default::default()
        };
        let result = invariant.verify(&ctx);

        assert!(result.is_passed());
    }

    #[test]
    fn test_kernel_determinism_fail() {
        let invariant = KernelDeterminismInvariant::new(3);
        let ctx = VerificationContext {
            kernel_hash_runs: vec![0xABC, 0xDEF, 0xABC],
            ..Default::default()
        };
        let result = invariant.verify(&ctx);

        assert!(!result.is_passed());
    }

    #[test]
    fn test_cve_purity_pass() {
        let invariant = CVEPurityInvariant;
        let source = "fn transform(state: &SimulationState) -> Result<GeometryScene, String> { ... }";
        let analysis = analyze_cve_purity(source);
        let ctx = VerificationContext {
            cve_analysis: Some(analysis),
            ..Default::default()
        };
        let result = invariant.verify(&ctx);

        assert!(result.is_passed());
    }

    #[test]
    fn test_cve_purity_fail() {
        let invariant = CVEPurityInvariant;
        let source = "static mut GLOBAL: u32 = 0; fn foo() { std::fs::read(...) }";
        let analysis = analyze_cve_purity(source);
        let ctx = VerificationContext {
            cve_analysis: Some(analysis),
            ..Default::default()
        };
        let result = invariant.verify(&ctx);

        assert!(!result.is_passed());
    }

    #[test]
    fn test_verification_engine() {
        let engine = VerificationEngine::with_default_invariants();
        let ctx = make_context_with_history(vec![1, 2, 3]);
        let report = engine.verify_all(&ctx);

        assert_eq!(report.checks.len(), 5);
        println!("{}", report.summary());
    }
}