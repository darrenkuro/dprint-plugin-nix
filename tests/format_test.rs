use dprint_plugin_nix::config::NixConfig;
use dprint_plugin_nix::format::format_text;
use rnix;

fn fmt(input: &str) -> String {
    let config = NixConfig::default();
    match format_text(input, &config) {
        Ok(None) => input.to_string(),
        Ok(Some(formatted)) => formatted,
        Err(e) => panic!("Parse error: {}", e),
    }
}

#[test]
fn literal_unchanged() {
    assert_eq!(fmt("42\n"), "42\n");
}

#[test]
fn simple_attr_set_inline() {
    // THE key test — short attr sets must stay on one line
    let result = fmt(r#"{ name = "foo"; greedy = true; }"#);
    println!("attr_set result: {:?}", result);
    // Should keep it on one line (well within 100 chars)
    assert!(!result.contains('\n') || result.trim_end().lines().count() == 1,
        "Short attr set should be inline, got: {:?}", result);
}

#[test]
fn list_inline() {
    let result = fmt("[ 1 2 3 ]");
    println!("list result: {:?}", result);
    assert!(!result.contains('\n') || result.trim_end().lines().count() == 1,
        "Short list should be inline, got: {:?}", result);
}

#[test]
fn let_in_basic() {
    let result = fmt("let\n  x = 1;\nin\nx\n");
    println!("let_in result: {:?}", result);
    assert!(result.contains("let"));
    assert!(result.contains("in"));
}

#[test]
fn if_else_basic() {
    let result = fmt("if true then 1 else 2\n");
    println!("if_else result: {:?}", result);
    assert!(result.contains("if"));
    assert!(result.contains("then"));
    assert!(result.contains("else"));
}

#[test]
fn rec_attr_set() {
    let result = fmt(r#"rec { name = "foo"; value = "bar"; }"#);
    println!("rec result: {:?}", result);
    assert!(result.contains("rec"));
}

#[test]
fn string_preserved() {
    let result = fmt(r#""hello world""#);
    println!("string result: {:?}", result);
    assert!(result.contains("hello world"));
}

#[test]
fn idempotent() {
    let inputs = vec![
        "42\n",
        r#"{ a = 1; }"#,
        "[ 1 2 3 ]",
        "let\n  x = 1;\nin\nx\n",
    ];
    for input in inputs {
        let first = fmt(input);
        let second = fmt(&first);
        assert_eq!(first, second, "Not idempotent for input: {:?}", input);
    }
}

#[test]
fn large_attr_set_expands() {
    // This attr set is too wide for one line — should expand
    let input = r#"{ name = "firefox"; greedy = true; description = "A free and open-source web browser developed by the Mozilla Foundation"; version = "120.0"; }"#;
    let result = fmt(input);
    println!("large attr_set:\n{}", result);
    // Should be multi-line since it exceeds 100 chars
    assert!(result.lines().count() > 1,
        "Large attr set should expand to multi-line, got: {:?}", result);
}

#[test]
fn lambda_basic() {
    let result = fmt("x: x + 1\n");
    println!("lambda result: {:?}", result);
    assert!(result.contains(":"));
}

#[test]
fn with_expr() {
    let result = fmt("with pkgs; [ vim git ]\n");
    println!("with result: {:?}", result);
    assert!(result.contains("with"));
}

#[test]
fn binary_op() {
    let result = fmt("1 + 2\n");
    println!("binop result: {:?}", result);
    assert!(result.contains("+"));
}

#[test]
fn nested_attr_set() {
    let result = fmt(r#"{ a = { x = 1; }; b = 2; }"#);
    println!("nested result: {:?}", result);
    assert!(result.contains("a ="));
}

#[test]
fn real_world_cask_list() {
    // The actual use case: homebrew cask list in home-manager
    let input = r#"[
  { name = "firefox"; greedy = true; }
  { name = "google-chrome"; greedy = true; }
  { name = "arc"; }
  { name = "1password"; greedy = true; }
  { name = "raycast"; greedy = true; }
  { name = "karabiner-elements"; greedy = true; }
  { name = "istat-menus"; greedy = true; }
  { name = "obs"; greedy = true; }
]
"#;
    let result = fmt(input);
    println!("cask list:\n{}", result);
    // Individual attr set entries must NOT be split across lines
    // (this is the main thing alejandra gets wrong)
    assert!(result.contains(r#"{ name = "firefox"; greedy = true; }"#),
        "Each cask entry should stay as one unit");
    assert!(result.contains(r#"{ name = "karabiner-elements"; greedy = true; }"#),
        "Even longer entries should stay inline");
}

#[test]
fn idempotent_cask_list() {
    let input = r#"[
  { name = "firefox"; greedy = true; }
  { name = "google-chrome"; greedy = true; }
  { name = "arc"; }
]
"#;
    let first = fmt(input);
    let second = fmt(&first);
    assert_eq!(first, second, "Cask list not idempotent");
}

#[test]
fn real_world_home_nix_snippet() {
    // Must be a complete Nix expression (attr set)
    let input = r#"{
  programs.zsh = {
    enable = true;
    history = {
      size = 100000;
      save = 100000;
      ignoreDups = true;
      share = true;
      extended = true;
    };
  };
}
"#;
    let result = fmt(input);
    println!("home.nix snippet:\n{}", result);
    assert!(result.contains("programs.zsh"));
    assert!(result.contains("history"));
    let second = fmt(&result);
    assert_eq!(result, second, "home.nix snippet not idempotent");
}

#[test]
fn format_whole_home_nix() {
    // Test that we can format the user's actual home.nix without panicking
    let input = std::fs::read_to_string(
        std::env::var("HOME").unwrap() + "/.config/home-manager/home.nix"
    );
    if let Ok(content) = input {
        let result = fmt(&content);
        println!("home.nix formatted ({} → {} bytes)", content.len(), result.len());
        // Verify result is valid Nix (parses without errors)
        let reparse = rnix::Root::parse(&result);
        assert!(reparse.errors().is_empty(),
            "Formatted home.nix has parse errors: {:?}", reparse.errors());
        // Verify idempotency
        let second = fmt(&result);
        assert_eq!(result, second, "home.nix not idempotent");
    }
}

#[test]
fn line_comment_standalone() {
    let input = "{\n  # a comment\n  x = 1;\n}\n";
    let result = fmt(input);
    println!("standalone comment: {:?}", result);
    assert!(result.contains("# a comment"), "Comment should be preserved");
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn trailing_comment_attr_set() {
    let input = "{\n  x = 1; # trailing\n  y = 2;\n}\n";
    let result = fmt(input);
    println!("trailing comment: {:?}", result);
    assert!(result.contains("# trailing"), "Trailing comment should be preserved");
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn blank_line_between_items() {
    let input = "{\n  x = 1;\n\n  y = 2;\n}\n";
    let result = fmt(input);
    println!("blank line: {:?}", result);
    // Should preserve the blank line separation
    assert!(result.contains("x = 1;"), "First item preserved");
    assert!(result.contains("y = 2;"), "Second item preserved");
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn comment_between_items() {
    let input = "{\n  x = 1;\n  # separator comment\n  y = 2;\n}\n";
    let result = fmt(input);
    println!("comment between: {:?}", result);
    assert!(result.contains("# separator comment"));
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn list_with_comments() {
    let input = "[\n  1\n  # comment\n  2\n  3 # trailing\n]\n";
    let result = fmt(input);
    println!("list comments: {:?}", result);
    assert!(result.contains("# comment"));
    assert!(result.contains("# trailing"));
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn let_in_with_comments() {
    let input = "let\n  x = 1; # why\n  # standalone\n  y = 2;\nin\nx + y\n";
    let result = fmt(input);
    println!("let_in comments: {:?}", result);
    assert!(result.contains("# why"), "Trailing comment preserved");
    assert!(result.contains("# standalone"), "Standalone comment preserved");
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn if_else_with_comment() {
    let input = "if true # reason\nthen 1\nelse 2\n";
    let result = fmt(input);
    println!("if_else comment: {:?}", result);
    assert!(result.contains("# reason"), "Comment preserved in if_else");
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn with_trailing_comment() {
    let input = "with pkgs; # use pkgs\n[ vim ]\n";
    let result = fmt(input);
    println!("with comment: {:?}", result);
    assert!(result.contains("# use pkgs"), "Comment preserved in with");
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn lambda_with_comment() {
    let input = "x: # param\nx + 1\n";
    let result = fmt(input);
    println!("lambda comment: {:?}", result);
    assert!(result.contains("# param"), "Comment preserved in lambda");
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn pattern_with_comments() {
    let input = "{ a, b, ... }: a + b\n";
    let result = fmt(input);
    println!("pattern: {:?}", result);
    assert!(result.contains("a"));
    assert!(result.contains("..."));
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}

#[test]
fn inherit_with_comment() {
    let input = "{\n  inherit a b; # from scope\n}\n";
    let result = fmt(input);
    println!("inherit comment: {:?}", result);
    assert!(result.contains("# from scope"), "Comment preserved in inherit");
    let second = fmt(&result);
    assert_eq!(result, second, "Not idempotent");
}
