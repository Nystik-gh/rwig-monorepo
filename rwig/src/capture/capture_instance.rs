use std::sync::{Arc, Mutex};

use dll_syringe::{error::InjectError, rpc::RemotePayloadProcedure};
use ipc_channel::ipc::{self, IpcBytesReceiver, IpcOneShotServer, IpcReceiver, IpcSender};
use rwig_common::{
    audio::MixFormat,
    ipc::IpcMessage,
    utils::error::{InjectionError, IpcError, PatchError, RwigError},
};

use crate::{
    hook::Injector,
    ipc::{
        connect_ipc, remote_disable_detour, remote_eject, remote_enable_detour,
        remote_get_mix_format,
    },
    stream::CaptureStream,
};

pub struct Capture {
    message_sender: IpcSender<IpcMessage>,
    message_receiver: IpcReceiver<IpcMessage>,
    capture_stream: Arc<Mutex<CaptureStream>>,
    pub mix_format: MixFormat,
    injector: Injector,
    pid: u32,
    ejected: bool,
}

impl Capture {
    pub fn new(pid: u32, injector: Injector) -> Result<Capture, RwigError> {
        let handshake_proc = injector.inject(pid)?;

        let (message_sender, message_receiver, bytes_receiver) = connect_ipc(handshake_proc)?;

        let mix_format = remote_get_mix_format(&message_sender, &message_receiver)
            .expect("failed getting mix format");

        let capture_stream = Arc::new(Mutex::new(CaptureStream::from_receiver(bytes_receiver)));

        Ok(Capture {
            message_sender,
            message_receiver,
            capture_stream,
            mix_format,
            injector,
            pid,
            ejected: false,
        })
    }

    pub fn start(&self) -> Result<(), IpcError> {
        remote_enable_detour(&self.message_sender, &self.message_receiver)?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), IpcError> {
        remote_disable_detour(&self.message_sender, &self.message_receiver)?;
        Ok(())
    }

    pub fn close(&self) -> Result<(), RwigError> {
        //remote_disable_detour(&self.message_sender, &self.message_receiver)?;
        if !self.ejected {
            remote_eject(&self.message_sender, &self.message_receiver)?;
            self.injector.eject(self.pid)?;
        }

        Ok(())
    }

    pub fn get_capture_stream(&self) -> Arc<Mutex<CaptureStream>> {
        return self.capture_stream.clone();
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.close().ok();
    }
}
