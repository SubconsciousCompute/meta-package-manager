use super::Manager;
use crate::Package;
use anyhow::Error;
use colored::{ColoredString, Colorize};
use strum::{EnumCount, IntoEnumIterator};
use tabled::{
    settings::{object::Rows, themes::Colorization, Color, Style},
    Table, Tabled,
};

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
#[allow(non_snake_case)]
struct Listing {
    Supported: Manager,
    Available: ColoredString,
}

impl Listing {
    fn new(pm: Manager) -> Self {
        Listing {
            Supported: pm,
            Available: if pm.init().is_some() {
                "Yes".green()
            } else {
                "No".red()
            },
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
