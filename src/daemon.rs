use crate::{
    tree::{Tree, Workspace, WorkspaceExtractor},
    window::{NodeWindowExtractor, Window},
};

use std::{
    collections::{HashMap, HashSet},
    thread, time,
};

use i3_ipc::{
    event::{Event, Subscribe, WindowChange, WindowData, WorkspaceChange, WorkspaceData},
    reply::Node,
    Connect, I3Stream, I3,
};

const SCRATCHPAD_WORKSPACE_NUM: i32 = -1;

lazy_static! {
    static ref WINDOW_EVENTS: HashSet<WindowChange> = {
        HashSet::from([
            WindowChange::New,
            WindowChange::Move,
            WindowChange::Title,
            WindowChange::Close,
        ])
    };
}

pub struct Output {
    name: String,
    width: isize,
    height: isize,
}

pub struct I3Daemon {
    // Track which window is on which workspace. This information is used at
    // WindowChange::Move events to rename the old workspace.
    window_to_workspace_num: HashMap<usize, i32>,
    outputs: Vec<Output>,
    last_seen_scratchpad_output: HashMap<usize, String>,
}

impl I3Daemon {
    pub fn new() -> Self {
        Self {
            window_to_workspace_num: HashMap::new(),
            outputs: vec![],
            last_seen_scratchpad_output: HashMap::new(),
        }
    }

    pub fn run(mut self) {
        let events = [Subscribe::Window, Subscribe::Workspace, Subscribe::Output];
        let mut i3stream = I3Stream::conn_sub(&events).unwrap();
        let mut i3 = I3::connect().unwrap();

        self.init_state(&mut i3);

        for i3_event in i3stream.listen() {
            match i3_event.unwrap() {
                Event::Workspace(e) => self.handle_workspace_event(&mut i3, e),
                Event::Window(e) => self.handle_window_event(&mut i3, e),
                Event::Output(_) => self.update_outputs(&mut i3),
                _ => unreachable!("Invalid event type"),
            }
        }
    }

    fn init_state(&mut self, i3: &mut I3Stream) {
        let root_node = i3.get_tree().unwrap();
        let tree = Tree::new(&root_node);

        for workspace in tree.workspaces.iter() {
            log::info!("Initializing workspace {}", workspace.num);

            self.register_windows_to_workspace(&workspace.windows, workspace);

            self.rename_workspace(i3, workspace);
        }

        self.update_outputs(i3);
    }

    fn update_outputs(&mut self, i3: &mut I3Stream) {
        self.outputs.clear();

        for i3_output in i3.get_outputs().unwrap().iter() {
            let output = Output {
                name: i3_output.name.clone(),
                width: i3_output.rect.width,
                height: i3_output.rect.width,
            };

            log::info!(
                "Output {} - {}x{}",
                output.name,
                output.width,
                output.height
            );

            self.outputs.push(output);
        }
    }

    fn handle_workspace_event(&mut self, i3: &mut I3Stream, event: Box<WorkspaceData>) {
        match event.change {
            WorkspaceChange::Empty => {
                let workspace_node = event.current.expect("No workspace in event");
                let workspace = workspace_node
                    .extract_workspace()
                    .expect("Could not extract workspace");

                log::info!("WorkspaceChange::Empty workspace {}", workspace.num);
                self.rename_workspace(i3, &workspace);
            }
            _ => return,
        }
    }

    fn handle_window_event(&mut self, i3: &mut I3Stream, event: Box<WindowData>) {
        if !WINDOW_EVENTS.contains(&event.change) {
            return;
        }

        let root_node = i3.get_tree().unwrap();
        let tree = Tree::new(&root_node);

        let windows = event.container.extract_windows();
        if windows.is_empty() {
            return;
        }

        let first_window_id = windows.first().expect("No windows in event").id;

        log::info!("WindowChange::{:?}", event.change);

        match event.change {
            WindowChange::New => {
                if let Some(workspace) = tree.find_workspace_for_window(first_window_id) {
                    self.register_windows_to_workspace(&windows, workspace);
                    self.rename_workspace(i3, workspace);
                }
            }
            WindowChange::Move => {
                let old_workspace_number =
                    *self.window_to_workspace_num.get(&first_window_id).unwrap();

                if old_workspace_number == SCRATCHPAD_WORKSPACE_NUM {
                    self.handle_scratchpad_window(first_window_id, &event.container, i3);
                }

                let new_workspace = tree.find_workspace_for_window(first_window_id).unwrap();
                let old_workspace = tree.find_workspace(old_workspace_number).unwrap();

                self.rename_workspace(i3, old_workspace);
                self.rename_workspace(i3, new_workspace);

                self.unregister_windows(&windows);
                self.register_windows_to_workspace(&windows, new_workspace);
            }
            WindowChange::Close => {
                self.unregister_windows(&windows);
                if let Some(workspace) = tree.find_workspace_for_window(first_window_id) {
                    self.rename_workspace(i3, workspace);
                }
            }
            WindowChange::Title => {
                if let Some(workspace) = tree.find_workspace_for_window(first_window_id) {
                    self.rename_workspace(i3, workspace);
                }
            }
            _ => return,
        }
    }

    fn handle_scratchpad_window(&mut self, window_id: usize, container: &Node, i3: &mut I3Stream) {
        let current_output = container
            .output
            .as_ref()
            .expect("Scratchpad window without output");

        let last_output = self
            .last_seen_scratchpad_output
            .entry(window_id)
            .or_insert(current_output.clone());

        if last_output == current_output {
            return;
        }

        if let Some(output) = self.outputs.iter().find(|o| o.name == *current_output) {
            log::info!(
                "Resizing scratchpad window {} on output {} - last seen on {}",
                window_id,
                current_output,
                last_output
            );

            let width = ((output.width as f32) * 0.95) as isize;
            let height = ((output.height as f32) * 0.95) as isize;

            self.run_command(i3, "border pixel 10");
            self.run_command(i3, format!("resize set width {} px", width).as_str());
            self.run_command(i3, format!("resize set height {} px", height).as_str());
            self.run_command(i3, "move pisition center");
        }
    }

    fn unregister_windows(&mut self, windows: &Vec<Window>) {
        for window in windows.iter() {
            if let Some(workspace_num) = self.window_to_workspace_num.remove(&window.id) {
                log::info!(
                    "Unregistering window {} on workspace {}",
                    window.id,
                    workspace_num
                );
            }
        }
    }

    fn register_windows_to_workspace(&mut self, windows: &Vec<Window>, workspace: &Workspace) {
        for window in windows.iter() {
            self.window_to_workspace_num
                .insert(window.id, workspace.num);

            log::info!(
                "Registering window {} on workspace {}",
                window.id,
                workspace.num
            );
        }
    }

    fn rename_workspace(&mut self, i3: &mut I3Stream, workspace: &Workspace) {
        if workspace.num < 0 {
            return; // Scratchpad
        }

        let expected_name = if let Some(window) = workspace.windows.first() {
            format!("{}: {}", workspace.num, window.get_name())
        } else {
            format!("{}", workspace.num)
        };

        if *workspace.name != expected_name {
            self.run_command(
                i3,
                format!(
                    "rename workspace \"{}\" to \"{}\"",
                    workspace.name, expected_name
                )
                .as_str(),
            );
        }
    }

    fn run_command(&self, i3: &mut I3Stream, command: &str) {
        thread::sleep(time::Duration::from_millis(100)); // There is a race between window movement and workspace renaming
        log::info!("Running command: {}", command);
        i3.run_command(command).unwrap();
    }
}
