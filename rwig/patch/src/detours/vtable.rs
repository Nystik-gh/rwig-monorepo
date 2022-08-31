use std::error::Error;

use windows::{
    core::Interface,
    Win32::{
        Media::Audio::{
            IAudioClient, IAudioClient_Vtbl, IAudioRenderClient, IAudioRenderClient_Vtbl,
            AUDCLNT_SHAREMODE_SHARED,
        },
        System::Com::CLSCTX_ALL,
    },
};

use crate::{utils::get_default_output_device, utils::get_device_name};

pub struct VTables {
    pub iaudioclient_vtbl: *const IAudioClient_Vtbl,
    pub iaudiorenderclient_vtbl: *const IAudioRenderClient_Vtbl,
}

pub fn get_vtables() -> Result<VTables, Box<dyn Error>> {
    let immdevice = get_default_output_device()?;
    let name = get_device_name(&immdevice)?;
    println!("Device: {}", name);
    unsafe {
        let iaduioclient: IAudioClient = immdevice.Activate(CLSCTX_ALL, std::ptr::null_mut())?;

        let pformat = iaduioclient.GetMixFormat()?;
        iaduioclient.Initialize(AUDCLNT_SHAREMODE_SHARED, 0, 0, 0, pformat, std::ptr::null())?;

        let iaudiorenderclient = iaduioclient.GetService::<IAudioRenderClient>()?;

        Ok(VTables {
            iaudioclient_vtbl: Interface::vtable(&iaduioclient),
            iaudiorenderclient_vtbl: Interface::vtable(&iaudiorenderclient),
        })
    }
}
