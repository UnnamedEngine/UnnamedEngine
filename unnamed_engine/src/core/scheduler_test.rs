// TODO: verify the use of Arc<Mutex> and try to find a better implementation
use std::{collections::{HashMap, VecDeque}, sync::{mpsc, Arc, Mutex}, thread::{self, JoinHandle}};

use strum::{Display, EnumCount};

/// Helper that defines a `FnOnce` that will be sent to the `ThreadPool` and
/// executed by a `Worker`.
pub type Job = Box<dyn FnOnce() + Send + 'static>;

/// All the variations of a `Worker`. Specially useful for defining dedicated
/// workers that will only execute one big `Job`.
#[derive(Debug, Clone, Copy, Display, EnumCount)]
pub enum WorkerKind {
    /// Represents a generic `Worker` that can be used for anything, this kind
    /// is used to execute tasks in parallel, the more we can get the better.
    /// After a graceful initialization, generic workers can be transformed in
    /// dedicted workers, this can be achieved by using
    /// `WorkerInstruction::Specialize`.
    Generic(usize),
}

/// All the variations of a `Task`.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Display)]
pub enum TaskKind {
    /// Task that will be directed to `WorkerKind::Generic`.
    Generic,
}

/// Specifies a `Job` and how to run it.
pub struct Task {
    /// Defines what `Job` will be executed.
    pub job: Job,
    /// Defines what kind of `Worker` is responsible for this task.
    pub kind: TaskKind,
}

/// All the instructions a `Worker` can receive during its lifetime.
pub enum WorkerInstruction {
    /// `Worker` should wait for instructions.
    Wait,
    /// `Worker` should execute the next valid `Task`.
    ExecuteTask,
    /// `Worker` should terminate the current `Task` and break the main loop.
    Terminate,
    /// `Worker` should specialize itself and initialize a new main loop.
    Specialize(WorkerKind),
}

/// All the possible states a `Worker` can be at.
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    /// `Worker` is currently waiting for a new `Task`.
    Idle,
    /// `Worker` is currently executing a `Task`.
    Executing,
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
    /// Generic `Worker` use this to receive instructions.
    receiver: Arc<Mutex<mpsc::Receiver<WorkerInstruction>>>,
}

pub struct WorkerInitializationDescriptor {
    /// All workers start as `WorkerKind::Generic` and thus need an unique id.
    id: usize,
    /// The receiver that
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<WorkerInstruction>>>) -> Self {
        let kind_arc = Arc::new(Mutex::new(WorkerKind::Generic(id)));
        let state_arc = Arc::new(Mutex::new(WorkerState::Idle));

        let kind_clone = Arc::clone(&kind_arc);
        let state_clone = Arc::clone(&state_arc);

        let thread = thread::spawn(move ||
            loop {
                // Loop until we successfully find a instruction
                let instruction;
                loop {
                    match receiver.lock() {
                        Ok(receiver) => {
                            match receiver.recv() {
                                Ok(received_instruction) => {
                                    instruction = received_instruction;
                                    break;
                                },
                                Err(err) => {
                                    log::error!("Failed to received instruction at worker '{}': {}", *kind_clone.lock().unwrap(), err.to_string());
                                },
                            }
                        },
                        Err(err) => {
                            log::error!("Failed to lock() instruction receiver at worker '{}': {}", *kind_clone.lock().unwrap(), err.to_string());
                        },
                    }
                }

                match instruction {
                    WorkerInstruction::Wait => {
                        log::warn!("Worker '{}' instructed to wait: this is not normal behavior", *kind_clone.lock().unwrap());
                        continue;
                    }
                    WorkerInstruction::ExecuteTask => {
                        {
                            log::info!("Worker '{}' instructed to execute a task", *kind_clone.lock().unwrap());
                        }

                        {
                            let mut state = state_clone.lock().unwrap();
                            *state = WorkerState::Executing;
                        }

                        (task.job)();

                        {
                            let mut state = state_clone.lock().unwrap();
                            *state = WorkerState::Idle;
                        }

                        {
                            log::info!("Worker '{}' finished the required task and is now idle", *kind_clone.lock().unwrap());
                        }
                    },
                    // TODO: allow terminated workers to be reused
                    WorkerInstruction::Terminate => {
                        log::info!("Worker '{}' instructed to terminate", *kind_clone.lock().unwrap());
                        break;
                    },
                    WorkerInstruction::Specialize(specialization) => {
                        let mut kind = kind_clone.lock().unwrap();
                        let state = state_clone.lock().unwrap();
                        match *kind {
                            WorkerKind::Generic(_) => {
                                match *state {
                                    WorkerState::Idle => {
                                        log::info!("Worker '{}' instructed to specalize into '{}'", *kind, &specialization);
                                        *kind = specialization;
                                    }
                                    _ => {
                                        log::error!("Failed to specialize '{}': a running worker cannot be specialized", *kind_clone.lock().unwrap());
                                    }
                                }
                            }
                            _ => {
                                log::error!("Failed to specialize '{}': only a generic worker can be specialized", *kind_clone.lock().unwrap());
                            }
                        }
                    },
                }
            }
        );

        Self {
            kind: kind_arc,
            state: state_arc,
            thread,
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    job_queue: Arc<Mutex<HashMap<TaskKind, VecDeque<Job>>>>,
    sender: mpsc::Sender<WorkerInstruction>,
}

impl ThreadPool {
    pub fn new() -> Self {
        let mut workers = Vec::default();
        let job_queue = Arc::new(Mutex::new(HashMap::new()));
        let (sender, receiver) = mpsc::channel();

        let available_threads = match thread::available_parallelism() {
            Ok(value) => {
                let value = value.get();
                if value < WorkerKind::COUNT {
                    log::warn!("Available threads are not sufficient: the engine requires {} threads, this machine appears to have {}", WorkerKind::COUNT, &value);
                    log::warn!("Performance will be significantly affected by this");
                }
                value
            },
            Err(err) => {
                log::warn!("Failed to get available threads: {}", err.to_string());
                log::warn!("Defaulting available threads to {}: performance might be affected", WorkerKind::COUNT);
                WorkerKind::COUNT
            },
        };

        for id in 0..available_threads {
            workers.push(Worker::new(id, receiver))
        }

        Self {
            workers,
            job_queue,
            sender,
        }
    }

    /// Method to submit a task to the `ThreadPool`.
    pub fn submit_task(&mut self, task: Task) {
        let job_queue = Arc::clone(&self.job_queue);
        let mut queue_guard = job_queue.lock().unwrap();
        queue_guard
            .entry(task.kind)
            .or_insert(VecDeque::default())
            .push_back(task.job);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
}
