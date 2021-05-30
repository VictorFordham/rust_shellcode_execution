// inspiration from https://make-a-demo-tool-in-rust.github.io/1-3-jit.html

use bindings::{
    Windows::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, VirtualAlloc, VIRTUAL_ALLOCATION_TYPE, VirtualProtect}, // okay, importing everything caused weird errors, don't do it
    Windows::Win32::System::SystemServices::{BOOL, PAGE_EXECUTE_READ, PAGE_READWRITE, PAGE_TYPE},
};
use std::ffi::c_void;
use std::mem;
use std::ops::{Index, IndexMut};

struct CodeBuffer {
    addr: *mut u8,
    len: usize,
    protected: bool,
    size: usize,
}

impl Index<usize> for CodeBuffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { & *self.addr.offset(index as isize) }
    }
}

impl IndexMut<usize> for CodeBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.addr.offset(index as isize) }
    }
}

impl CodeBuffer {
    #[cfg(target_os = "windows")]
    fn alloc_with_size(size: usize) -> Self {
        let addr: *mut u8;
        let protections = PAGE_TYPE::from(PAGE_READWRITE);

        unsafe {
            let raw_addr: *mut c_void;

            raw_addr = VirtualAlloc(
                std::ptr::null_mut(),
                size,
                VIRTUAL_ALLOCATION_TYPE::from(MEM_RESERVE | MEM_COMMIT),
                protections,
            );

            if raw_addr.is_null() {
                panic!("Unsuccessful buffer allocation");
            }

            addr = mem::transmute(raw_addr);
        }

        Self {
            addr: addr,
            len: 0,
            protected: true,
            size: size,
        }
    }

    #[cfg(target_os = "linux")]
    fn alloc_with_size(size: usize) -> Self {

    }

    pub fn new() -> Self {
        Self::alloc_with_size(100)
    }

    pub fn load(&mut self, code: Vec<u8>) -> bool {
        if  self.len + code.len() >= self.size {
            return false;
        }

        if !self.protected {
            self.change_protect(true);
        }

        for byte in code.into_iter() {
            unsafe { *self.addr.offset(self.len as isize) = byte; }
            self.len += 1;
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

    pub fn to_fn(&mut self) -> unsafe extern "C" fn() -> u8 {
        if self.protected {
            self.change_protect(false);
        }

        unsafe {
            let func: unsafe extern "C" fn() -> u8 = mem::transmute(self.addr);
            return func;
        }
    }

    #[cfg(target_os = "windows")]
    fn change_protect(&mut self, protect: bool) {
        let new_prot: PAGE_TYPE;
        let mut old_prot: PAGE_TYPE;

        if protect {
            new_prot = PAGE_TYPE::from(PAGE_READWRITE);
            old_prot = PAGE_TYPE::from(PAGE_EXECUTE_READ);
        } else {
            new_prot = PAGE_TYPE::from(PAGE_EXECUTE_READ);
            old_prot = PAGE_TYPE::from(PAGE_READWRITE);
        }

        unsafe {
            let raw_addr: *mut c_void = mem::transmute(self.addr);
            
            let result: BOOL = VirtualProtect(
                raw_addr,
                self.size,
                new_prot,
                &mut old_prot
            );

            self.protected = protect;

            if result == false {
                panic!("VirtualProtect call failed");
            }
        }
    }
}

fn main() -> windows::Result<()> {

    let mut buf = CodeBuffer::new();

    buf.push_u8(0xb8);
    buf.push_u8(0x80);
    buf.push_u8(0x00);
    buf.push_u8(0x00);
    buf.push_u8(0x00);
    buf.push_u8(0xc3);

    let func: unsafe extern "C" fn() -> u8 = buf.to_fn();

    unsafe { println!("{}", func()); }

    Ok(())
}