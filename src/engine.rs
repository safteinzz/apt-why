//! Distro-agnostic analysis over a [`World`]: the algorithms that turn raw
//! package data into answers. None of this knows or cares whether the data came
//! from apt, pacman, or dnf — that's the backend's job. This is the part that
//! was painful in bash (graph BFS, window heuristics) and is trivial here.

use crate::model::World;

/// How many reverse-deps make a package "foundational" (a core library many
/// things need) rather than something a user would consider removing.
pub const FOUNDATION_THRESHOLD: usize = 25;

/// Kernel / firmware / microcode packages: never "safe to remove", and grouped
/// specially in the report. Name-based, matching the original tool's rules.
pub fn is_kernel_pkg(name: &str) -> bool {
    name.starts_with("linux-")
        || name == "intel-microcode"
        || name == "amd64-microcode"
        || name.starts_with("firmware-")
        || name.starts_with("initramfs")
}

/// The headline answer to "why the hell is this here": walk the reverse-dep
/// graph outward from `start` until we reach a *manually* installed package,
/// and return the path `[start … manual_root]`. That root is the thing the user
/// actually chose to install; everything on the path was pulled in for it.
///
/// Returns `None` if no manual ancestor is found (an orphan / untraceable
/// auto-install). BFS guarantees the shortest such chain.
pub fn bfs_root(world: &World, start: &str) -> Option<Vec<String>> {
    use std::collections::{HashMap, HashSet, VecDeque};

    let mut visited: HashSet<&str> = HashSet::new();
    let mut parent: HashMap<&str, &str> = HashMap::new();
    let mut queue: VecDeque<&str> = VecDeque::new();

    visited.insert(start);
    queue.push_back(start);

    let mut found: Option<&str> = None;
    'search: while let Some(cur) = queue.pop_front() {
        for dep in world.rdeps_of(cur) {
            let dep = dep.as_str();
            if !visited.insert(dep) {
                continue;
            }
            parent.insert(dep, cur);
            if world.is_manual(dep) {
                found = Some(dep);
                break 'search;
            }
            queue.push_back(dep);
        }
    }

    let found = found?;

    // Reconstruct the path from the manual root back to start, then reverse it
    // so it reads start → … → root.
    let mut path = vec![found.to_string()];
    let mut node = found;
    while let Some(&p) = parent.get(node) {
        path.push(p.to_string());
        node = p;
    }
    path.reverse();
    Some(path)
}

/// Packages installed around the same time as `pkg` — a strong context clue for
/// "what did I install this alongside?". We find `pkg`'s install time, then
/// widen the window (3d → 1d → 12h → 6h) until it holds a manageable number of
/// packages, mirroring the original heuristic. Returns names, deduped & sorted.
pub fn same_session(world: &World, pkg: &str) -> Vec<String> {
    let anchor = match world.packages.get(pkg).and_then(|p| p.install_epoch) {
        Some(e) => e,
        None => return Vec::new(),
    };

    // Widest-first so we prefer more context, but fall back to tighter windows
    // if a busy install day would otherwise return hundreds of packages.
    const WINDOWS: [i64; 4] = [259_200, 86_400, 43_200, 21_600]; // 3d 1d 12h 6h

    let collect = |half: i64| -> Vec<String> {
        let (lo, hi) = (anchor - half, anchor + half);
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for (epoch, name) in &world.install_log {
            if *epoch < lo || *epoch > hi {
                continue;
            }
            if name == pkg {
                continue;
            }
            if seen.insert(name.clone()) {
                out.push(name.clone());
            }
        }
        out.sort();
        out
    };

    for half in WINDOWS {
        let result = collect(half);
        if result.len() <= 20 {
            return result;
        }
    }
    collect(21_600)
}

/// A rough "how long ago" string for a unix timestamp, e.g. "3 months ago".
/// Complements the absolute date rather than replacing it.
pub fn relative_time(epoch: i64) -> String {
    let secs = chrono::Utc::now().timestamp() - epoch;
    if secs < 0 {
        return "in the future".to_string();
    }
    let days = secs / 86_400;
    let plural = |n: i64| if n == 1 { "" } else { "s" };
    match days {
        0 => "today".to_string(),
        1 => "yesterday".to_string(),
        2..=6 => format!("{days} days ago"),
        7..=29 => {
            let w = days / 7;
            format!("{w} week{} ago", plural(w))
        }
        30..=364 => {
            let m = days / 30;
            format!("{m} month{} ago", plural(m))
        }
        _ => {
            let y = days / 365;
            format!("{y} year{} ago", plural(y))
        }
    }
}

/// Format an installed size given in KB into a compact human string.
pub fn format_size(kb: u64) -> String {
    if kb == 0 {
        "n/a".to_string()
    } else if kb >= 1024 {
        format!("{:.1} MB", kb as f64 / 1024.0)
    } else {
        format!("{kb} KB")
    }
}
