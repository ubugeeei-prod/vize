//! Printing the P0-13 seeded-defect assertion report (`seed-defects.rs`).

use crate::davinci_fpfn::{DiagnosticRow, SeedAssertReport};

pub fn print_assert_report(report: &SeedAssertReport) {
    println!(
        "assert: class-a detected={}/{} class-b detected={}/{} baseline mapped={} verdict={}",
        report.class_a.detected,
        report.class_a.expected,
        report.class_b.detected,
        report.class_b.expected,
        report.baseline_shift.mapped,
        report.verdict
    );
    for miss in &report.class_a.misses {
        println!(
            "MISS class-a {}:{}:{}-{}:{} {} identifier={}",
            miss.path,
            miss.line,
            miss.column,
            miss.end_line,
            miss.end_column,
            miss.rule_id,
            miss.identifier
        );
    }
    for miss in &report.baseline_shift.misses {
        println!("MISS baseline {}", describe_row(miss));
    }
    for row in &report.baseline_shift.unmappable {
        println!("UNMAPPABLE baseline {}", describe_row(row));
    }
    for row in &report.unexpected {
        println!("UNEXPECTED {}", describe_row(row));
    }
}

fn describe_row(row: &DiagnosticRow) -> String {
    format!(
        "{}:{}:{}-{}:{} {}",
        row.path, row.line, row.column, row.end_line, row.end_column, row.rule_id
    )
}
