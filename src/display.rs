use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, CellAlignment, Color,
    ContentArrangement, Table,
};
use console::{measure_text_width, style, Term};

pub fn terminal_width() -> u16 {
    let w = Term::stdout().size().1;
    if w == 0 { 80 } else { w }
}

pub fn print_rule(title: &str, bold: bool) {
    let w = terminal_width() as usize;

    if title.is_empty() {
        println!("{}", "\u{2500}".repeat(w));
        return;
    }

    let visual_len = measure_text_width(title) + 2;
    let styled = if bold {
        format!(" {} ", style(title).bold())
    } else {
        format!(" {title} ")
    };

    if visual_len >= w {
        println!("{styled}");
        return;
    }

    let remaining = w - visual_len;
    let left = remaining / 2;
    let right = remaining - left;
    println!(
        "{}{}{}",
        "\u{2500}".repeat(left),
        styled,
        "\u{2500}".repeat(right)
    );
}

pub fn status_cell(status: &str) -> Cell {
    let (color, icon) = match status {
        "In Progress" => (Color::Yellow, "\u{23f3}"),
        "Review" => (Color::Blue, "\u{1f441} "),
        "Done" => (Color::Green, "\u{2713} "),
        "To Do" => (Color::White, "\u{25cb} "),
        "Selected for Development" => (Color::Cyan, "\u{1f3af}"),
        _ => (Color::White, "  "),
    };
    Cell::new(format!("{icon} {status}")).fg(color)
}

pub fn make_table(width: u16) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(width.saturating_sub(4));
    table
}

pub fn print_table_indented(table: &Table) {
    for line in table.to_string().lines() {
        println!("  {line}");
    }
}

pub fn right_cell(text: impl Into<String>) -> Cell {
    Cell::new(text.into()).set_alignment(CellAlignment::Right)
}
