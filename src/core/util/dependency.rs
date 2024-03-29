use crate::fs::AbsPath;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

/// Dependency Manager
///
/// A directed dependency graph that track dependencies between files. An edge `A -> B` means that `A` depends on `B`.
pub struct DepManager {
    out_edge_counts: HashMap<AbsPath, usize>, // Count how many dependencies one vertex has
    in_edges: HashMap<AbsPath, HashSet<AbsPath>>, // V -> K edges (V depends on K)
    finished: HashSet<AbsPath>,               // Set of finished vertices
}

impl DepManager {
    /// Create empty dependency manager
    pub fn new() -> Self {
        Self {
            out_edge_counts: HashMap::new(),
            in_edges: HashMap::new(),
            finished: HashSet::new(),
        }
    }

    /// Add multiple dependencies to a file
    ///
    /// The first argument `A` depends on all of the file `B`s in the second argument.
    /// This will add `A -> B` edges to the graph for every B in the second argument.
    ///
    /// Return true if `A` has any out edge.
    pub fn add_dependency(&mut self, depender: &AbsPath, dependencies: &[AbsPath]) -> bool {
        if dependencies.is_empty() {
            return false;
        }
        let mut added = false;
        let dependency_count = self.out_edge_counts.entry(depender.clone()).or_insert(0);
        for dependency in dependencies {
            if self.finished.contains(dependency) {
                continue;
            }
            let dependers = self.in_edges.entry(dependency.clone()).or_default();
            // add depender -> dependency edge
            if dependers.insert(depender.clone()) {
                *dependency_count += 1;
            }
            added = true;
        }
        // It's fine to not revert the added vertice even if it has no out edge
        // because we use in_edges to traverse the graph.
        // The empty vertices will never be visited
        added
    }

    /// Notify a file `B` has finished processing
    ///
    /// This assumes `B` has no out edges and removes all (in) edges of `B`.
    /// For each `A -> B` edge removed, if `A` has no more out edges after the removal, `A` is added to the output.
    pub fn notify_finish(&mut self, finished: &AbsPath) -> HashSet<AbsPath> {
        self.finished.insert(finished.clone());
        // Get all dependers of finished
        let mut output = HashSet::new();
        let in_edges = match self.in_edges.remove(finished) {
            Some(in_edges) => in_edges,
            None => return output,
        };
        for depender in in_edges {
            let count = self.out_edge_counts.get_mut(&depender).unwrap();
            if *count <= 1 {
                self.out_edge_counts.remove(&depender);
                output.insert(depender);
            } else {
                *count -= 1;
            }
        }
        output
    }

    /// Convert the remaining graph to a map of `depender -> [dependencies]`
    pub fn take_remaining(self) -> HashMap<AbsPath, HashSet<AbsPath>> {
        let mut out_edges = HashMap::new();
        for (k, v) in self.in_edges {
            for depender in v {
                out_edges
                    .entry(depender)
                    .or_insert_with(HashSet::new)
                    .insert(k.clone());
            }
        }
        out_edges
    }
}

pub fn print_dep_map(map: &HashMap<AbsPath, HashSet<AbsPath>>) -> String {
    let mut out = String::new();
    for (k, v) in map {
        let _ = writeln!(out, "{k}");
        for (i, dep) in v.iter().enumerate() {
            let _ = if i == v.len() - 1 {
                writeln!(out, "╰╴{dep}")
            } else {
                writeln!(out, "├╴{dep}")
            };
        }
        out.push('\n')
    }
    out
}

#[cfg(test)]
mod ut {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_empty() {
        let mut dm = DepManager::new();
        let finished = AbsPath::new(PathBuf::from("/a"));
        let free = dm.notify_finish(&finished);
        assert_eq!(free, HashSet::new());
    }

    #[test]
    fn test_insert_empty() {
        let mut dm = DepManager::new();
        let finished = AbsPath::new(PathBuf::from("/a"));
        assert!(!dm.add_dependency(&finished, &[]));
        let free = dm.notify_finish(&finished);
        assert_eq!(free, HashSet::new());
    }

    #[test]
    fn test_one() {
        let mut dm = DepManager::new();
        let a = AbsPath::new(PathBuf::from("/a"));
        let b = AbsPath::new(PathBuf::from("/b"));
        assert!(dm.add_dependency(&a, &[b.clone()]));
        let free = dm.notify_finish(&b);
        assert_eq!(free, [a].into_iter().collect());
    }

    #[test]
    fn test_one_no_depender() {
        let mut dm = DepManager::new();
        let a = AbsPath::new(PathBuf::from("/a"));
        let b = AbsPath::new(PathBuf::from("/b"));
        let c = AbsPath::new(PathBuf::from("/c"));
        assert!(dm.add_dependency(&a, &[b.clone()]));
        let free = dm.notify_finish(&c);
        assert_eq!(free, HashSet::new());
        let a_deps = [b].into_iter().collect::<HashSet<_>>();
        assert_eq!(
            dm.take_remaining(),
            [(a, a_deps)].into_iter().collect::<HashMap<_, _>>()
        );
    }

    #[test]
    fn test_one_depends_on_two() {
        let mut dm = DepManager::new();
        let a = AbsPath::new(PathBuf::from("/a"));
        let b = AbsPath::new(PathBuf::from("/b"));
        let c = AbsPath::new(PathBuf::from("/c"));
        assert!(dm.add_dependency(&a, &[b.clone(), c.clone()]));
        let free = dm.notify_finish(&b);
        assert_eq!(free, HashSet::new());
        let free = dm.notify_finish(&c);
        assert_eq!(free, [a].into_iter().collect());
    }

    #[test]
    fn test_diamond() {
        let mut dm = DepManager::new();
        let a = AbsPath::new(PathBuf::from("/a"));
        let b = AbsPath::new(PathBuf::from("/b"));
        let c = AbsPath::new(PathBuf::from("/c"));
        let d = AbsPath::new(PathBuf::from("/d"));
        assert!(dm.add_dependency(&a, &[b.clone(), c.clone()]));
        assert!(dm.add_dependency(&b, &[d.clone()]));
        assert!(dm.add_dependency(&c, &[d.clone()]));
        let free = dm.notify_finish(&d);
        assert_eq!(free, [b.clone(), c.clone()].into_iter().collect());
        let free = dm.notify_finish(&c);
        assert_eq!(free, HashSet::new());
        let free = dm.notify_finish(&b);
        assert_eq!(free, [a].into_iter().collect());
    }

    #[test]
    fn test_circle() {
        let mut dm = DepManager::new();
        let a = AbsPath::new(PathBuf::from("/a"));
        let b = AbsPath::new(PathBuf::from("/b"));
        let c = AbsPath::new(PathBuf::from("/c"));
        assert!(dm.add_dependency(&a, &[b.clone(), c.clone()]));
        assert!(dm.add_dependency(&b, &[a.clone()]));
        let free = dm.notify_finish(&c);
        assert_eq!(free, HashSet::new());
        let a_deps = [b.clone()].into_iter().collect::<HashSet<_>>();
        let b_deps = [a.clone()].into_iter().collect::<HashSet<_>>();
        assert_eq!(
            dm.take_remaining(),
            [(a, a_deps), (b, b_deps)]
                .into_iter()
                .collect::<HashMap<_, _>>()
        );
    }

    #[test]
    fn test_do_not_add_done() {
        let mut dm = DepManager::new();
        let a = AbsPath::new(PathBuf::from("/a"));
        let b = AbsPath::new(PathBuf::from("/b"));
        let free = dm.notify_finish(&b);
        assert_eq!(free, HashSet::new());
        assert!(!dm.add_dependency(&a, &[b.clone()]));
        assert!(!dm.add_dependency(&a, &[b.clone(), b.clone()]));
        let free = dm.notify_finish(&b);
        assert_eq!(free, HashSet::new());
        assert_eq!(dm.take_remaining(), HashMap::new());
    }
}
