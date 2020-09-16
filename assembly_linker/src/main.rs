use std::fs::File;
use std::io::{repeat, Read, Write};
use std::ops::Sub;
use std::process::{Command, Output};

fn calculate_sector_total(
    start_c: u8,
    start_h: u8,
    start_s: u8,
    end_c: u8,
    end_h: u8,
    end_s: u8,
) -> u32 {
    let mut result: u32 = 0;
    let mut c_head = start_h;
    let mut c_sector = start_s;
    let mut c_cylinder = start_c;
    while c_head < end_h || c_sector < end_s || c_cylinder < end_c {
        result += 1;
        c_sector += 1;
        if c_sector == 64 {
            c_sector = 1;
            c_head += 1;
            if c_head == 255 {
                c_head = 0;
                c_cylinder += 1;
            }
        }
    }
    result += 1;
    result
}

#[derive(Clone, Copy)]
struct Partition {
    status: u8,
    start_cylinder: u8,
    start_head: u8,
    start_sector: u8,
    p_type: u8,
    end_cylinder: u8,
    end_head: u8,
    end_sector: u8,
    //lba: u32, //todo calculate this
}

impl Partition {
    fn total_sectors(&self) -> u32 {
        if self.status == 0x00 {
            return 0x00;
        }
        calculate_sector_total(
            self.start_cylinder,
            self.start_head,
            self.start_sector,
            self.end_cylinder,
            self.end_head,
            self.end_sector,
        )
    }
    fn start_lba(&self) -> u32 {
        if self.status == 0x00 {
            return 0x00;
        }
        calculate_sector_total(
            0,
            0,
            2,
            self.start_cylinder,
            self.start_head,
            self.start_sector,
        )
    }

    fn print(&self) {
        println!("\tStatus: {}\n\tFirst:\n\t\tHead: {}\n\t\tSector: {}\n\t\tCylinder: {}\n\tType: {}\n\tLast:\n\t\tHead: {}\n\t\tSector: {}\n\t\tCylinder: {}\n\tLBA: {}\n\tSectors in partition: {}",
                 self.status,
                 self.start_head,
                 self.start_sector,
                 self.start_cylinder,
                 self.p_type,
                 self.end_head,
                 self.end_sector,
                 self.end_cylinder,
                 self.start_lba(),
                 self.total_sectors())
    }

    fn write(&self, vector: &mut Vec<u8>) {
        vector.push(self.status);
        vector.push(self.start_head);
        vector.push(self.start_sector);
        vector.push(self.start_cylinder);
        vector.push(self.p_type);
        vector.push(self.end_head);
        vector.push(self.end_sector);
        vector.push(self.end_cylinder);
        let integer = &self.start_lba().to_le_bytes();
        vector.write(integer).unwrap();
        let integer2 = &self.total_sectors().to_le_bytes();
        vector.write(integer2).unwrap();
    }

    fn read(vector: &mut Vec<u8>) -> Partition {
        let status = vector.remove(0);
        if status == 0x00 {
            let part = Partition {
                status,
                start_cylinder: 0,
                start_head: 0,
                start_sector: 0,
                p_type: 0,
                end_cylinder: 0,
                end_head: 0,
                end_sector: 0,
            };
            vector.drain(0..15);
            part
        } else {
            let part = Partition {
                status,
                start_head: vector.remove(0),
                start_sector: vector.remove(0),
                start_cylinder: vector.remove(0),
                p_type: vector.remove(0),
                end_head: vector.remove(0),
                end_sector: vector.remove(0),
                end_cylinder: vector.remove(0),
            };
            vector.resize(vector.len() - 8, 0x00);
            part
        }
    }
}

struct MBR {
    bootstrap: [u8; 440],
    signature: u32,
    copy_protected: bool,
    partition_1: Partition,
    partition_2: Partition,
    partition_3: Partition,
    partition_4: Partition,
}

impl MBR {
    fn print(&self) {
        println!("Copy_Protected: {}", self.copy_protected);
        if self.partition_1.status == 0x00 {
            println!("No Partitions");
            return;
        }
        println!("P1: ");
        self.partition_1.print();
        if self.partition_2.status == 0x00 {
            return;
        }
        println!("P2: ");
        self.partition_2.print();
        if self.partition_3.status == 0x00 {
            return;
        }
        println!("P3: ");
        self.partition_3.print();
        if self.partition_4.status == 0x00 {
            return;
        }
        println!("P4: ");
        self.partition_4.print();
    }

    fn write(&self, vector: &mut Vec<u8>) {
        vector.write(&self.bootstrap).unwrap();
        let integer = &self.signature.to_le_bytes();
        vector.write(integer).unwrap();
        let num = if self.copy_protected { 0x5A } else { 0x00 };
        vector.push(num);
        vector.push(num);
        self.partition_1.write(vector);
        self.partition_2.write(vector);
        self.partition_3.write(vector);
        self.partition_4.write(vector);
        vector.push(0x55);
        vector.push(0xAA);
    }

    fn read(vector: &mut Vec<u8>) -> MBR {
        let mut bootstrap = [0; 440];
        for i in 0..440 {
            bootstrap[i] = vector.remove(0);
        }
        let signature = as_u32_le(
            vector.remove(0),
            vector.remove(0),
            vector.remove(0),
            vector.remove(0),
        );
        let cp_a = vector.remove(0);
        let cp_b = vector.remove(0);
        let copy_protected = cp_a == 0x5A && cp_b == 0x5A;
        let mut mbr = MBR {
            bootstrap,
            signature,
            copy_protected,
            partition_1: Partition::read(vector),
            partition_2: Partition::read(vector),
            partition_3: Partition::read(vector),
            partition_4: Partition::read(vector),
        };
        mbr
    }
}

struct CommandLine {
    base: String,
    args: String,
    windows: bool,
}

impl CommandLine {
    fn run(&self) -> Output {
        return if self.windows {
            Command::new("cmd")
                .args(&["/C", format!("{} {}", self.base, self.args).as_str()])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(format!("{} {}", self.base, self.args).as_str())
                .output()
                .expect("failed to execute process")
        };
    }
}

fn as_string(string: &mut Vec<String>) -> String {
    if string.is_empty() {
        return "".to_string();
    }
    let mut builder = "".to_string();
    while !string.is_empty() {
        let mut str = string.pop().unwrap();
        while !str.is_empty() {
            builder.push(str.pop().unwrap());
        }
        builder.push(' ');
    }
    builder.pop().unwrap();
    string_reverse(builder)
}

fn as_string_primitive(string: &mut Vec<&str>) -> String {
    let mut vec2 = vec![];
    while !string.is_empty() {
        vec2.push(string.pop().unwrap().to_string())
    }
    vec2.reverse();
    as_string(&mut vec2)
}

fn string_reverse(mut string: String) -> String {
    let mut builder = "".to_string();
    while !string.is_empty() {
        builder.push(string.pop().unwrap());
    }
    builder
}

/*fn craft_command(exe: String, mut args: Vec<String>) -> CommandLine {
    CommandLine {
        base: exe,
        args: as_string(&mut args),
    }
}*/

fn craft_command(exe: &str, mut args: Vec<&str>, is_windows: bool) -> CommandLine {
    CommandLine {
        base: exe.to_string(),
        args: as_string_primitive(&mut args),
        windows: is_windows,
    }
}

fn craft_powershell_command(exe: &str, mut args: Vec<&str>) -> CommandLine {
    let mut vec2 = vec!["-NoProfile", "-NonInteractive", "-command", exe];
    while !args.is_empty() {
        vec2.push(args.pop().unwrap());
    }
    return craft_command("powershell", vec2, true);
}

fn clang_version(ver: &Vec<u8>) -> String {
    let mut builder = "".to_string();
    let string = std::str::from_utf8(&*ver).unwrap().to_string();
    let start = string.find("version").unwrap_or(string.len()) + 6;
    if start > string.len() {
        return "".to_string();
    }
    let mut chars = string.chars();
    chars.nth(start);
    loop {
        let tmp = chars.nth(0).unwrap();
        if tmp.is_ascii_digit() || tmp == '.' {
            builder.push(tmp);
        } else if !builder.is_empty() {
            return builder;
        }
    }
}

fn version_major(string: String) -> u16 {
    if string.is_empty() {
        return 0;
    }
    let mut chars = string.chars();
    let mut builder = "".to_string();
    let mut tmp = chars.nth(0).unwrap_or('E');
    while tmp.is_ascii_digit() {
        builder.push(tmp);
        tmp = chars.nth(0).unwrap_or('E');
    }
    return builder.parse().unwrap_or(0);
}

fn as_u32_le(x: u8, y: u8, z: u8, t: u8) -> u32 {
    ((x as u32) << 0) + ((y as u32) << 8) + ((z as u32) << 16) + ((t as u32) << 24)
}

fn sectors_to_bytes(sectors: u32) {
    println!("Bytes: {}", sectors * 512)
}

fn build_mbr(is_windows: bool) -> bool {
    let diskpath = std::path::Path::new("hdd.img");
    let bios = std::path::Path::new("boot.bin");
    let result = craft_command(
        "nasm",
        vec!["-f bin", "assembly/bootloader/mbr/main.asm", "-o boot.bin"],
        is_windows,
    )
    .run();
    println!("{}", std::str::from_utf8(&result.stdout).unwrap());
    if !result.status.success() {
        eprintln!("Error: {}", std::str::from_utf8(&result.stderr).unwrap());
        return false;
    }

    let mut bootable = File::open(bios).unwrap();
    let len = bootable.metadata().unwrap().len();
    if len > 512 {
        eprintln!("{} > 512! too big for MBR!", len);
        return false;
    }
    if len < 512 {
        eprintln!(
            "WARN: {} < 512! Smaller than full boot.bin size! Did you forget to pad the file?",
            len
        );
    }
    let mut buffer: Vec<u8> = vec![];
    bootable.read_to_end(&mut buffer).unwrap();
    while buffer.len() < 440 {
        buffer.push(0)
    }
    if buffer.len() > 440 {
        buffer.drain(440..);
    }
    let mut file = File::create(diskpath).unwrap();
    let empty = Partition {
        status: 0,
        start_cylinder: 0,
        start_head: 0,
        start_sector: 0,
        p_type: 0,
        end_cylinder: 0,
        end_head: 0,
        end_sector: 0,
    };
    let mut mbr = MBR {
        bootstrap: [0x00; 440],
        signature: 0,
        copy_protected: false,
        partition_1: Partition {
            status: 0x80,
            start_cylinder: 0,
            start_head: 32,
            start_sector: 33,
            p_type: 0x7F,
            end_cylinder: 2,
            end_head: 140,
            end_sector: 10,
        },
        partition_2: empty,
        partition_3: empty,
        partition_4: empty,
    };
    for i in 0..440 {
        mbr.bootstrap[i] = buffer.remove(0);
    }
    let mut vec: Vec<u8> = vec![];
    mbr.write(&mut vec);
    file.write_all(&vec).unwrap();
    mbr.print();
    return true;
}

fn main() {
    let is_windows = cfg!(target_os = "windows");
    let nasm_exist = craft_command("nasm", vec!["-v"], is_windows)
        .run()
        .status
        .success();
    let cargo_exist = craft_command("cargo", vec!["-V"], is_windows).run();
    let rustc_exist = craft_command("rustc", vec!["--version", "--verbose"], is_windows).run();
    let llvm_clang_exist = craft_command("clang", vec!["-v"], is_windows).run();
    let llvm_clang_major = version_major(clang_version(&llvm_clang_exist.stderr));
    let rustc_clang_major = version_major(clang_version(&rustc_exist.stdout));
    println!(
        "Has nasm: {}\nHas cargo: {}\nHas llvm_clang: {}\nHas rustc: {}\n\nVersion clang: {}\nVersion rustc_clang: {}",
        nasm_exist,
        cargo_exist.status.success(),
        llvm_clang_exist.status.success(),
        rustc_exist.status.success(),
        llvm_clang_major,
        rustc_clang_major,
    );
    if rustc_clang_major > llvm_clang_major {
        println!(
            "LLVM outdated or not installed globally! Need at least major version: {} Have major version: {}",
            rustc_clang_major,
            llvm_clang_major,
        );
        return;
    }
    if !nasm_exist {
        println!("Nasm not found! Please install before continuing.");
        return;
    }
    println!("Choose an option:\n\t0) Build all\n\t1) Build Bootloader\n\t2) Build Kernel");
    build_mbr(is_windows);
    //todo get user input here
}

#[cfg(test)]
mod tests {
    use crate::{craft_command, craft_powershell_command};

    #[test]
    fn check_caller() {
        let is_windows = cfg!(target_os = "windows");
        let mut commands = vec![craft_command("echo", vec!["hello", "world"], is_windows)];
        if is_windows {
            commands.push(craft_powershell_command("pwd", vec![]));
        } else {
            commands.push(craft_command("pwd", vec![], false));
        }
        while !commands.is_empty() {
            let command = commands.pop().unwrap();
            let output = command.run();
            if output.status.success() {
                println!(
                    "{}\nResult: {}",
                    output.status,
                    std::str::from_utf8(&output.stdout).unwrap()
                );
            } else {
                eprintln!(
                    "{}\nResult: {}",
                    output.status,
                    std::str::from_utf8(&output.stderr).unwrap()
                )
            }
            assert!(output.status.success());
        }
    }
}
