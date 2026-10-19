//! Shared process-local admission. Clone a pool to share its limit across callers.
use crate::{
    Error, batch,
    runtime::{PreparedTool, RunResult},
};
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// No internal waiting queue. Saturated calls return `Error::Busy` before execution.
/// Separate pool instances have separate limits; callers must route all work through
/// the shared instance. This is not authentication, a rate limit, or an RSS bound.
#[derive(Clone)]
pub struct ExecutionPool {
    slots: Arc<Semaphore>,
}

impl ExecutionPool {
    /// Allow between one and 64 simultaneous invocations across this pool's clones.
    pub fn new(capacity: usize) -> Result<Self, Error> {
        if !(1..=64).contains(&capacity) {
            return Err(Error::Policy);
        }
        Ok(Self {
            slots: Arc::new(Semaphore::new(capacity)),
        })
    }

    fn reserve(&self, count: usize) -> Result<OwnedSemaphorePermit, Error> {
        self.slots
            .clone()
            .try_acquire_many_owned(count as u32)
            .map_err(|_| Error::Busy)
    }

    pub async fn run(
        &self,
        tool: &PreparedTool,
        input: &[u8],
        seed: u64,
    ) -> Result<RunResult, Error> {
        if input.len() > tool.manifest().limits.input_bytes {
            return Err(Error::InputLimit);
        }
        let _permit = self.reserve(1)?;
        tool.run(input, seed).await
    }

    /// Reserve the whole batch window atomically or start nothing. The reservation
    /// remains owned by running tasks until completion or cancellation cleanup.
    /// Each batch also obeys the byte/count limits in `batch::run_batch`.
    pub async fn run_batch(
        &self,
        tool: Arc<PreparedTool>,
        invocations: Vec<batch::Invocation>,
        concurrency: usize,
    ) -> Result<Vec<RunResult>, Error> {
        batch::validate(&tool, &invocations, concurrency)?;
        if invocations.is_empty() {
            return Ok(vec![]);
        }
        let permit = Arc::new(self.reserve(concurrency.min(invocations.len()))?);
        batch::schedule(invocations, concurrency, move |invocation| {
            let tool = tool.clone();
            let permit = permit.clone();
            async move {
                let _permit = permit;
                tool.run(&invocation.input, invocation.seed).await
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clones_share_atomic_reservations_and_independent_pools_do_not() {
        assert!(ExecutionPool::new(0).is_err());
        assert!(ExecutionPool::new(65).is_err());
        let pool = ExecutionPool::new(3).unwrap();
        let clone = pool.clone();
        let two = pool.reserve(2).unwrap();
        assert!(matches!(clone.reserve(2), Err(Error::Busy)));
        let one = clone.reserve(1).unwrap();
        assert!(matches!(pool.reserve(1), Err(Error::Busy)));
        assert!(ExecutionPool::new(1).unwrap().reserve(1).is_ok());
        drop(two);
        drop(one);
        assert!(clone.reserve(3).is_ok());
    }
}
