//! Bounded invocation batches for one prepared trust/policy snapshot.
use crate::{
    Error,
    runtime::{PreparedTool, RunResult},
};
use std::{future::Future, sync::Arc};
use tokio::task::JoinSet;

pub const MAX_INVOCATIONS: usize = 64;
pub const MAX_CONCURRENCY: usize = 16;
pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;

/// Input and public deterministic fixture seed for one fresh guest instance.
pub struct Invocation {
    pub input: Vec<u8>,
    pub seed: u64,
}

/// Execute at most 64 invocations, returning results in input order.
///
/// All input and worst-case captured-output budgets are checked before any guest
/// starts. Concurrency is per batch, not a global quota. Each invocation retains
/// its own fuel/deadline; queued time is outside that deadline. Drop this future
/// to abort outstanding tasks cooperatively; queued items are never spawned.
/// Compilation, total process memory and concurrent callers require host limits.
pub async fn run_batch(
    tool: Arc<PreparedTool>,
    invocations: Vec<Invocation>,
    concurrency: usize,
) -> Result<Vec<RunResult>, Error> {
    if invocations.len() > MAX_INVOCATIONS || !(1..=MAX_CONCURRENCY).contains(&concurrency) {
        return Err(Error::Policy);
    }
    if invocations.is_empty() {
        return Ok(vec![]);
    }
    let limits = &tool.manifest().limits;
    let mut input_bytes = 0usize;
    for invocation in &invocations {
        if invocation.input.len() > limits.input_bytes {
            return Err(Error::InputLimit);
        }
        input_bytes = input_bytes
            .checked_add(invocation.input.len())
            .ok_or(Error::Policy)?;
    }
    let output_bytes = limits
        .output_bytes
        .checked_mul(2)
        .and_then(|size| size.checked_mul(invocations.len()))
        .ok_or(Error::Policy)?;
    if input_bytes > MAX_INPUT_BYTES || output_bytes > MAX_OUTPUT_BYTES {
        return Err(Error::Policy);
    }
    schedule(invocations, concurrency, move |invocation| {
        let tool = Arc::clone(&tool);
        async move { tool.run(&invocation.input, invocation.seed).await }
    })
    .await
}

async fn schedule<T, F, Fut>(
    items: Vec<T>,
    concurrency: usize,
    operation: F,
) -> Result<Vec<RunResult>, Error>
where
    T: Send + 'static,
    F: Fn(T) -> Fut,
    Fut: Future<Output = Result<RunResult, Error>> + Send + 'static,
{
    let mut results: Vec<Option<RunResult>> = (0..items.len()).map(|_| None).collect();
    let mut pending = items.into_iter().enumerate();
    let mut running = JoinSet::new();
    loop {
        while running.len() < concurrency {
            let Some((index, item)) = pending.next() else {
                break;
            };
            let future = operation(item);
            running.spawn(async move { (index, future.await) });
        }
        let Some(completed) = running.join_next().await else {
            break;
        };
        match completed {
            Ok((index, Ok(result))) => results[index] = Some(result),
            failed => {
                running.shutdown().await;
                return Err(match failed {
                    Ok((_, Err(error))) => error,
                    _ => Error::Runtime,
                });
            }
        }
    }
    results
        .into_iter()
        .map(|result| result.ok_or(Error::Runtime))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Outcome;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    struct Active(Arc<AtomicUsize>);
    impl Drop for Active {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }
    fn result(index: usize) -> RunResult {
        RunResult {
            outcome: Outcome::Success,
            stdout: vec![index as u8],
            stderr: vec![],
            fuel_consumed: 1,
        }
    }
    #[tokio::test]
    async fn concurrency_is_bounded_and_completion_is_reordered() {
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let values = schedule((0..12).collect(), 3, |index| {
            let active = active.clone();
            let maximum = maximum.clone();
            async move {
                let count = active.fetch_add(1, Ordering::SeqCst) + 1;
                let _guard = Active(active);
                maximum.fetch_max(count, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis((12 - index) as u64)).await;
                Ok(result(index))
            }
        })
        .await
        .unwrap();
        assert_eq!(maximum.load(Ordering::SeqCst), 3);
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert_eq!(
            values.iter().map(|r| r.stdout[0]).collect::<Vec<_>>(),
            (0..12).collect::<Vec<_>>()
        );
    }
    #[tokio::test]
    async fn dropping_batch_aborts_running_tasks_without_starting_queue() {
        let active = Arc::new(AtomicUsize::new(0));
        let started = Arc::new(AtomicUsize::new(0));
        let batch = schedule((0..12).collect(), 2, |_| {
            let active = active.clone();
            let started = started.clone();
            async move {
                active.fetch_add(1, Ordering::SeqCst);
                started.fetch_add(1, Ordering::SeqCst);
                let _guard = Active(active);
                std::future::pending::<Result<RunResult, Error>>().await
            }
        });
        assert!(
            tokio::time::timeout(Duration::from_millis(20), batch)
                .await
                .is_err()
        );
        for _ in 0..5 {
            tokio::task::yield_now().await;
        }
        assert_eq!(started.load(Ordering::SeqCst), 2);
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }
    #[tokio::test]
    async fn task_failure_cleans_up_siblings() {
        let active = Arc::new(AtomicUsize::new(0));
        let value = schedule(vec![0, 1], 2, |index| {
            let active = active.clone();
            async move {
                active.fetch_add(1, Ordering::SeqCst);
                let _guard = Active(active);
                if index == 0 {
                    tokio::task::yield_now().await;
                    return Err(Error::Runtime);
                }
                std::future::pending::<Result<RunResult, Error>>().await
            }
        })
        .await;
        assert!(matches!(value, Err(Error::Runtime)));
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn task_panic_is_sanitized_and_queue_is_not_started() {
        let started = Arc::new(AtomicUsize::new(0));
        let value = schedule(vec![0, 1, 2], 2, |index| {
            let started = started.clone();
            async move {
                started.fetch_add(1, Ordering::SeqCst);
                if index == 0 {
                    tokio::task::yield_now().await;
                    panic!("synthetic task panic");
                }
                std::future::pending::<Result<RunResult, Error>>().await
            }
        })
        .await;
        assert!(matches!(value, Err(Error::Runtime)));
        assert_eq!(started.load(Ordering::SeqCst), 2);
    }
}
