use colored::{ColoredString, Colorize};
use strum::{EnumCount, IntoEnumIterator};
use tabled::{
    settings::{object::Rows, themes::Colorization, Color, Style},
    Table, Tabled,
};

use crate::{
    common::AvailablePackageManager, managers::MetaPackageManager, PackageManager,
    PackageManagerCommands,
};

/// Takes a format string and prints it in the format "Info {format_str}"
#[macro_export]
macro_rules! notify {
    ($($fmt:tt)+) => {
        {
            println!("{args}", args = format_args!($($fmt)+))
        }
    };
}

/// Struct used for printing supported package managers in a table
#[derive(Tabled)]
#[tabled(rename_all = "PascalCase")]
struct Listing {
    supported: ColoredString,
    /// CSV of support formats.
    file_extensions: ColoredString,
    available: ColoredString,
}

impl Listing {
    pub(crate) fn new(pm: AvailablePackageManager) -> Self {
        let mpm = MetaPackageManager::new(pm.clone());
        Listing {
            supported: format!("{pm}").green(),
            file_extensions: mpm
                .supported_pkg_formats()
                .iter()
                .map(|pkg| pkg.file_extention())
                .collect::<Vec<_>>()
                .join(", ")
                .green(),
            available: if mpm.is_available() {
                "Yes".green()
            } else {
                "No".red()
            },
        }
    }
}

/// Creates a table and prints supported package managers with availability
/// information
pub fn print_managers() {
    notify!(
        "Total {} package managers are supported",
        AvailablePackageManager::COUNT
    );
    let table = Table::new(AvailablePackageManager::iter().map(Listing::new));
    print_table(table);
}

/// Takes a `Table` type and sets appropriate styling options, then prints in
pub fn print_table(mut table: Table) {
    table
        .with(Style::rounded().remove_horizontals())
        .with(Colorization::exact([Color::FG_CYAN], Rows::first()));
    println!("{table}");
}

/// Log error
pub fn log_error(err: anyhow::Error) {
    eprintln!("{} {err}", "Error:".red().bold());
    for (i, cause) in err.chain().skip(1).enumerate() {
        if i == 0 {
            eprintln!("\n{}", "Caused by:".yellow().bold());
        }
        eprintln!("({}) {cause}", i + 1);
    }
}
