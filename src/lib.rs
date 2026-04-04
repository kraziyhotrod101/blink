use pamsm::{pam_module, Pam, PamError, PamFlags, PamServiceModule};
use std::os::unix::net::UnixStream;
use std::io::{Read, Write};
use std::time::Duration;

struct Blink;

impl PamServiceModule for Blink {
    fn authenticate(_pamh: Pam, _flags: PamFlags, _args: Vec<String>) -> PamError {
        // Connect to the daemon
        let mut stream = match UnixStream::connect("/var/run/blink.sock") {
            Ok(s) => s,
            Err(_) => return PamError::AUTH_ERR,
        };

        // Set a 5-second timeout so we don't hang sudo forever
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

        // Send "Check Face" trigger
        if stream.write_all(&[1]).is_err() {
            return PamError::AUTH_ERR;
        }

        // Wait for response
        let mut response = [0; 1];
        if stream.read_exact(&mut response).is_ok() {
            if response[0] == 1 {
                return PamError::SUCCESS;
            }
        }

        PamError::AUTH_ERR
    }

    fn setcred(_: Pam, _: PamFlags, _: Vec<String>) -> PamError {
        PamError::SUCCESS
    }
}

pam_module!(Blink);