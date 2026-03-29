use crate::error::CliError;
use serde::Serialize;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Format {
    Table,
    Json,
    Yaml,
}

pub fn print_json<T: Serialize>(value: &T) -> Result<(), CliError> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn print_yaml<T: Serialize>(value: &T) -> Result<(), CliError> {
    print!("{}", serde_yaml::to_string(value)?);
    Ok(())
}

/// Format a table as a string (for testing) or print to stdout.
pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return "(no results)".to_string();
    }

    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let mut lines = Vec::new();

    let header_line: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{:<width$}", h, width = widths[i]))
        .collect();
    lines.push(header_line.join("  "));

    let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    lines.push(sep.join("  "));

    for row in rows {
        let line: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let w = widths.get(i).copied().unwrap_or(0);
                format!("{:<width$}", cell, width = w)
            })
            .collect();
        lines.push(line.join("  "));
    }

    lines.join("\n")
}

pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    println!("{}", format_table(headers, rows));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_table_empty() {
        let result = format_table(&["ID", "NAME"], &[]);
        assert_eq!(result, "(no results)");
    }

    #[test]
    fn format_table_aligns_columns() {
        let rows = vec![
            vec!["1".to_string(), "short".to_string()],
            vec!["2".to_string(), "a longer name".to_string()],
        ];
        let result = format_table(&["ID", "NAME"], &rows);
        let lines: Vec<&str> = result.lines().collect();

        assert_eq!(lines.len(), 4); // header + separator + 2 rows
        assert!(lines[0].starts_with("ID"));
        assert!(lines[0].contains("NAME"));
        assert!(lines[1].contains("--")); // separator
        assert!(lines[2].starts_with("1 "));
        assert!(lines[3].starts_with("2 "));
    }

    #[test]
    fn format_table_wide_values_expand_columns() {
        let rows = vec![vec!["abcdef-1234-5678".to_string(), "x".to_string()]];
        let result = format_table(&["ID", "V"], &rows);
        let lines: Vec<&str> = result.lines().collect();

        // ID column should be as wide as the data value
        assert!(lines[0].len() == lines[2].len());
    }
}
