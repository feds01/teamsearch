use divan::Bencher;
use teamsearch_benchmark::{Repo, Size};
use teamsearch_workspace::codeowners::CodeOwners;

// CodSpeed measures the allocations itself, in its memory mode.
#[cfg(not(codspeed))]
#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

/// The files that the `lookup` benchmark looks up the owners of.
const LOOKUPS: usize = 16;

fn main() {
    divan::main();
}

/// Parses the CODEOWNERS file and builds the matcher of every owned path, as
/// each command does before anything else.
#[divan::bench(args = Size::ALL)]
fn parse(bencher: Bencher, size: Size) {
    let repo = Repo::new(size);
    bencher.bench(|| CodeOwners::parse_from_file(&repo.codeowners, &repo.root).unwrap());
}

/// Looks up the teams that own a few files, as `teamsearch lookup` does.
#[divan::bench(args = Size::ALL)]
fn lookup(bencher: Bencher, size: Size) {
    let repo = Repo::new(size);
    let codeowners = repo.parse_codeowners();
    let paths = repo.sample(LOOKUPS);

    // CodSpeed has no counters, and warns about each one.
    #[cfg(not(codspeed))]
    let bencher = bencher.counter(divan::counter::ItemsCount::new(paths.len()));

    bencher.bench(|| paths.iter().map(|path| codeowners.lookup(path)).collect::<Vec<_>>());
}

/// Checks whether any team owns each file, one after another, as `teamsearch
/// orphans` does with each file that it finds.
#[divan::bench(args = Size::ALL)]
fn is_owned(bencher: Bencher, size: Size) {
    let repo = Repo::new(size);
    let codeowners = repo.parse_codeowners();

    // CodSpeed has no counters, and warns about each one.
    #[cfg(not(codspeed))]
    let bencher = bencher.counter(divan::counter::ItemsCount::new(repo.files.len()));

    bencher.bench(|| repo.files.iter().filter(|path| !codeowners.is_owned(path)).count());
}
