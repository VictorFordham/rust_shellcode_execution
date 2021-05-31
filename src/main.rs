// inspiration from https://make-a-demo-tool-in-rust.github.io/1-3-jit.html
#[cfg(target_os = "windows")]
use bindings::{
    Windows::Win32::System::Memory::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, VirtualAlloc, VIRTUAL_ALLOCATION_TYPE, VirtualFree, VirtualProtect}, // okay, importing everything caused weird errors, don't do it
    Windows::Win32::System::SystemServices::{BOOL, PAGE_EXECUTE_READ, PAGE_READWRITE, PAGE_TYPE},
};
#[cfg(target_os = "linux")]
use libc;
use std::ffi::c_void;
use std::iter::{Iterator};
use std::mem;
use std::ops::{Drop, Index, IndexMut};

const PAGE_SIZE: usize = 4096;

struct ByteBuffer {
    addr: *mut u8,
    i: usize,
    len: usize,
    protected: bool,
    size: usize,
}

impl Drop for ByteBuffer {
    #[cfg(target_os = "windows")]
    fn drop(&mut self) {
        unsafe {
            let raw_addr = mem::transmute(self.addr);

            let result = VirtualFree(
                raw_addr,
                0,
                MEM_RELEASE
            );

            if result == false {
                panic!("Failed to free buffer");
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn drop(&mut self) {

    }
}

impl Index<usize> for ByteBuffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { & *self.addr.offset(index as isize) }
    }
}

impl IndexMut<usize> for ByteBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.addr.offset(index as isize) }
    }
}

// impl Iterator for ByteBuffer {
//     type Item = u8;

//     fn next(&mut self) -> Option<u8> {
//         if self.i == 0
//     }
// }

impl ByteBuffer {
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
            i: 0,
            len: 0,
            protected: true,
            size: size,
        }
    }

    #[cfg(target_os = "linux")]
    fn alloc_with_size(size: usize) -> Self {

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

    #[cfg(target_os = "linux")]
    fn change_protect(&mut self, protect: bool) {
        unsafe {
            let raw_addr: *mut c_void = mem::transmute(self.addr);

            if protect {
                libc::mprotect(raw_addr, self.size, libc::PROT_READ | libc::PROT_WRITE);
            } else {
                libc::mprotect(raw_addr, self.size, libc::PROT_EXEC | libc::PROT_READ);
            }

            self.protected = protect;
        }
    }

    pub fn executable(&mut self) {
        if self.protected {
            self.change_protect(false);
        }
    }

    pub fn new() -> Self {
        Self::alloc_with_size(PAGE_SIZE)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn load(&mut self, code: Vec<u8>) -> bool {
        if  self.len + code.len() >= self.size {
            return false;
        }

        self.writable();

        for byte in code.into_iter() {
            unsafe { *self.addr.offset(self.len as isize) = byte; }
            self.len += 1;
        }

        return true;
    }

    pub fn push_u8(&mut self, val: u8) -> bool {
        return self.load(vec![val]);
    }

    pub fn push_u16_be(&mut self, val: u16) -> bool {
        let mut vals: Vec<u8> = Vec::new();
        vals.push(((val >> 8) & 0xff) as u8);
        vals.push((val & 0xff) as u8);
        return self.load(vals);
    }

    pub fn push_u16_le(&mut self, val: u16) -> bool {
        let mut vals: Vec<u8> = Vec::new();
        vals.push((val & 0xff) as u8);
        vals.push(((val >> 8) & 0xff) as u8);
        return self.load(vals);
    }

    pub fn push_u32_be(&mut self, val: u32) -> bool {
        let mut vals: Vec<u8> = Vec::new();
        vals.push(((val >> 24) & 0xff) as u8);
        vals.push(((val >> 16) & 0xff) as u8);
        vals.push(((val >>  8) & 0xff) as u8);
        vals.push((val & 0xff) as u8);
        return self.load(vals);
    }

    pub fn push_u32_le(&mut self, val: u32) -> bool {
        let mut vals: Vec<u8> = Vec::new();
        vals.push((val & 0xff) as u8);
        vals.push(((val >>  8) & 0xff) as u8);
        vals.push(((val >> 16) & 0xff) as u8);
        vals.push(((val >> 24) & 0xff) as u8);
        return self.load(vals);
    }

    pub fn push_u64_be(&mut self, val: u64) -> bool {
        let mut vals: Vec<u8> = Vec::new();
        vals.push(((val >> 56) & 0xff) as u8);
        vals.push(((val >> 48) & 0xff) as u8);
        vals.push(((val >> 40) & 0xff) as u8);
        vals.push(((val >> 32) & 0xff) as u8);
        vals.push(((val >> 24) & 0xff) as u8);
        vals.push(((val >> 16) & 0xff) as u8);
        vals.push(((val >>  8) & 0xff) as u8);
        vals.push((val & 0xff) as u8);
        return self.load(vals);
    }

    pub fn push_u64_le(&mut self, val: u64) -> bool {
        let mut vals: Vec<u8> = Vec::new();
        vals.push((val & 0xff) as u8);
        vals.push(((val >>  8) & 0xff) as u8);
        vals.push(((val >> 16) & 0xff) as u8);
        vals.push(((val >> 24) & 0xff) as u8);
        vals.push(((val >> 32) & 0xff) as u8);
        vals.push(((val >> 40) & 0xff) as u8);
        vals.push(((val >> 48) & 0xff) as u8);
        vals.push(((val >> 56) & 0xff) as u8);
        return self.load(vals);
    }

    pub fn raw_addr(&mut self) -> *mut c_void {
        unsafe {
            mem::transmute::<_, *mut c_void>(self.addr)
        }
    }

    pub fn run(&mut self, ptr: Option<*mut c_void>) -> *mut c_void {
        self.executable();

        unsafe {
            let func: unsafe extern "C" fn(*mut c_void) -> *mut c_void = mem::transmute(self.addr);
            func(ptr.unwrap_or(std::ptr::null_mut()))
        }
    }

    pub fn writable(&mut self) {
        if !self.protected {
            self.change_protect(true);
        }
    }
}

fn main() -> windows::Result<()> {

    let mut buf = ByteBuffer::new();

    buf.push_u8(0xb8);
    buf.push_u32_le(0x80);
    buf.push_u8(0xc3);

    buf.executable();
    let func: unsafe extern "C" fn() -> u8 = unsafe {mem::transmute(buf.raw_addr())};

    unsafe { println!("{}", func()); }

    buf.writable();
    buf[1] = 0x10;

    println!("{:?}", buf.run(None));

    Ok(())
}