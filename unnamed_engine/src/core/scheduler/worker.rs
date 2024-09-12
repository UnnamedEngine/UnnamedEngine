use std::{sync::{mpsc, Arc, Mutex}, thread::{self, JoinHandle}};

use strum::{Display, EnumCount};

use super::Job;

/// All the variations of a `Worker`. Specially useful for defining dedicated
/// workers that will only execute one big `Job`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Display, EnumCount)]
pub enum WorkerKind {
    /// Represents a generic `Worker` that can be used for anything, this kind
    /// is used to execute tasks in parallel, the more we can get the better.
    /// After a graceful initialization, generic workers can be transformed in
    /// dedicted workers, this can be achieved by using
    /// `WorkerInstruction::Specialize`.
    Generic(usize),
}

/// All the possible states a `Worker` can be at.
#[derive(Debug, PartialEq, Eq, Display, Clone, Copy)]
pub enum WorkerState {
    /// `Worker` is currently waiting for a new `Job`.
    Idle,
    /// `Worker` is currently executing a `Job`.
    Executing,
}

/// All the instructions a `Worker` can receive during its lifetime.
pub enum WorkerInstruction {
    /// `Worker` should wait for instructions.
    Wait,
    /// `Worker` should execute the passed `Job`.
    Execute(Job),
    /// `Worker` should terminate the current `Job` and break the main loop.
    Terminate,
    /// `Worker` should specialize itself with the passed `Job`.
    Specialize(WorkerKind, Job),
}

/// Wrapper for a `JoinHandle` that contains extra information to help manage
/// the thread.
pub struct Worker {
    /// Defines the kind of the current `Worker`.
    kind: Arc<Mutex<WorkerKind>>,
    /// Flags the current state of the `Worker`.
    state: Arc<Mutex<WorkerState>>,
    /// A handle for the thread this `Worker` is responsible for.
    thread: JoinHandle<()>,
    /// Used to receive instructions.
    receiver: Arc<Mutex<mpsc::Receiver<WorkerInstruction>>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<WorkerInstruction>>>) -> Self {
        let kind = Arc::new(Mutex::new(WorkerKind::Generic(id)));
        let state = Arc::new(Mutex::new(WorkerState::Idle));

        let kind_clone = Arc::clone(&kind);
        let state_clone = Arc::clone(&state);
        let receiver_clone = Arc::clone(&receiver);

        let thread = thread::spawn(move || {
            let kind = kind_clone.lock().unwrap().clone();
            loop {
                // Loop until we sucessfuully find a instruction
                let instruction;
                loop {
                    match receiver_clone.lock() {
                        Ok(receiver) => {
                            match receiver.recv() {
                                Ok(received_instruction) => {
                                    instruction = received_instruction;
                                    break;
                                },
                                Err(err) => {
                                    log::error!("Failed to received instruction at worker '{}': {}", &kind, err.to_string());
                                },
                            }
                        },
                        Err(err) => {
                            log::error!("Failed to lock() instruction receiver at worker '{}': {}", &kind, err.to_string());
                        },
                    }
                }

                match instruction {
                    WorkerInstruction::Wait => {
                        log::warn!("Worker '{}' instructed to wait: this is not normal behavior", &kind);
                        continue;
                    },
                    WorkerInstruction::Execute(job) => {
                        log::info!("Worker '{}' instructed to execute a task", &kind);

                        {
                            let mut state = state_clone.lock().unwrap();
                            *state = WorkerState::Executing;
                        }

                        job();

                        {
                            let mut state = state_clone.lock().unwrap();
                            *state = WorkerState::Idle;
                        }

                        log::info!("Worker '{}' finished the required job and is now idle", &kind);
                    },
                    // TODO: find a way to terminate dedicated workers
                    // TODO: allow terminated workers to be reused
                    WorkerInstruction::Terminate => {
                        log::info!("Worker '{}' instructed to terminate", &kind);
                        break;
                    },
                    WorkerInstruction::Specialize(specialization, job) => {
                        let mut kind = kind_clone.lock().unwrap();
                        let mut state = state_clone.lock().unwrap();
                        let kind_copy = kind.clone();
                        match kind_copy {
                            WorkerKind::Generic(id) => {
                                log::info!("Worker '{}' instructed to specialize into '{}'", &*kind, &specialization);
                                *kind = specialization;
                                *state = WorkerState::Executing;
                                job();
                                *state = WorkerState::Idle;
                                *kind = WorkerKind::Generic(id);
                                log::info!("Worker '{}' finished the specialized job, got converted back into a generic worker and is now idle", &kind);
                            },
                            _ => {
                                log::error!("Failed to specialize '{}': only a generic worker can be specialized", &*kind);
                            }
                        }
                    },
                }
            }
        });

        Self {
            kind,
            state,
            thread,
            receiver,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn create_worker() {
        let (_, receiver) = mpsc::channel();
        let worker = Worker::new(0, Arc::new(Mutex::new(receiver)));
        assert_eq!(*worker.kind.lock().unwrap(), WorkerKind::Generic(0));
        assert_eq!(*worker.state.lock().unwrap(), WorkerState::Idle);
    }

    #[test]
    fn worker_execute() {
        let (sender, receiver) = mpsc::channel();
        let worker = Worker::new(0, Arc::new(Mutex::new(receiver)));

        let _ = sender.send(WorkerInstruction::Execute(Box::new(|| {
            thread::sleep(Duration::from_secs(2));
        })));

        // Wait some time to make sure the job got started
        thread::sleep(Duration::from_secs(1));
        assert_eq!(*worker.state.lock().unwrap(), WorkerState::Executing);

        // Wait some time to make sure the job got completed
        thread::sleep(Duration::from_secs(3));
        assert_eq!(*worker.state.lock().unwrap(), WorkerState::Idle);

        // Terminate the worker
        let _ = sender.send(WorkerInstruction::Terminate);
    }

    #[test]
    fn worker_execute_multiple() {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..4).map(|id| {
            Worker::new(id, Arc::clone(&receiver))
        }).collect::<Vec<_>>();

        workers.iter().for_each(|_| {
            let _ = sender.send(WorkerInstruction::Execute(Box::new(|| {
                thread::sleep(Duration::from_secs(2));
            })));
        });

        // Wait some time to make sure the jobs got started
        thread::sleep(Duration::from_secs(1));
        workers.iter().for_each(|worker| {
            assert_eq!(*worker.state.lock().unwrap(), WorkerState::Executing);
        });

        // Wait some time to make sure the jobs got completed
        thread::sleep(Duration::from_secs(3));
        workers.iter().for_each(|worker| {
            assert_eq!(*worker.state.lock().unwrap(), WorkerState::Idle);
        });

        // Terminate the workers
        workers.iter().for_each(|_| {
            let _ = sender.send(WorkerInstruction::Terminate);
        });
    }

    #[test]
    fn worker_execute_more_than_available() {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let workers = (0..4).map(|id| {
            Worker::new(id, Arc::clone(&receiver))
        }).collect::<Vec<_>>();

        // Send double the amount of jobs
        (0..8).for_each(|_| {
            let _ = sender.send(WorkerInstruction::Execute(Box::new(|| {
                thread::sleep(Duration::from_secs(2));
            })));
        });

        // Wait some time to make sure the jobs got started
        thread::sleep(Duration::from_secs(1));
        workers.iter().for_each(|worker| {
            assert_eq!(*worker.state.lock().unwrap(), WorkerState::Executing);
        });

        // Wait some time to make sure the second wave of jobs got started
        thread::sleep(Duration::from_secs(3));
        workers.iter().for_each(|worker| {
            assert_eq!(*worker.state.lock().unwrap(), WorkerState::Executing);
        });

        // Wait some time to make sure all jobs got completed
        thread::sleep(Duration::from_secs(3));
        workers.iter().for_each(|worker| {
            assert_eq!(*worker.state.lock().unwrap(), WorkerState::Idle);
        });

        // Terminate the workers
        workers.iter().for_each(|_| {
            let _ = sender.send(WorkerInstruction::Terminate);
        });
    }
}
