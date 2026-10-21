//! Explicit local catalog admission, completed before any compilation or serving.
use crate::{
    Error, Policy, TrustStore, VerifiedPackage,
    mcp::MAX_TOOLS,
    package::MAX_ENVELOPE_BYTES,
    runtime::{PreparedTool, Runtime},
    storage::{read_file, read_json},
};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

pub const MAX_CATALOG_SOURCE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema_version: u32,
    packages: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    manifest: PathBuf,
    component: PathBuf,
}

/// Paths in the operator-supplied catalog resolve relative to its parent.
/// All entries share one explicit authority snapshot. No discovery or download.
pub fn prepare_catalog(
    path: &Path,
    trust: &TrustStore,
    policy: &Policy,
) -> Result<Vec<PreparedTool>, Error> {
    let catalog: Catalog = read_json(path, 64 * 1024)?;
    if catalog.schema_version != 1
        || catalog.packages.is_empty()
        || catalog.packages.len() > MAX_TOOLS
    {
        return Err(Error::Format);
    }
    let parent = path.parent().ok_or(Error::Format)?;
    let mut remaining = MAX_CATALOG_SOURCE_BYTES;
    let mut names = BTreeSet::new();
    let mut packages = Vec::new();
    for entry in catalog.packages {
        let component = read_file(
            &parent.join(entry.component),
            remaining.min(policy.max_component_bytes),
        )?;
        remaining -= component.len();
        let package = VerifiedPackage::verify(
            &read_file(&parent.join(entry.manifest), MAX_ENVELOPE_BYTES)?,
            component,
            trust,
            policy,
        )?;
        if !names.insert(package.manifest().name.clone()) {
            return Err(Error::Format);
        }
        packages.push(package);
    }
    let runtime = Runtime::new()?;
    packages
        .into_iter()
        .map(|package| runtime.prepare(package))
        .collect()
}
