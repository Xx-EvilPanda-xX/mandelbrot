use std::fmt;
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
}

pub struct Worker {
    thread: Option<thread::JoinHandle<Vec<u8>>>,
    id: u32,
}

impl ThreadPool {
    pub fn new(num_workers: u32) -> ThreadPool {
        let mut pool = ThreadPool {
            workers: Vec::with_capacity(num_workers as usize),
        };

        for i in 0..num_workers {
            pool.workers.push(Worker {
                thread: None,
                id: i,
            });
        }

        pool
    }

    pub fn run_job<F>(&mut self, job: F) -> Result<u32, PoolError>
        where F: FnOnce() -> Vec<u8> + Send + 'static
    {
        for w in &mut self.workers {
            match w.thread {
                Some(_) => continue,
                None => {
                    w.thread = Some(thread::spawn(job));
                    return Ok(w.id);
                }
            }
        }

        Err(PoolError {
            why: String::from("No more threads available"),
        })
    }

    pub fn join_all(&mut self, dims: (u32, u32), thread_ids: &[u32]) -> Result<Vec<u8>, PoolError> {
        let mut out_buf = vec![0; dims.0 as usize * dims.1 as usize * 3];
        let mut index = 0;

        for thread_id in thread_ids {
            let handle =
                self
                .workers
                .iter_mut()
                .find(|Worker { id, .. }| id == thread_id)
                .ok_or(PoolError { why: String::from("no such render thread") })?
                .thread
                .take()
                .ok_or(PoolError { why: String::from("render thread already killed") })?;

            match handle.join() {
                Ok(ret) => {
                    for pix in ret {
                        out_buf[index] = pix;
                        index += 1;
                    }
                }
                Err(_) => {
                    return Err(PoolError {
                        why: String::from("failed to join thread"),
                    });
                }
            }
        }

        if index > 0 {
            Ok(out_buf)
        } else {
            Err(PoolError {
                why: String::from("no threads running"),
            })
        }
    }
}

#[derive(Debug)]
pub struct PoolError {
    why: String,
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "JobError: {}", self.why)?;
        Ok(())
    }
}
