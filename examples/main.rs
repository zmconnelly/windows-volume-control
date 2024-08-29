
use windows_volume_control::AudioController;

fn main() {
    unsafe {
        let mut controller = AudioController::init(None);
        controller.get_sessions();
        controller.get_default_audio_enpoint_volume_control();
        controller.get_all_process_sessions();

        let master_session = controller.get_session_by_name("master".to_string());
        println!("Master Volume: {:?}",master_session.unwrap().get_volume());


        let session_names = controller.get_all_session_names();
        println!("{:?}",session_names);

        let all_sessions = controller.get_all_sessions();
        for session in all_sessions {
            println!("Session: {} Volume: {}",session.get_name(),session.get_volume());
        }
    }
}
