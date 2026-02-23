use rexpect::session::spawn_command;
use std::{fs, path::Path, process::Command};

fn zim_bin() -> String {
    env!("CARGO_BIN_EXE_zim").to_string()
}

/// input ファイルをテンポラリにコピーして zim を起動し、keys を送って終了後に
/// expected ゴールデンファイルと比較する
fn run_golden_test(input: &str, expected: &str, keys: &str) {
    // 入力ファイルをテンポラリにコピー（元ファイルを変更しない）
    let temp = tempfile::NamedTempFile::new().unwrap();
    if Path::new(input).exists() {
        fs::copy(input, temp.path()).unwrap();
    }

    let mut cmd = Command::new(zim_bin());
    cmd.arg(temp.path());

    let mut p = spawn_command(cmd, Some(5000)).unwrap();

    // zim の初期描画（ステータスバー）が完了するまで待機してから keys を送信する
    p.exp_string("lines").unwrap();

    // \n を Enter として送信（termion 4.x 互換）
    p.send(keys).unwrap();
    p.exp_eof().unwrap();

    let actual = fs::read_to_string(temp.path()).unwrap();

    if std::env::var("UPDATE_GOLDEN").is_ok() {
        // ゴールデンファイル更新モード
        if let Some(parent) = Path::new(expected).parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(expected, &actual).unwrap();
        println!("Updated: {}", expected);
    } else {
        let expected_content = fs::read_to_string(expected).unwrap_or_else(|_| {
            panic!(
                "ゴールデンファイルが見つかりません: {}\nUPDATE_GOLDEN=1 cargo test --test golden で生成してください",
                expected
            )
        });
        assert_eq!(actual, expected_content, "diff detected: {}", expected);
    }
}

#[test]
fn test_open_save_ascii() {
    run_golden_test(
        "test_data/test.txt",
        "test_data/expected/test.txt",
        ":wq\n",
    );
}

#[test]
fn test_open_save_japanese() {
    run_golden_test(
        "test_data/test_japanese.txt",
        "test_data/expected/test_japanese.txt",
        ":wq\n",
    );
}

#[test]
fn test_insert_ascii() {
    run_golden_test(
        "test_data/test.txt",
        "test_data/expected/insert_ascii.txt",
        "iHello zim\x1b:wq\n",
    );
}

#[test]
fn test_delete_line() {
    run_golden_test(
        "test_data/test.txt",
        "test_data/expected/delete_line.txt",
        "dd:wq\n",
    );
}

#[test]
fn test_open_line_below() {
    run_golden_test(
        "test_data/test.txt",
        "test_data/expected/open_line_below.txt",
        "oNew line\x1b:wq\n",
    );
}

#[test]
fn test_insert_japanese() {
    run_golden_test(
        "test_data/test.txt",
        "test_data/expected/insert_japanese.txt",
        "iこんにちは\x1b:wq\n",
    );
}

#[test]
fn test_yank_paste_line() {
    run_golden_test(
        "test_data/test.txt",
        "test_data/expected/yank_paste_line.txt",
        "yyp:wq\n",
    );
}

#[test]
fn test_delete_char_x() {
    run_golden_test(
        "test_data/test.txt",
        "test_data/expected/delete_char_x.txt",
        "x:wq\n",
    );
}

#[test]
fn test_backspace_join_lines() {
    run_golden_test(
        "test_data/test.txt",
        "test_data/expected/backspace_join_lines.txt",
        "ji\x7f\x1b:wq\n",
    );
}
