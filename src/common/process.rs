use shared_child::SharedChild;
use std::sync::Arc;
use std::process::Command;
use crate::common::ApplicationError;

#[derive(Clone)]
pub struct ProcessHandle {
    process: Arc<SharedChild>,
}


impl ProcessHandle {
    pub fn new(mut command: &mut Command) -> Result<ProcessHandle, ApplicationError> {
        Ok(ProcessHandle {
            process: Arc::new(SharedChild::spawn(&mut command)?),
        })
    }

    pub fn kill(&self) {
        let _ = self.process.kill();
    }

    pub fn pid(&self) -> u32 {
        self.process.id()
    }


    pub fn is_running(&self) -> Result<(), ApplicationError> {
        let terminate_result = self.process.try_wait();
        match terminate_result {
            Ok(expected_status) => match expected_status {
                // Process already exited. Terminate was successful.
                Some(_status) => Ok(()),
                None => Err(ApplicationError::transport(format!(
                    "Process [pid={}] is still running.",
                    self.process.id()
                )))
            },
            Err(error) => Err(ApplicationError::transport(format!(
                "Failed to wait for process [pid={}]. Error: {}",
                self.process.id(),
                error
            )))
        }
    }

}