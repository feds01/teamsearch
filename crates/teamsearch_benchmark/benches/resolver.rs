use std::slice;

use divan::Bencher;
use teamsearch_benchmark::{Repo, Size, TEAM};
use teamsearch_workspace::resolver::find_files_in_paths;

// CodSpeed measures the allocations itself, in its memory mode.
#[cfg(not(codspeed))]
#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

/// Walks the repository for the files that a team owns, as `teamsearch find
/// --team` does before it searches them.
#[divan::bench(args = Size::ALL)]
fn find_files(bencher: Bencher, size: Size) {
    let repo = Repo::new(size);
    let settings = repo.team_settings(TEAM);

    // CodSpeed has no counters, and warns about each one.
    #[cfg(not(codspeed))]
    let bencher = bencher.counter(divan::counter::ItemsCount::new(repo.files.len()));

    bencher
        .bench(|| find_files_in_paths(&repo.root, slice::from_ref(&repo.root), &settings).unwrap());
}
