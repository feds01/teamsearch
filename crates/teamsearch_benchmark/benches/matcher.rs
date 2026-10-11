use divan::Bencher;
use teamsearch_benchmark::{Repo, Search, Size};
use teamsearch_matcher::search_file;

// CodSpeed measures the allocations itself, in its memory mode.
#[cfg(not(codspeed))]
#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

/// The files that the `search` benchmark searches.
const SEARCHES: usize = 50;

fn main() {
    divan::main();
}

/// Searches some of the files for the pattern, keeping those that match, as
/// `teamsearch find` does with each file that it resolves.
#[divan::bench(args = Search::all())]
fn search(bencher: Bencher, search: &Search) {
    let repo = Repo::new(Size::Small);
    let paths = repo.sample(SEARCHES);

    // CodSpeed has no counters, and warns about each one.
    #[cfg(not(codspeed))]
    let bencher = {
        let bytes: u64 = paths.iter().map(|path| path.metadata().unwrap().len()).sum();
        bencher.counter(divan::counter::BytesCount::new(bytes))
    };

    bencher.bench(|| {
        paths
            .iter()
            .map(|&path| search_file(search.pattern(), path.clone()).unwrap())
            .filter(|matches| !matches.is_empty())
            .collect::<Vec<_>>()
    });
}
