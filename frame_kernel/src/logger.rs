use alloc::string::ToString;

use crate::println;
use crate::write_channel::WriteChannel;

pub struct Logger<'a, C: WriteChannel> {
    channel: &'a C,
    pub show_debug: bool,
    pub show_verbose: bool,
    pub show_info: bool,
    pub show_warn: bool,
    pub show_err: bool,
    pub show_wtf: bool
}

impl<'a, C: WriteChannel> Logger<'a, C> {

    pub fn new(channel: &C) -> Logger<C> {
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

    pub fn debug(&self, data: &str) {
        if !self.show_debug {
            return;
        }
        self.channel.write(&("&8[D] > ".to_string() + data));
    }

    pub fn verbose(&self, data: &str) {
        if !self.show_verbose {
            return;
        }
        self.channel.write(&("&8[&7V&8] &7> &7".to_string() + data));
    }

    pub fn info(&self, data: &str) {
        if !self.show_info {
            return;
        }
        self.channel.write(&("&8[&bI&8] &7> &f".to_string() + data));
    }

    pub fn warn(&self, data: &str) {
        if !self.show_warn {
            return;
        }
        self.channel.write(&("&8[&eW&8] &7> &e".to_string() + data));
    }

    pub fn error(&self, data: &str) {
        if !self.show_err {
            return;
        }
        self.channel.write(&("&8[&cE&8] &7> &c".to_string() + data));
    }

    pub fn wtf(&self, data: &str) {
        if !self.show_wtf {
            return;
        }
        self.channel.write(&("&8[&4!&8] &7> &4".to_string() + data));
    }
}