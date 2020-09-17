use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::vga_buffer_outdated::Color;

fn color(fg: Color, bg: Color) -> u8 { // create an attribute byte from 2 colors
    ((bg as u8) << 4 | (fg as u8))
}

const SCREEN_HEIGHT: u8 = 25;
const SCREEN_WIDTH: u8 = 80;

const DATA_BUFFER_SIZE: u8 = 124;

const VGA_TEXTMODE_PTR: *mut u8 = 0xb8000 as *mut u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenChar { // a screenchar that can be displayed
    ascii: u8,
    attr: u8
}

impl ScreenChar {
    pub fn new(c: u8, attr: u8) -> ScreenChar {
        ScreenChar {
            ascii: c,
            attr
        }
    }
}

/// Unsafe because the user must call a valid location and use of pointer offsets and writing
pub unsafe fn write_byte(vga: *mut u8, byte: u8, row: u8, col: u8) { // TODO: THIS IS BROKEN. FIX.
    vga.offset((((row as isize) * ((SCREEN_WIDTH * 2) as isize)) + ((col) as isize)) as isize).write(byte); // write to the correct position
    // multiplies row by width because thats how many characters are in a row
}

/// Unsafe because the user must call a valid location and use of unsafe function self.write_byte(...)
pub unsafe fn write_raw(vga: *mut u8, b: u8, attr: u8, row: u8, col: u8) {
    write_byte(vga, b, row, col); // write byte
    write_byte(vga, attr, row, col + 1); // write attribute at offset
}

/// Unsafe because of call to self.write(...)
pub unsafe fn write(vga: *mut u8, sc: ScreenChar, row: u8, col: u8) {
    write_raw(vga, sc.ascii, sc.attr, row, col); // write a screenchar
}

pub struct Writer {
    col_pos: u8, // the current column position on the screen
    row_pos: u8, // the current row position on the screen
    def_attr: u8, // the default attribute byte for writing
    buf_index: u8, // the current position of the screen in the data buffer
    buffer: [[ScreenChar; SCREEN_WIDTH as usize]; DATA_BUFFER_SIZE as usize], // all data written to the screen, including what is not displayed
}

impl Writer {
    pub fn new() -> Writer {
        Writer {
            col_pos: 0,
            row_pos: 0,
            def_attr: color(Color::White, Color::Black),
            buf_index: 0,
            buffer: [[ScreenChar::new(b' ', color(Color::White, Color::Black)); SCREEN_WIDTH as usize]; DATA_BUFFER_SIZE as usize],
        }
    }

    pub fn clear(&mut self) {
        self.buffer = [[ScreenChar::new(b' ', color(Color::White, Color::Black)); SCREEN_WIDTH as usize]; DATA_BUFFER_SIZE as usize];
        self.col_pos = 0;
        self.row_pos = 0;
        self.buf_index = 0;
        self.draw();
    }

    pub fn mov_u(&mut self) {
        // TODO
    }

    pub fn mov_d(&mut self) {
        if self.row_pos >= self.buf_index + 25 { // if at the bottom of the screen
            if self.buf_index >= DATA_BUFFER_SIZE - 25 { // if at the end of the buffer
                // shift all lines up by 1 index (overwriting index 0)
                for x in 0..DATA_BUFFER_SIZE {
                    if x != DATA_BUFFER_SIZE - 1 {
                        self.buffer[x as usize] = self.buffer[(x + 1) as usize];
                    }
                }
                // set the bottom line to an empty line
                self.buffer[(DATA_BUFFER_SIZE - 1) as usize] = [ScreenChar::new(b' ', self.def_attr); SCREEN_WIDTH as usize];
            } else {
                self.buf_index += 1;
            }
        } else { // otherwise should just move the row down by one
            self.row_pos += 1;
        }
    }

    pub fn ret(&mut self) {
        self.col_pos = 0;
    }

    pub fn newline(&mut self) {
        self.ret();
        self.mov_d();
        self.draw();
    }

    pub fn backspace(&mut self) {
        // TODO
    }

    pub fn write_byte_colored(&mut self, byte: u8, color: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => self.ret(),
            b'\x08' => self.backspace(),
            byte => {
                self.buffer[(self.row_pos) as usize][(self.col_pos as usize)] = ScreenChar {
                    ascii: byte,
                    attr: color
                };

                self.col_pos += 1;
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.write_byte_colored(byte, self.def_attr);
    }

    fn write_valid_byte(&mut self, byte: u8) {
        match byte {
            // match the non color code byte
            // printable ASCII byte or newline
            0x20..=0x7e | b'\n' | b'\r' | b'\x08' => self.write_byte(byte),
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

    pub fn write_string(&mut self, s: &str) {
        let mut colored = false;
        for x in 0..s.bytes().len() {
            let byte = s.bytes().nth(x).unwrap();
            if colored {
                match byte {
                    // determine the color TODO: Custom Background colors?
                    b'0' => self.def_attr = color(Color::Black, Color::Black),
                    b'1' => self.def_attr = color(Color::Blue, Color::Black),
                    b'2' => self.def_attr = color(Color::Green, Color::Black),
                    b'3' => self.def_attr = color(Color::Cyan, Color::Black),
                    b'4' => self.def_attr = color(Color::Red, Color::Black),
                    b'5' => self.def_attr = color(Color::Magenta, Color::Black),
                    b'6' => self.def_attr = color(Color::Brown, Color::Black),
                    b'7' => self.def_attr = color(Color::LightGray, Color::Black),
                    b'8' => self.def_attr = color(Color::DarkGray, Color::Black),
                    b'9' => self.def_attr = color(Color::LightBlue, Color::Black),
                    b'a' => self.def_attr = color(Color::LightGreen, Color::Black),
                    b'b' => self.def_attr = color(Color::LightCyan, Color::Black),
                    b'c' => self.def_attr = color(Color::LightRed, Color::Black),
                    b'd' => self.def_attr = color(Color::Pink, Color::Black),
                    b'e' => self.def_attr = color(Color::Yellow, Color::Black),
                    b'f' => self.def_attr = color(Color::White, Color::Black),
                    _ => {
                        // if not a color code, just print the normal text
                        self.write_byte(b'&'); // COLOR INDICATOR CHAR SET HERE!
                        colored = self.color_check(x, s);
                        if colored {
                            continue;
                        }
                        self.write_valid_byte(byte);
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
            self.write_valid_byte(byte);
        }
        self.draw();
    }

    pub fn draw(&mut self) { // TODO: THIS IS BROKEN. FIX.
        for row in 0..SCREEN_HEIGHT {
            let mut output_col = 0;
            for col in 0..SCREEN_WIDTH {
                unsafe {
                    write(VGA_TEXTMODE_PTR, self.buffer[(row + self.buf_index) as usize][col as usize], row, output_col); // write the current char to the screen
                }

                output_col += 2;
            }

        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_textmode::_print(format_args!($($arg)*)));
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