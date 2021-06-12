use rollup_state_manager::test_utils::messages::{parse_msg, WrappedMessage};
use std::fs::File;
use std::io::{BufRead, BufReader};
pub fn load_msgs_from_file(filepath: &str, sender: crossbeam_channel::Sender<WrappedMessage>) -> Option<std::thread::JoinHandle<()>> {
    let filepath = filepath.to_string();
    Some(std::thread::spawn(move || {
        let file = File::open(filepath).unwrap();
        // since
        for l in BufReader::new(file).lines() {
            let msg = parse_msg(l.unwrap()).unwrap();
            sender.try_send(msg).unwrap();
        }
    }))
}

pub fn load_msgs_from_mq(_broker: &str, _sender: crossbeam_channel::Sender<WrappedMessage>) -> Option<std::thread::JoinHandle<()>> {
    // TODO
    None
}