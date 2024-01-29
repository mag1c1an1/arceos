/// File related syscalls.
use axmem::{UserInPtr, UserOutPtr};
use axlog::ax_println;

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1;
const FD_STDERR: usize = 2;
const CHUNK_SIZE: usize = 256;

pub fn sys_write(fd: usize, buf: UserInPtr<u8>, len: usize) -> isize {
    match fd {
        FD_STDOUT | FD_STDERR => {
            let mut count = 0;
            while count < len {
                let chunk_len = CHUNK_SIZE.min(len);
                let chunk: [u8; CHUNK_SIZE] = unsafe { buf.add(count).read_array(chunk_len) };
                ax_println!("{}", core::str::from_utf8(&chunk[..chunk_len]).unwrap());
                count += chunk_len;
            }
            count as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

// pub fn sys_read(fd: usize, mut buf: UserOutPtr<u8>, len: usize) -> isize {
//     match fd {
//         FD_STDIN => {
//             assert_eq!(len, 1, "Only support len = 1 in sys_read!");
//             loop {
//                 if let Some(c) = console_getchar() {
//                     buf.write(c);
//                     return 1;
//                 } else {
//                     CurrentTask::get().yield_now();
//                 }
//             }
//         }
//         _ => {
//             panic!("Unsupported fd in sys_read!");
//         }
//     }
// }

/// readv/writev使用的结构体
#[repr(C)]
pub struct IoVec {
    pub base: *mut u8,
    pub len: usize,
}

pub fn sys_writev(fd: usize, iov: *const IoVec, iov_cnt: usize) -> isize {
    match fd {
        FD_STDOUT | FD_STDERR => {
            let mut write_len = 0;
            for i in 0..iov_cnt {
                let io: &IoVec = unsafe { &(*iov.add(i)) };
                if io.base.is_null() || io.len == 0 {
                    continue;
                }
                let res = sys_write(fd, (io.base as usize).into(), io.len);
                if res >= 0 {
                    write_len += res;
                } else {
                    return res;
                }
            }
            write_len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

// pub fn sys_readv(fd: usize, iov: *const IoVec, iov_cnt: usize) -> isize {
//     match fd {
//         FD_STDOUT | FD_STDERR => {
//             let mut count = 0;
//             while count < len {
//                 let chunk_len = CHUNK_SIZE.min(len);
//                 let chunk: [u8; CHUNK_SIZE] = unsafe { buf.add(count).read_array(chunk_len) };
//                 print!("{}", core::str::from_utf8(&chunk[..chunk_len]).unwrap());
//                 count += chunk_len;
//             }
//             count as isize
//         }
//         _ => {
//             panic!("Unsupported fd in sys_write!");
//         }
//     }
// }
