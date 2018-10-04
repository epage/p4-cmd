use std::fmt;
use std::path;
use std::process;
use std::str;

use chrono;
use chrono::TimeZone;

use dirs;
use files;
use print;
use sync;
use where_;

#[derive(Clone, Debug)]
pub struct P4 {
    custom_p4: Option<path::PathBuf>,
    port: Option<String>,
    user: Option<String>,
    password: Option<String>,
    client: Option<String>,
    retries: Option<usize>,
}

impl P4 {
    pub fn new() -> Self {
        Self {
            custom_p4: None,
            port: None,
            user: None,
            password: None,
            client: None,
            retries: None,
        }
    }

    /// Overrides the `p4` command used.
    ///
    /// This is useful for "portable" installs (not in system path) and performance (caching the
    /// `PATH` lookup via `where` crate).
    pub fn set_p4_cmd(mut self, custom_p4: Option<path::PathBuf>) -> Self {
        self.custom_p4 = custom_p4;
        self
    }

    /// Overrides any P4PORT setting with the specified protocol:host:port.
    pub fn set_port(mut self, port: Option<String>) -> Self {
        self.port = port;
        self
    }

    /// Overrides any P4USER, USER, or USERNAME setting with the specified user name.
    pub fn set_user(mut self, user: Option<String>) -> Self {
        self.user = user;
        self
    }

    /// Overrides any P4PASSWD setting with the specified passwo
    pub fn set_password(mut self, password: Option<String>) -> Self {
        self.password = password;
        self
    }

    /// Overrides any P4CLIENT setting with the specified client name.
    pub fn set_client(mut self, client: Option<String>) -> Self {
        self.client = client;
        self
    }

    /// Number of times a command should be retried if the network times out (takes longer than N
    /// seconds to respond to a single I/O operation) during command execution.
    pub fn set_retries(mut self, retries: Option<usize>) -> Self {
        self.retries = retries;
        self
    }

    /// Write a depot file to standard output
    ///
    /// Retrieve the contents of a depot file to the client's standard output.
    /// The file is not synced.  If file is specified using client syntax,
    /// Perforce uses the client view to determine the corresponding depot
    /// file.
    ///
    /// By default, the head revision is printed.  If the file argument
    /// includes a revision, the specified revision is printed.  If the
    /// file argument has a revision range,  then only files selected by
    /// that revision range are printed, and the highest revision in the
    /// range is printed. For details about revision specifiers, see 'p4
    /// help revisions'.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let p4 = p4_cmd::P4::new();
    /// let files = p4.print("//depot/dir/file").run().unwrap();
    /// for file in files {
    ///     println!("{:?}", file);
    /// }
    /// ```
    pub fn print<'p, 'f>(&'p self, file: &'f str) -> print::PrintCommand<'p, 'f> {
        print::PrintCommand::new(self, file)
    }

    /// Synchronize the client with its view of the depot
    ///
    /// Sync updates the client workspace to reflect its current view (if
    /// it has changed) and the current contents of the depot (if it has
    /// changed). The client view maps client and depot file names and
    /// locations.
    ///
    /// Sync adds files that are in the client view and have not been
    /// retrieved before.  Sync deletes previously retrieved files that
    /// are no longer in the client view or have been deleted from the
    /// depot.  Sync updates files that are still in the client view and
    /// have been updated in the depot.
    ///
    /// By default, sync affects all files in the client workspace. If file
    /// arguments are given, sync limits its operation to those files.
    /// The file arguments can contain wildcards.
    ///
    /// If the file argument includes a revision specifier, then the given
    /// revision is retrieved.  Normally, the head revision is retrieved.
    ///
    /// If the file argument includes a revision range specification,
    /// only files selected by the revision range are updated, and the
    /// highest revision in the range is used.
    ///
    /// See 'p4 help revisions' for help specifying revisions or ranges.
    ///
    /// Normally, sync does not overwrite workspace files that the user has
    /// manually made writable.  Setting the 'clobber' option in the
    /// client specification disables this safety check.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let p4 = p4_cmd::P4::new();
    /// let dirs = p4.sync("//depot/dir/*").run().unwrap();
    /// for dir in dirs {
    ///     println!("{:?}", dir);
    /// }
    /// ```
    pub fn sync<'p, 'f>(&'p self, file: &'f str) -> sync::SyncCommand<'p, 'f> {
        sync::SyncCommand::new(self, file)
    }

    /// List files in the depot.
    ///
    /// List details about specified files: depot file name, revision,
    /// file, type, change action and changelist number of the current
    /// head revision. If client syntax is used to specify the file
    /// argument, the client view mapping is used to determine the
    /// corresponding depot files.
    ///
    /// By default, the head revision is listed.  If the file argument
    /// specifies a revision, then all files at that revision are listed.
    /// If the file argument specifies a revision range, the highest revision
    /// in the range is used for each file. For details about specifying
    /// revisions, see 'p4 help revisions'.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let p4 = p4_cmd::P4::new();
    /// let files = p4.files("//depot/dir/*").run().unwrap();
    /// for file in files {
    ///     println!("{:?}", file);
    /// }
    /// ```
    pub fn files<'p, 'f>(&'p self, file: &'f str) -> files::FilesCommand<'p, 'f> {
        files::FilesCommand::new(self, file)
    }

    /// List depot subdirectories
    ///
    /// List directories that match the specified file pattern (dir).
    /// This command does not support the recursive wildcard (...).
    /// Use the * wildcard instead.
    ///
    /// Perforce does not track directories individually. A path is treated
    /// as a directory if there are any undeleted files with that path as a
    /// prefix.
    ///
    /// By default, all directories containing files are listed. If the dir
    /// argument includes a revision range, only directories containing files
    /// in the range are listed. For details about specifying file revisions,
    /// see 'p4 help revisions'.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let p4 = p4_cmd::P4::new();
    /// let dirs = p4.dirs("//depot/dir/*").run().unwrap();
    /// for dir in dirs {
    ///     println!("{:?}", dir);
    /// }
    /// ```
    pub fn dirs<'p, 'f, 's>(&'p self, dir: &'f str) -> dirs::DirsCommand<'p, 'f, 's> {
        dirs::DirsCommand::new(self, dir)
    }

    /// Show how file names are mapped by the client view
    ///
    /// Where shows how the specified files are mapped by the client view.
    /// For each argument, three names are produced: the name in the depot,
    /// the name on the client in Perforce syntax, and the name on the client
    /// in local syntax.
    ///
    /// If the file parameter is omitted, the mapping for all files in the
    /// current directory and below) is returned.
    ///
    /// Note that 'p4 where' does not determine where any real files reside.
    /// It only displays the locations that are mapped by the client view.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let p4 = p4_cmd::P4::new();
    /// let files = p4.where_().file("//depot/dir/*").run().unwrap();
    /// for file in files {
    ///     println!("{:?}", file);
    /// }
    /// ```
    pub fn where_<'p, 'f>(&'p self) -> where_::WhereCommand<'p, 'f> {
        where_::WhereCommand::new(self)
    }

    pub(crate) fn connect(&self) -> process::Command {
        let p4_cmd = self
            .custom_p4
            .as_ref()
            .map(path::PathBuf::as_path)
            .unwrap_or_else(|| path::Path::new("p4"));
        let mut cmd = process::Command::new(p4_cmd);
        cmd.args(&["-Gs", "-C utf8"]);
        if let Some(ref port) = self.port {
            cmd.args(&["-p", port.as_str()]);
        }
        if let Some(ref user) = self.user {
            cmd.args(&["-u", user.as_str()]);
        }
        if let Some(ref password) = self.password {
            cmd.args(&["-P", password.as_str()]);
        }
        if let Some(ref client) = self.client {
            cmd.args(&["-c", client.as_str()]);
        }
        cmd
    }

    pub(crate) fn connect_with_retries(&self, retries: Option<usize>) -> process::Command {
        let mut cmd = self.connect();
        if let Some(retries) = retries.or(self.retries) {
            let retries = format!("{}", retries);
            cmd.args(&["-r", &retries]);
        }
        cmd
    }
}

pub type Time = chrono::DateTime<chrono::Utc>;

// Keeping around for future use.
#[allow(dead_code)]
pub(crate) fn to_timestamp(time: &Time) -> i64 {
    time.timestamp()
}

pub(crate) fn from_timestamp(timestamp: i64) -> Time {
    chrono::Utc.timestamp(timestamp, 0)
}

/// Action performed on a file at a given revision.
///
/// # Example
///
/// ```rust
/// assert_eq!(p4_cmd::Action::MoveDelete.to_string(), "move/delete");
/// assert_eq!("move/delete".parse::<p4_cmd::Action>().unwrap(), p4_cmd::Action::MoveDelete);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    #[doc(hidden)]
    __Nonexhaustive,

    Add,
    Edit,
    Delete,
    Branch,
    MoveAdd,
    MoveDelete,
    Integrate,
    Import,
    Purge,
    Archive,

    Unknown(String),
}

impl str::FromStr for Action {
    type Err = fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ft = match s {
            "add" => Action::Add,
            "edit" => Action::Edit,
            "delete" => Action::Delete,
            "branch" => Action::Branch,
            "move/add" => Action::MoveAdd,
            "move/delete" => Action::MoveDelete,
            "integrate" => Action::Integrate,
            "import" => Action::Import,
            "purge" => Action::Purge,
            "archive" => Action::Archive,
            s => Action::Unknown(s.to_owned()),
        };
        Ok(ft)
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Action::Add => "add",
            Action::Edit => "edit",
            Action::Delete => "delete",
            Action::Branch => "branch",
            Action::MoveAdd => "move/add",
            Action::MoveDelete => "move/delete",
            Action::Integrate => "integrate",
            Action::Import => "import",
            Action::Purge => "purge",
            Action::Archive => "archive",
            Action::Unknown(ref s) => s.as_str(),
            Action::__Nonexhaustive => unreachable!("This is a private variant"),
        };
        write!(f, "{}", value)
    }
}

/// Perforce base file type.
///
/// # Example
///
/// ```rust
/// assert_eq!(p4_cmd::BaseFileType::Utf8.to_string(), "utf8");
/// assert_eq!("utf8".parse::<p4_cmd::BaseFileType>().unwrap(), p4_cmd::BaseFileType::Utf8);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseFileType {
    #[doc(hidden)]
    __Nonexhaustive,

    /// Text file
    ///
    /// Synced as text in the workspace. Line-ending translations are performed automatically.
    ///
    /// Stored as: deltas in RCS format
    Text,
    /// Non-text file
    ///
    /// Synced as binary files in the workspace. Stored compressed within the depot.
    ///
    /// Stored as: full file, compressed
    Binary,
    /// Symbolic link
    ///
    /// Helix Server applications on UNIX, OS X, recent versions of Windows treat these files as
    /// symbolic links. On other platforms, these files appear as (small) text files.
    ///
    /// On Windows, you require admin privileges or an appropriate group policy must be set, otherwise, you get text files.
    ///
    /// Stored as: deltas in RCS format
    Symlink,
    /// Unicode file
    ///
    /// Services operating in unicode mode support the unicode file type. These files are
    /// translated into the local character set specified by P4CHARSET.
    ///
    /// Line-ending translations are performed automatically.
    ///
    /// Services not in unicode mode do not support the unicode file type.
    ///
    ///Stored as: RCS deltas in UTF-8 format
    Unicode,
    /// Unicode file
    ///
    /// Synced in the client workspace with the UTF-8 BOM (byte order mark).
    ///
    /// Whether the service is in unicode mode or not, files are transferred as UTF-8 in the client workspace.
    ///
    /// Line-ending translations are performed automatically.
    ///
    /// Stored as: RCS deltas in UTF-8 format without the UTF-8 BOM (byte order mark).
    Utf8,
    /// Unicode file
    ///
    /// Whether the service is in unicode mode or not, files are transferred as UTF-8, and
    /// translated to UTF-16 (with byte order mark, in the byte order appropriate for the user's
    /// machine) in the client workspace.
    ///
    /// Line-ending translations are performed automatically.
    ///
    /// Stored as: RCS deltas in UTF-8 format
    Utf16,

    Unknown(String),
}

impl Default for BaseFileType {
    fn default() -> Self {
        BaseFileType::Text
    }
}

impl str::FromStr for BaseFileType {
    type Err = fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ft = match s {
            "text" => BaseFileType::Text,
            "binary" => BaseFileType::Binary,
            "symlink" => BaseFileType::Symlink,
            "unicode" => BaseFileType::Unicode,
            "utf8" => BaseFileType::Utf8,
            "utf16" => BaseFileType::Utf16,
            s => BaseFileType::Unknown(s.to_owned()),
        };
        Ok(ft)
    }
}

impl fmt::Display for BaseFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            BaseFileType::Text => "text",
            BaseFileType::Binary => "binary",
            BaseFileType::Symlink => "symlink",
            BaseFileType::Unicode => "unicode",
            BaseFileType::Utf8 => "utf8",
            BaseFileType::Utf16 => "utf16",
            BaseFileType::Unknown(ref s) => s.as_str(),
            BaseFileType::__Nonexhaustive => unreachable!("This is a private variant"),
        };
        write!(f, "{}", value)
    }
}

/// Perforce file type modifiers.
///
/// # Example
///
/// ```rust
/// let mut modifiers = p4_cmd::FileTypeModifiers::new();
/// modifiers.exclusive = true;
/// assert_eq!(modifiers.to_string(), "l");
/// assert_eq!("l".parse::<p4_cmd::FileTypeModifiers>().unwrap(), modifiers);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileTypeModifiers {
    /// File is always writable on client
    pub always_writeable: bool,
    /// Execute bit set on client
    pub executable: bool,
    /// RCS keyword expansion
    pub rcs_expansion: bool,
    /// Exclusive open (locking)
    pub exclusive: bool,
    /// Perforce stores the full compressed version of each file revision
    pub full: bool,
    /// Perforce stores deltas in RCS format
    pub deltas: bool,
    /// Perforce stores full file per revision, uncompressed
    pub full_uncompressed: bool,
    /// Only the head revision is stored
    pub head: bool,
    /// Only the most recent n revisions are stored
    pub revisions: Option<usize>,
    /// Preserve original modtime
    pub modtime: bool,
    /// Archive trigger required
    pub archive: bool,
    non_exhaustive: (),
}

impl FileTypeModifiers {
    pub fn new() -> Self {
        Default::default()
    }
}

impl str::FromStr for FileTypeModifiers {
    type Err = fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut modifiers = FileTypeModifiers::default();

        for flag in s.chars() {
            match flag {
                'w' => modifiers.always_writeable = true,
                'x' => modifiers.executable = true,
                'k' => modifiers.rcs_expansion = true,
                'l' => modifiers.exclusive = true,
                'C' => modifiers.full = true,
                'D' => modifiers.deltas = true,
                'F' => modifiers.full_uncompressed = true,
                'S' => modifiers.head = true,
                // TODO: handle `revisions`.
                'm' => modifiers.modtime = true,
                'X' => modifiers.archive = true,
                _ => return Err(fmt::Error),
            }
        }

        Ok(modifiers)
    }
}

impl fmt::Display for FileTypeModifiers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.always_writeable {
            write!(f, "w")?;
        }
        if self.executable {
            write!(f, "x")?;
        }
        if self.rcs_expansion {
            write!(f, "k")?;
        }
        if self.exclusive {
            write!(f, "l")?;
        }
        if self.full {
            write!(f, "C")?;
        }
        if self.deltas {
            write!(f, "D")?;
        }
        if self.full_uncompressed {
            write!(f, "S")?;
        }
        if self.head {
            write!(f, "S")?;
        }
        if let Some(revisions) = self.revisions {
            write!(f, "S{}", revisions)?;
        }
        if self.modtime {
            write!(f, "m")?;
        }
        if self.archive {
            write!(f, "X")?;
        }

        Ok(())
    }
}

/// Perforce file type.
///
/// # Example
///
/// ```rust
/// let mut modifiers = p4_cmd::FileTypeModifiers::default();
/// modifiers.exclusive = true;
/// let ft = p4_cmd::FileType::new()
///     .base(p4_cmd::BaseFileType::Binary)
///     .modifiers(Some(modifiers));
/// assert_eq!(ft.to_string(), "binary+l");
/// assert_eq!("binary+l".parse::<p4_cmd::FileType>().unwrap(), ft);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileType {
    /// The base Perforce file type
    pub base: BaseFileType,
    pub modifiers: Option<FileTypeModifiers>,
    non_exhaustive: (),
}

impl FileType {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn base(mut self, base: BaseFileType) -> Self {
        self.base = base;
        self
    }

    pub fn modifiers(mut self, modifiers: Option<FileTypeModifiers>) -> Self {
        self.modifiers = modifiers;
        self
    }
}

impl str::FromStr for FileType {
    type Err = fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut itr = s.splitn(2, '+');
        let base = itr.next().ok_or(fmt::Error)?;
        let base = base.parse().map_err(|_| fmt::Error)?;

        let modifiers = itr
            .next()
            .map(|f| {
                let modifiers: FileTypeModifiers = f.parse()?;
                Ok(modifiers)
            }).map_or(Ok(None), |r| r.map(Some))?;

        let ft = FileType {
            base,
            modifiers,
            non_exhaustive: (),
        };

        Ok(ft)
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.base)?;
        if let Some(ref modifiers) = self.modifiers {
            write!(f, "+{}", modifiers)?;
        }
        Ok(())
    }
}
