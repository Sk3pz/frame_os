use std::fs::File;
use std::io::{repeat, Read, Write};
use std::process::{Command, Output};

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

fn calculate_sector_total(start_c: u8, start_h: u8, start_s: u8, end_c: u8, end_h: u8, end_s: u8) {
    let mut result: u32 = 0;
    let mut c_head = start_h;
    let mut c_sector = start_s;
    let mut c_cylinder = start_c;
    while c_head != end_h || c_sector != end_s || c_cylinder != end_c {
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
    println!("Result Sectors: {}", result);
    sectors_to_bytes(result);
}

fn read_mbr() {
    calculate_sector_total(0, 32, 33, 2, 140, 10);
    let diskpath = std::path::Path::new("os.img");
    let mut bootable = File::open(diskpath).unwrap();
    let mut buffer: Vec<u8> = vec![];
    while buffer.len() < 512 {
        buffer.push(0);
    }
    bootable.read(&mut *buffer).unwrap();
    buffer.drain(0..446);
    buffer.reverse();
    println!(
        "P1:\n\tStatus: {}\n\tFirst:\n\t\tHead: {}\n\t\tSector: {}\n\t\tCylinder: {}\n\tType: {}\n\tLast:\n\t\tHead: {}\n\t\tSector: {}\n\t\tCylinder: {}\n\tLBA: {}\n\tSectors in partition: {}",
        buffer.pop().unwrap(),
        buffer.pop().unwrap(),
        buffer.pop().unwrap(),
        buffer.pop().unwrap(),
        buffer.pop().unwrap(),
        buffer.pop().unwrap(),
        buffer.pop().unwrap(),
        buffer.pop().unwrap(),
        as_u32_le(buffer.pop().unwrap(), buffer.pop().unwrap(), buffer.pop().unwrap(), buffer.pop().unwrap()),
        as_u32_le(buffer.pop().unwrap(), buffer.pop().unwrap(), buffer.pop().unwrap(), buffer.pop().unwrap())
    );
}

fn build_mbr(is_windows: bool) -> bool {
    read_mbr();
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

    let mut file = File::create(diskpath).unwrap();
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
    buffer.push(0xbe); //Write Signature
    buffer.push(0xbe);
    buffer.push(0xbe);
    buffer.push(0x7F);
    buffer.push(0x00); //To copy protect: 0x5A else 0x00
    buffer.push(0x00); //To copy protect: 0x5A else 0x00
    file.write(&*buffer).unwrap(); //Write boot data
    buffer.resize(0, 0x00);
    buffer.push(0x80); //Bootable = 0x80. OFF = 0x00. normal = 0xbe
    buffer.push(0x20); //First Head
    buffer.push(0x21); //First Sector
    buffer.push(0x00); //First Cylinder
    buffer.push(0x7F); //Reserved for individual or local use and temporary or experimental projects
    buffer.push(0x8C); //Last Head
    buffer.push(0x0A); //Last Sector
    buffer.push(0x02); //Last Cylinder
    let lba = 2048_u32.to_le_bytes();
    let sectors = 38912_u32.to_le_bytes();
    buffer.push(*lba.get(0).unwrap());
    buffer.push(*lba.get(1).unwrap());
    buffer.push(*lba.get(2).unwrap());
    buffer.push(*lba.get(3).unwrap());
    buffer.push(*sectors.get(0).unwrap());
    buffer.push(*sectors.get(1).unwrap());
    buffer.push(*sectors.get(2).unwrap());
    buffer.push(*sectors.get(3).unwrap());
    file.write(&*buffer).unwrap();
    println!("Wrote partition Header 1");
    buffer.drain(..);
    buffer.resize(16, 0x00); //Empty Partition
    for i in 0..3 {
        println!("Wrote partition Header {}", i + 2);
        file.write(&*buffer).unwrap();
    }
    buffer.resize(0, 0x00);
    buffer.push(0x55);
    buffer.push(0xaa);
    file.write(&*buffer).unwrap();
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
