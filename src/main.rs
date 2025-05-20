use i3ipc::event::inner::WindowChange;
use i3ipc::event::Event;
use i3ipc::reply::{Node, NodeLayout};
use i3ipc::{I3Connection, I3EventListener, Subscription};
use std::cmp::Ordering;
use std::{env, process};
use std::fs::File;
use std::io::Write;
use std::process::exit;

fn set_layout(conn: &mut I3Connection) {
    let tree = conn.get_tree().expect("Failed to get tree");
    let (focused, parent) = find_focused(&tree, None);
    if parent.is_some() {
        match parent.unwrap().layout {
            NodeLayout::Stacked | NodeLayout::Tabbed => {}
            _ => {
                // height > width
                if focused.rect.3 > focused.rect.2 {
                    if parent.unwrap().layout == NodeLayout::SplitH {
                        conn.run_command("split v").expect("Error setting layout");
                    }
                } else {
                    if parent.unwrap().layout == NodeLayout::SplitV {
                        conn.run_command("split h").expect("Error setting layout");
                    }
                }
            }
        }
    }
}

fn find_focused<'a>(tree: &'a Node, parent: Option<&'a Node>) -> (&'a Node, Option<&'a Node>) {
    let id = tree.focus.get(0);
    match id {
        None => (tree, parent),
        Some(id) => {
            let mut focused_child: Option<&Node> = None;
            for child in &tree.nodes {
                if child.id.cmp(id) == Ordering::Equal {
                    focused_child = Some(child);
                }
            }
            match focused_child {
                None => (tree, parent),
                Some(c) => find_focused(c, Some(tree))
            }

        }
    }
}

fn print_help(args: &Vec<String>) {
    println!("Usage: {} [-p PIDFILE]\n", args[0]);
    println!("Options:\n    -p PIDFILE    Saves the PID for this program in the filename specified\n")
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut pid_file: Option<&String> = None;

    for arg in &args[1..] {
        if arg == "-h" {
            print_help(&args);
            exit(0);
        }
        if arg == "-p" {
            pid_file = Some(&args[2]);
        }
    }

    if pid_file.is_some() {
        let mut file = File::create(pid_file.unwrap())
            .expect("Failed to create or open file");
        let pid = process::id();
        let pid = pid.to_string();
        file.write_all(pid.as_bytes())
            .expect("Failed to write PID file");
    }

    let mut connection = I3Connection::connect().expect("Failed to connect to i3");
    let mut event_listener = I3EventListener::connect().expect("Failed to connect to i3 events");
    let subs = [ Subscription::Window ];
    event_listener.subscribe(&subs).expect("Failed to subscribe to events");
    for event_result in event_listener.listen() {
        let event: Event = event_result.expect("Failed to get event");
        match event {
            Event::WindowEvent(info) => {
                match info.change {
                    WindowChange::Focus => {
                        set_layout(&mut connection);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
