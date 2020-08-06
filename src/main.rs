extern crate pty;
extern crate nix;
extern crate libc;

#[macro_use]
extern crate simple_error;

use std::os::unix::io::FromRawFd;
use std::os::unix::io::AsRawFd;
use simple_error::SimpleError;
use std::io::{Stdin, Read, Write};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>>  {
    // 1 -- First get the original terminal attributes
    let stdin = std::io::stdin();
    let window : libc::winsize = unsafe {
        get_window(&stdin)?
    };

    let stdin = std::io::stdin().as_raw_fd();
    let stdout = std::io::stdout().as_raw_fd();

    let orig_attr = nix::sys::termios::tcgetattr(stdin)?;

    if let Err(err) = script(&window, stdin, &orig_attr, stdout) {
        nix::sys::termios::tcsetattr(
            stdin,
            nix::sys::termios::SetArg::TCSANOW,
            &orig_attr,
        )?;
        Err(err)
    } else {
        Ok(())
    }
}

fn script(window: &libc::winsize, stdin: i32, stdin_attr: &nix::sys::termios::Termios, stdout: i32) -> Result<(), Box<dyn std::error::Error>> {
    let fork_result = nix::pty::forkpty(Some(window), Some(stdin_attr))?;

    match fork_result.fork_result {

        // the child simply exec's into a shell
        nix::unistd::ForkResult::Child => {
            nix::sys::termios::tcsetattr(
                stdin,
                nix::sys::termios::SetArg::TCSANOW,
                &stdin_attr,
            )?;
            let shell = "/bin/bash";
            let c_str = std::ffi::CString::new(shell).expect("CString::new failed");
            nix::unistd::execv(&c_str, &[&c_str])?;
        }

        // the parent will relay data between terminal and pty master
        nix::unistd::ForkResult::Parent { .. } => {
            let mut master_file : std::fs::File = unsafe {
                std::fs::File::from_raw_fd(fork_result.master)
            };

            // this should print '/dev/ptmx' as the master device
            // https://linux.die.net/man/4/ptmx
            // Each file descriptor obtained by opening /dev/ptmx
            // is an independent PTM with its own associated pseudoterminal slaves (PTS)

            let mut output_file = std::fs::File::create("typescript")?;

            let mut tty = nix::sys::termios::tcgetattr(stdin)?;
            nix::sys::termios::cfmakeraw(&mut tty);
            nix::sys::termios::tcsetattr(
                stdin,
                nix::sys::termios::SetArg::TCSANOW,
                &tty,
            )?;

            let mut in_fds = nix::sys::select::FdSet::new();

            let mut buffer = [0; 256];
            loop {

                in_fds.clear();
                in_fds.insert(stdin);
                in_fds.insert(fork_result.master);

                let _ = nix::sys::select::select(None, Some(&mut in_fds), None, None, None)?;

                // if the terminal has any input available, then the program reads some of that
                // input and writes it to the pseudoterminal master
                if in_fds.contains(stdin) {
                    let bytes_read = nix::unistd::read(stdin, &mut buffer)?;
                    let bytes_written = master_file.write(&buffer[..bytes_read])?;

                    if bytes_read != bytes_written {
                        bail!("partial failed read[{}]/write[{}] (masterFd)", bytes_read, bytes_written);
                    }
                }

                // if the pseudterminal master has input available, this program reads some of that
                // input and writes it to the terminal and file
                if in_fds.contains(fork_result.master) {
                    let bytes_read = master_file.read(&mut buffer)?;

                    let bytes_written = nix::unistd::write(stdout, &buffer[..bytes_read])?;
                    if bytes_written != bytes_read {
                        bail!("partial failed read[{}]/write[{}] (stdout)", bytes_read, bytes_written);
                    }

                    let bytes_written = output_file.write(&buffer[..bytes_read])?;
                    if bytes_written != bytes_read {
                        bail!("partial failed read[{}]/write[{}] (output file)", bytes_read, bytes_written);
                    }
                }

            }
        }
    }

    Ok(())
}

unsafe fn get_window(stdin: &Stdin) -> Result<libc::winsize, SimpleError> {
    let mut window: std::mem::MaybeUninit<libc::winsize> = std::mem::MaybeUninit::uninit();
    let result = libc::ioctl(stdin.as_raw_fd(), libc::TIOCGWINSZ, window.as_mut_ptr());
    if result < 0 {
        bail!("Failed to get window size.");
    }
    Ok(window.assume_init())
}
