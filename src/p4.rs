use std::fmt;
use std::process;
use std::str;

use chrono;
use chrono::TimeZone;

use files;

#[derive(Clone, Debug)]
pub struct P4 {
    port: Option<String>,
    user: Option<String>,
    password: Option<String>,
    client: Option<String>,
}

impl P4 {
    pub fn new() -> Self {
        Self {
            port: None,
            user: None,
            password: None,
            client: None,
        }
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
    /// let files = p4.files("//depot/dir/*").run();
    /// for file in files {
    ///     println!("{:?}", file);
    /// }
    /// ```
    pub fn files<'p, 'f>(&'p self, file: &'f str) -> files::Files<'p, 'f> {
        files::Files::new(self, file)
    }

    pub(crate) fn connect(&self) -> process::Command {
        let mut cmd = process::Command::new("p4");
        cmd.args(&["-Gs", "-C utf8"]);
        if let Some(ref port) = self.port {
            cmd.args(&["-p", port.as_str()]);
        }
        if let Some(ref user) = self.user {
            cmd.args(&["-c", user.as_str()]);
        }
        if let Some(ref password) = self.password {
            cmd.args(&["-c", password.as_str()]);
        }
        if let Some(ref client) = self.client {
            cmd.args(&["-c", client.as_str()]);
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
            Action::Edit=> "edit",
            Action::Delete=> "delete",
            Action::Branch=> "branch",
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
            })
            .map_or(Ok(None), |r| r.map(Some))?;

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
