extern crate pty;
extern crate nix;
extern crate libc;

#[macro_use]
extern crate simple_error;

use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use simple_error::SimpleError;
use std::io::Write;

struct Child {
    shell: std::ffi::CString,
}

impl Child {
    fn from_env() -> std::result::Result<Child, Box<dyn std::error::Error>> {
        // we check the SHELL environment variable otherwise default
        // to /bin/sh
        let shell = std::env::var_os("SHELL")
            .unwrap_or(std::ffi::OsString::from("/bin/sh"))
            .into_string()
            .map_err(|_| { SimpleError::new("could not decode SHELL environment variable") })?;
        let shell = std::ffi::CString::new(shell)?;
        Ok(Child{shell})
    }

    fn run(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // the child simply execs into the shell
        match nix::unistd::execv(&self.shell, &[&self.shell]) {
            Ok(_) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

struct Parent {
    child: nix::unistd::Pid,
    stdin: RawFd,
    stdout: RawFd,
    master_pty: RawFd,
    typescript: std::fs::File,
}

impl Parent {
    fn stdin_raw_mode(&self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut tty = nix::sys::termios::tcgetattr(self.stdin)?;
        nix::sys::termios::cfmakeraw(&mut tty);
        nix::sys::termios::tcsetattr(self.stdin, nix::sys::termios::SetArg::TCSANOW, &tty)
            .map_err(|err| { err.into() })
    }

    fn run(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        println!("Starting scriptr...");
        println!("Currently in parent process.");
        println!("Child Pid: {:?}", self.child);
        println!("Master Fd: {:?}", self.master_pty);

        self.stdin_raw_mode()?;

        let mut in_fds = nix::sys::select::FdSet::new();
        let mut buffer = [0; 256];
        loop {
            in_fds.clear();
            in_fds.insert(self.stdin);
            in_fds.insert(self.master_pty);

            let _ = nix::sys::select::select(None, Some(&mut in_fds), None, None, None)?;

            // if the terminal has any input available, then the program reads some of that
            // input and writes it to the pseudo-terminal master
            if in_fds.contains(self.stdin) {
                let bytes_read_result = nix::unistd::read(self.stdin, &mut buffer);

                // IO error here is a happy case since the child process might have died
                // hide it by returning OK
                let bytes_read = match bytes_read_result {
                    Ok(bytes_read) => bytes_read,
                    Err(_) => return Ok(())
                };

                let bytes_written = nix::unistd::write(self.master_pty, &buffer[..bytes_read])?;

                if bytes_read != bytes_written {
                    bail!("partial failed read[{}]/write[{}] (masterFd)", bytes_read, bytes_written);
                }
            }

            // if the pseudo-terminal master has input available, this program reads some of that
            // input and writes it to the terminal and file
            if in_fds.contains(self.master_pty) {
                let bytes_read_result = nix::unistd::read(self.master_pty, &mut buffer);

                // IO error here is a happy case since the child process might have died
                // hide it by returning OK
                let bytes_read = match bytes_read_result {
                    Ok(bytes_read) => bytes_read,
                    Err(_) => return Ok(())
                };

                let bytes_written = nix::unistd::write(self.stdout, &buffer[..bytes_read])?;
                if  bytes_written != bytes_read {
                    bail!("partial failed read[{}]/write[{}] (stdout)", bytes_read, bytes_written);
                }

                let bytes_written = self.typescript.write(&buffer[..bytes_read])?;
                if bytes_written != bytes_read {
                    bail!("partial failed read[{}]/write[{}] (output file)", bytes_read, bytes_written);
                }
            }
        }
    }
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error> >  {
    let stdin = std::io::stdin().as_raw_fd();
    let stdout = std::io::stdout().as_raw_fd();

    // Get the original terminal attributes
    // we want to restore the terminal to the original attributes
    // once the program finishes
    let tty = nix::sys::termios::tcgetattr(stdin)?;

    // grab the window information for the fork
    let window : libc::winsize = get_window(stdin)?;

    let child = Child::from_env()?;

    // create a child process that is connected to this process via a pseudo-terminal
    let pty_fork_result = nix::pty::forkpty(Some(&window), Some(&tty))?;

    let run_result = match pty_fork_result.fork_result {
        nix::unistd::ForkResult::Child => {
            child.run()
        },
        nix::unistd::ForkResult::Parent { child } => {
            let master_pty = pty_fork_result.master;
            let typescript = std::fs::File::create("typescript")?;
            let mut parent = Parent{
                child,
                stdin,
                stdout,
                master_pty,
                typescript,
            };
            parent.run()
        }
    };

    // Restore the original tty settings to remove the non-canonical mode we set
    let reset_tty_result = nix::sys::termios::tcsetattr(stdin, nix::sys::termios::SetArg::TCSANOW, &tty)
        .map_err(|err| err.into());

    return run_result.and(reset_tty_result);
}

fn get_window(stdin: RawFd) -> Result<libc::winsize, SimpleError> {
    let mut window = libc::winsize{
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let result = unsafe {
        libc::ioctl(stdin, libc::TIOCGWINSZ, &mut window)
    };

    if result < 0 {
        bail!("Failed to get window size.");
    }

    Ok(window)
}

