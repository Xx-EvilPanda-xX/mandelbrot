use std::thread;
use std::error;
use std::fmt;

type Job = Box<dyn FnOnce() -> Vec<u8> + Send + 'static>;
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
            pool.workers.push( Worker {
                thread: None,
                id: i,
            });
        }

        pool
    }

    pub fn run_job(&mut self, job: Job) -> Result<u32, PoolError> {
        for w in &mut self.workers {
            match w.thread {
                Some(_) => continue,
                None => {
                    w.thread = Some(thread::spawn(job));
                    return Ok(w.id);
                }
            }
        };

        Err(PoolError { why: String::from("No more threads available") })
    }

    pub fn join_all(self, dims: (u32, u32)) -> Result<Vec<u8>, PoolError> {
        let mut out_buf = vec![0; dims.0 as usize * dims.1 as usize * 3];
        let mut index = 0;

        for w in self.workers {
            if let Some(handle) = w.thread {
                match handle.join() {
                    Ok(ret) => {
                        for pix in ret {
                            out_buf[index] = pix;
                            index += 1;
                        }
                    },
                    Err(_) => return Err(PoolError { why: String::from("Failed to join thread") })
                }
            }
        }

        if index > 0 {
            Ok(out_buf)
        } else {
            Err(PoolError { why: String::from("No threads running") })
        }
    }
}

#[derive(Debug)]
pub struct PoolError {
    why: String
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), std::fmt::Error> { 
        write!(f, "JobError: {}", self.why)?;
        Ok(())
    }
}
