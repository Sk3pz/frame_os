use alloc::sync::Arc;
use alloc::vec::Vec;

use crossbeam_queue::ArrayQueue;

pub struct Command {
    // TODO
}

impl Command {
    pub fn new() -> Command {
        Command {
            // TODO
        }
    }

    // TODO
}

pub struct CommandExecutor {
    command_queue: Arc<ArrayQueue<Command>>,
    running: bool
}

impl CommandExecutor {
    pub fn new() -> CommandExecutor {
        CommandExecutor {
            command_queue: Arc::new(ArrayQueue::new(100)), // 100 command limit
            running: false
        }
    }

    pub fn spawn(&mut self, cmd: Command) {
        self.command_queue.push(cmd).expect("command queue full!");
    }

    fn execute_ready_commands(&mut self) {
        while !self.command_queue.is_empty() {
            let current_cmd = self.command_queue.pop();

            // TODO
        }
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub async fn run(&mut self) {
        self.running = true;

        while self.running {
            self.execute_ready_commands();
        }
    }

}