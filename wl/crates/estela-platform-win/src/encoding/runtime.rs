use std::{marker::PhantomData, rc::Rc};

use windows::Win32::{
    Foundation::RPC_E_CHANGED_MODE,
    Media::MediaFoundation::{MFShutdown, MFStartup, MFSTARTUP_FULL, MF_VERSION},
    System::Com::{CoInitializeEx, CoUninitialize, COINIT_MULTITHREADED},
};

use crate::Result;

pub(super) struct Runtime {
    started: bool,
    uninitialize: bool,
    _thread: PhantomData<Rc<()>>,
}

impl Runtime {
    pub(super) fn start() -> Result<Self> {
        // SAFETY: The runtime balances initialization on this same thread.
        let status = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if status.is_err() && status != RPC_E_CHANGED_MODE {
            status.ok()?;
        }
        let mut runtime = Self {
            started: false,
            uninitialize: status.is_ok(),
            _thread: PhantomData,
        };
        // SAFETY: COM is initialized and the owned runtime balances shutdown.
        unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL)? };
        runtime.started = true;
        Ok(runtime)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if self.started {
            // SAFETY: All owned MF objects are released before this runtime.
            let _ = unsafe { MFShutdown() };
        }
        if self.uninitialize {
            // SAFETY: Only a successful CoInitializeEx call sets this flag.
            unsafe { CoUninitialize() };
        }
    }
}
