use alloc::string::String;

pub trait WriteChannel {
    fn write(&self, data: String);
}

pub struct ChannelSTDOUT {
    // TODO
}

impl WriteChannel for ChannelSTDOUT {
    fn write(&self, data: String) {
        println!(data);
    }
}

pub struct ChannelSTDIN {
    // TODO
}

impl WriteChannel for ChannelSTDIN {
    fn write(&self, data: String) {
        println!(data);
    }
}

pub struct ChannelSTDERR {
    // TODO
}

impl WriteChannel for ChannelSTDERR {
    fn write(&self, data: String) {
        println!(data);
    }
}