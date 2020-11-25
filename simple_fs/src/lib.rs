pub struct Folder {
    path: String,
}

impl Folder {
    pub fn new() -> Folder {}
    pub fn clone(&self) -> Folder {
        Folder {
            path: self.path.clone(),
        }
    }
    pub fn mkdir(&self) { // TODO: return error codes if fails or panic idk
                          // TODO: Implement
    }
    pub fn exists(&self) -> bool {
        false // TODO: Implement
    }
}

pub struct File {
    path: Folder,
    // path without the file name
    name: String,
    // name of the file
    full_path: String, // path + name
}

impl File {
    pub fn new(name: &str, loc: Folder) -> File {
        if !loc.exists() {
            // ensure the folder exists (if not we are creating)
            loc.mkdir();
        }

        let full = &loc.path + &name;
        let f = File {
            path: loc,
            name: name.to_string(),
            full_path: full,
        };

        if !f.exists() {
            f.create();
        }

        f
    }

    pub fn create(&self) { // TODO: return error codes if fails or panic idk
                           // TODO: Implement
    }

    pub fn exists(&self) -> bool {
        true // TODO: Implement
    }
}

#[no_mangle]
pub extern "C" fn get_number() -> isize {
    42 as isize
}

#[cfg(test)]
mod tests {
    use crate::Folder;

    #[test]
    fn it_works() {
        let root_folder: Folder = Folder::new();
        root_folder.mkdir();
    }
}
