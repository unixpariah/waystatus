use hyprland::{event_listener::EventListener, shared::HyprDataActive};
use inotify::{Inotify, WatchMask};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use tokio::sync::broadcast;

#[derive(Debug, Serialize, Deserialize)]
pub enum Trigger {
    WorkspaceChanged,
    TimePassed(u64),
    FileChange(PathBuf),
}

pub enum WorkspaceListener {
    Hyprland(EventListener),
}

pub struct WorkspaceListenerData {
    tx: broadcast::Sender<bool>,
    listener: WorkspaceListener,
}

pub struct TimeListenerData {
    tx: broadcast::Sender<bool>,
    interval: u64,
    original_interval: u64,
}

pub struct FileChangeListenerData {
    tx: broadcast::Sender<bool>,
    inotify: Inotify,
}

pub struct Listeners {
    pub workspace_listener: Arc<Mutex<Option<WorkspaceListenerData>>>,
    pub file_change_listener: Arc<Mutex<Option<FileChangeListenerData>>>,
    pub time_passed_listener: Arc<Mutex<Vec<TimeListenerData>>>,
}

impl Listeners {
    pub fn new() -> Self {
        Self {
            file_change_listener: Arc::new(Mutex::new(None)),
            workspace_listener: Arc::new(Mutex::new(None)),
            time_passed_listener: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn new_workspace_listener(&mut self) -> broadcast::Receiver<bool> {
        if let Some(workspace_listener) = self.workspace_listener.lock().unwrap().as_ref() {
            return workspace_listener.tx.subscribe();
        }

        let listener;

        let (tx, rx) = broadcast::channel(1);

        listener = if hyprland::data::Workspace::get_active().is_ok() {
            let mut listener = EventListener::new();
            {
                let tx = tx.clone();
                listener.add_workspace_destroy_handler(move |_| {
                    _ = tx.send(true);
                });
            }

            {
                let tx = tx.clone();
                listener.add_workspace_change_handler(move |_| {
                    _ = tx.send(true);
                });
            }

            {
                let tx = tx.clone();
                listener.add_active_monitor_change_handler(move |_| {
                    _ = tx.send(true);
                });
            }

            WorkspaceListener::Hyprland(listener)
        } else {
            return rx;
        };

        self.workspace_listener =
            Arc::new(Mutex::new(Some(WorkspaceListenerData { tx, listener })));
        rx
    }

    pub fn start_listeners(&mut self) {
        let time_passed_listener = Arc::clone(&self.time_passed_listener);
        let file_change_listener = Arc::clone(&self.file_change_listener);
        let workspace_listener = Arc::clone(&self.workspace_listener);

        // TLDR: thread sorts listeners by interval, waits for the shortest interval sends the message
        // to the listeners whose interval has passed and resets the interval in a loop
        if !time_passed_listener.lock().unwrap().is_empty() {
            thread::spawn(move || {
                if let Ok(mut time_passed_listener) = time_passed_listener.lock() {
                    loop {
                        time_passed_listener.sort_by(|a, b| a.interval.cmp(&b.interval));
                        let min_interval = time_passed_listener[0].interval;
                        thread::sleep(std::time::Duration::from_millis(min_interval));
                        for data in time_passed_listener.iter_mut() {
                            if data.interval <= min_interval {
                                _ = data.tx.send(true);
                                data.interval = data.original_interval;
                            } else {
                                data.interval -= min_interval;
                            }
                        }
                    }
                }
            });
        }

        if file_change_listener.lock().unwrap().is_some() {
            thread::spawn(move || {
                if let Ok(mut file_change_listener) = file_change_listener.lock() {
                    loop {
                        let mut buffer = [0; 1024];
                        if let Some(file_change_listener) = file_change_listener.as_mut() {
                            let events = file_change_listener
                                .inotify
                                .read_events_blocking(&mut buffer)
                                .expect("Failed to read events");

                            events.for_each(|_| {
                                _ = file_change_listener.tx.send(true);
                            });
                        }
                    }
                }
            });
        }

        if workspace_listener.lock().unwrap().is_some() {
            thread::spawn(move || {
                if let Ok(mut workspace_listener) = workspace_listener.lock() {
                    if let Some(listener) = workspace_listener.as_mut() {
                        match &mut listener.listener {
                            WorkspaceListener::Hyprland(listener) => {
                                let _ = listener.start_listener();
                            }
                        }
                    }
                }
            });
        }
    }

    pub fn new_time_passed_listener(&mut self, interval: u64) -> broadcast::Receiver<bool> {
        let (tx, rx) = broadcast::channel(1);

        let data = TimeListenerData {
            tx,
            interval,
            original_interval: interval,
        };

        let time_passed_listener = &self.time_passed_listener;
        if let Ok(mut time_passed_listener) = time_passed_listener.lock() {
            time_passed_listener.push(data);
        }

        rx
    }

    pub fn new_file_change_listener(&mut self, path: &PathBuf) -> broadcast::Receiver<bool> {
        let (tx, rx) = broadcast::channel(1);

        if let Ok(mut file_change_listener) = self.file_change_listener.lock() {
            if file_change_listener.is_none() {
                *file_change_listener = Some(FileChangeListenerData {
                    tx,
                    inotify: Inotify::init().expect("Failed to setup inotify"),
                });
            }

            file_change_listener
                .as_mut()
                .unwrap()
                .inotify
                .watches()
                .add(path, WatchMask::MODIFY)
                .expect("Failed to add watch");
        }

        rx
    }
}
