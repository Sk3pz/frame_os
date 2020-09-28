use alloc::borrow::ToOwned;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::outb;

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
    Gray = 8,
    LightBlue = 9,
    LightGreen = 10, // a
    LightCyan = 11,  // b
    LightRed = 12,   // c
    Pink = 13,       // d
    Yellow = 14,     // e
    White = 15,      // f
}

fn color(fg: Color, bg: Color) -> u8 { // create an attribute byte from 2 colors
    ((bg as u8) << 4 | (fg as u8))
}

const SCREEN_HEIGHT: u8 = 25;
const SCREEN_WIDTH: u8 = 80;

const DATA_BUFFER_SIZE: u8 = 124;

const VGA_TEXTMODE_PTR: *mut u8 = 0xb8000 as *mut u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenChar {
    // a screenchar that can be displayed
    ascii: u8,
    attr: u8,
}

impl ScreenChar {
    pub fn new(c: u8, attr: u8) -> ScreenChar {
        ScreenChar {
            ascii: c,
            attr,
        }
    }
}

/// Unsafe because the user must call a valid location and use of pointer offsets and writing
pub unsafe fn vga_write_byte(byte: u8, row: u8, col: u8) { // TODO: THIS IS BROKEN. FIX.
    VGA_TEXTMODE_PTR.offset((((row as isize) * ((SCREEN_WIDTH * 2) as isize)) + ((col) as isize)) as isize).write(byte); // write to the correct position
    // multiplies row by width because thats how many characters are in a row
}

/// Unsafe because the user must call a valid location and use of unsafe function vga_write_byte(...)
pub unsafe fn vga_write_raw(b: u8, attr: u8, row: u8, col: u8) {
    vga_write_byte(b, row, col); // write byte
    vga_write_byte(attr, row, col + 1); // write attribute at offset
}

/// Unsafe because of call to vga_write_raw(...)
/// Unsafe because of call to set_vga_cursor_pos(...)
pub unsafe fn vga_write(sc: ScreenChar, row: u8, col: u8) {
    vga_write_raw(sc.ascii, sc.attr, row, col); // write a screenchar
}

/// Unsafe due to calls to outb(...)
/// Unsafe writing to port
pub unsafe fn set_vga_cursor_pos(x: u8, y: u8) {
    let pos: u16 = ((y as u16) * (SCREEN_WIDTH as u16) + (x as u16)) as u16;

    outb(0x3D4, 0x0F);
    outb(0x3D5, (pos & (0xFF as u16)) as u16);
    outb(0x3D4, 0x0E);
    outb(0x3D5, ((pos >> 8) & 0xFF) as u16);
}

pub struct Writer {
    col_pos: u8, // the current column position in the buffer
    row_pos: u8, // the current row position in the buffer
    def_attr: u8, // the default attribute byte for writing
    current_fg: Color,
    current_bg: Color,
    drawing: bool, // true if the system is allowed to write to the screen
    screen_buf_pos: u8, // the current position of the start of the screen in the data buffer
    buffer: [[ScreenChar; SCREEN_WIDTH as usize]; DATA_BUFFER_SIZE as usize], // all data written to the screen, including what is not displayed
}

// TODO: rework system to write to the bottom of the buffer first, and push everything up

impl Writer {
    pub fn new() -> Writer {
        Writer {
            col_pos: 0,
            row_pos: 0,
            def_attr: color(Color::White, Color::Black),
            current_fg: Color::White,
            current_bg: Color::Black,
            drawing: true,
            screen_buf_pos: 0,
            buffer: [[ScreenChar::new(b' ', color(Color::White, Color::Black)); SCREEN_WIDTH as usize]; DATA_BUFFER_SIZE as usize],
        }
    }

    /// locks the system from drawing to the screen (NOT THE BUFFER)
    pub fn lock_drawing(&mut self) {
        self.drawing = false;
    }

    /// unlocks the system from drawing to the screen
    pub fn unlock_drawing(&mut self) {
        self.drawing = true;
    }

    /// Moves the screen up in the buffer
    /// returns true if moved up successfully
    pub fn move_screen_up(&mut self) -> bool {
        if self.row_pos >= 1 {
            self.row_pos -= 1;
            if self.screen_buf_pos > 0 && self.screen_buf_pos > self.row_pos {
                self.screen_buf_pos -= 1;
            }
            self.draw();
            return true;
        }
        false
    }

    pub fn move_cursor_left(&mut self, wrap: bool) {
        // if no wrapping is needed
        if self.col_pos >= 1 {
            self.col_pos -= 1; // move the cursor position back
            self.draw();
            return; // nothing left to do
        }

        if !wrap { // if not wrapping, just move cursor to back of line and end
            self.col_pos = 0;
            self.draw();
            return;
        } else {
            self.move_screen_up();
        }

        self.draw();
    }

    pub fn move_cursor_right(&mut self, wrap: bool) {
        self.col_pos += 1; // increase the column position

        if self.col_pos > SCREEN_WIDTH { // requires a new line
            if !wrap { // if not wrapping, move cursor to the end of the line and return
                self.col_pos = SCREEN_WIDTH - 1;
                self.draw();
                return;
            }

            self.move_screen_down(); // move the screen down as many times as asked
            self.col_pos = 0; // set the remainder to the column position on the current line
        }
        self.draw();
    }

    /// Moves the screen down in the buffer
    pub fn move_screen_down(&mut self) {
        if self.screen_buf_pos + self.row_pos >= DATA_BUFFER_SIZE - 1 {
            // shift all lines up to make room for new lines at the end of the buffer
            for x in 0..DATA_BUFFER_SIZE {
                if x != DATA_BUFFER_SIZE - 1 {
                    self.buffer[x as usize] = self.buffer[(x + 1) as usize]; // set the value in buffer[x + 1] to buffer[x] to move everything up
                } else {
                    self.buffer[x as usize] = [ScreenChar::new(b' ', self.def_attr); SCREEN_WIDTH as usize]; // clear last entry
                }
            }
            self.draw();
            return;
        }

        // check the current pos on screen is not the end
        if self.row_pos < self.screen_buf_pos + (SCREEN_HEIGHT - 1) { // for when not moving position in buffer, only on screen
            self.row_pos += 1;
            self.draw();
            return;
        }

        // check that the position of the screen is before the end of the buffer
        if self.screen_buf_pos < DATA_BUFFER_SIZE - SCREEN_HEIGHT { // shifts the screen down by one and sets the row position accordingly
            self.screen_buf_pos += 1;
            self.row_pos += 1;
            self.draw();
        }
    }

    /// the carriage return '\r' returns the current position on the line to 0
    pub fn carriage_ret(&mut self) {
        self.col_pos = 0; // go back to the start of the current line
    }

    /// moves the current row position down by one
    pub fn newline(&mut self) {
        self.move_screen_down(); // move the line down 1
        self.carriage_ret(); // go to start of line
        self.draw(); // draw the change
    }

    /// goes back a character and sets it to a space
    pub fn backspace(&mut self) {
        self.move_cursor_left(true);
        self.write_string(" ");
        self.move_cursor_left(true);
    }

    pub fn clear(&mut self) {
        self.buffer = [[ScreenChar::new(b' ', color(Color::White, Color::Black)); SCREEN_WIDTH as usize]; DATA_BUFFER_SIZE as usize];
        self.col_pos = 0;
        self.set_fg_color(b'f');
        self.row_pos = 0;
        self.screen_buf_pos = 0;
        self.draw();
    }

    /// Writes a byte to the buffer with a colored attribute byte following
    pub fn write_byte_colored(&mut self, byte: u8, color: u8) {
        match byte {
            b'\n' => self.newline(), // newline
            b'\r' => self.carriage_ret(), // return carriage
            b'\x08' => self.backspace(), // backspace
            byte => { // if the byte is a normal character, print it to the buffer
                // set the correct location
                self.buffer[self.row_pos as usize][self.col_pos as usize] = ScreenChar::new(byte, color);

                self.col_pos += 1; // increment the column position in the buffer (column = character in the line)
                if self.col_pos >= 80 { // check if the current column position is at the end of the line
                    self.newline(); // go to the next line
                }
            }
        }
    }

    /// Writes a byte to the buffer with the default color attribute of the writer
    pub fn write_byte(&mut self, byte: u8) {
        self.write_byte_colored(byte, self.def_attr); // write the byte with the current default color attribute
    }

    /// Writes a character to the screen, but checks it to make sure its something that can be printed
    fn write_valid_byte(&mut self, byte: u8) {
        match byte {
            // match the non color code byte
            // printable ASCII byte or newline
            0x20..=0x7e | b'\n' | b'\r' | b'\x08' => self.write_byte(byte),
            // not part of printable ASCII range
            _ => self.write_byte(0xfe),
        }
    }

    /// check if the current byte is a color attribute identifier (foreground)
    /// returns true if color is changed
    fn set_fg_color(&mut self, cbyte: u8) -> bool {
        match cbyte {
            b'0' => {
                self.def_attr = color(Color::Black, self.current_bg);
                self.current_fg = Color::Black;
            }
            b'1' => {
                self.def_attr = color(Color::Blue, self.current_bg);
                self.current_fg = Color::Blue;
            }
            b'2' => {
                self.def_attr = color(Color::Green, self.current_bg);
                self.current_fg = Color::Green;
            }
            b'3' => {
                self.def_attr = color(Color::Cyan, self.current_bg);
                self.current_fg = Color::Cyan;
            }
            b'4' => {
                self.def_attr = color(Color::Red, self.current_bg);
                self.current_fg = Color::Red;
            }
            b'5' => {
                self.def_attr = color(Color::Magenta, self.current_bg);
                self.current_fg = Color::Magenta;
            }
            b'6' => {
                self.def_attr = color(Color::Brown, self.current_bg);
                self.current_fg = Color::Brown;
            }
            b'7' => {
                self.def_attr = color(Color::LightGray, self.current_bg);
                self.current_fg = Color::LightGray;
            }
            b'8' => {
                self.def_attr = color(Color::Gray, self.current_bg);
                self.current_fg = Color::Gray;
            }
            b'9' => {
                self.def_attr = color(Color::LightBlue, self.current_bg);
                self.current_fg = Color::LightBlue;
            }
            b'a' => {
                self.def_attr = color(Color::LightGreen, self.current_bg);
                self.current_fg = Color::LightGreen;
            }
            b'b' => {
                self.def_attr = color(Color::LightCyan, self.current_bg);
                self.current_fg = Color::LightCyan;
            }
            b'c' => {
                self.def_attr = color(Color::LightRed, self.current_bg);
                self.current_fg = Color::LightRed;
            }
            b'd' => {
                self.def_attr = color(Color::Pink, self.current_bg);
                self.current_fg = Color::Pink;
            }
            b'e' => {
                self.def_attr = color(Color::Yellow, self.current_bg);
                self.current_fg = Color::Yellow;
            }
            b'f' => {
                self.def_attr = color(Color::White, self.current_bg);
                self.current_fg = Color::White;
            }
            _ => return false
        }
        true
    }

    /// check if the current byte is a color attribute identifier (background)
    /// returns true if color is changed
    fn set_bg_color(&mut self, cbyte: u8) -> bool {
        match cbyte {
            b'0' => {
                self.def_attr = color(self.current_fg, Color::Black);
                self.current_bg = Color::Black;
            }
            b'1' => {
                self.def_attr = color(self.current_fg, Color::Blue);
                self.current_bg = Color::Blue;
            }
            b'2' => {
                self.def_attr = color(self.current_fg, Color::Green);
                self.current_bg = Color::Green;
            }
            b'3' => {
                self.def_attr = color(self.current_fg, Color::Cyan);
                self.current_bg = Color::Cyan;
            }
            b'4' => {
                self.def_attr = color(self.current_fg, Color::Red);
                self.current_bg = Color::Red;
            }
            b'5' => {
                self.def_attr = color(self.current_fg, Color::Magenta);
                self.current_bg = Color::Magenta;
            }
            b'6' => {
                self.def_attr = color(self.current_fg, Color::Brown);
                self.current_bg = Color::Brown;
            }
            b'7' => {
                self.def_attr = color(self.current_fg, Color::LightGray);
                self.current_bg = Color::LightGray;
            }
            b'8' => {
                self.def_attr = color(self.current_fg, Color::Gray);
                self.current_bg = Color::Gray;
            }
            b'9' => {
                self.def_attr = color(self.current_fg, Color::LightBlue);
                self.current_bg = Color::LightBlue;
            }
            b'a' => {
                self.def_attr = color(self.current_fg, Color::LightGreen);
                self.current_bg = Color::LightGreen;
            }
            b'b' => {
                self.def_attr = color(self.current_fg, Color::LightCyan);
                self.current_bg = Color::LightCyan;
            }
            b'c' => {
                self.def_attr = color(self.current_fg, Color::LightRed);
                self.current_bg = Color::LightRed;
            }
            b'd' => {
                self.def_attr = color(self.current_fg, Color::Pink);
                self.current_bg = Color::Pink;
            }
            b'e' => {
                self.def_attr = color(self.current_fg, Color::Yellow);
                self.current_bg = Color::Yellow;
            }
            b'f' => {
                self.def_attr = color(self.current_fg, Color::White);
                self.current_bg = Color::White;
            }
            _ => return false
        }
        true
    }

    /// Writes a string literal to the buffer using write_byte(...)
    pub fn write_string(&mut self, s: &str) {
        let mut colored_fg = false; // flag the next byte might be a fg color byte
        let mut colored_bg = false; // flag the next byte might be a bg color byte
        for x in 0..s.bytes().len() { // loop through all the bytes in the string
            let byte = s.bytes().nth(x).unwrap(); // get the raw value of the current byte
            if colored_fg { // if colored fg is flagged
                if self.set_fg_color(byte) { // changes the color if needed
                    colored_fg = false; // flip colored flag
                    continue; // set color, nothing left to do this loop iteration
                } else {
                    colored_fg = false; // unflag colored
                    self.write_valid_byte(b'&'); // print out '&' as it wasnt a colored byte
                }
            }
            if colored_bg {
                if self.set_bg_color(byte) {
                    colored_bg = false;
                    continue;
                } else {
                    colored_bg = false;
                    self.write_valid_byte(b'%');
                }
            }

            if byte == b'&' { // flag colored if needed
                colored_fg = true;
                continue;
            }
            if byte == b'%' {
                colored_bg = true;
                continue;
            }

            self.write_valid_byte(byte); // output current byte

        }
        self.draw();
    }

    /// Draws the portion of the buffer marked by screen_buf_pos to the screen
    pub fn draw(&mut self) {
        if !self.drawing {
            return;
        }
        for row in 0..SCREEN_HEIGHT { // all the rows (lines) on the screen
            for col in 0..SCREEN_WIDTH { // all the characters on the current line
                let byte = self.buffer[(self.screen_buf_pos + row) as usize][col as usize];
                unsafe {
                    // Write the current screenchar to the screen
                    vga_write(byte, row, col * 2);
                    // get the current row position in the buffer ^         col * 2 to account for attribute bytes ^
                }
            }
        }
        unsafe {
            let mut r = self.row_pos - self.screen_buf_pos;
            if r > 25 {
                r = 24;
            }
            set_vga_cursor_pos(self.col_pos, r);
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

#[macro_export]
macro_rules! clear_vga {
    () => ($crate::vga_textmode::_clear());
}

#[doc(hidden)]
pub fn _clear() {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().clear();
    });
}

pub(crate) fn get_writer<'a>() -> &'a Mutex<Writer> {
    &WRITER
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}