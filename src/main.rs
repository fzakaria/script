extern crate pty;
extern crate nix;
extern crate libc;

#[macro_use]
extern crate simple_error;

use std::os::unix::io::AsRawFd;
use simple_error::SimpleError;
use std::io::Stdin;

fn main() -> Result<(), Box<dyn std::error::Error> >  {
    // 1 -- First get the original terminal attributes
    let stdin = std::io::stdin();
    let orig_attr = nix::sys::termios::tcgetattr(stdin.as_raw_fd())?;

    unsafe {
        let window = get_window(stdin)?;
        println!("{:?}", window)
    }

    Ok(())
}

unsafe fn get_window(stdin: Stdin) -> Result<libc::winsize, SimpleError> {
    let mut window: std::mem::MaybeUninit<libc::winsize> = std::mem::MaybeUninit::uninit();
    let result = libc::ioctl(stdin.as_raw_fd(), libc::TIOCGWINSZ, window.as_mut_ptr());
    if result < 0 {
        bail!("Failed to get window size.");
    }
    Ok(window.assume_init())
}

