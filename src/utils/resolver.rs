use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::path::PathBuf;

use crate::utils::manifest::PackageManifest;

/// A resolved package with its source directory in the registry.
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub name: String,
    pub source: PathBuf,
}

/// Build and resolve the dependency graph starting from the given package.
///
/// Performs a depth-first traversal: dependencies are resolved before the
/// package itself, yielding a topological install order. Detects circular
/// dependencies and shared (diamond) dependencies, which are installed once.
///
/// - `visited`: packages already fully resolved (shared deps are skipped).
/// - `in_stack`: packages currently on the DFS stack (cycle detection).
pub fn resolve(
    name: &str,
    registry_dir: &PathBuf,
    visited: &mut HashSet<String>,
    in_stack: &mut HashSet<String>,
) -> Result<Vec<ResolvedPackage>> {
    let pkg_dir = registry_dir.join("packages").join(name);

    if !pkg_dir.exists() {
        bail!("Package '{}' not found in registry", name);
    }

    let manifest = PackageManifest::from_dir(&pkg_dir)
        .with_context(|| format!("Failed to read manifest for '{name}'"))?;

    // Circular dependency detection
    if in_stack.contains(name) {
        let deps_in_cycle: Vec<&String> = manifest.dependencies.keys().collect();
        bail!(
            "Circular dependency detected: {} → {}",
            name,
            deps_in_cycle
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" → ")
        );
    }

    if visited.contains(name) {
        return Ok(Vec::new());
    }

    visited.insert(name.to_string());
    in_stack.insert(name.to_string());

    let mut resolved = Vec::new();

    // Resolve dependencies first (depth-first)
    for (dep_name, _constraint) in &manifest.dependencies {
        let sub_resolved = resolve(dep_name, registry_dir, visited, in_stack)?;
        resolved.extend(sub_resolved);
    }

    // Remove from stack after all deps are resolved
    in_stack.remove(name);

    // Add self
    resolved.push(ResolvedPackage {
        name: manifest.package.name.clone(),
        source: pkg_dir,
    });

    Ok(resolved)
}

/// Resolve packages into install batches (levels).
///
/// Each batch contains packages whose dependencies are all in earlier
/// batches, so packages within a batch can be installed in parallel.
/// Batch order is a valid topological order.
pub fn resolve_batched(
    root: &str,
    registry_dir: &PathBuf,
) -> Result<Vec<Vec<ResolvedPackage>>> {
    let order = resolve(root, registry_dir, &mut HashSet::new(), &mut HashSet::new())?;

    // Map package name → (index in order, dependencies of that package)
    let mut index_of: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (i, pkg) in order.iter().enumerate() {
        index_of.insert(pkg.name.as_str(), i);
    }

    // Compute batch index for each package:
    // batch(pkg) = 1 + max(batch(dep) for each dep)
    let mut batch_of: Vec<usize> = vec![0; order.len()];
    let mut deps_of: Vec<Vec<String>> = Vec::with_capacity(order.len());

    for pkg in &order {
        let pkg_dir = registry_dir.join("packages").join(&pkg.name);
        let deps = PackageManifest::from_dir(&pkg_dir)
            .map(|m| m.dependencies.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        deps_of.push(deps);
    }

    let mut max_batch = 0;
    for i in 0..order.len() {
        let mut b = 1; // base level for a package with no deps
        for dep in &deps_of[i] {
            if let Some(&di) = index_of.get(dep.as_str()) {
                b = b.max(batch_of[di] + 1);
            }
        }
        batch_of[i] = b;
        max_batch = max_batch.max(b);
    }

    let mut batches: Vec<Vec<ResolvedPackage>> = vec![Vec::new(); max_batch];
    for (i, pkg) in order.into_iter().enumerate() {
        batches[batch_of[i] - 1].push(pkg);
    }

    Ok(batches)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a temp registry with packages.
    fn make_registry(pkgs: &[(&str, &str, &[(&str, &str)])]) -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);

        let dir = std::env::temp_dir().join(format!(
            "inx-test-{}-{}",
            std::process::id(),
            id
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("packages")).unwrap();

        for (name, version, deps) in pkgs {
            let pkg_dir = dir.join("packages").join(name);
            std::fs::create_dir_all(&pkg_dir).unwrap();

            let mut toml = format!(
                "[package]\nname = \"{name}\"\nversion = \"{version}\"\n"
            );
            if !deps.is_empty() {
                toml.push_str("\n[dependencies]\n");
                for (dep, constraint) in *deps {
                    toml.push_str(&format!("{dep} = \"{constraint}\"\n"));
                }
            }
            std::fs::write(pkg_dir.join("inx.toml"), toml).unwrap();
        }

        dir
    }

    #[test]
    fn resolves_simple_package() {
        let reg = make_registry(&[("a", "1.0.0", &[])]);
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let result = resolve("a", &reg, &mut visited, &mut stack).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "a");
        let _ = std::fs::remove_dir_all(&reg);
    }

    #[test]
    fn resolves_dependencies_in_order() {
        let reg = make_registry(&[
            ("a", "1.0.0", &[("b", "1.0.0")]),
            ("b", "1.0.0", &[("c", "1.0.0")]),
            ("c", "1.0.0", &[]),
        ]);
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let result = resolve("a", &reg, &mut visited, &mut stack).unwrap();
        let names: Vec<&str> = result.iter().map(|p| p.name.as_str()).collect();
        // c before b before a
        assert_eq!(names, vec!["c", "b", "a"]);
        let _ = std::fs::remove_dir_all(&reg);
    }

    #[test]
    fn resolves_diamond_dependency_once() {
        let reg = make_registry(&[
            ("a", "1.0.0", &[("b", "1.0.0"), ("c", "1.0.0")]),
            ("b", "1.0.0", &[("d", "1.0.0")]),
            ("c", "1.0.0", &[("d", "1.0.0")]),
            ("d", "1.0.0", &[]),
        ]);
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let result = resolve("a", &reg, &mut visited, &mut stack).unwrap();
        let names: Vec<&str> = result.iter().map(|p| p.name.as_str()).collect();
        // d must come before both b and c; b and c before a.
        // Sibling order (b vs c) is not guaranteed (HashMap iteration order).
        assert_eq!(names[0], "d");
        assert!(names[1] == "b" || names[1] == "c");
        let last = names.last().copied().unwrap_or("");
        assert_eq!(last, "a");
        assert_eq!(names.len(), 4);
        let _ = std::fs::remove_dir_all(&reg);
    }

    #[test]
    fn detects_circular_dependency() {
        let reg = make_registry(&[
            ("a", "1.0.0", &[("b", "1.0.0")]),
            ("b", "1.0.0", &[("a", "1.0.0")]),
        ]);
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let result = resolve("a", &reg, &mut visited, &mut stack);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Circular dependency"), "error was: {err}");
        let _ = std::fs::remove_dir_all(&reg);
    }

    #[test]
    fn reports_missing_dependency() {
        let reg = make_registry(&[("a", "1.0.0", &[("missing", "1.0.0")])]);
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let result = resolve("a", &reg, &mut visited, &mut stack);
        assert!(result.is_err());
        let _ = std::fs::remove_dir_all(&reg);
    }

    #[test]
    fn batched_resolution_groups_independent_packages() {
        let reg = make_registry(&[
            ("a", "1.0.0", &[("b", "1.0.0"), ("c", "1.0.0")]),
            ("b", "1.0.0", &[("d", "1.0.0")]),
            ("c", "1.0.0", &[("d", "1.0.0")]),
            ("d", "1.0.0", &[]),
        ]);
        let batches = resolve_batched("a", &reg).unwrap();

        // Batch 1: d alone (no deps)
        // Batch 2: b and c (siblings, installable in parallel)
        // Batch 3: a (root)
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 1);
        assert_eq!(batches[0][0].name, "d");
        assert_eq!(batches[1].len(), 2);
        assert_eq!(batches[2].len(), 1);
        assert_eq!(batches[2][0].name, "a");
        let _ = std::fs::remove_dir_all(&reg);
    }
}