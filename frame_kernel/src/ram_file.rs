use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// a file stored in ram instead of a hard drive
/// used as a placeholder for things that require files while files are not yet implemented.
pub struct RAMFile {
    name: String, // just name, no need for a path because it is in ram not stored in a directory
    data: Vec<String>, // the "lines" of the ram file
}

impl RAMFile {
    pub fn new(name: String) -> RAMFile {
        RAMFile {
            name,
            data: Vec::new()
        }
    }

    /// writes data to the ram file (overwrites current data)
    pub fn write(&mut self, data: String) {
        self.data.clear();
        self.append(data);
    }

    /// writes data to the ram file (adds to end)
    pub fn append(&mut self, data: String) {
        for l in data.split("\n") {
            self.data.push(l.to_string());
        }
    }

    /// returns a vector of the "lines" in the ram file
    pub fn read(&self) -> Vec<String> {
        self.data.clone()
    }


}