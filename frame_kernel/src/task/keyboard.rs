use core::{
    pin::Pin,
    task::{Context, Poll},
};

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, KeyCode, layouts, ScancodeSet1};
use spin::Mutex;

use crate::{clear_vga, print, println};
use crate::vga_textmode::get_writer;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

struct KSBool {
    ks: bool
}

impl KSBool {
    pub fn new(val: bool) -> KSBool {
        KSBool {
            ks: val
        }
    }

    pub fn set(&mut self, val: bool) {
        self.ks = val;
    }
}

lazy_static! {
    pub static ref reset_keystrokes: Mutex<KSBool> = Mutex::new(KSBool::new(false)); // a mutex to allow other screen changing functions to reset the keystroke count of the user
    // probably a horrible way of doing this. TODO: Redo.
}

pub async fn handle_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);

    let mut keystrokes = 0; // keystrokes tracks how many characters the user has typed to allow backspacing to be handled

    while let Some(scancode) = scancodes.next().await {
        if reset_keystrokes { // check if other section requested reset of keystrokes, and accommodate
            keystrokes = 0;
            reset_keystrokes.lock().set(true);
        }
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        print!("{}", character);
                        keystrokes += 1;
                    },
                    DecodedKey::RawKey(key) => {
                        match key {
                            KeyCode::ArrowDown => {
                                get_writer().lock().move_screen_down();
                            }
                            KeyCode::ArrowUp => {
                                get_writer().lock().move_screen_up();
                            }
                            KeyCode::ArrowLeft => {
                                get_writer().lock().move_cursor_left(true);
                            }
                            KeyCode::ArrowRight => {
                                get_writer().lock().move_cursor_right(true);
                            }
                            KeyCode::Backspace => { // handle backspacing based on keystrokes
                                if keystrokes != 0 {
                                    // Probably bad, as if the keystrokes gets cleared right before this, it still backspaces... TODO: redo keystrokes
                                    print!("{:?}", key); // print backspace
                                    keystrokes -= 1; // remove one key
                                }
                            }
                            key => {
                                print!("{:?}", key);
                                keystrokes += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}