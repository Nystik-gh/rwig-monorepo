use std::{error::Error, mem::transmute};

use detour::static_detour;
use windows::{
    core::HRESULT,
    Win32::Media::Audio::{IAudioClient, IAudioClient_Vtbl},
};

static_detour! {
  pub static GET_CURRENT_PADDING_DETOUR: extern "system" fn(IAudioClient, *mut u32) -> HRESULT;
  pub static GET_BUFFER_SIZE_DETOUR: extern "system" fn(IAudioClient, *mut u32) -> HRESULT;
}

pub fn get_current_padding(this: IAudioClient, pnumpaddingframes: *mut u32) -> HRESULT {
    //println!("get_current_padding, {:?}", pnumpaddingframes);
    unsafe { GET_CURRENT_PADDING_DETOUR.call(this, pnumpaddingframes) }
}

pub fn get_buffer_size(this: IAudioClient, pnumbufferframes: *mut u32) -> HRESULT {
    //println!("get_buffer_size");
    unsafe { GET_BUFFER_SIZE_DETOUR.call(this, pnumbufferframes) }
}

pub unsafe fn initialize_audio_client_detours(
    vtable: *const IAudioClient_Vtbl,
) -> Result<(), detour::Error> {
    println!("initialize_audio_client_detour");
    let vtbl = &(*vtable);

    if let Err(e) = GET_CURRENT_PADDING_DETOUR
        .initialize(transmute(vtbl.GetCurrentPadding), get_current_padding)
    {
        match e {
            detour::Error::AlreadyInitialized => (),
            _ => return Err(e),
        }
    }

    if let Err(e) =
        GET_BUFFER_SIZE_DETOUR.initialize(transmute(vtbl.GetBufferSize), get_buffer_size)
    {
        match e {
            detour::Error::AlreadyInitialized => (),
            _ => return Err(e),
        }
    }

    Ok(())
}

pub unsafe fn attach_audio_client_detours() -> Result<(), detour::Error> {
    println!("attach_audio_client_detour");

    GET_CURRENT_PADDING_DETOUR.enable()?;
    GET_BUFFER_SIZE_DETOUR.enable()?;

    Ok(())
}

pub unsafe fn detach_audio_client_detours() -> Result<(), detour::Error> {
    println!("detach_audio_client_detour");

    GET_CURRENT_PADDING_DETOUR.disable()?;
    GET_BUFFER_SIZE_DETOUR.disable()?;

    Ok(())
}
