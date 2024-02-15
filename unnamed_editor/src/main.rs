//! ## Editor Entry-Point
mod editor;

use editor::editor::Editor;

fn main() {
    let mut editor = Editor::new("UnnamedEditor".to_string());
    editor.start();
}
