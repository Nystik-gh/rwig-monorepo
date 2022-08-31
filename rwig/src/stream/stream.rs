use std::io::Read;

use ipc_channel::ipc::IpcBytesReceiver;

use std::io::Error as IoError;

pub struct CaptureStream {
    receiver: IpcBytesReceiver,
}

impl CaptureStream {
    pub fn from_receiver(receiver: IpcBytesReceiver) -> CaptureStream {
        CaptureStream { receiver }
    }
}

impl Read for CaptureStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let frames = match self.receiver.try_recv() {
            Ok(res) => res,
            Err(e) => match e {
                ipc_channel::ipc::TryRecvError::IpcError(err) => match err {
                    ipc_channel::ipc::IpcError::Bincode(_) => {
                        return Err(IoError::new(std::io::ErrorKind::InvalidData, "bad data"))
                    }
                    ipc_channel::ipc::IpcError::Io(io_err) => return Err(io_err),
                    ipc_channel::ipc::IpcError::Disconnected => {
                        return Err(IoError::new(
                            std::io::ErrorKind::BrokenPipe,
                            "channel disconnected",
                        ))
                    }
                },
                ipc_channel::ipc::TryRecvError::Empty => {
                    return Err(IoError::new(std::io::ErrorKind::Interrupted, "no data"))
                }
            },
        };

        let f_len = frames.len();

        let mut n = 0;
        for (i, b) in buf.iter_mut().enumerate() {
            if i < f_len {
                *b = frames[i];
                n += 1;
            } else {
                break;
            }
        }

        Ok(n)
    }
}

unsafe impl Send for CaptureStream {}
unsafe impl Sync for CaptureStream {}
