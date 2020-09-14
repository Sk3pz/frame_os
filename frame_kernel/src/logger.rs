use alloc::string::{String, ToString};

use crate::println;
use crate::write_channel::WriteChannel;

pub struct Logger {
    channel: dyn WriteChannel,
    pub show_debug: bool,
    pub show_verbose: bool,
    pub show_info: bool,
    pub show_warn: bool,
    pub show_err: bool,
    pub show_wtf: bool
}

impl Logger {

    pub fn new<C: WriteChannel>(channel: &C) -> Logger {
        Logger {
            channel,
            show_debug: true,
            show_verbose: true,
            show_info: true,
            show_warn: true,
            show_err: true,
            show_wtf: true
        }
    }

    pub fn set_channel<C: WriteChannel>(&mut self, channel: &C) {
        self.channel = channel;
    }

    pub fn debug(&self, data: &str) {
        if !self.show_debug {
            return;
        }
        self.channel.write("&8[DEBUG] > " + data);
    }

    pub fn verbose(&self, data: &str) {
        if !self.show_verbose {
            return;
        }
        self.channel.write("&8[&7VERBOSE&8] &7> &7" + data);
    }

    pub fn info(&self, data: &str) {
        if !self.show_info {
            return;
        }
        self.channel.write("&8[&bINFO&8] &7> &f" + data);
    }

    pub fn warn(&self, data: &str) {
        if !self.show_warn {
            return;
        }
        self.channel.write("&8[&eWARN&8] &7> &e" + data);
    }

    pub fn error(&self, data: &str) {
        if !self.show_err {
            return;
        }
        self.channel.write("&8[&cERROR&8] &7> &c" + data);
    }

    pub fn wtf(&self, data: &str) {
        if !self.show_wtf {
            return;
        }
        self.channel.write("&8[&4FAILURE&8] &7> &c" + data);
    }
}