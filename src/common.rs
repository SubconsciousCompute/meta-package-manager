//! Common types and traits.

use std::{
    borrow::Cow,
    fmt::Display,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use anyhow::Context;

/// Representation of a package manager command
///
/// All the variants are the type of commands that a type that imlements
/// [``PackageManagerCommands``] and [``PackageManager``] (should) support.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cmd {
    Install,
    Uninstall,
    Update,
    UpdateAll,
    List,
    Sync,
    AddRepo,
    Search,
}

/// A representation of a package
///
/// This struct contains package's name and version information (optional).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Package {
    /// name of the package
    name: String,

    /// Untyped version, might be replaced with a strongly typed one
    version: Option<String>,

    /// Url of this package. A local package can be passed as "file://" URI.
    url: Option<url::Url>,
}

impl Package {
    /// Create new Package with name and version.
    pub fn new(name: &str, version: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            version: version.map(|v| v.to_string()),
            url: None,
        }
    }

    /// Name of the package
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Package name for cli.
    pub fn cli_display(&self, pkg_delimiter: char) -> String {
        if let Some(url) = self.url() {
            if url.scheme() == "file" {
                tracing::debug!("Found on-disk path. Stripping file://...");
                return url.as_str().strip_prefix("file://").unwrap().to_string();
            }
            return url.as_str().to_string();
        }
        if let Some(version) = &self.version {
            return format!("{}{}{}", self.name, pkg_delimiter, version);
        }
        self.name.clone()
    }

    /// Get version information if present
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Get version information if present
    pub fn url(&self) -> Option<&url::Url> {
        self.url.as_ref()
    }

    /// Turn remote url to local file based URI
    pub fn make_available_on_disk(
        &mut self,
        output: Option<&std::path::Path>,
        force: bool,
    ) -> anyhow::Result<std::path::PathBuf> {
        use std::io::Write;

        anyhow::ensure!(
            self.url().is_some(),
            "There is no URL associated with this package"
        );
        let url = self.url().context("missing URL")?;
        anyhow::ensure!(
            url.scheme() != "file",
            "Package already points to local file {url:?}"
        );

        let pkgpath = match output {
            Some(p) => p.into(),
            None => std::env::temp_dir().join(
                url.path_segments()
                    .context("missing path in url")?
                    .last()
                    .context("missing filepath in url")?,
            ),
        };

        // download to disk.
        if !pkgpath.exists() || force {
            tracing::debug!("Downloading package from `{url}` (force={force})...");
            let resp = reqwest::blocking::Client::builder()
                .timeout(None)
                .build()?
                .get(url.as_str())
                .send()?;

            let bytes = resp.bytes()?;
            tracing::debug!(" ... fetched {} MB.", bytes.len() / 1024 / 1024);

            let mut buffer = std::fs::File::create(&pkgpath)?;
            buffer.write_all(&bytes)?;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        anyhow::ensure!(pkgpath.is_file(), "Failed to download {url} -> {pkgpath:?}");
        self.url = format!("file://{}", pkgpath.display()).parse().ok();
        Ok(pkgpath)
    }
}

impl std::str::FromStr for Package {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        if let Ok(url) = url::Url::parse(s) {
            let name = url
                .path_segments()
                .context("can not determine pane from the url")?
                .last()
                .expect("can't determine package name from the url");
            let mut fragments = std::collections::HashMap::new();
            for frag in url.fragment().unwrap_or("").split(',') {
                let mut fs = frag.splitn(2, '=');
                if let Some(key) = fs.next() {
                    if let Some(value) = fs.next() {
                        fragments.insert(key, value.to_string());
                    }
                }
            }
            return Ok(Self {
                name: name.to_string(),
                version: fragments.remove("version"),
                url: Some(url),
            });
        }

        if let Some((name, version)) = s.split_once('@') {
            Ok(Package::new(name, Some(version)))
        } else {
            Ok(Package::new(s, None))
        }
    }
}

impl std::convert::From<&str> for Package {
    fn from(s: &str) -> Self {
        s.parse().expect("invalid format")
    }
}

impl std::convert::From<&std::path::Path> for Package {
    fn from(p: &std::path::Path) -> Self {
        let s = format!("file://{}", p.display());
        s.parse().expect("invalid format")
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self.version() {
            // might be changed later for a better format
            write!(f, "{} - {}", self.name, v)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl tabled::Tabled for Package {
    const LENGTH: usize = 40;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.name.clone().into(),
            self.version.as_deref().unwrap_or("~").into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec!["name".into(), "version".into()]
    }
}

/// Available package manager. This is from cli because I can't use
/// MetaPackageManager as `clap::ValueEnum`.
#[derive(
    Clone,
    PartialEq,
    Debug,
    clap::ValueEnum,
    strum::Display,
    strum::EnumIter,
    strum::EnumCount,
    strum::EnumString,
)]
#[strum(ascii_case_insensitive)]
pub enum AvailablePackageManager {
    Apt,
    Brew,
    Choco,
    Dnf,
    Yum,
    Zypper,
}

/// Operation type to execute using [``Package::execute_pkg_command``]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    Install,
    Uninstall,
    Update,
}

/// Pkg Format.
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum PkgFormat {
    Bottle,
    Exe,
    Msi,
    Rpm,
    Deb,
}

impl PkgFormat {
    /// File extension of package.
    pub fn file_extention(&self) -> String {
        match self {
            Self::Bottle => "tar.gz",
            Self::Exe => "exe",
            Self::Msi => "msi",
            Self::Rpm => "rpm",
            Self::Deb => "deb",
        }
        .to_string()
    }
}

/// Command result is a tuple of ExitStatus, stdout lines
pub struct CommandResult(pub std::process::ExitStatus, pub Vec<String>);

impl CommandResult {
    /// Command executed successfully?
    pub fn success(&self) -> bool {
        self.0.success()
    }
}

impl std::fmt::Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "<code={:?}, success={}, lines={}>",
            self.0.code(),
            self.0.success(),
            self.1.len()
        )
    }
}

/// Execute a command and stream its output. Collect response.
pub fn run_command<S: AsRef<str> + std::convert::AsRef<std::ffi::OsStr>>(
    mut cmd: Command,
    args: &[S],
    stream_to_stdout: bool,
) -> anyhow::Result<CommandResult> {
    let mut result = vec![];
    let mut child = cmd.args(args).stdout(Stdio::piped()).spawn()?;
    {
        let stdout = child.stdout.as_mut().unwrap();
        let stdout_reader = BufReader::new(stdout);
        let stdout_lines = stdout_reader.lines();

        for line in stdout_lines.filter_map(Result::ok) {
            if stream_to_stdout {
                println!(">> {line}");
            } else {
                tracing::debug!(">> {line}");
            }
            result.push(line);
        }
    }
    let ec = child.wait()?;
    tracing::trace!(">>> command response: {}", ec);
    Ok(CommandResult(ec, result))
}
