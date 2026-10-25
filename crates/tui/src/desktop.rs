use crate::commands::{CMD_ABOUT, CMD_LOGS, alt_char_event, ctrl_char_event};
use i18n::i18n;
use tv::desktop::Desktop;
use tv::menu::{Menu, MenuBar};
use tv::status::{StatusDef, StatusLine};
use tv::view::{Rect, View};
use tv::{Command, Key, KeyEvent};

pub fn make_desktop(r: Rect) -> Box<dyn View> {
    let mut r = r;
    r.a.y += 1;
    r.b.y -= 1;
    Box::new(Desktop::new(r, |br| Some(Desktop::init_background(br))))
    //
    // let b = Rect::new(4,2,54,16);
    // let main_windows = Window::new(
    //     b,
    //     Some("Main".to_string()),
    //     0
    // );
    // desktop.insert_view(Box::new(main_windows));
    // Box::new(desktop)
}

fn status_defs() -> Vec<StatusDef> {
    StatusDef::list()
        .def_all(|stat| {
            stat.item("~F1~ Help", KeyEvent::from(Key::F(1)), Command::HELP)
                .item("~F9~ Menu", KeyEvent::from(Key::F(9)), Command::MENU)
                .item(i18n!("~Alt-L~ Logs"), alt_char_event('l'), CMD_LOGS)
                .item("~Ctrl-Q~ Quit", ctrl_char_event('q'), Command::QUIT)
        })
        .build()
}

pub fn make_status_line(r: Rect) -> Box<dyn View> {
    let mut r = r;
    r.a.y = r.b.y - 1;
    Box::new(StatusLine::new(r, status_defs()))
}

fn app_menu() -> Menu {
    Menu::builder()
        .submenu("~F~ile", alt_char_event('f'), |m| {
            m.command_key(i18n!("~L~ogs"), CMD_LOGS, alt_char_event('l'), "Alt-L")
                .command_key("~Q~uit", Command::QUIT, ctrl_char_event('q'), "Ctrl-Q")
        })
        .submenu("~H~elp", alt_char_event('h'), |m| {
            m.command_key("~A~bout...", CMD_ABOUT, ctrl_char_event('a'), "Ctrl-A")
        })
        .build()
}

pub fn make_menu_bar(r: Rect) -> Box<dyn View> {
    let mut r = r;
    r.b.y = r.a.y + 1;
    Box::new(MenuBar::new(r, app_menu()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tv::menu::MenuItem;
    use tv::menu::menu_view::hot_key;

    /// Alt+L resolves to CMD_LOGS through the File submenu accelerator.
    #[test]
    fn alt_l_maps_to_cmd_logs() {
        assert_eq!(hot_key(&app_menu(), alt_char_event('l')), Some(CMD_LOGS));
    }

    /// The File submenu declares a "~L~ogs" item bound to CMD_LOGS, Alt+L.
    #[test]
    fn file_menu_has_logs_item() {
        let menu = app_menu();
        let file = menu.items.iter().find_map(|item| match item {
            MenuItem::SubMenu { name, menu, .. } if name == "~F~ile" => Some(menu),
            _ => None,
        });
        let file = file.expect("File submenu exists");
        let logs = file.items.iter().find_map(|item| match item {
            MenuItem::Command {
                name,
                command,
                key_code,
                ..
            } if *command == CMD_LOGS => Some((name, key_code)),
            _ => None,
        });
        let (name, key_code) = logs.expect("Logs item exists in File menu");
        assert_eq!(name, "~L~ogs");
        assert_eq!(*key_code, Some(alt_char_event('l')));
    }

    /// No other accelerator in the menu collides with Alt+L.
    #[test]
    fn alt_l_is_unique() {
        let menu = app_menu();
        let hits = |m: &Menu| -> usize {
            m.items
                .iter()
                .filter(|item| match item {
                    MenuItem::Command { key_code, .. } => *key_code == Some(alt_char_event('l')),
                    _ => false,
                })
                .count()
        };
        assert_eq!(hits(&menu), 0, "top-level items carry no Alt+L");
        assert_eq!(hot_key(&menu, alt_char_event('l')), Some(CMD_LOGS));
    }

    /// The status def binds Alt+L to CMD_LOGS with a visible label.
    #[test]
    fn status_line_has_alt_l_logs() {
        let defs = status_defs();
        let all = defs
            .iter()
            .flat_map(|d| d.items.iter())
            .find(|item| item.command == CMD_LOGS)
            .expect("status line binds CMD_LOGS");
        assert_eq!(all.key_code, Some(alt_char_event('l')));
        assert!(all.text.is_some(), "status item is visible");
    }
}
