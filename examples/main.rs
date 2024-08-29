use std::time::Duration;

use windows_volume_control::AudioController;

fn main() {
    unsafe {
        let mut controller = AudioController::init(None);
        controller.get_sessions();
        controller.get_default_audio_enpoint_volume_control();
        controller.get_all_process_sessions();
        let test = controller.get_all_session_names();
        let master_session = controller.get_session_by_name("master".to_string());
        println!("{:?}",master_session.unwrap().get_volume());
    }
}
