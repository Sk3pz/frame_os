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
    let mut start = string.find("version").unwrap() + 6;
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

fn main() {
    let is_windows = cfg!(target_os = "windows");
    let nasm_exist = craft_command("nasm", vec!["-v"], is_windows)
        .run()
        .status
        .success();
    let cargo_exist = craft_command("cargo", vec!["-V"], is_windows).run();
    let rustc_exist = craft_command("rustc", vec!["--version", "--verbose"], is_windows).run();
    let llvm_clang_exist = craft_command("clang", vec!["-v"], is_windows).run();
    println!(
        "Has nasm: {}\nHas cargo: {}\nHas llvm_clang: {}\nHas rustc: {}\n\nVersion clang: {}\nVersion rustc_clang: {}",
        nasm_exist,
        cargo_exist.status.success(),
        llvm_clang_exist.status.success(),
        rustc_exist.status.success(),
        clang_version(&llvm_clang_exist.stderr),
        clang_version(&rustc_exist.stdout),
    )
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
