use anyhow::Error;
use colored::{ColoredString, Colorize};
use strum::{EnumCount, IntoEnumIterator};
use tabled::{
    settings::{object::Rows, themes::Colorization, Color, Style},
    Table, Tabled,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{utils::manager::Manager, Package};

use super::manager::PkgFormat;

/// Takes a format string and prints it in the format "Info {format_str}"
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
    supported: Manager,
    /// CSV of support formats.
    file_extensions: ColoredString,
    available: ColoredString,
}

impl Listing {
    fn new(pm: Manager) -> Self {
        Listing {
            supported: pm,
            file_extensions: pm
                .supported_pkg_formats()
                .iter()
                .map(|pkg| pkg.file_extention())
                .collect::<Vec<_>>()
                .join(", ")
                .green(),
            available: if pm.init().is_some() {
                "Yes".green()
            } else {
                "No".red()
            },
        }
    }
}

#[cfg(feature = "json")]
/// Type used for printing a list of supported package managers as JSON
#[derive(Serialize, Deserialize, Debug)]
struct PkgManangerInfo {
    name: Manager,
    available: bool,
    file_extensions: Vec<PkgFormat>,
}

#[cfg(feature = "json")]
impl PkgManangerInfo {
    fn new(pm: Manager) -> Self {
        PkgManangerInfo {
            name: pm,
            file_extensions: pm.supported_pkg_formats(),
            available: pm.init().is_some(),
        }
    }
}

/// Struct used for printing packages in a table
#[derive(Tabled)]
#[allow(non_snake_case)]
struct PkgListing<'a> {
    Package: &'a str,
    Version: ColoredString,
}

impl<'a> PkgListing<'a> {
    fn new(pkg: &'a Package<'a>) -> Self {
        PkgListing {
            Package: pkg.name(),
            Version: if let Some(v) = pkg.version() {
                v.green()
            } else {
                "~".white() // no version info is present
            },
        }
    }
}

#[cfg(feature = "json")]
/// Prints list of packages with version information in JSON format to stdout
pub fn print_packages_json(pkgs: Vec<Package>) -> anyhow::Result<()> {
    use anyhow::Context;

    let stdout = std::io::stdout();
    serde_json::to_writer_pretty(stdout, &pkgs).context("failed to package list in JSON")?;
    Ok(())
}

/// Creates a table and prints list of packages with version information
pub fn print_packages(pkgs: Vec<Package>) {
    let table = Table::new(pkgs.iter().map(PkgListing::new));
    print_table(table);
}

/// Creates a table and prints supported package managers with availability
/// information
pub fn print_managers() {
    notify!(
        "a total of {} package managers are supported",
        Manager::COUNT
    );
    let table = Table::new(Manager::iter().map(Listing::new));
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
