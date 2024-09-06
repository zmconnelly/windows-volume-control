use session::{ApplicationSession, EndPointSession, Session};
use std::{ffi::OsString, os::windows::ffi::OsStringExt, process::exit};
use windows::{
    core::Interface,
    Win32::{
        Media::Audio::{
            eMultimedia, eRender, Endpoints::IAudioEndpointVolume, IAudioSessionControl, IAudioSessionControl2,
            IAudioSessionEnumerator, IAudioSessionManager2, IMMDevice, IMMDeviceEnumerator, ISimpleAudioVolume,
            MMDeviceEnumerator,
        },
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CLSCTX_ALL, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
                COINIT_MULTITHREADED,
            },
            ProcessStatus::K32GetProcessImageFileNameA,
            Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        },
    },
};

pub mod session;

pub struct AudioController {
    default_device: Option<IMMDevice>,
    imm_device_enumerator: Option<IMMDeviceEnumerator>,
    sessions: Vec<Box<dyn Session>>,
}

pub enum CoinitMode {
    MultiTreaded,
    ApartmentThreaded,
}

impl AudioController {
    pub unsafe fn init(coinit_mode: Option<CoinitMode>) -> Self {
        let mut coinit: windows::Win32::System::Com::COINIT = COINIT_MULTITHREADED;
        if let Some(x) = coinit_mode {
            match x {
                CoinitMode::ApartmentThreaded => coinit = COINIT_APARTMENTTHREADED,
                CoinitMode::MultiTreaded => coinit = COINIT_MULTITHREADED,
            }
        }
        CoInitializeEx(None, coinit).unwrap();

        Self {
            default_device: None,
            imm_device_enumerator: None,
            sessions: vec![],
        }
    }

    pub unsafe fn load_current_sessions(&mut self) {
        self.sessions.clear();
        self.get_sessions();
        self.get_default_audio_enpoint_volume_control();
        self.get_all_process_sessions();
    }

    unsafe fn get_sessions(&mut self) {
        self.imm_device_enumerator = Some(
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER).unwrap_or_else(|err| {
                eprintln!("ERROR: Couldn't get Media device enumerator: {err}");
                exit(1);
            }),
        );
    }

    unsafe fn get_default_audio_enpoint_volume_control(&mut self) {
        if self.imm_device_enumerator.is_none() {
            eprintln!("ERROR: Function called before creating enumerator");
            return;
        }

        self.default_device = Some(
            self.imm_device_enumerator
                .clone()
                .unwrap()
                .GetDefaultAudioEndpoint(eRender, eMultimedia)
                .unwrap_or_else(|err| {
                    eprintln!("ERROR: Couldn't get Default audio endpoint {err}");
                    exit(1);
                }),
        );
        let simple_audio_volume: IAudioEndpointVolume = self
            .default_device
            .clone()
            .unwrap()
            .Activate(CLSCTX_ALL, None)
            .unwrap_or_else(|err| {
                eprintln!("ERROR: Couldn't get Endpoint volume control: {err}");
                exit(1);
            });

        self.sessions.push(Box::new(EndPointSession::new(
            simple_audio_volume,
            "master".to_string(),
            0,
        )));
    }

    unsafe fn get_all_process_sessions(&mut self) {
        if self.default_device.is_none() {
            eprintln!("ERROR: Default device hasn't been initialized so the cant find the audio processes...");
            return;
        }

        let session_manager2: IAudioSessionManager2 = self
            .default_device
            .as_ref()
            .unwrap()
            .Activate(CLSCTX_INPROC_SERVER, None)
            .unwrap_or_else(|err| {
                eprintln!("ERROR: Couldnt get AudioSessionManager for enumerating over processes... {err}");
                exit(1);
            });

        let session_enumerator: IAudioSessionEnumerator =
            session_manager2.GetSessionEnumerator().unwrap_or_else(|err| {
                eprintln!("ERROR: Couldnt get session enumerator... {err}");
                exit(1);
            });

        for i in 0..session_enumerator.GetCount().unwrap() {
            let normal_session_control: Option<IAudioSessionControl> = session_enumerator.GetSession(i).ok();
            if normal_session_control.is_none() {
                eprintln!("ERROR: Couldn't get session control of audio session...");
                continue;
            }

            let session_control: Option<IAudioSessionControl2> = normal_session_control.unwrap().cast().ok();
            if session_control.is_none() {
                eprintln!("ERROR: Couldn't convert from normal session control to session control 2");
                continue;
            }

            let pid = session_control.as_ref().unwrap().GetProcessId().unwrap();
            if pid == 0 {
                continue;
            }
            let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok();
            if process.is_none() {
                eprintln!("ERROR: Couldn't get process information of process id {pid}");
                continue;
            }
            let mut filename: [u8; 128] = [0; 128];
            K32GetProcessImageFileNameA(process.unwrap(), &mut filename);
            let mut new_filename: Vec<u8> = vec![];
            for i in filename.iter() {
                if i == &(0 as u8) {
                    continue;
                }
                new_filename.push(i.clone());
            }
            let mut str_filename = match String::from_utf8(new_filename) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("ERROR: Filename couldn't be converted to string, {err}");
                    continue;
                }
            };
            str_filename = match str_filename.split("\\").last() {
                Some(data) => data.to_string().replace(".exe", ""),
                None => {
                    continue;
                }
            };
            let audio_control: ISimpleAudioVolume = match session_control.unwrap().cast() {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("ERROR: Couldn't get the simpleaudiovolume from session controller: {err}");
                    continue;
                }
            };
            let application_session = ApplicationSession::new(audio_control, str_filename, pid);
            self.sessions.push(Box::new(application_session));
        }
    }

    pub unsafe fn get_all_session_names(&self) -> Vec<String> {
        self.sessions.iter().map(|session| session.get_name()).collect()
    }

    pub unsafe fn get_all_sessions(&self) -> Vec<&Box<dyn Session>> {
        self.sessions.iter().collect()
    }

    pub unsafe fn get_session_with_name(&self, name: String) -> Option<&Box<dyn Session>> {
        self.sessions
            .iter()
            .find(|session| session.get_name().to_lowercase() == name.to_lowercase())
    }

    pub unsafe fn get_all_sessions_with_name(&self, name: String) -> Vec<&Box<dyn Session>> {
        self.sessions
            .iter()
            .filter(|session| session.get_name().to_lowercase() == name.to_lowercase())
            .collect()
    }
}
