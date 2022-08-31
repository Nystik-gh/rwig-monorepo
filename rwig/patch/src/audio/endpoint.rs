use std::sync::{Arc, Mutex};

use ipc_channel::ipc::IpcBytesSender;
use once_cell::sync::OnceCell;
use rwig_common::{audio::MixFormat, utils::error::PatchError};

pub static ENDPOINT: OnceCell<Arc<Mutex<Option<SimulatedEndpoint>>>> = OnceCell::new();

pub struct EndpointBuffer {
    buffer_size: usize,
    pbuffer: *mut [u8],
    padding: u32,
}

impl EndpointBuffer {
    pub fn new(buffer_size: usize) -> EndpointBuffer {
        let buf = vec![0u8; buffer_size].into_boxed_slice();
        let pbuffer = Box::into_raw(buf);
        EndpointBuffer {
            buffer_size,
            pbuffer,
            padding: 0,
        }
    }

    pub fn read_buffer(&self, length: usize) -> Vec<u8> {
        let mut out = vec![];
        unsafe {
            for (i, byte) in (*self.pbuffer).iter().enumerate() {
                if i == length {
                    break;
                }
                out.push(*byte)
            }
        }
        out
    }
}

pub struct SimulatedEndpoint {
    pub buffer_size: usize,
    pub format: MixFormat,
    pub buffer: EndpointBuffer,
    sender: IpcBytesSender,
    buffer_lock: bool,
}

unsafe impl Send for SimulatedEndpoint {}
unsafe impl Sync for SimulatedEndpoint {}

impl SimulatedEndpoint {
    pub fn new(sender: IpcBytesSender, buffer_size: usize, format: MixFormat) -> SimulatedEndpoint {
        SimulatedEndpoint {
            buffer_size,
            format,
            buffer: EndpointBuffer::new(buffer_size),
            sender,
            buffer_lock: false,
        }
    }

    pub fn get_padding(&self) -> usize {
        0
    }

    pub fn get_buffer_pointer(&self) -> *mut [u8] {
        self.buffer.pbuffer
    }

    pub fn release_buffer(&self, frames_written: u32) {
        let length = frames_written as usize * self.format.format.block_align as usize;
        let data = self.buffer.read_buffer(length);
        //self.sender.send(data.as_slice()).ok();
    }
}

pub fn set_endpoint(
    //receiver: IpcReceiver<IpcMessage>,
    endpoint: SimulatedEndpoint,
) -> Result<(), PatchError> {
    match ENDPOINT.get() {
        Some(arc) => {
            if let Ok(mut opt) = arc.lock() {
                let _ = opt.insert(endpoint);
            }
        }
        None => {
            return ENDPOINT
                .set(Arc::new(Mutex::new(Some(endpoint))))
                .map_err(|_| {
                    PatchError::ChannelUnavailable("failed to set MESSAGE_SENDER".to_string())
                });
        }
    };

    Ok(())
}

pub fn get_endpoint() -> Arc<Mutex<Option<SimulatedEndpoint>>> {
    let arc = ENDPOINT.get().expect("ENDPOINT is not initialized");
    arc.clone()
}
