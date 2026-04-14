// crabjar/agent-core/filesystem/src/lib.rs

pub mod filesystem;
pub mod in_memory;

pub use filesystem::{DirEntry, FileStat, FileSystem};
pub use in_memory::InMemoryFs;
