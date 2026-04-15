use std::path::Path;

use crate::rfclib::index;

/// Executes the `reindex` command: completely rebuilds the index from scratch
pub fn execute(project_root: &Path) -> Result<(), String> {
    let idx = index::rebuild_index(project_root)?;

    println!("Reindexed {} RFCs.", idx.rfcs.len());

    Ok(())
}
