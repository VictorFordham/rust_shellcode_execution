// inspiration from https://make-a-demo-tool-in-rust.github.io/1-3-jit.html

use bindings::{
    Windows::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, VirtualAlloc, VIRTUAL_ALLOCATION_TYPE, VirtualProtect}, // okay, importing everything caused weird errors, don't do it
    Windows::Win32::System::SystemServices::{BOOL, PAGE_EXECUTE_READ, PAGE_READWRITE, PAGE_TYPE},
};
use std::ffi::c_void;
use std::mem;

struct Buffer {
    addr: *mut u8,
    offset: isize,
    protections: PAGE_TYPE,
    size: usize,
}

impl Buffer {
    #[cfg(target_os = "windows")]
    pub fn new(size: usize) -> Self {
        let addr: *mut u8;
        let protections = PAGE_TYPE::from(PAGE_READWRITE);

        unsafe {
            let raw_addr: *mut c_void;

            raw_addr = VirtualAlloc(
                std::ptr::null_mut(),
                size,
                VIRTUAL_ALLOCATION_TYPE::from(MEM_RESERVE | MEM_COMMIT),
                protections
            );

            if raw_addr.is_null() {
                panic!("Unsuccessful buffer allocation");
            }

            addr = mem::transmute(raw_addr);
        }
        println!("{:?}", addr);
        Buffer {
            addr: addr,
            offset: 0,
            protections: protections,
            size: size,
        }
    }

    pub fn load(&mut self, code: Vec<u8>) -> bool {
        if  self.offset + (code.len() as isize) >= self.size as isize {
            return false;
        }

        if self.protections != PAGE_TYPE::from(PAGE_READWRITE) {
            self.change_protect(PAGE_TYPE::from(PAGE_READWRITE));
        }

        for byte in code.into_iter() {
            unsafe { *self.addr.offset(self.offset) = byte; }
            self.offset += 1;
        }

        return true;
    }

    pub fn push_u8(&mut self, val: u8) -> bool {
        return self.load(vec![val]);
    }

    pub fn push_u16(&mut self, val: u16) -> bool {
        let mut vals: Vec<u8> = Vec::new();
        vals.push(((val >> 8) & 0xff) as u8);
        vals.push((val & 0xff) as u8);
        return self.load(vals);
    }

    pub fn run(&mut self) {

        if self.protections != PAGE_TYPE::from(PAGE_EXECUTE_READ) {
            self.change_protect(PAGE_TYPE::from(PAGE_EXECUTE_READ));
        }

        unsafe {
            let func: unsafe extern "C" fn() -> u8 = mem::transmute(self.addr);
            let result = func();

            println!("{}", result);
        }

    }

    #[cfg(target_os = "windows")]
    fn change_protect(&mut self, new_prot: PAGE_TYPE) {
        unsafe {
            let raw_addr: *mut c_void = mem::transmute(self.addr);
            
            let result: BOOL = VirtualProtect(
                raw_addr,
                self.size,
                new_prot,
                &mut self.protections
            );

            self.protections = new_prot;

            if result == false {
                panic!("VirtualProtect call failed");
            }
        }
    }
}

fn main() -> windows::Result<()> {

    let mut buf = Buffer::new(100);

    buf.push_u8(0xb8);
    buf.push_u8(0x80);
    buf.push_u8(0x00);
    buf.push_u8(0x00);
    buf.push_u8(0x00);
    buf.push_u8(0xc3);

    buf.run();

    Ok(())
}