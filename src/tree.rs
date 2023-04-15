use std::collections::VecDeque;
use std::fmt::Debug;

use i3_ipc::reply::{Node, NodeType};

use crate::window::{NodeWindowExtractor, Window};

#[derive(Debug)]
pub struct Tree<'a> {
    pub workspaces: Vec<Workspace<'a>>,
}

#[derive(Debug)]
pub struct Workspace<'a> {
    pub num: i32,
    pub name: &'a String,
    pub output: &'a String,
    pub windows: Vec<Window<'a>>,
}

impl<'a> Tree<'a> {
    pub fn new(root_node: &'a Node) -> Self {
        Self {
            workspaces: Self::extract_workspaces(root_node),
        }
    }

    fn extract_workspaces(root_node: &'a Node) -> Vec<Workspace> {
        let mut queue = VecDeque::new();
        queue.extend(&root_node.nodes);

        let mut workspaces = vec![];

        while let Some(node) = queue.pop_front() {
            if node.node_type == NodeType::Workspace {
                let workspace = Workspace {
                    num: node.num.expect("Workspace without number"),
                    name: node.name.as_ref().expect("Workspace without name"),
                    output: node.output.as_ref().expect("Workspace without output"),
                    windows: node.extract_windows(),
                };

                workspaces.push(workspace);
            } else {
                queue.extend(&node.nodes);
            }
        }

        workspaces
    }

    pub fn find_workspace_for_window(&self, window_id: usize) -> Option<&'a Workspace> {
        for workspace in self.workspaces.iter() {
            if let Some(_) = workspace.windows.iter().find(|w| w.id == window_id) {
                return Some(workspace);
            }
        }

        None
    }

    pub fn find_workspace(&self, workspace_num: i32) -> Option<&'a Workspace> {
        for workspace in self.workspaces.iter() {
            if workspace.num == workspace_num {
                return Some(workspace);
            }
        }

        None
    }
}
