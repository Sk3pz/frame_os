use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, Output};

#[derive(Clone, Copy)]
struct CHS {
    cylinder: u8,
    head: u8,
    sector: u8,
}

impl CHS {
    fn calculate_sector_total(start_chs: CHS, end_chs: CHS) -> u32 {
        let mut result: u32 = 0;
        let mut current_chs = CHS {
            cylinder: start_chs.cylinder,
            head: start_chs.head,
            sector: start_chs.sector,
        };
        while current_chs.cylinder < end_chs.cylinder
            || current_chs.head < end_chs.head
            || current_chs.sector < end_chs.sector
        {
            current_chs.increment();
            result += 1;
        }
        result
    }

    fn calculate_sector_distance(&self, other: CHS) -> u32 {
        if self.cylinder == other.cylinder {
            if self.head == other.head {
                if self.sector == other.sector {
                    0_u32
                } else if self.sector > other.sector {
                    CHS::calculate_sector_total(other, *self)
                } else {
                    CHS::calculate_sector_total(*self, other)
                }
            } else if self.head > other.head {
                CHS::calculate_sector_total(other, *self)
            } else {
                CHS::calculate_sector_total(*self, other)
            }
        } else if self.cylinder > other.cylinder {
            CHS::calculate_sector_total(other, *self)
        } else {
            CHS::calculate_sector_total(*self, other)
        }
    }

    fn as_lba(&self) -> u32 {
        let start_chs = CHS {
            cylinder: 0,
            head: 0,
            sector: 1,
        };
        self.calculate_sector_distance(start_chs)
    }

    fn increment(&mut self) {
        self.sector += 1;
        if self.sector == 64 {
            self.sector = 1;
            self.head += 1;
            if self.head == 255 {
                self.head = 0;
                self.cylinder += 1;
            }
        }
    }

    fn decrement(&mut self) -> bool {
        if self.sector <= 1 {
            if self.head == 0 {
                if self.cylinder == 0 {
                    return false;
                } else {
                    self.head = 254;
                    self.sector = 63;
                    self.cylinder -= 1;
                }
            } else {
                self.sector = 63;
                self.head -= 1;
            }
        } else {
            self.sector -= 1
        }
        true
    }

    fn from_lba(lba: u32) -> CHS {
        let mut current = CHS {
            cylinder: 0,
            head: 0,
            sector: 1,
        };
        let mut result = 0_u32;
        while result < lba {
            current.increment();
            result += 1;
        }
        current
    }
}

#[derive(Clone, Copy)]
struct Partition {
    status: u8,
    start_chs: CHS,
    p_type: u8,
    end_chs: CHS,
}

impl Partition {
    fn total_sectors(&self) -> u32 {
        if self.status == 0x00 {
            return 0x00;
        }
        self.start_chs.calculate_sector_distance(self.end_chs) + 1_u32
    }
    fn start_lba(&self) -> u32 {
        if self.status == 0x00 {
            return 0x00;
        }
        self.start_chs.as_lba()
    }

    fn print_record(&self) {
        println!("\tStatus: {}\n\tFirst:\n\t\tHead: {}\n\t\tSector: {}\n\t\tCylinder: {}\n\tType: {}\n\tLast:\n\t\tHead: {}\n\t\tSector: {}\n\t\tCylinder: {}\n\tLBA: {}\n\tSectors in partition: {}",
                 self.status,
                 self.start_chs.head,
                 self.start_chs.sector,
                 self.start_chs.cylinder,
                 self.p_type,
                 self.end_chs.head,
                 self.end_chs.sector,
                 self.end_chs.cylinder,
                 self.start_lba(),
                 self.total_sectors())
    }

    fn write_record(&self, vector: &mut Vec<u8>) {
        vector.push(self.status);
        vector.push(self.start_chs.head);
        vector.push(self.start_chs.sector);
        vector.push(self.start_chs.cylinder);
        vector.push(self.p_type);
        vector.push(self.end_chs.head);
        vector.push(self.end_chs.sector);
        vector.push(self.end_chs.cylinder);
        let integer = &self.start_lba().to_le_bytes();
        vector.write(integer).unwrap();
        let integer2 = &self.total_sectors().to_le_bytes();
        vector.write(integer2).unwrap();
    }

    fn read_record(vector: &mut Vec<u8>) -> Partition {
        let status = vector.remove(0);
        if status == 0x00 {
            let part = Partition {
                status,
                start_chs: CHS {
                    cylinder: 0,
                    head: 0,
                    sector: 0,
                },
                p_type: 0,
                end_chs: CHS {
                    cylinder: 0,
                    head: 0,
                    sector: 0,
                },
            };
            vector.drain(0..15);
            part
        } else {
            let part = Partition {
                status,
                start_chs: CHS {
                    head: vector.remove(0),
                    sector: vector.remove(0),
                    cylinder: vector.remove(0),
                },
                p_type: vector.remove(0),
                end_chs: CHS {
                    head: vector.remove(0),
                    sector: vector.remove(0),
                    cylinder: vector.remove(0),
                },
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
    blank_space: Partition,
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
        self.partition_1.print_record();
        if self.partition_2.status == 0x00 {
            return;
        }
        println!("P2: ");
        self.partition_2.print_record();
        if self.partition_3.status == 0x00 {
            return;
        }
        println!("P3: ");
        self.partition_3.print_record();
        if self.partition_4.status == 0x00 {
            return;
        }
        println!("P4: ");
        self.partition_4.print_record();
    }

    fn write(&self, file: &mut File) {
        let mut boot_vec: Vec<u8> = vec![];
        boot_vec.write(&self.bootstrap).unwrap();
        let integer = &self.signature.to_le_bytes();
        boot_vec.write(integer).unwrap();
        let num = if self.copy_protected { 0x5A } else { 0x00 };
        boot_vec.push(num);
        boot_vec.push(num);
        self.partition_1.write_record(&mut boot_vec);
        self.partition_2.write_record(&mut boot_vec);
        self.partition_3.write_record(&mut boot_vec);
        self.partition_4.write_record(&mut boot_vec);
        boot_vec.push(0x55);
        boot_vec.push(0xAA);
        file.write_all(&boot_vec).unwrap();
        boot_vec.clear();
        boot_vec.resize(512, 0x00);
        if self.partition_1.status != 0x00 {
            let mut x = self.partition_1.start_lba() + self.partition_1.total_sectors() - 1;
            for _ in 0..x {
                file.write_all(&boot_vec).unwrap();
            }
            if self.partition_2.status != 0x00 {
                x = (self.partition_2.start_lba() - x) + self.partition_2.total_sectors() - 1;
                for _ in 0..x {
                    file.write_all(&boot_vec).unwrap();
                }
                if self.partition_3.status != 0x00 {
                    x = (self.partition_3.start_lba() - x) + self.partition_3.total_sectors() - 1;
                    for _ in 0..x {
                        file.write_all(&boot_vec).unwrap();
                    }
                    if self.partition_4.status != 0x00 {
                        x = (self.partition_4.start_lba() - x) + self.partition_4.total_sectors()
                            - 1;
                        for _ in 0..x {
                            file.write_all(&boot_vec).unwrap();
                        }
                    }
                }
            }
        }
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
            partition_1: Partition::read_record(vector),
            partition_2: Partition::read_record(vector),
            partition_3: Partition::read_record(vector),
            partition_4: Partition::read_record(vector),
            blank_space: Partition {
                status: 0x00,
                start_chs: CHS {
                    cylinder: 0,
                    head: 0,
                    sector: 0,
                },
                p_type: 0x00,
                end_chs: CHS {
                    cylinder: 0,
                    head: 0,
                    sector: 0,
                },
            },
        };
        if mbr.partition_1.status != 0x00 {
            let start = CHS {
                cylinder: 0,
                head: 0,
                sector: 2,
            };
            let end = CHS {
                cylinder: mbr.partition_1.start_chs.cylinder,
                head: mbr.partition_1.start_chs.head,
                sector: mbr.partition_1.start_chs.sector,
            };
        }
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
    let sector = CHS::from_lba(0);
    println!(
        "LBA to Sector: H: {} S: {} C: {}",
        sector.head, sector.sector, sector.cylinder
    );
    let disk_path = std::path::Path::new("hdd.img");
    let bootloader = std::path::Path::new("boot.bin");
    let bootloader2 = std::path::Path::new("boot2.bin"); //todo
    let mut result = craft_command(
        "nasm",
        vec!["-f bin", "assembly/bootloader/mbr/part1.asm", "-o boot.bin"],
        is_windows,
    )
    .run();
    println!("{}", std::str::from_utf8(&result.stdout).unwrap());
    if !result.status.success() {
        eprintln!("Error: {}", std::str::from_utf8(&result.stderr).unwrap());
        return false;
    }

    result = craft_command(
        "nasm",
        vec![
            "-f bin",
            "assembly/bootloader/mbr/part2.asm",
            "-o boot2.bin",
        ],
        is_windows,
    )
    .run();
    println!("{}", std::str::from_utf8(&result.stdout).unwrap());
    if !result.status.success() {
        eprintln!("Error: {}", std::str::from_utf8(&result.stderr).unwrap());
        return false;
    }
    let mut bootable = File::open(bootloader).unwrap();
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
    if buffer.len() < 440 {
        buffer.resize(440, 0x00);
    }
    if buffer.len() > 440 {
        buffer.drain(440..);
    }
    let mut file = File::create(disk_path).unwrap();
    let empty = Partition {
        status: 0,
        start_chs: CHS {
            cylinder: 0,
            head: 0,
            sector: 0,
        },
        p_type: 0,
        end_chs: CHS {
            cylinder: 0,
            head: 0,
            sector: 0,
        },
    };
    let part_19_megabytes = Partition {
        status: 0x80,
        start_chs: CHS {
            cylinder: 0,
            head: 32,
            sector: 33,
        },
        p_type: 0x7F,
        end_chs: CHS {
            cylinder: 2,
            head: 140,
            sector: 10,
        },
    };
    let mut mbr = MBR {
        bootstrap: [0x00; 440],
        signature: 0xbebebe59,
        copy_protected: false,
        blank_space: empty,
        partition_1: part_19_megabytes,
        partition_2: empty,
        partition_3: empty,
        partition_4: empty,
    };
    for i in 0..440 {
        mbr.bootstrap[i] = buffer.remove(0);
    }
    mbr.write(&mut file);
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
