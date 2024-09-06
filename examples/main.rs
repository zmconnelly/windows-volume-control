use windows_volume_control::AudioController;

fn main() {
    unsafe {
        let mut controller = AudioController::init(None);
        controller.load_current_sessions();

        let test = controller.get_all_session_names();
        println!("{:?}", test);

        controller.load_current_sessions();

        let master_session = controller.get_session_with_name("master".to_string());
        println!("Master Volume: {:?}", master_session.unwrap().get_volume());

        let session_names = controller.get_all_session_names();
        println!("{:?}", session_names);

        let all_sessions = controller.get_all_sessions();
        for session in all_sessions {
            println!(
                "Session: {} Volume: {} Mute: {} PID: {}",
                session.get_name(),
                session.get_volume(),
                session.get_mute(),
                session.get_pid()
            );
        }
    }
}
