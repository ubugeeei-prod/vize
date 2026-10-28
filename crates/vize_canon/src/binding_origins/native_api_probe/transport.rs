//! Test-only byte custody: Unix relay to one standalone native stdio process.
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    net::Shutdown,
    os::unix::net::{UnixListener, UnixStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};
use vize_carton::cstr;

struct RecordedReader<R> {
    input: R,
    capture: File,
}
impl<R: Read> Read for RecordedReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = self.input.read(buffer)?;
        self.capture.write_all(buffer.split_at(count).0)?;
        Ok(count)
    }
}
struct RecordedWriter<W> {
    output: W,
    capture: File,
}
impl<W: Write> Write for RecordedWriter<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let count = self.output.write(buffer)?;
        self.capture.write_all(buffer.split_at(count).0)?;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()?;
        self.capture.flush()
    }
}
struct NativeChild(Child);
impl Drop for NativeChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

pub(super) struct NativeTap {
    child: NativeChild,
    socket: PathBuf,
    stop: Sender<()>,
    control: Receiver<UnixStream>,
    relay: Option<JoinHandle<io::Result<(u64, u64)>>>,
}
impl NativeTap {
    pub(super) fn start(launcher: &Path, backend: &Path, root: &Path, output: &Path) -> Self {
        let socket = output.join("api-relay.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let mut child = NativeChild(
            Command::new("setpriv")
                .args(["--pdeathsig", "KILL", "--"])
                .arg(launcher)
                .args(["--api", "--async", "--cwd"])
                .arg(root)
                .current_dir(root)
                .env("VIZE_ORIGIN_API_BACKEND", backend)
                .env("VIZE_ORIGIN_API_CAPTURE", output)
                .env("VIZE_ORIGIN_API_PARENT_PID", std::process::id().to_string())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(File::create(output.join("native.stderr.txt")).unwrap())
                .spawn()
                .unwrap(),
        );
        let mut native_in = RecordedWriter {
            output: child.0.stdin.take().unwrap(),
            capture: File::create(output.join("client-to-native.wire.bin")).unwrap(),
        };
        let mut native_out = RecordedReader {
            input: child.0.stdout.take().unwrap(),
            capture: File::create(output.join("native-to-client.wire.bin")).unwrap(),
        };
        let (stop, stopped) = mpsc::channel();
        let (control_sender, control) = mpsc::channel();
        let relay = thread::spawn(move || {
            let mut front = loop {
                if stopped.try_recv().is_ok() {
                    return Ok((0, 0));
                }
                match listener.accept() {
                    Ok((front, _)) => break front,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => return Err(error),
                }
            };
            front.set_nonblocking(false)?;
            control_sender.send(front.try_clone()?).unwrap();
            let mut read_front = front.try_clone()?;
            let incoming = thread::spawn(move || io::copy(&mut read_front, &mut native_in));
            let outgoing = io::copy(&mut native_out, &mut front);
            let _ = front.shutdown(Shutdown::Both);
            let incoming = incoming
                .join()
                .map_err(|_| io::Error::other("input relay panicked"))??;
            Ok((incoming, outgoing?))
        });
        Self {
            child,
            socket,
            stop,
            control,
            relay: Some(relay),
        }
    }
    pub(super) fn socket(&self) -> &Path {
        &self.socket
    }
    pub(super) fn pid(&self) -> u32 {
        self.child.0.id()
    }
    pub(super) fn finish(&mut self) -> io::Result<(u64, u64)> {
        if self.child.0.try_wait()?.is_none() {
            self.child.0.kill()?;
        }
        self.child.0.wait()?;
        let _ = self.stop.send(());
        if let Ok(front) = self.control.try_recv() {
            let _ = front.shutdown(Shutdown::Both);
        }
        let result = self
            .relay
            .take()
            .map(|relay| {
                relay
                    .join()
                    .map_err(|_| io::Error::other("relay panicked"))?
            })
            .unwrap_or(Ok((0, 0)));
        assert!(!Path::new(cstr!("/proc/{}", self.child.0.id()).as_str()).exists());
        let _ = fs::remove_file(&self.socket);
        result
    }
}
impl Drop for NativeTap {
    fn drop(&mut self) {
        if self.relay.is_some() {
            let _ = self.finish();
        }
    }
}
