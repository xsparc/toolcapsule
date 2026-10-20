//! Bounded process-local preparation reuse; every lookup verifies current authority.
use crate::{
    Error, Policy, TrustStore, VerifiedPackage,
    runtime::{PreparedTool, Runtime},
};
use std::{collections::VecDeque, sync::Arc};

struct Entry {
    tool: Arc<PreparedTool>,
    source_bytes: usize,
}

/// Least-recently-used prepared tools, with entry and retained source-byte limits.
/// JIT memory and externally retained Arcs are not covered by these bounds.
pub struct PreparationCache {
    runtime: Runtime,
    entries: VecDeque<Entry>,
    max_entries: usize,
    max_source_bytes: usize,
    source_bytes: usize,
}

impl PreparationCache {
    pub fn new(max_entries: usize, max_source_bytes: usize) -> Result<Self, Error> {
        if !(1..=64).contains(&max_entries) || !(1..=256 * 1024 * 1024).contains(&max_source_bytes)
        {
            return Err(Error::Policy);
        }
        Ok(Self {
            runtime: Runtime::new()?,
            entries: VecDeque::new(),
            max_entries,
            max_source_bytes,
            source_bytes: 0,
        })
    }

    /// Verify the exact envelope and component against the supplied current trust
    /// and policy BEFORE looking up cached code. Compilation is synchronous;
    /// call from the host's bounded preparation worker, outside guest deadlines.
    pub fn prepare(
        &mut self,
        envelope: &[u8],
        component: Vec<u8>,
        trust: &TrustStore,
        policy: &Policy,
    ) -> Result<Arc<PreparedTool>, Error> {
        let package = VerifiedPackage::verify(envelope, component, trust, policy)?;
        let size = package.component().len();
        if size > self.max_source_bytes {
            return Err(Error::Policy);
        }
        if let Some(index) = self.entries.iter().position(|entry| {
            entry.tool.manifest_sha256() == package.manifest_sha256()
                && entry.tool.signer() == package.signer()
        }) {
            let entry = self.entries.remove(index).ok_or(Error::Runtime)?;
            let tool = entry.tool.clone();
            self.entries.push_back(entry);
            return Ok(tool);
        }
        // Failed compilation does not evict useful entries. Peak compilation memory
        // therefore needs an external limit, as it does for Runtime::prepare.
        let tool = Arc::new(self.runtime.prepare(package)?);
        while self.entries.len() >= self.max_entries
            || self.source_bytes + size > self.max_source_bytes
        {
            let entry = self.entries.pop_front().ok_or(Error::Runtime)?;
            self.source_bytes -= entry.source_bytes;
        }
        self.source_bytes += size;
        self.entries.push_back(Entry {
            tool: tool.clone(),
            source_bytes: size,
        });
        Ok(tool)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn source_bytes(&self) -> usize {
        self.source_bytes
    }

    /// Drop cache-owned references. Previously returned tools remain usable;
    /// revoke those separately in the host when changing trust or policy.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.source_bytes = 0;
    }
}
