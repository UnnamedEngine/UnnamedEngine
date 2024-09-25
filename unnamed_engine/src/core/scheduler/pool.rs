use std::{collections::HashSet, thread};

use crossbeam::channel::{unbounded, Receiver, Sender};
use strum::EnumCount;
use thiserror::Error;

use super::worker::{Worker, WorkerInstruction, WorkerKind, WorkerNotification};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    /// Failed to send instruction
    #[error("Failed to send instruction")]
    InstructionSendFail,
    /// This specialization already exists inside the `WorkerPool`.
    #[error("Specialized worker already exists for `{0}`")]
    SpecializationAlreadyExists(WorkerKind),
    /// Cannot specialize `WorkerKind::Generic` into another
    /// `WorkerKind::Generic`.
    #[error("Cannot specialize generic worker into another generic worker")]
    CannotSpecializeIntoGeneric,
    /// Cannot shrink number of workers to zero or less.
    #[error("Cannot srhink number of workers to zer or less")]
    CannotShrinkToZeroOrLess,
}

pub struct WorkerPool {
    /// All owned `Worker` instances.
    workers: Vec<Worker>,
    /// Helper to prevent a duplicate dedicated `Worker`.
    dedicated: HashSet<WorkerKind>,
    /// Used to send `WorkerInstruction` to first available
    /// `WorkerKind::Generic`.
    instruction_sender: Sender<WorkerInstruction>,
    /// A `Worker` uses a clone of this to receive instructions from the
    /// `WorkerPool`.
    ///
    /// _Stored to help modify amount of `Worker` instances._
    instruction_receiver: Receiver<WorkerInstruction>,
    /// Used to receive `WorkerNotification`.
    notification_receiver: Receiver<WorkerNotification>,
    /// A `Worker` uses a clone of this to communicate with the `WorkerPool`.
    ///
    /// _Stored to help modify amount of `Worker` instances._
    notification_sender: Sender<WorkerNotification>,
}

impl Default for WorkerPool {
    fn default() -> Self {
        let available_threads = match thread::available_parallelism() {
            Ok(available_threads) => available_threads.get(),
            Err(err) => {
                log::warn!(
                    "Failed to query available threads: {}. Assuming {} threads.",
                    err,
                    WorkerKind::COUNT,
                );
                WorkerKind::COUNT
            }
        };

        Self::new(available_threads - 1)
    }
}

impl WorkerPool {
    /// Creates a new `Pool` with the passed `size`.
    ///
    /// _If you want to create a `Pool` with maximum size, refer to
    /// `Pool::default()`_.
    pub fn new(size: usize) -> Self {
        if size < WorkerKind::COUNT {
            log::warn!(
                "Current hardware doesn't have the minimum amount of threads \
                required to run this application. Required {} found {}",
                WorkerKind::COUNT,
                size,
            );
        }

        let (instruction_sender, instruction_receiver) = unbounded();

        let (notification_sender, notification_receiver) = unbounded();

        // Initialize requested workers
        let workers = (0..size)
            .map(|id| Worker::new(
                id,
                instruction_receiver.clone(),
                notification_sender.clone(),
            ))
            .collect::<Vec<_>>();

        let dedicated = HashSet::default();

        Self {
            workers,
            dedicated,
            instruction_sender,
            instruction_receiver,
            notification_receiver,
            notification_sender,
        }
    }

    /// Grow workers with passed amount.
    ///
    /// Returns the final size.
    pub fn grow(&mut self, growth: usize) -> usize {
        let current_size = self.workers.len();
        let mut new_workers = (current_size..growth + current_size)
            .map(|id| Worker::new(
                id,
                self.instruction_receiver.clone(),
                self.notification_sender.clone(),
            ))
            .collect::<Vec<_>>();

        self.workers.append(&mut new_workers);

        self.workers.len()
    }

    /// Returns the amount of workers.
    pub fn len(&self) -> usize {
        self.workers.len()
    }

    /// Returns `true` if the `WorkerPool` contains no `Worker` instances.
    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    /// Send a instruction to be executed by the first `Worker` that finds it.
    pub fn send(&mut self, instruction: WorkerInstruction) -> Result<(), Error> {
        self.process_notifications();

        if let WorkerInstruction::Specialize(kind, _) = instruction {
            match kind {
                WorkerKind::Generic(_) => {
                    return Err(Error::CannotSpecializeIntoGeneric);
                },
                _ => {
                    if !self.dedicated.insert(kind) {
                        return Err(Error::SpecializationAlreadyExists(kind));
                    }
                },
            }
        }
        match self.instruction_sender.send(instruction) {
            Ok(_) => Ok(()),
            Err(_) => {
                Err(Error::InstructionSendFail)
            }
        }
    }

    /// Internal function that process all notifications received from workers.
    fn process_notifications(&mut self) {
        while let Ok(notification) = self.notification_receiver.try_recv() {
            if let WorkerNotification::SpecializedJobCompleted(kind) = notification {
                self.dedicated.remove(&kind);
                log::info!(
                    "Received notification that specialized worker '{}' has completed its job",
                    kind,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn create_pool() {
        let pool_size = WorkerKind::COUNT;
        let pool = WorkerPool::new(pool_size);
        assert!(pool.workers.len() == pool_size, "Incorrect pool size");
    }

    #[test]
    fn specialize() {
        let pool_size = WorkerKind::COUNT;
        let mut pool = WorkerPool::new(pool_size);
        let result = pool.send(WorkerInstruction::Specialize(WorkerKind::Dummy, Box::new(|| {
            thread::sleep(Duration::from_millis(100));
        })));

        assert!(result.is_ok(), "Failed to specialize worker");
    }

    #[test]
    fn specialize_duplicate() {
        let pool_size = WorkerKind::COUNT;
        let mut pool = WorkerPool::new(pool_size);
        let _ = pool.send(WorkerInstruction::Specialize(WorkerKind::Dummy, Box::new(|| {
            thread::sleep(Duration::from_millis(100));
        })));
        let result = pool.send(WorkerInstruction::Specialize(WorkerKind::Dummy, Box::new(|| {
            thread::sleep(Duration::from_millis(100));
        })));

        assert_eq!(
            result.err().unwrap(),
            Error::SpecializationAlreadyExists(WorkerKind::Dummy),
            "Should not be able to specialize same kind two times",
        );
    }

    #[test]
    fn specialize_duplicate_after_termination() {
        let pool_size = WorkerKind::COUNT;
        let mut pool = WorkerPool::new(pool_size);
        let _ = pool.send(WorkerInstruction::Specialize(WorkerKind::Dummy, Box::new(|| {
            thread::sleep(Duration::from_millis(100));
        })));

        // Just to be sure our specialized execution is completed
        thread::sleep(Duration::from_millis(110));

        let result = pool.send(WorkerInstruction::Specialize(WorkerKind::Dummy, Box::new(|| {
            thread::sleep(Duration::from_millis(100));
        })));

        assert!(
            result.is_ok(),
            "Failed to specialize same after termination: {}",
            result.err().unwrap(),
        );
    }

    #[test]
    fn pool_grow() {
        let pool_size = WorkerKind::COUNT;
        let mut pool = WorkerPool::new(pool_size);

        pool.grow(4);

        assert_eq!(
            pool.workers.len(),
            WorkerKind::COUNT + 4,
            "Pool grow is not adding workers",
        );
    }
}
