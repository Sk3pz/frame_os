

pub enum SupportedOS {
    FRAMEOS, // TODO: LINUX, WINDOWS
}

pub enum CallType { // TODO: Might not handle this this way.
    // Todo...
}

pub unsafe extern "C" fn syscall(t: CallType, mut args: ...) {}