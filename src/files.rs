use std::vec;

use error;
use p4;

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
#[derive(Debug, Clone)]
pub struct Files<'p, 'f> {
    connection: &'p p4::P4,
    file: Vec<&'f str>,

    list_revisions: bool,
    syncable_only: bool,
    ignore_case: bool,
    max: Option<usize>,
}

impl<'p, 'f> Files<'p, 'f> {
    pub fn new(connection: &'p p4::P4, file: &'f str) -> Self {
        Self {
            connection,
            file: vec![file],
            list_revisions: false,
            syncable_only: false,
            ignore_case: false,
            max: None,
        }
    }

    pub fn file(mut self, file: &'f str) -> Self {
        self.file.push(file);
        self
    }

    /// The -a flag displays all revisions within the specific range, rather
    /// than just the highest revision in the range.
    pub fn list_revisions(mut self, list_revisions: bool) -> Self {
        self.list_revisions = list_revisions;
        self
    }

    /// The -e flag displays files with an action of anything other than
    /// deleted, purged or archived.  Typically this revision is always
    /// available to sync or integrate from.
    pub fn syncable_only(mut self, syncable_only: bool) -> Self {
        self.syncable_only = syncable_only;
        self
    }

    /// The -i flag is used to ignore the case of the file argument when
    /// listing files in a case sensitive server.
    pub fn ignore_case(mut self, ignore_case: bool) -> Self {
        self.ignore_case = ignore_case;
        self
    }

    /// The -m flag limits files to the first 'max' number of files.
    pub fn set_max(mut self, max: Option<usize>) -> Self {
        self.max = max;
        self
    }

    /// Run the `files` command.
    pub fn run(self) -> Result<FilesIter, error::P4Error> {
        let mut cmd = self.connection.connect();
        cmd.arg("files");
        if self.list_revisions {
            cmd.arg("-a");
        }
        if self.syncable_only {
            cmd.arg("-e");
        }
        if self.ignore_case {
            cmd.arg("-i");
        }
        if let Some(max) = self.max {
            cmd.arg(format!("-m {}", max));
        }
        for file in self.file {
            cmd.arg(file);
        }
        let data = cmd.output().map_err(|e| {
            error::ErrorKind::SpawnFailed
                .error()
                .set_cause(e)
                .set_context(format!("Command: {:?}", cmd))
        })?;
        let (_remains, (mut items, exit)) = files_parser::files(&data.stdout).map_err(|_| {
            error::ErrorKind::ParseFailed
                .error()
                .set_context(format!("Command: {:?}", cmd))
        })?;
        items.push(exit);
        Ok(FilesIter(items.into_iter()))
    }
}

pub type FileItem = error::Item<File>;

#[derive(Debug)]
pub struct FilesIter(vec::IntoIter<FileItem>);

impl Iterator for FilesIter {
    type Item = FileItem;

    #[inline]
    fn next(&mut self) -> Option<FileItem> {
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
pub struct File {
    pub depot_file: String,
    pub rev: usize,
    pub change: usize,
    pub action: p4::Action,
    pub file_type: p4::FileType,
    pub time: p4::Time,
    non_exhaustive: (),
}

mod files_parser {
    use super::*;

    use super::super::parser::*;

    named!(file<&[u8], File>,
        do_parse!(
            depot_file: depot_file >>
            rev: rev >>
            change: change >>
            action: action >>
            file_type: file_type >>
            time: time >>
            (
                File {
                    depot_file: depot_file.path.to_owned(),
                    rev: rev.rev,
                    change: change.change,
                    action: action.action.parse().expect("Unknown to capture all"),
                    file_type: file_type.ft.parse().expect("Unknown to capture all"),
                    time: p4::from_timestamp(time.time),
                    non_exhaustive: (),
                }
            )
        )
    );

    named!(item<&[u8], FileItem>,
        alt!(
            map!(file, data_to_item) |
            map!(error, error_to_item) |
            map!(info, info_to_item)
        )
    );

    named!(pub files<&[u8], (Vec<FileItem>, FileItem)>,
        pair!(
            many0!(item),
            map!(exit, exit_to_item)
        )
    );
}
