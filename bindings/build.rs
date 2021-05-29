fn main() {
    windows::build!(
        Windows::Win32::System::Memory::{MEM_COMMIT, MEM_RESERVE, VirtualAlloc, VIRTUAL_ALLOCATION_TYPE, VirtualProtect},
        Windows::Win32::System::SystemServices::{BOOL, PAGE_EXECUTE_READ, PAGE_READWRITE, PAGE_TYPE},

    );
}