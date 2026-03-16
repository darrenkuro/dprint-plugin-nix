use dprint_plugin_nix::config::NixConfig;
use dprint_plugin_nix::format::format_text;

fn fmt(input: &str) -> String {
    let config = NixConfig::default();
    match format_text(input, &config) {
        Ok(None) => input.to_string(),
        Ok(Some(formatted)) => formatted,
        Err(e) => panic!("Parse error: {}", e),
    }
}

#[test]
fn diff_home_nix() {
    let path = format!("{}/.config/home-manager/home.nix", std::env::var("HOME").unwrap());
    let original = std::fs::read_to_string(&path).expect("Could not read home.nix");
    let formatted = fmt(&original);

    if original == formatted {
        println!("No changes!");
        return;
    }

    // Print a unified diff
    let orig_lines: Vec<&str> = original.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();

    println!("=== Changes (original → formatted) ===\n");

    let max = orig_lines.len().max(fmt_lines.len());
    let mut i = 0;
    while i < max {
        let orig = orig_lines.get(i).copied().unwrap_or("");
        let fmt = fmt_lines.get(i).copied().unwrap_or("");
        if orig != fmt {
            // Find the extent of the changed block
            let block_start = i;
            while i < max {
                let o = orig_lines.get(i).copied().unwrap_or("");
                let f = fmt_lines.get(i).copied().unwrap_or("");
                if o == f { break; }
                i += 1;
            }
            println!("@@ line {} @@", block_start + 1);
            for j in block_start..i {
                if let Some(o) = orig_lines.get(j) {
                    println!("-{}", o);
                }
            }
            for j in block_start..i {
                if let Some(f) = fmt_lines.get(j) {
                    println!("+{}", f);
                }
            }
            println!();
        } else {
            i += 1;
        }
    }

    println!("Original: {} lines, {} bytes", orig_lines.len(), original.len());
    println!("Formatted: {} lines, {} bytes", fmt_lines.len(), formatted.len());
}
