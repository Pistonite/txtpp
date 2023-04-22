use std::collections::{HashMap, HashSet};

use crate::path::AbsPath;

/// Dependency Manager
/// 
/// A directed dependency graph that track dependencies between files. An edge `A -> B` means that `A` depends on `B`.
pub struct DepManager {
    out_edge_counts: HashMap<AbsPath, usize>, // Count how many dependencies one vertex has
    in_edges: HashMap<AbsPath, HashSet<AbsPath>>, // V -> K edges (V depends on K)
}

impl DepManager {
    /// Create empty dependency manager
    pub fn new() -> Self {
        Self {
            out_edge_counts: HashMap::new(),
            in_edges: HashMap::new(),
        }
    }

    /// Add multiple dependencies to a file
    /// 
    /// The first argument `A` depends on all of the file `B`s in the second argument.
    /// This will add `A -> B` edges to the graph for every B in the second argument.
    pub fn add_dependency(&mut self, depender: &AbsPath, dependencies: &[AbsPath]) {
        if dependencies.is_empty() {
            return;
        }
        let dependency_count = self.out_edge_counts.entry(depender.clone()).or_insert(0);
        for dependency in dependencies {
            let dependers = self.in_edges.entry(dependency.clone()).or_insert(HashSet::new());
            // add depender -> dependency edge
            if dependers.insert(depender.clone()) {
                *dependency_count += 1;
            }
        }

    }

    /// Notify a file `B` has finished processing
    /// 
    /// This assumes `B` has no out edges and removes all (in) edges of `B`.
    /// For each `A -> B` edge removed, if `A` has no more out edges after the removal, `A` is added to the output.
    pub fn notify_finish(&mut self, finished: &AbsPath) -> HashSet<AbsPath> {
        // Get all dependers of finished
        let mut output = HashSet::new();
        let in_edges = match self.in_edges.remove(finished) {
            Some(in_edges) => in_edges,
            None => return output,
        };
        for depender in in_edges {
            let count = self.out_edge_counts.get_mut(&depender).unwrap();
            if *count <= 1{
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
                out_edges.entry(depender).or_insert_with(HashSet::new).insert(k.clone());
            }
        }
        out_edges
    }
}

#[cfg(test)]
mod ut {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_empty() {
        let mut dm = DepManager::new();
        let finished = AbsPath::try_from(PathBuf::from("/a")).unwrap();
        let free = dm.notify_finish(&finished);
        assert_eq!(free, HashSet::new());
    }

    #[test]
    fn test_insert_empty() {
        let mut dm = DepManager::new();
        let finished = AbsPath::try_from(PathBuf::from("/a")).unwrap();
        dm.add_dependency(&finished, &[]);
        let free = dm.notify_finish(&finished);
        assert_eq!(free, HashSet::new());
    }

    #[test]
    fn test_one() {
        let mut dm = DepManager::new();
        let a = AbsPath::try_from(PathBuf::from("/a")).unwrap();
        let b = AbsPath::try_from(PathBuf::from("/b")).unwrap();
        dm.add_dependency(&a, &[b.clone()]);
        let free = dm.notify_finish(&b);
        assert_eq!(free, [a].into_iter().collect());
    }

    #[test]
    fn test_one_no_depender() {
        let mut dm = DepManager::new();
        let a = AbsPath::try_from(PathBuf::from("/a")).unwrap();
        let b = AbsPath::try_from(PathBuf::from("/b")).unwrap();
        let c = AbsPath::try_from(PathBuf::from("/c")).unwrap();
        dm.add_dependency(&a, &[b.clone()]);
        let free = dm.notify_finish(&c);
        assert_eq!(free, HashSet::new());
        let a_deps = [b].into_iter().collect::<HashSet<_>>();
        assert_eq!(dm.take_remaining(), [(a, a_deps)].into_iter().collect::<HashMap<_, _>>());
    }

    #[test]
    fn test_one_depends_on_two() {
        let mut dm = DepManager::new();
        let a = AbsPath::try_from(PathBuf::from("/a")).unwrap();
        let b = AbsPath::try_from(PathBuf::from("/b")).unwrap();
        let c = AbsPath::try_from(PathBuf::from("/c")).unwrap();
        dm.add_dependency(&a, &[b.clone(), c.clone()]);
        let free = dm.notify_finish(&b);
        assert_eq!(free, HashSet::new());
        let free = dm.notify_finish(&c);
        assert_eq!(free, [a].into_iter().collect());
    }

    #[test]
    fn test_diamond() {
        let mut dm = DepManager::new();
        let a = AbsPath::try_from(PathBuf::from("/a")).unwrap();
        let b = AbsPath::try_from(PathBuf::from("/b")).unwrap();
        let c = AbsPath::try_from(PathBuf::from("/c")).unwrap();
        let d = AbsPath::try_from(PathBuf::from("/d")).unwrap();
        dm.add_dependency(&a, &[b.clone(), c.clone()]);
        dm.add_dependency(&b, &[d.clone()]);
        dm.add_dependency(&c, &[d.clone()]);
        let free = dm.notify_finish(&d);
        assert_eq!(free, [b.clone(),c.clone()].into_iter().collect());
        let free = dm.notify_finish(&c);
        assert_eq!(free, HashSet::new());
        let free = dm.notify_finish(&b);
        assert_eq!(free, [a].into_iter().collect());
    }

    #[test]
    fn test_circle() {
        let mut dm = DepManager::new();
        let a = AbsPath::try_from(PathBuf::from("/a")).unwrap();
        let b = AbsPath::try_from(PathBuf::from("/b")).unwrap();
        let c = AbsPath::try_from(PathBuf::from("/c")).unwrap();
        dm.add_dependency(&a, &[b.clone(), c.clone()]);
        dm.add_dependency(&b, &[a.clone()]);
        let free = dm.notify_finish(&c);
        assert_eq!(free, HashSet::new());
        let a_deps = [b.clone()].into_iter().collect::<HashSet<_>>();
        let b_deps = [a.clone()].into_iter().collect::<HashSet<_>>();
        assert_eq!(dm.take_remaining(), [(a, a_deps), (b, b_deps)].into_iter().collect::<HashMap<_, _>>());
    }
}