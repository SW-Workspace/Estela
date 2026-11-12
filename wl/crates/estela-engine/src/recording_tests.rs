use super::*;

struct TestSource;

impl Session for TestSource {
    fn next_frame(&mut self, _timeout: Duration) -> estela_capture::Result<Option<Frame>> {
        Ok(Some(Frame {
            width: 64,
            height: 48,
            rgba: vec![128; 64 * 48 * 4],
        }))
    }
    fn pointer(&self) -> estela_capture::Result<Pointer> {
        Ok(Pointer::default())
    }
    fn cursor_embedded(&self) -> bool {
        false
    }
}

fn wait_for(events: &Receiver<Event>, predicate: impl Fn(&Event) -> bool) -> Event {
    loop {
        let event = events.recv_timeout(Duration::from_secs(10)).unwrap();
        if let Event::Failed(message) = &event {
            panic!("{message}");
        }
        if predicate(&event) {
            return event;
        }
    }
}

#[test]
#[ignore = "Requires dynamically linked LGPL FFmpeg on PATH."]
fn pauses_resumes_and_finalizes_only_after_stop() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("estela-session-{unique}.estela"));
    let output_path = path.clone();
    let (commands, receiver) = mpsc::channel();
    let (sender, events) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        record_session(Box::new(TestSource), output_path, 30, receiver, &sender).unwrap()
    });
    wait_for(&events, |event| matches!(event, Event::Prepared { .. }));
    assert!(!path.exists());
    commands.send(Command::Start).unwrap();
    wait_for(&events, |event| matches!(event, Event::Progress { .. }));
    commands.send(Command::Pause).unwrap();
    wait_for(&events, |event| matches!(event, Event::Paused));
    assert!(events.recv_timeout(Duration::from_millis(150)).is_err());
    assert!(!path.exists());
    commands.send(Command::Resume).unwrap();
    wait_for(&events, |event| matches!(event, Event::Progress { .. }));
    commands.send(Command::Stop).unwrap();
    wait_for(&events, |event| matches!(event, Event::Finished { .. }));
    worker.join().unwrap();
    assert!(estela_format::reader::open(&path).is_ok());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn canceling_preparation_does_not_create_a_file() {
    let path = std::env::temp_dir().join(format!("estela-canceled-{}.estela", std::process::id()));
    let (commands, receiver) = mpsc::channel();
    let (sender, events) = mpsc::channel();
    commands.send(Command::Discard).unwrap();
    record_session(Box::new(TestSource), path.clone(), 30, receiver, &sender).unwrap();
    assert!(events
        .try_iter()
        .any(|event| matches!(event, Event::Discarded)));
    assert!(!path.exists());
}

struct SlowSource;

impl Session for SlowSource {
    fn next_frame(&mut self, timeout: Duration) -> estela_capture::Result<Option<Frame>> {
        std::thread::sleep(Duration::from_millis(80));
        TestSource.next_frame(timeout)
    }
    fn pointer(&self) -> estela_capture::Result<Pointer> {
        Ok(Pointer::default())
    }
    fn cursor_embedded(&self) -> bool {
        false
    }
}

#[test]
#[ignore = "Requires dynamically linked LGPL FFmpeg on PATH."]
fn slow_capture_preserves_wall_clock_duration() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("estela-slow-{unique}.estela"));
    let output_path = path.clone();
    let (commands, receiver) = mpsc::channel();
    let (sender, events) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        record_session(Box::new(SlowSource), output_path, 60, receiver, &sender).unwrap()
    });
    wait_for(&events, |event| matches!(event, Event::Prepared { .. }));
    let started = Instant::now();
    commands.send(Command::Start).unwrap();
    wait_for(
        &events,
        |event| matches!(event, Event::Progress { seconds } if *seconds >= 1.0),
    );
    commands.send(Command::Stop).unwrap();
    let elapsed = started.elapsed().as_secs_f64();
    let event = wait_for(&events, |event| matches!(event, Event::Finished { .. }));
    if let Event::Finished { recording, .. } = event {
        assert!((recording.duration.0 as f64 / 1e9 - elapsed).abs() < 0.3);
    }
    worker.join().unwrap();
    std::fs::remove_file(path).unwrap();
}
