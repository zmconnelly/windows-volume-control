use windows_volume_control::AudioController;

fn main() {
    unsafe {
        let controller = AudioController::init(None);

        let master_session = controller.get_session_by_name("master".to_string());
        println!("Master Volume: {:?}", master_session.unwrap().get_volume());

        let session_names = controller.get_all_session_names();
        println!("{:?}", session_names);

        let all_sessions = controller.get_all_sessions();
        for session in all_sessions {
            println!("Session: {} Volume: {}", session.get_name(), session.get_volume());
        }
    }
}
