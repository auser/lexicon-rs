use console::Style;

pub fn heading(text: &str) {
    let style = Style::new().bold().cyan();
    println!("{}", style.apply_to(text));
}

pub fn success(text: &str) {
    let style = Style::new().green();
    println!("{} {}", style.apply_to("✓"), text);
}

pub fn warning(text: &str) {
    let style = Style::new().yellow();
    println!("{} {}", style.apply_to("⚠"), text);
}

pub fn error(text: &str) {
    let style = Style::new().red().bold();
    eprintln!("{} {}", style.apply_to("✗"), text);
}

pub fn info(text: &str) {
    println!("  {text}");
}

pub fn divider() {
    println!("{}", "─".repeat(60));
}
