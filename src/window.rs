use std::collections::VecDeque;
use std::fmt::Debug;

use i3_ipc::reply::Node;

#[derive(Debug)]
pub struct Window<'a> {
    pub id: usize,
    pub window_class: &'a String,
    pub window_title: &'a String,
}

pub trait NodeWindowExtractor {
    fn extract_windows(&self) -> Vec<Window>;
}

impl<'a> Window<'a> {
    pub fn get_name(&self) -> String {
        match self.window_class.as_str() {
            "jetbrains-pycharm" => "pycharm".to_owned(),
            "jetbrains-idea-ce" => {
                let project_name_text = self.window_title.split(" [").next();

                if let Some(project_name) = project_name_text {
                    format!("{} {}", "", project_name)
                } else {
                    "".to_owned()
                }
            }
            "Slack" => "".to_owned(),
            "Alacritty" => {
                let session_name = self
                    .window_title
                    .split(':')
                    .next()
                    .and_then(|n| n.strip_prefix(" "));

                if session_name.is_some() && !session_name.unwrap().starts_with("WS") {
                    format!("{} {}", "", session_name.unwrap())
                } else {
                    "".to_owned()
                }
            }
            "firefox" => "".to_owned(),
            "Thunar" => "".to_owned(),
            "qBittorrent" => " qB".to_owned(),
            "vlc" => "".to_owned(),
            "Zathura" => "".to_owned(),
            "Galculator" => "".to_owned(),
            "beekeeper-studio" => " SQL".to_owned(),
            _ => self.window_class.clone(),
        }
    }
}

impl NodeWindowExtractor for Node {
    fn extract_windows(&self) -> Vec<Window> {
        let mut queue = VecDeque::from([self]);

        let mut windows = vec![];

        while let Some(node) = queue.pop_front() {
            if let Some(props) = node.window_properties.as_ref() {
                let window = Window {
                    id: node.id,
                    window_class: props.class.as_ref().expect("Window without class"),
                    window_title: props.title.as_ref().expect("Window without title"),
                };

                windows.push(window);
            } else {
                queue.extend(&node.nodes);
                queue.extend(&node.floating_nodes);
            }
        }

        return windows;
    }
}
