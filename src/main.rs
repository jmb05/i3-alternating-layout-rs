use i3ipc::event::inner::WindowChange;
use i3ipc::event::Event;
use i3ipc::reply::{Node, NodeLayout};
use i3ipc::{I3Connection, I3EventListener, Subscription};
use std::cmp::Ordering;
use std::fs::File;
use std::io::Write;
use std::process::exit;
use std::{env, process};

fn set_layout(conn: &mut I3Connection) {
    match conn.get_tree() {
        Ok(tree) => {
            let (focused, parent_opt) = find_focused(&tree, None);
            match parent_opt {
                None => {}
                Some(parent) => {
                    match parent.layout {
                        NodeLayout::Stacked | NodeLayout::Tabbed => {}
                        _ => {
                            // height > width
                            if focused.rect.3 > focused.rect.2 {
                                if parent.layout == NodeLayout::SplitH {
                                    match conn.run_command("split v") {
                                        Ok(_) => {}
                                        Err(err) => eprintln!("Error setting layout to vertical\n{}", err)
                                    }
                                }
                            } else {
                                if parent.layout == NodeLayout::SplitV {
                                    match conn.run_command("split h") {
                                        Ok(_) => {}
                                        Err(err) => eprintln!("Error setting layout to horizontal\n{}", err)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(err) => eprintln!("Error getting i3 window tree\n{}", err)
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
    println!("Options:\n    -p PIDFILE    Saves the PID for this program in the filename specified\n");
    println!("    -h, --help    Print this help info");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut pid_file: Option<&String> = None;

    for arg in &args[1..] {
        if arg == "-h" || arg == "--help" {
            print_help(&args);
            exit(0);
        }
        if arg == "-p" {
            pid_file = Some(&args[2]);
        }
    }

    if pid_file.is_some() {
        match File::create(pid_file.unwrap()) {
            Ok(mut file) => {
                let pid = process::id();
                let pid = pid.to_string();
                match file.write_all(pid.as_bytes()) {
                    Ok(_) => {}
                    Err(err) => eprintln!("Failed to write PID file\n{}", err)
                }
            }
            Err(err) => eprintln!("Error creating pid file\n{}", err),
        }
    }

    let mut connection = I3Connection::connect().expect("Failed to connect to i3, is i3 running?");
    let mut event_listener = I3EventListener::connect().expect("Failed to connect to i3 events");
    let subs = [ Subscription::Window ];
    event_listener.subscribe(&subs).expect("Failed to subscribe to i3 events");
    for event_result in event_listener.listen() {
        match event_result {
            Ok(event) => 
                match event {
                    Event::WindowEvent(info) =>
                        match info.change {
                            WindowChange::Focus => set_layout(&mut connection),
                            _ => {}
                        }
                    _ => {}
                }
            Err(err) => eprintln!("Error getting event from i3\n{}", err)
        }
    }
}
