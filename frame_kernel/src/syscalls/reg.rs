

pub enum Register {
    R15 = 0, R14 = 1, R13 = 2, R12 = 3, RBP = 4,
    RBX = 5, R11 = 6, R10 = 7, R9 = 8, R8 = 9, RAX = 10,
    RCX = 11, RDX = 12, RSI = 13, RDI = 14,

    ORIG_RAX = 15, RIP = 16, CS = 17, EFLAGS = 18,
    RSP = 19, SS = 20, FS_BASE = 21, GS_BASE = 22,
    DS = 23, ES = 24, FS = 25, GS = 26,
}