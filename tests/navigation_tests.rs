use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs;
use tempfile::tempdir;
use termexplorer::config::Config;
use termexplorer::filesystem::operations::{create_dir, create_file};
use termexplorer::state::{ActivePane, FocusedPanel};
use termexplorer::App;

#[tokio::test]
async fn test_arrow_keys_navigate_in_and_out_of_directories() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    // Create directories and files
    let sub_a = root.join("dir_alpha");
    let sub_b = root.join("dir_beta");
    let file_c = root.join("file_gamma.txt");

    create_dir(&sub_a).unwrap();
    create_dir(&sub_b).unwrap();
    create_file(&file_c).unwrap();

    // Also populate sub_a so it has content inside
    create_file(&sub_a.join("inner_file.txt")).unwrap();

    let mut app = App::new(Config::default()).await.unwrap();
    app.navigate_into(root.to_path_buf()).await;
    app.state_mut().focused = FocusedPanel::Explorer;

    // Verify initial state
    assert_eq!(app.state().explorer.current_dir, root);
    assert_eq!(app.state().explorer.selected_index, 0);
    assert_eq!(app.state().selected_entry().unwrap().name, "dir_alpha");

    // 1. Move Down with Down Arrow
    let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    app.handle_key(down_key).await.unwrap();
    assert_eq!(app.state().explorer.selected_index, 1);
    assert_eq!(app.state().selected_entry().unwrap().name, "dir_beta");

    // 2. Move Up with Up Arrow
    let up_key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
    app.handle_key(up_key).await.unwrap();
    assert_eq!(app.state().explorer.selected_index, 0);
    assert_eq!(app.state().selected_entry().unwrap().name, "dir_alpha");

    // 3. Navigate into directory with Right Arrow
    let right_key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
    app.handle_key(right_key).await.unwrap();
    assert_eq!(app.state().explorer.current_dir, sub_a);
    assert_eq!(app.state().explorer.entries.len(), 1);
    assert_eq!(app.state().explorer.entries[0].name, "inner_file.txt");

    // 4. Navigate out of directory to parent with Left Arrow
    let left_key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
    app.handle_key(left_key).await.unwrap();
    assert_eq!(app.state().explorer.current_dir, root);
    // Selection should be preserved directly on the folder we just exited
    assert_eq!(app.state().selected_entry().unwrap().name, "dir_alpha");
    assert_eq!(app.state().explorer.selected_index, 0);

    // 5. Move Down to dir_beta and test Enter / Backspace symmetry
    app.handle_key(down_key).await.unwrap();
    assert_eq!(app.state().selected_entry().unwrap().name, "dir_beta");
    assert_eq!(app.state().explorer.selected_index, 1);

    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(enter_key).await.unwrap();
    assert_eq!(app.state().explorer.current_dir, sub_b);

    let backspace_key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
    app.handle_key(backspace_key).await.unwrap();
    assert_eq!(app.state().explorer.current_dir, root);
    // Selection preserved on dir_beta
    assert_eq!(app.state().selected_entry().unwrap().name, "dir_beta");
    assert_eq!(app.state().explorer.selected_index, 1);
}

#[tokio::test]
async fn test_vim_hjkl_navigation() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let sub1 = root.join("folder_1");
    let sub2 = root.join("folder_2");
    create_dir(&sub1).unwrap();
    create_dir(&sub2).unwrap();

    let mut app = App::new(Config::default()).await.unwrap();
    app.navigate_into(root.to_path_buf()).await;
    app.state_mut().focused = FocusedPanel::Explorer;

    // j: move down
    app.handle_key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().explorer.selected_index, 1);
    assert_eq!(app.state().selected_entry().unwrap().name, "folder_2");

    // k: move up
    app.handle_key(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().explorer.selected_index, 0);
    assert_eq!(app.state().selected_entry().unwrap().name, "folder_1");

    // l: enter directory
    app.handle_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().explorer.current_dir, sub1);

    // h: parent directory
    app.handle_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().explorer.current_dir, root);
    assert_eq!(app.state().selected_entry().unwrap().name, "folder_1");
}

#[tokio::test]
async fn test_preview_focus_on_file_and_left_or_esc_return() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let file_path = root.join("hello.txt");
    fs::write(&file_path, "Hello TermExplorer").unwrap();

    let mut app = App::new(Config::default()).await.unwrap();
    app.navigate_into(root.to_path_buf()).await;
    app.state_mut().focused = FocusedPanel::Explorer;

    assert_eq!(app.state().selected_entry().unwrap().name, "hello.txt");

    // Right on file -> focuses Preview panel
    let right_key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
    app.handle_key(right_key).await.unwrap();
    assert_eq!(app.state().focused, FocusedPanel::Preview);

    // Left on Preview -> returns focus to Explorer
    let left_key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
    app.handle_key(left_key).await.unwrap();
    assert_eq!(app.state().focused, FocusedPanel::Explorer);

    // Enter on file -> also focuses Preview panel
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(enter_key).await.unwrap();
    assert_eq!(app.state().focused, FocusedPanel::Preview);

    // Esc on Preview -> returns focus to Explorer
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key(esc_key).await.unwrap();
    assert_eq!(app.state().focused, FocusedPanel::Explorer);
}

#[tokio::test]
async fn test_dot_toggles_hidden_files() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    create_file(&root.join("visible.txt")).unwrap();
    create_file(&root.join(".hidden.txt")).unwrap();

    let mut app = App::new(Config::default()).await.unwrap();
    app.navigate_into(root.to_path_buf()).await;
    app.state_mut().focused = FocusedPanel::Explorer;
    app.state_mut().config.show_hidden = false;
    // Refresh to apply show_hidden = false
    let f5_key = KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE);
    app.handle_key(f5_key).await.unwrap();

    // Initially dotfiles hidden
    assert!(!app.state().config.show_hidden);
    assert_eq!(app.state().explorer.entries.len(), 1);

    // Press '.' to toggle hidden files
    let dot_key = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE);
    app.handle_key(dot_key).await.unwrap();

    assert!(app.state().config.show_hidden);
    assert_eq!(app.state().explorer.entries.len(), 2);

    // Press '.' again to hide dotfiles
    app.handle_key(dot_key).await.unwrap();
    assert!(!app.state().config.show_hidden);
    assert_eq!(app.state().explorer.entries.len(), 1);
}

#[tokio::test]
async fn test_dual_pane_arrow_navigation() {
    let tmp = tempdir().unwrap();
    let root = tmp.path();

    let dir_left = root.join("left_tree");
    let dir_right = root.join("right_tree");
    create_dir(&dir_left).unwrap();
    create_dir(&dir_right).unwrap();

    create_file(&dir_left.join("left_file.txt")).unwrap();
    create_dir(&dir_right.join("right_sub")).unwrap();

    let mut app = App::new(Config::default()).await.unwrap();
    app.navigate_into(root.to_path_buf()).await;
    app.state_mut().focused = FocusedPanel::Explorer;

    // Toggle dual pane on with F7
    app.handle_key(KeyEvent::new(KeyCode::F(7), KeyModifiers::NONE))
        .await
        .unwrap();
    assert!(app.state().dual_pane_enabled);

    // Switch focus to secondary pane with Tab
    app.handle_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
        .await
        .unwrap();
    assert_eq!(app.state().focused, FocusedPanel::ExplorerSecondary);
    assert_eq!(app.state().active_pane, ActivePane::Secondary);

    // In secondary pane, navigate with Right and Left
    let right_key = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
    app.handle_key(right_key).await.unwrap();

    // Now secondary pane should have entered dir_left (or selected dir)
    let sec_dir = app.state().explorer_secondary.current_dir.clone();
    assert_ne!(sec_dir, root);

    // Left should go back to parent
    let left_key = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
    app.handle_key(left_key).await.unwrap();
    assert_eq!(app.state().explorer_secondary.current_dir, root);
}
