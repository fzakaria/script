extern crate pty;
extern crate nix;
extern crate libc;

#[macro_use]
extern crate simple_error;

use std::os::unix::io::FromRawFd;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use simple_error::SimpleError;
use std::io::{Read, Write};

fn run( stdin: RawFd, stdout: RawFd, fork_result: nix::pty::ForkptyResult) -> std::result::Result<(), Box<dyn std::error::Error> >  {

    match fork_result.fork_result {

        // the child simply exec's into a shell
        // we check the SHELL environment variable otherwise default to /bin/sh
        nix::unistd::ForkResult::Child => {
            let shell = std::env::var_os("SHELL")
                .unwrap_or(std::ffi::OsString::from("/bin/sh"))
                .into_string().expect("We expected to convert from OString to String");


            let c_str = std::ffi::CString::new(shell).expect("CString::new failed");
            nix::unistd::execv(&c_str, &[&c_str])?;
        }

        // the parent will relay data between terminal and pty master
        nix::unistd::ForkResult::Parent { child, .. } => {
            let mut master_file : std::fs::File = unsafe {
                std::fs::File::from_raw_fd(fork_result.master)
            };

            // this should print '/dev/ptmx' as the master device
            // https://linux.die.net/man/4/ptmx
            // Each file descriptor obtained by opening /dev/ptmx
            // is an independent PTM with its own associated pseudoterminal slaves (PTS)
            println!("Starting scriptr...");
            println!("Currently in parent process.");
            println!("Child Pid: {:?}", child);
            println!("Master Fd: {:?}", master_file);

            let mut output_file = std::fs::File::create("typescript")?;

            let mut tty = nix::sys::termios::tcgetattr(stdin)?;
            nix::sys::termios::cfmakeraw(&mut tty);
            nix::sys::termios::tcsetattr(stdin, nix::sys::termios::SetArg::TCSAFLUSH, &tty)?;

            let mut in_fds = nix::sys::select::FdSet::new();

            let mut buffer = [0; 256];
            loop {

                in_fds.clear();
                in_fds.insert(stdin);
                in_fds.insert(fork_result.master);

                let _ = nix::sys::select::select(None, Some(&mut in_fds), None, None, None)?;

                // if the terminal has any input available, then the program reads some of that
                // input and writes it to the pseudo-terminal master
                if in_fds.contains(stdin) {
                    let bytes_read_result = nix::unistd::read(stdin, &mut buffer);

                    // IO error here is a happy case since the child process might have died
                    // hide it by returning OK
                    let bytes_read = match bytes_read_result {
                        Ok(bytes_read) => bytes_read,
                        Err(_) => return Ok(())
                    };

                    let bytes_written = master_file.write(&buffer[..bytes_read])?;

                    //flush it
                    master_file.flush()?;
                    if bytes_read != bytes_written {
                        bail!("partial failed read[{}]/write[{}] (masterFd)", bytes_read, bytes_written);
                    }
                }

                // if the pseudo-terminal master has input available, this program reads some of that
                // input and writes it to the terminal and file
                if in_fds.contains(fork_result.master) {
                    let bytes_read_result = master_file.read(&mut buffer);

                    // IO error here is a happy case since the child process might have died
                    // hide it by returning OK
                    let bytes_read = match bytes_read_result {
                        Ok(bytes_read) => bytes_read,
                        Err(_) => return Ok(())
                    };

                    let bytes_written = nix::unistd::write(stdout, &buffer[..bytes_read])?;
                    if  bytes_written != bytes_read {
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

fn main() -> std::result::Result<(), Box<dyn std::error::Error> >  {
    let stdin = std::io::stdin().as_raw_fd();
    let stdout = std::io::stdout().as_raw_fd();

    // Get the original terminal attributes
    // we want to restore the terminal to the original attributes
    // once the program finishes
    let tty = nix::sys::termios::tcgetattr(stdin)?;

    // grab the window information for the fork
    let window : libc::winsize = unsafe {
        get_window(stdin)?
    };

    // create a child process that is connected to this process via a pseudo-terminal
    let fork_result = nix::pty::forkpty(Some(&window), Some(&tty))?;

    // run the script program read-print-loop
    let run_result = run(stdin, stdout, fork_result);

    // Restore the original tty settings to remove the non-canonical mode we set
    let reset_tty_result = nix::sys::termios::tcsetattr(stdin, nix::sys::termios::SetArg::TCSANOW, &tty)
        .map_err(|err| err.into());

    return run_result.and(reset_tty_result);
}

unsafe fn get_window(stdin: RawFd) -> Result<libc::winsize, SimpleError> {
    let mut window: std::mem::MaybeUninit<libc::winsize> = std::mem::MaybeUninit::uninit();
    let result = libc::ioctl(stdin, libc::TIOCGWINSZ, window.as_mut_ptr());
    if result < 0 {
        bail!("Failed to get window size.");
    }
    Ok(window.assume_init())
}

