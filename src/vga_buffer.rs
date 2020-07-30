use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use core::fmt;

// ================= COLOR ENUM MAPPING

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color { // create color variables for readability (even though I have the color ids memorized...)
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// ================= STATIC WRITER MUTEX

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// ================= COLOR HANDLING

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8); // TODO: Rework Color System: Store FG and BG separately for color specification.

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// ================= SCREENCHAR - A DISPLAYABLE ASCII CHARACTER

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// ================= WRITING BUFFER

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// ================= WRITER

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte_colored(&mut self, byte: u8, color: ColorCode) {
        match byte {
            b'\n' => self.new_line(),
            b'\r' => self.reset_cursor(),
            b'\x08' => self.backspace(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color,
                });
                self.column_position += 1;
            }
        }
    }
    pub fn write_byte(&mut self, byte:u8) {
        self.write_byte_colored(byte, self.color_code);
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn reset_cursor(&mut self) { // reset to start of current line
        self.column_position = 0;
    }

    // TODO: WARNING: BACKSPACING TOO FAR BREAKS INPUT! FIX!
    // Issue may reside with newlines
    fn backspace(&mut self) { // TODO: Fix to only allow for typed characters to be backspaced
        self.column_position -= 1; // move back one character
        self.write_byte(b' '); // write a space (clear char)
        self.column_position -= 1; // put cursor into correct position
    }

    fn write_str_continue(&mut self, byte: u8) {
        match byte { // match the non color code byte
            // printable ASCII byte or newline
            0x20..=0x7e | b'\n' | b'\r' | b'\x08' => self.write_byte(byte),
            // Todo Register Character Escapes here!
            // not part of printable ASCII range
            _ => self.write_byte(0xfe),
        }
    }

    fn color_check(&mut self, x: usize, s: &str) -> bool {
        s.bytes().nth(x).unwrap() == b'&' && s.bytes().len() > x + 1
            && ((s.bytes().nth(x + 1).unwrap() >= b'0' && s.bytes().nth(x + 1).unwrap() <= b'9')
                || (s.bytes().nth(x + 1).unwrap() >= b'a' && s.bytes().nth(x + 1).unwrap() <= b'f'))
    }

    pub fn write_string(&mut self, s: &str) { // TODO: Fix bug where you cant type '&'
        let mut colored = false;
        for x in 0..s.bytes().len() {
            let byte = s.bytes().nth(x).unwrap();
            if colored {
                match byte { // determine the color TODO: Custom Background colors?
                    b'0' => self.color_code = ColorCode::new(Color::Black, Color::Black),
                    b'1' => self.color_code = ColorCode::new(Color::Blue, Color::Black),
                    b'2' => self.color_code = ColorCode::new(Color::Green, Color::Black),
                    b'3' => self.color_code = ColorCode::new(Color::Cyan, Color::Black),
                    b'4' => self.color_code = ColorCode::new(Color::Red, Color::Black),
                    b'5' => self.color_code = ColorCode::new(Color::Magenta, Color::Black),
                    b'6' => self.color_code = ColorCode::new(Color::Brown, Color::Black),
                    b'7' => self.color_code = ColorCode::new(Color::LightGray, Color::Black),
                    b'8' => self.color_code = ColorCode::new(Color::DarkGray, Color::Black),
                    b'9' => self.color_code = ColorCode::new(Color::LightBlue, Color::Black),
                    b'a' => self.color_code = ColorCode::new(Color::LightGreen, Color::Black),
                    b'b' => self.color_code = ColorCode::new(Color::LightCyan, Color::Black),
                    b'c' => self.color_code = ColorCode::new(Color::LightRed, Color::Black),
                    b'd' => self.color_code = ColorCode::new(Color::Pink, Color::Black),
                    b'e' => self.color_code = ColorCode::new(Color::Yellow, Color::Black),
                    b'f' => self.color_code = ColorCode::new(Color::White, Color::Black),
                    _ => { // if not a color code, just print the normal text
                        self.write_byte(b'&'); // COLOR INDICATOR CHAR SET HERE!
                        colored = self.color_check(x, s);
                        if colored { continue; }
                        self.write_str_continue(byte);
                        continue; // as to not set colored to false if needed
                    }
                }
                colored = false;
                continue; // Continue the loop as there is nothing else to do.
            }
            colored = self.color_check(x, s);
            if colored { continue; }
            self.write_str_continue(byte);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// ================= PRINTING MACROS

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}
