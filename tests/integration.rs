use ripdiff::config::Theme;
use ripdiff::diff::get_file_diff;
use ripdiff::state::get_joined_paths;
use std::collections::HashMap;

const OLD_DIR: &str = "test_files/old";
const NEW_DIR: &str = "test_files/new";

#[test]
fn test_get_joined_paths_only_new_has_entries() {
    let old = HashMap::new();
    let mut new = HashMap::new();
    new.insert(
        format!("{NEW_DIR}/nested"),
        vec!["unchanged.txt".to_string()],
    );

    let result = get_joined_paths(&format!("{NEW_DIR}/nested"), &old, &new);
    assert_eq!(result, vec!["unchanged.txt"]);
}

#[test]
fn test_get_joined_paths_both_have_same_directory() {
    let mut old = HashMap::new();
    old.insert(
        format!("{OLD_DIR}/nested"),
        vec!["unchanged.txt".to_string()],
    );
    let mut new = HashMap::new();
    new.insert(
        format!("{OLD_DIR}/nested"),
        vec!["modified.txt".to_string()],
    );

    let result = get_joined_paths(&format!("{OLD_DIR}/nested"), &old, &new);
    assert_eq!(result, vec!["unchanged.txt", "modified.txt"]);
}

#[test]
fn test_diff_unchanged_file_identical_panes() {
    let theme = Theme::default();
    let (old_p, new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/unchanged.txt"),
        0,
        20,
        &theme,
    );

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 20));
    let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 20));
    old_p.render(Rect::new(0, 0, 80, 20), &mut old_buf);
    new_p.render(Rect::new(0, 0, 80, 20), &mut new_buf);
    assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
}

#[test]
fn test_diff_modified_file_different_panes() {
    let theme = Theme::default();
    let (old_p, new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/modified.txt"),
        0,
        20,
        &theme,
    );

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 20));
    let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 20));
    old_p.render(Rect::new(0, 0, 80, 20), &mut old_buf);
    new_p.render(Rect::new(0, 0, 80, 20), &mut new_buf);
    assert_ne!(format!("{:?}", old_buf), format!("{:?}", new_buf));
}

#[test]
fn test_diff_added_file_old_side_error() {
    let theme = Theme::default();
    let (old_p, _new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/added_only_new.txt"),
        0,
        10,
        &theme,
    );

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
    old_p.render(Rect::new(0, 0, 80, 10), &mut buf);
    assert!(format!("{:?}", buf).contains("Error"));
}

#[test]
fn test_diff_deleted_file_new_side_error() {
    let theme = Theme::default();
    let (_old_p, new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/deleted_only_old.txt"),
        0,
        10,
        &theme,
    );

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
    new_p.render(Rect::new(0, 0, 80, 10), &mut buf);
    assert!(format!("{:?}", buf).contains("Error"));
}

#[test]
fn test_diff_empty_file_identical_panes() {
    let theme = Theme::default();
    let (old_p, new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/empty.txt"),
        0,
        10,
        &theme,
    );

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
    let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
    old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
    new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
    assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
}

#[test]
fn test_diff_deeply_nested_unchanged_file() {
    let theme = Theme::default();
    let (old_p, new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/nested/deep/deeper/unchanged.txt"),
        0,
        10,
        &theme,
    );

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
    let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
    old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
    new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
    assert_eq!(format!("{:?}", old_buf), format!("{:?}", new_buf));
}

#[test]
fn test_diff_whitespace_changes() {
    let theme = Theme::default();
    let (old_p, new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/whitespace_diff.txt"),
        0,
        10,
        &theme,
    );

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    let mut old_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
    let mut new_buf = Buffer::empty(Rect::new(0, 0, 80, 10));
    old_p.render(Rect::new(0, 0, 80, 10), &mut old_buf);
    new_p.render(Rect::new(0, 0, 80, 10), &mut new_buf);
    assert_ne!(format!("{:?}", old_buf), format!("{:?}", new_buf));
}

#[test]
fn test_binary_file_no_panic() {
    let theme = Theme::default();
    let (_old_p, _new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/binary.bin"),
        0,
        10,
        &theme,
    );
}

#[test]
fn test_unicode_filename_no_panic() {
    let theme = Theme::default();
    let (_old_p, _new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/中文.txt"),
        0,
        10,
        &theme,
    );
}

#[test]
fn test_file_with_spaces_in_name_no_panic() {
    let theme = Theme::default();
    let (_old_p, _new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/file with spaces.txt"),
        0,
        10,
        &theme,
    );
}

#[test]
fn test_scroll_offset_returns_subset_of_lines() {
    let theme = Theme::default();
    let (old_p, _new_p) = get_file_diff(
        OLD_DIR,
        NEW_DIR,
        &format!("{OLD_DIR}/many_lines.txt"),
        5,
        10,
        &theme,
    );

    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;

    let mut buf = Buffer::empty(Rect::new(0, 0, 80, 10));
    old_p.render(Rect::new(0, 0, 80, 10), &mut buf);
    assert!(format!("{:?}", buf).contains("006"));
}
