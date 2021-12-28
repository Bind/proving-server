use std::sync::mpsc;

pub fn worker(trigger: mpsc::Receiver<()>) {
    loop {
        trigger.recv().unwrap();
        println!("Hello!");
    }
}
