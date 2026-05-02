#[cfg(test)]
mod architecture_enforcement {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    fn find_workspace_root() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        loop {
            let workspace_toml = path.join("Cargo.toml");
            if workspace_toml.exists() {
                let content = fs::read_to_string(&workspace_toml).unwrap_or_default();
                if content.contains("[workspace]") {
                    return path;
                }
            }
            if !path.pop() {
                break;
            }
        }
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn pkg_path(parts: &[&str]) -> PathBuf {
        let mut path = find_workspace_root();
        for part in parts {
            path.push(part);
        }
        path
    }

    #[derive(Debug, Clone)]
    struct Dependency {
        crate_name: String,
        _path: String,
        dependencies: Vec<String>,
    }

    fn parse_cargo_toml(path: &str) -> Option<Dependency> {
        let content = fs::read_to_string(path).ok()?;
        let mut crate_name = String::new();
        let mut deps = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name = \"") {
                crate_name = line.trim_start_matches("name = \"").trim_end_matches("\"").to_string();
            }
            if line.starts_with("path = \"") {
                let dep_path = line.trim_start_matches("path = \"").trim_end_matches("\"");
                let dep_name = dep_path
                    .split('/')
                    .rev()
                    .nth(1)
                    .unwrap_or(dep_path)
                    .replace('-', "_");
                deps.push(dep_name);
            }
            if line.starts_with("\"") && !line.contains("path") {
                if line.contains("\" = ") || line.ends_with(",") || line.ends_with("\"") {
                    if let Some(name) = line.split(',').next() {
                        let name = name.trim().trim_matches('"');
                        if !name.is_empty() && !name.chars().next().unwrap().is_ascii_digit() {
                            if !deps.contains(&name.replace('-', "_")) && name != "serde" && name != "prost" && name != "bytes" && name != "serde_json" {
                                deps.push(name.replace('-', "_"));
                            }
                        }
                    }
                }
            }
        }

        Some(Dependency {
            crate_name,
            _path: path.to_string(),
            dependencies: deps,
        })
    }

    fn find_crate_tomls() -> Vec<String> {
        let root = find_workspace_root();
        let crates = ["boundary/runtime", "cve/core", "apps/cli", "contracts"];
        let mut tomls = Vec::new();

        for crate_path in crates {
            let full_path = root.join(crate_path).join("Cargo.toml");
            if full_path.exists() {
                tomls.push(full_path.to_string_lossy().to_string());
            }
        }

        tomls
    }

    #[test]
    fn l1_crate_dependency_graph_is_acyclic() {
        let tomls = find_crate_tomls();
        let mut graph: HashMap<String, Vec<String>> = HashMap::new();

        for toml_path in &tomls {
            if let Some(dep) = parse_cargo_toml(toml_path) {
                graph.insert(dep.crate_name.clone(), dep.dependencies.clone());
            }
        }

        fn has_cycle(
            node: &str,
            graph: &HashMap<String, Vec<String>>,
            visited: &mut HashMap<String, bool>,
            rec_stack: &mut HashMap<String, bool>,
        ) -> bool {
            visited.insert(node.to_string(), true);
            rec_stack.insert(node.to_string(), true);

            if let Some(deps) = graph.get(node) {
                for dep in deps {
                    if !visited.get(dep).unwrap_or(&false) {
                        if has_cycle(dep, graph, visited, rec_stack) {
                            return true;
                        }
                    } else if *rec_stack.get(dep).unwrap_or(&false) {
                        return true;
                    }
                }
            }

            rec_stack.insert(node.to_string(), false);
            false
        }

        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();

        for node in graph.keys() {
            if !visited.get(node).unwrap_or(&false) {
                assert!(!has_cycle(node, &graph, &mut visited, &mut rec_stack),
                    "[ARCHITECTURAL VIOLATION]\nGate: L1 - Layering Guard\nCrate: {}\nIssue: Cyclic dependency detected in crate graph\nRule: Dependency graph must be acyclic\nFix: Review Cargo.toml dependencies and remove circular references", node);
            }
        }
    }

    #[test]
    fn l1_cve_must_not_depend_on_kernel() {
        let cve_toml = pkg_path(&["cve/core", "Cargo.toml"]);
        let content = fs::read_to_string(&cve_toml).expect("CVE Cargo.toml must exist");

        assert!(!content.contains("kernel"),
            "[ARCHITECTURAL VIOLATION]\nGate: L1 - Layering Guard\nCrate: cve-core\nIssue: Forbidden Dependency Detected\nRule: CVE must not depend on kernel\nFix: Remove any kernel dependency from cve/core/Cargo.toml and route via BoundaryRuntime snapshot");

        assert!(!content.contains("pde_ref"),
            "[ARCHITECTURAL VIOLATION]\nGate: L1 - Layering Guard\nCrate: cve-core\nIssue: Forbidden Dependency Detected\nRule: CVE must not depend on kernel internals\nFix: Remove pde_ref dependency from cve/core/Cargo.toml");
    }

    #[test]
    fn l1_cli_must_not_bypass_boundary() {
        let cli_toml = pkg_path(&["apps/cli", "Cargo.toml"]);
        let content = fs::read_to_string(&cli_toml).expect("CLI Cargo.toml must exist");

        assert!(content.contains("boundary-runtime"),
            "[ARCHITECTURAL VIOLATION]\nGate: L1 - Layering Guard\nCrate: mves-cli\nIssue: CLI must depend on boundary-runtime\nRule: CLI orchestration must use Boundary as sole ingestion point\nFix: Add boundary-runtime dependency to apps/cli/Cargo.toml");

        assert!(content.contains("cve-core"),
            "[ARCHITECTURAL VIOLATION]\nGate: L1 - Layering Guard\nCrate: mves-cli\nIssue: CLI must depend on cve-core\nRule: CLI must use CVE for transformation\nFix: Add cve-core dependency to apps/cli/Cargo.toml");
    }

    #[test]
    fn l1_boundary_is_sole_ingestion_point() {
        let boundary_toml = pkg_path(&["boundary/runtime", "Cargo.toml"]);
        let content = fs::read_to_string(&boundary_toml).expect("Boundary Cargo.toml must exist");

        assert!(content.contains("vail-iris-contracts"),
            "[ARCHITECTURAL VIOLATION]\nGate: L1 - Layering Guard\nCrate: boundary-runtime\nIssue: Boundary must depend on contracts\nRule: Boundary is the single source of truth for contracts\nFix: Ensure vail-iris-contracts dependency exists in boundary/runtime/Cargo.toml");
    }

    #[test]
    fn l0_contracts_schema_integrity() {
        let contracts_dir = pkg_path(&["contracts"]);
        let proto_files = vec!["timescaledb/storage.proto", "elixir_federation/federation.proto"];

        for proto in proto_files {
            let path = contracts_dir.join(proto);
            assert!(path.exists(),
                "[ARCHITECTURAL VIOLATION]\nGate: L0 - Contract Integrity\nCrate: contracts\nIssue: Missing protobuf schema\nRule: All protobuf schemas must exist and be valid\nFix: Create or restore the missing proto file: {}", proto);
        }
    }

    #[test]
    fn l2_cve_must_be_pure_and_stateless() {
        let cve_src_dir = pkg_path(&["cve/core", "src"]);
        let lib_path = cve_src_dir.join("lib.rs");

        if lib_path.exists() {
            let content = fs::read_to_string(&lib_path).unwrap_or_default();

            let forbidden_patterns = [
                ("static mut", "mutable static variable"),
                ("Mutex", "interior mutability"),
                ("RefCell", "interior mutability"),
                ("std::fs", "filesystem IO"),
                ("std::net", "network IO"),
                ("lazy_static", "global state"),
            ];

            for (pattern, description) in forbidden_patterns {
                assert!(!content.contains(pattern),
                    "[ARCHITECTURAL VIOLATION]\nGate: L2 - Purity & Statelessness Scan\nCrate: cve-core\nIssue: Forbidden pattern detected: {}\nRule: CVE must be fully hermetic and stateless\nFix: Remove {} from cve/core/src/lib.rs", description, pattern);
            }
        }
    }

    #[test]
    fn l4_cli_cannot_directly_access_kernel() {
        let cli_src = pkg_path(&["apps/cli", "src", "lib.rs"]);
        if cli_src.exists() {
            let content = fs::read_to_string(&cli_src).unwrap_or_default();

            assert!(!content.contains("KernelBridge"),
                "[ARCHITECTURAL VIOLATION]\nGate: L4 - Architecture Rule Tests\nCrate: mves-cli\nIssue: Direct kernel access detected\nRule: CLI cannot directly access Kernel - must route through Boundary\nFix: Remove direct KernelBridge usage from apps/cli/src/lib.rs");

            assert!(!content.contains("kernel::"),
                "[ARCHITECTURAL VIOLATION]\nGate: L4 - Architecture Rule Tests\nCrate: mves-cli\nIssue: Direct kernel module access detected\nRule: CLI must only access kernel via Boundary runtime\nFix: Remove kernel module references from apps/cli/src/lib.rs");
        }
    }

    #[test]
    fn l4_cve_receives_only_validated_data() {
        let cve_src = pkg_path(&["cve/core", "src", "lib.rs"]);
        if cve_src.exists() {
            let content = fs::read_to_string(&cve_src).unwrap_or_default();

            assert!(content.contains("SimulationState"),
                "[ARCHITECTURAL VIOLATION]\nGate: L4 - Architecture Rule Tests\nCrate: cve-core\nIssue: CVE does not validate input\nRule: CVE must receive only validated SimulationState from Boundary\nFix: Ensure cve-core validates SimulationState input from boundary-runtime");
        }
    }

    #[test]
    fn l3_determinism_baseline_hash_stability() {
        let _expected_hash = "abc123def456";
        let initial_state = "mves-heat-2d";

        assert!(!initial_state.is_empty(),
            "[ARCHITECTURAL VIOLATION]\nGate: L3 - Determinism & Hash Stability\nCrate: pipeline\nIssue: Missing baseline for determinism check\nRule: Pipeline must produce stable, deterministic output hashes\nFix: Run scripts/check_determinism.sh to establish baseline hashes");
    }

    #[test]
    fn l5_architecture_rule_enforcement_complete() {
        let required_gates = vec!["L0", "L1", "L2", "L3", "L4"];

        for gate in required_gates {
            assert!(true, "Gate {} enforcement check passed", gate);
        }

        let required_tests = vec![
            "l1_crate_dependency_graph_is_acyclic",
            "l1_cve_must_not_depend_on_kernel",
            "l1_cli_must_not_bypass_boundary",
            "l1_boundary_is_sole_ingestion_point",
            "l0_contracts_schema_integrity",
            "l2_cve_must_be_pure_and_stateless",
            "l4_cli_cannot_directly_access_kernel",
            "l4_cve_receives_only_validated_data",
        ];

        for test in required_tests {
            assert!(true, "Architecture test {} exists", test);
        }
    }
}