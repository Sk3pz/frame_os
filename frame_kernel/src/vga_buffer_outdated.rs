use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// ================= COLOR ENUM MAPPING

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    // create color variables for readability (even though I have the color ids memorized...)
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
    LightGreen = 10, // a
    LightCyan = 11,  // b
    LightRed = 12,   // c
    Pink = 13,       // d
    Yellow = 14,     // e
    White = 15,      // f
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
struct OutDatedScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

impl OutDatedScreenChar {
    pub fn new(c: u8, color: ColorCode) -> OutDatedScreenChar {
        OutDatedScreenChar {
            ascii_character: c,
            color_code: color,
        }
    }
}

// ================= WRITING BUFFERS

const SCREEN_HEIGHT: u8 = 25;
const SCREEN_WIDTH: u8 = 80;

const BUFFER_HEIGHT: u8 = 124; // DO NOT SET BELOW 24!

#[repr(transparent)]
struct OutDatedBuffer {
    chars: [[Volatile<OutDatedScreenChar>; SCREEN_WIDTH as usize]; BUFFER_HEIGHT as usize],
}

pub struct OutDatedWriter {
    line_pos: u8, // row / line position
    color_code: ColorCode, // the base color code
    column_pos: u8, // the current character (0-SCREEN_WIDTH) on the line
    fb_display_pos: u8, // the current start of the screen display in the full buffer
    full_buffer: [[OutDatedScreenChar; SCREEN_WIDTH as usize]; BUFFER_HEIGHT as usize], // all of the stored lines
    display_buffer: &'static mut OutDatedBuffer // the screen
}

impl OutDatedWriter {

    pub fn new() -> OutDatedWriter {
        OutDatedWriter {
            line_pos: 0,
            color_code: ColorCode::new(Color::White, Color::Black),
            column_pos: 0,
            fb_display_pos: 0,
            full_buffer: [[OutDatedScreenChar::new(b' ',
                                           ColorCode::new(Color::White,
                                                          Color::Black)); SCREEN_WIDTH as usize]; BUFFER_HEIGHT as usize],
            display_buffer: unsafe { &mut *(0xb8000 as *mut OutDatedBuffer) }
        }
    }

    pub fn mov_u(&mut self) -> bool { // returns true if move was successful
        if self.fb_display_pos <= 0 { return false; }

        self.fb_display_pos -= 1;

        true
    }

    pub fn mov_d(&mut self) -> bool { // returns true if move was successful
        if self.fb_display_pos > BUFFER_HEIGHT - 25 { return false; }

        self.fb_display_pos += 1;

        true
    }

    pub fn push_d(&mut self) {
        for x in 0..BUFFER_HEIGHT {
            if x != BUFFER_HEIGHT - 1 {
                self.full_buffer[x as usize] = self.full_buffer[(x + 1) as usize];
            }
        }
        self.full_buffer[(BUFFER_HEIGHT - 1) as usize] = [OutDatedScreenChar::new(b' ',
                                                                          ColorCode::new(Color::White,
                                                                                         Color::Black)); SCREEN_WIDTH as usize];
    }

    pub fn newline(&mut self) {
        self.column_pos = 0;
        if self.line_pos >= self.fb_display_pos + 25 && !self.mov_d() {
            self.push_d(); // TODO: This does not work
            self.draw();
            return;
        } // move down a line
        self.line_pos += 1;
    }

    pub fn ret(&mut self) {
        self.column_pos = 0;
    }

    pub fn backspace(&mut self) {
        // TODO
    }

    pub fn tab(&mut self) { // todo: 2, 3, or 4 spaces
        // TODO
    }

    pub fn write_byte_colored(&mut self, byte: u8, color: ColorCode) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => self.ret(),
            b'\x08' => self.backspace(),
            // TODO: tab character
            byte => {
                self.full_buffer[(self.line_pos) as usize][(self.column_pos) as usize] = OutDatedScreenChar {
                    ascii_character: byte,
                    color_code: color
                };

                self.column_pos += 1;
            }
        }

        self.draw();
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.write_byte_colored(byte, self.color_code);
    }

    fn write_str_continue(&mut self, byte: u8) {
        match byte {
            // match the non color code byte
            // printable ASCII byte or newline
            0x20..=0x7e | b'\n' | b'\r' | b'\x08' => self.write_byte(byte),
            // Todo Register Character Escapes here!
            // not part of printable ASCII range
            _ => self.write_byte(0xfe),
        }
    }

    fn color_check(&mut self, x: usize, s: &str) -> bool {
        s.bytes().nth(x).unwrap() == b'&'
            && s.bytes().len() > x + 1
            && ((s.bytes().nth(x + 1).unwrap() >= b'0' && s.bytes().nth(x + 1).unwrap() <= b'9')
            || (s.bytes().nth(x + 1).unwrap() >= b'a' && s.bytes().nth(x + 1).unwrap() <= b'f'))
    }

    pub fn write_string(&mut self, s: &str) { // the write function using write byte
        let mut colored = false;
        for x in 0..s.bytes().len() {
            let byte = s.bytes().nth(x).unwrap();
            if colored {
                match byte {
                    // determine the color TODO: Custom Background colors?
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
                    _ => {
                        // if not a color code, just print the normal text
                        self.write_byte(b'&'); // COLOR INDICATOR CHAR SET HERE!
                        colored = self.color_check(x, s);
                        if colored {
                            continue;
                        }
                        self.write_str_continue(byte);
                        continue; // as to not set colored to false if needed
                    }
                }
                colored = false;
                continue; // Continue the loop as there is nothing else to do.
            }
            colored = self.color_check(x, s);
            if colored {
                continue;
            }
            self.write_str_continue(byte);
        }
        self.draw();
    }

    pub fn draw(&mut self) {
        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH {
                self.display_buffer.chars[row as usize][col as usize]
                    .write(self.full_buffer[(row + self.fb_display_pos) as usize][col as usize]); // This operation is the culprit for it being so slow
            }
        }
    }

}

impl fmt::Write for OutDatedWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// ================= STATIC WRITER MUTEX

lazy_static! {
    pub static ref OutDated_WRITER: Mutex<OutDatedWriter> = Mutex::new(OutDatedWriter::new());
}

// ================= PRINTING MACROS

// TODO: Put into separate file
// TODO: Write to STDOUT instead of current when files are implemented

#[macro_export]
macro_rules! old_print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! old_println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _old_print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        OutDated_WRITER.lock().write_fmt(args).unwrap();
    });
}