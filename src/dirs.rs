use std::vec;

use error;
use p4;

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
#[derive(Debug, Clone)]
pub struct Dirs<'p, 'f, 's> {
    connection: &'p p4::P4,
    dir: Vec<&'f str>,

    client_only: bool,
    stream: Option<&'s str>,
    include_deleted: bool,
    include_synced: bool,
    ignore_case: bool,
}

impl<'p, 'f, 's> Dirs<'p, 'f, 's> {
    pub fn new(connection: &'p p4::P4, dir: &'f str) -> Self {
        Self {
            connection,
            dir: vec![dir],
            client_only: false,
            stream: None,
            include_deleted: false,
            include_synced: false,
            ignore_case: false,
        }
    }

    pub fn dir(mut self, dir: &'f str) -> Self {
        self.dir.push(dir);
        self
    }

    /// The -C flag lists only directories that fall within the current
    /// client view.
    pub fn client_only(mut self, client_only: bool) -> Self {
        self.client_only = client_only;
        self
    }

    /// The -S flag limits output to depot directories mapped in a stream's
    /// client view.
    pub fn set_stream(mut self, stream: &'s str) -> Self {
        self.stream = Some(stream);
        self
    }

    /// The -D flag includes directories containing only deleted files.
    pub fn include_deleted(mut self, include_deleted: bool) -> Self {
        self.include_deleted = include_deleted;
        self
    }

    /// The -H flag lists directories containing files synced to the current
    /// client workspace.
    pub fn include_synced(mut self, include_synced: bool) -> Self {
        self.include_synced = include_synced;
        self
    }

    /// The -i flag is used to ignore the case of the file pattern when
    /// listing directories in a case sensitive server. This flag is not
    /// compatible with the -C option.
    pub fn ignore_case(mut self, ignore_case: bool) -> Self {
        self.ignore_case = ignore_case;
        self
    }

    /// Run the `dirs` command.
    pub fn run(self) -> Result<DirsIter, error::P4Error> {
        let mut cmd = self.connection.connect();
        cmd.arg("dirs");
        if self.client_only {
            cmd.arg("-C");
        }
        if let Some(stream) = self.stream {
            cmd.args(&["-S", stream]);
        }
        if self.include_deleted {
            cmd.arg("-D");
        }
        if self.include_synced {
            cmd.arg("-H");
        }
        if self.ignore_case {
            cmd.arg("-i");
        }
        for dir in self.dir {
            cmd.arg(dir);
        }
        let data = cmd.output().map_err(|e| {
            error::ErrorKind::SpawnFailed
                .error()
                .set_cause(e)
                .set_context(format!("Command: {:?}", cmd))
        })?;
        let (_remains, (mut items, exit)) = dirs_parser::dirs(&data.stdout).map_err(|_| {
            error::ErrorKind::ParseFailed
                .error()
                .set_context(format!("Command: {:?}", cmd))
        })?;
        items.push(exit);
        Ok(DirsIter(items.into_iter()))
    }
}

pub type DirItem = error::Item<Dir>;

#[derive(Debug)]
pub struct DirsIter(vec::IntoIter<DirItem>);

impl Iterator for DirsIter {
    type Item = DirItem;

    #[inline]
    fn next(&mut self) -> Option<DirItem> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.0.count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dir {
    pub dir: String,
    non_exhaustive: (),
}

mod dirs_parser {
    use super::super::parser::*;

    named!(dir_<&[u8], super::Dir>,
        do_parse!(
            dir: dir >>
            (
                super::Dir {
                    dir: dir.dir.to_owned(),
                    non_exhaustive: (),
                }
            )
        )
    );

    named!(item<&[u8], super::DirItem>,
        alt!(
            map!(dir_, data_to_item) |
            map!(error, error_to_item)
        )
    );

    named!(pub dirs<&[u8], (Vec<super::DirItem>, super::DirItem)>,
        pair!(
            many0!(item),
            map!(exit, exit_to_item)
        )
    );
}
