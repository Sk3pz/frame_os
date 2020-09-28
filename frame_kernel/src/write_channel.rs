use alloc::string::String;

use crate::println;

pub const stdout: ChannelSTDOUT = ChannelSTDOUT {};
pub const stdin: ChannelSTDIN = ChannelSTDIN {};
pub const stderr: ChannelSTDERR = ChannelSTDERR {};

pub trait WriteChannel {
    fn write(&self, data: &str);
}

pub struct ChannelSTDOUT {
    // TODO
}

impl WriteChannel for ChannelSTDOUT {
    fn write(&self, data: &str) {
        println!("{}", data);
    }
}

pub struct ChannelSTDIN {
    // TODO
}

impl WriteChannel for ChannelSTDIN {
    fn write(&self, data: &str) {
        println!("ON STDIN: {}", data);
    }
}

pub struct ChannelSTDERR {
    // TODO
}

impl WriteChannel for ChannelSTDERR {
    fn write(&self, data: &str) {
        println!("ON STDERR: {}", data);
    }
}