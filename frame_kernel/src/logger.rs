use alloc::string::{String, ToString};

use crate::println;

pub enum LogChannel { // TODO: Update when files are added
    STDOUT, // set to println!() until files implemented
    STDERR, // set to println!() until files implemented
    STDIN,  // set to println!() until files implemented
}

pub struct Logger {
    channel: LogChannel,
    pub show_debug: bool,
    pub show_verbose: bool,
    pub show_info: bool,
    pub show_warn: bool,
    pub show_err: bool,
    pub show_wtf: bool
}

impl Logger {

    pub fn new(chan: LogChannel) -> Logger {
        Logger {
            channel: chan,
            show_debug: true,
            show_verbose: true,
            show_info: true,
            show_warn: true,
            show_err: true,
            show_wtf: true
        }
    }

    pub fn set_channel(&mut self, chan: LogChannel) {
        match chan {
            LogChannel::STDOUT => {
                // todo: all changing code to go to stdout

                self.channel = LogChannel::STDOUT;
            }
            LogChannel::STDIN => {
                // todo: all changing code to go to stdin

                self.channel = LogChannel::STDIN;
            }
            LogChannel::STDERR => {
                // todo: all changing code to go to stderr

                self.channel = LogChannel::STDERR;
            }
        }
    }

    pub fn debug(&self, data: &str) {
        if !self.show_debug {
            return;
        }
        match self.channel {
            LogChannel::STDOUT => println!("&8[DEBUG] > {}", data),
            LogChannel::STDIN  => println!("&8[DEBUG] > {}", data),
            LogChannel::STDERR => println!("&8[DEBUG] > {}", data)
        }
    }

    pub fn verbose(&self, data: &str) {
        if !self.show_verbose {
            return;
        }
        match self.channel {
            LogChannel::STDOUT => println!("&8[&7VERBOSE&8] &7> &7{}", data),
            LogChannel::STDIN  => println!("&8[&7VERBOSE&8] &7> &7{}", data),
            LogChannel::STDERR => println!("&8[&7VERBOSE&8] &7> &7{}", data)
        }
    }

    pub fn info(&self, data: &str) {
        if !self.show_info {
            return;
        }
        match self.channel {
            LogChannel::STDOUT => println!("&8[&bINFO&8] &7> &f{}", data),
            LogChannel::STDIN  => println!("&8[&bINFO&8] &7> &f{}", data),
            LogChannel::STDERR => println!("&8[&bINFO&8] &7> &f{}", data)
        }
    }

    pub fn warn(&self, data: &str) {
        if !self.show_warn {
            return;
        }
        match self.channel {
            LogChannel::STDOUT => println!("&8[&eWARN&8] &7> &e{}", data),
            LogChannel::STDIN  => println!("&8[&eWARN&8] &7> &e{}", data),
            LogChannel::STDERR => println!("&8[&eWARN&8] &7> &e{}", data)
        }
    }

    pub fn error(&self, data: &str) {
        if !self.show_err {
            return;
        }
        match self.channel {
            LogChannel::STDOUT => println!("&8[&cERROR&8] &7> &c{}", data),
            LogChannel::STDIN  => println!("&8[&cERROR&8] &7> &c{}", data),
            LogChannel::STDERR => println!("&8[&cERROR&8] &7> &c{}", data)
        }
    }

    pub fn wtf(&self, data: &str) {
        if !self.show_wtf {
            return;
        }
        match self.channel {
            LogChannel::STDOUT => println!("&8[&4FAILURE&8] &7> &c{}", data),
            LogChannel::STDIN  => println!("&8[&4FAILURE&8] &7> &c{}", data),
            LogChannel::STDERR => println!("&8[&4FAILURE&8] &7> &c{}", data)
        }
    }
}