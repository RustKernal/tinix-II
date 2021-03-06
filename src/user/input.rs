use core::{fmt::Arguments, num::ParseIntError, ops::Range};
use alloc::{collections::VecDeque, string::String};
use x86_64::instructions::interrupts::without_interrupts;
use pc_keyboard::*;
use x86_64::instructions::port::*;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{clear_row, io::devices::console::BACKSPACE, kernel::arch::enable_irq, log, time};

pub fn init() -> crate::kernel::InitResult<()> {
    enable_irq(1);
    crate::kernel::arch::set_interrupt(1, on_key_pressed)
    .expect("Unable To Setup Keyboard Interrupt");
    Ok(())
}

fn on_key_pressed(_irq : u8) {
    without_interrupts(|| {
        KEYBOARD.lock().process_scancode();
    });
}

pub fn serial_read() -> Option<u8> {
    Some(crate::kernel::hardware::uart::read_u8())
}

pub fn serial_write(byte : u8){
    crate::kernel::hardware::uart::write_u8(byte);
}

pub macro serial_print {
    ($($arg:tt)*) => { $crate::user::input::_print(format_args!($($arg)*)) }
}

pub macro serial_println {
    ($($arg:tt)*) => { $crate::user::input::serial_print!("{}\r\n",format_args!($($arg)*)) }
}

pub fn _print(args : Arguments) {
    without_interrupts(|| {
        crate::kernel::hardware::uart::write_str(args);
    });
}

pub fn set_text_color(color : u8) {
    serial_print!("\u{1b}[1;{}m", color)
}

pub fn reset_text_color() {
    serial_print!("\u{1b}[0m")
}
    

lazy_static! {
    static ref KEYBOARD : Mutex<KeyBoard> = Mutex::new(KeyBoard::new());
}


struct KeyBoard {
    //IO Ports
    data    : Port<u8>, //Port 0x60
    //pc-keyboard
    kb      : Keyboard<layouts::Uk105Key, ScancodeSet1>,


    buffer : VecDeque<char>,
}

impl KeyBoard {
    pub fn new() -> Self {
        KeyBoard {
            data : Port::new(0x60),
            kb : Keyboard::new(layouts::Uk105Key, ScancodeSet1, HandleControl::Ignore),
            buffer : VecDeque::new(),
        }
    }

    pub fn process_scancode(&mut self) {
        let result : Option<char> = {
                if let Ok(Some(event)) = self.kb.add_byte(unsafe {self.data.read()}) {
                    if let Some(key) = self.kb.process_keyevent(event) {
                        match key {
                            DecodedKey::Unicode(c) => Some(c),
                            _ => None
                        }
                    } else {None}
                } else {None}
            };
        if result.is_some() {
             self.buffer.push_back(result.unwrap());
        }
    }

    pub fn last_key(&mut self) -> Option<char> {
        self.buffer.pop_front()
    }
}

pub fn key() -> Option<char> {
    let mut lk : Option<char> = None;
    without_interrupts(|| {
        lk = KEYBOARD.lock().last_key();
    });
    lk
} 

pub fn string(prompt : &str) -> String {
    let mut s = String::new();
    clear_row!();
    log!("{}{}\r",prompt, s);
    'consume_chars: loop {
        if let Some(key) = key() {
            if key == '\n' {break 'consume_chars}
            if key == BACKSPACE as char {s.pop();}
            s.push(key);
            clear_row!();
            log!("{}{}\r",prompt, s);
        }
        
        time::sleep_ticks(10); //Check 100 Times per second
    }
    s
}

pub fn number(prompt : &str, range : Range<usize>) -> usize {
    loop {
        let inp = self::string(prompt);
        let result : Result<usize, ParseIntError>  = inp.parse::<usize>();
        if result.is_ok() {
            let value = result.expect("");
            if range.contains(&value) {
                return value;
            }
        }
    }
    
}