use windows::Win32::Media::Audio::{Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl}, AUDIO_VOLUME_NOTIFICATION_DATA};
use windows_volume_control::AudioController;

#[windows::core::implement(IAudioEndpointVolumeCallback)]
struct VolumeChangeCallback;

impl IAudioEndpointVolumeCallback_Impl for VolumeChangeCallback {
    fn OnNotify(&self, pnotify: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> ::windows::core::Result<()> {
        unsafe {
            println!("volume changed: {}", (*pnotify).fMasterVolume);
        }
        return Ok(());
    }
}

fn main() {
    unsafe {
        let mut controller = AudioController::init(None);
        controller.get_sessions();
        controller.get_default_audio_enpoint_volume_control();
        controller.get_all_process_sessions();

        let session = controller.get_session_by_name("master".to_string()).unwrap();

        if let Some(session_endpoint_volume) = session.get_audio_endpoint_volume() {
            let volume_callback: IAudioEndpointVolumeCallback = VolumeChangeCallback {}.into();
            session_endpoint_volume.RegisterControlChangeNotify(&volume_callback).unwrap();
            println!("Initialised audio event listener for session 'master'");
            loop {};
        }
    }
}