//! Definitions for [Member] and all related functionality. A [Member] is part
//! of a workspace and represents all of the associated information with a
//! single file that is being processed by the linting tool.

use std::path::PathBuf;

#[derive(Clone)]
pub struct Member {
    /// The fully canonicalised path of the member.
    pub path: PathBuf,

    /// The raw file contents of the member.
    pub contents: String,
}

impl Member {
    /// Create a new [Member] with the given contents.
    pub fn new(path: PathBuf, contents: String) -> Self {
        Member { path, contents }
    }
}

index_vec::define_index_type! {
    // Define StrIdx to use only 32 bits internally (you can use usize, u16,
    // and even u8).
    pub struct MemberId = u32;
}
