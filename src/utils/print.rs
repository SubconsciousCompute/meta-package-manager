use anyhow::Error;
use colored::{ColoredString, Colorize};
use strum::{EnumCount, IntoEnumIterator};
use tabled::{
    settings::{object::Rows, themes::Colorization, Color, Style},
    Table, Tabled,
};

use crate::{common::AvailablePackageManager, managers::MetaPackageManager};

use super::manager::PkgFormat;

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
        Listing {
            supported: format!("{pm:?}").green(),
            file_extensions: pm
                .supported_pkg_formats()
                .iter()
                .map(|pkg| pkg.file_extention())
                .collect::<Vec<_>>()
                .join(", ")
                .green(),
            available: if MetaPackageManager::try_new(pm).is_ok() {
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
        "a total of {} package managers are supported",
        AvailablePackageManager::COUNT
    );
    let table = Table::new(AvailablePackageManager::iter().map(Listing::new));
    print_table(table);
}

/// Prints list of supported package managers in JSON format to stdout
#[cfg(feature = "json")]
pub fn print_managers_json() -> anyhow::Result<()> {
    use anyhow::Context;

    let managers: Vec<_> = Manager::iter().map(PkgManangerInfo::new).collect();
    let stdout = std::io::stdout();
    serde_json::to_writer_pretty(stdout, &managers)
        .context("failed to print support package managers in JSON")?;
    Ok(())
}

/// Takes a `Table` type and sets appropriate styling options, then prints in
pub fn print_table(mut table: Table) {
    table
        .with(Style::rounded().remove_horizontals())
        .with(Colorization::exact([Color::FG_CYAN], Rows::first()));
    println!("{table}");
}

pub fn log_error(err: Error) {
    eprintln!("{} {err}", "Error:".red().bold());
    for (i, cause) in err.chain().skip(1).enumerate() {
        if i == 0 {
            eprintln!("\n{}", "Caused by:".yellow().bold());
        }
        eprintln!("({}) {cause}", i + 1);
    }
}
