use std::path;
use std::vec;

use error;
use p4;

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
#[derive(Debug, Clone)]
pub struct Sync<'p, 'f> {
    connection: &'p p4::P4,
    file: Vec<&'f str>,

    force: bool,
    preview: bool,
    server_only: bool,
    client_only: bool,
    verify: bool,
    max_files: Option<usize>,
    parallel: Option<usize>,
}

impl<'p, 'f> Sync<'p, 'f> {
    pub fn new(connection: &'p p4::P4, file: &'f str) -> Self {
        Self {
            connection: connection,
            file: vec![file],
            force: false,
            preview: false,
            server_only: false,
            client_only: false,
            verify: false,
            max_files: None,
            parallel: None,
        }
    }

    pub fn file(mut self, dir: &'f str) -> Self {
        self.file.push(dir);
        self
    }

    /// The -f flag forces resynchronization even if the client already
    /// has the file, and overwriting any writable files.  This flag doesn't
    /// affect open files.
    pub fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// The -n flag previews the operation without updating the workspace.
    pub fn preview(mut self, preview: bool) -> Self {
        self.preview = preview;
        self
    }

    /// The -k flag updates server metadata without syncing files. It is
    /// intended to enable you to ensure that the server correctly reflects
    /// the state of files in the workspace while avoiding a large data
    /// transfer. Caution: an erroneous update can cause the server to
    /// incorrectly reflect the state of the workspace.
    pub fn server_only(mut self, server_only: bool) -> Self {
        self.server_only = server_only;
        self
    }

    /// The -p flag populates the client workspace, but does not update the
    /// server to reflect those updates.  Any file that is already synced or
    /// opened will be bypassed with a warning message.  This option is very
    /// useful for build clients or when publishing content without the
    /// need to track the state of the client workspace.
    pub fn client_only(mut self, client_only: bool) -> Self {
        self.client_only = client_only;
        self
    }

    /// The -s flag adds a safety check before sending content to the client
    /// workspace.  This check uses MD5 digests to compare the content on the
    /// clients workspace against content that was last synced.  If the file
    /// has been modified outside of Perforce's control then an error message
    /// is displayed and the file is not overwritten.  This check adds some
    /// extra processing which will affect the performance of the operation.
    /// Clients with 'allwrite' and 'noclobber' set do this check by default.
    pub fn verify(mut self, verify: bool) -> Self {
        self.verify = verify;
        self
    }

    /// The -m flag limits sync to the first 'max' number of files. This
    /// option is useful in conjunction with tagged output and the '-n'
    /// flag, to preview how many files will be synced without transferring
    /// all the file data.
    pub fn max_files(mut self, max_files: usize) -> Self {
        self.max_files = Some(max_files);
        self
    }

    /// The --parallel flag specifies options for parallel file transfer. If
    /// your administrator has enabled parallel file transfer by setting the
    /// net.parallel.max configurable, and if there are sufficient resources
    /// across the system, a sync command may execute more rapidly by
    /// transferring multiple files in parallel. Specify threads=N to request
    /// files be sent concurrently, using N independent network connections.
    /// The N threads grab work in batches; specify batch=N to control the
    /// number of files in a batch, or batchsize=N to control the number of
    /// bytes in a batch. A sync that is too small will not initiate parallel
    /// file transfers; specify min=N to control the minimum number of files
    /// in a parallel sync, or minsize=N to control the minimum number of
    /// bytes in a parallel sync. Requesting progress indicators causes the
    /// --parallel flag to be ignored.
    ///
    /// Auto parallel sync may be enabled by setting the net.parallel.threads
    /// configurable to the desired number of threads to be used by all sync
    /// commands. This value must be less than or equal to the value of
    /// net.parallel.max. Other net.parallel.* configurables may be specified
    /// as well, but are not required. See 'p4 help configurables' to see
    /// the options and their defaults. Auto parallel sync is turned off by
    /// unsetting the net.parallel.threads configurable. A user may override
    /// the configured auto parallel sync options on the command line, or may
    /// disable it via 'p4 sync --parallel=0'.
    pub fn parallel(mut self, parallel: usize) -> Self {
        self.parallel = Some(parallel);
        self
    }

    /// Run the `sync` command.
    pub fn run(self) -> Result<SyncIter, error::P4Error> {
        let mut cmd = self.connection.connect();
        cmd.arg("sync");
        if self.force {
            cmd.arg("-f");
        }
        if self.preview {
            cmd.arg("-n");
        }
        if self.server_only {
            cmd.arg("-k");
        }
        if self.client_only {
            cmd.arg("-p");
        }
        if self.verify {
            cmd.arg("-s");
        }
        if let Some(max_files) = self.max_files {
            let max_files = format!("{}", max_files);
            cmd.args(&["-m", &max_files]);
        }
        if let Some(parallel) = self.parallel {
            let parallel = format!("{}", parallel);
            cmd.args(&["--parallel", &parallel]);
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
        Ok(SyncIter(items.into_iter()))
    }
}

pub type FileItem = error::Item<File>;

#[derive(Debug)]
pub struct SyncIter(vec::IntoIter<FileItem>);

impl Iterator for SyncIter {
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
pub enum FileContent {
    #[doc(hidden)]
    __Nonexhaustive,

    Text(Vec<String>),
    Binary(Vec<u8>),
}

impl FileContent {
    pub fn as_text(&self) -> Option<&[String]> {
        match self {
            FileContent::Text(c) => Some(&c),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            FileContent::Binary(c) => Some(&c),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub depot_file: String,
    pub client_file: path::PathBuf,
    pub rev: usize,
    pub action: p4::Action,
    pub file_size: usize,
    non_exhaustive: (),
}

mod files_parser {
    use super::*;

    use super::super::parser::*;

    named!(pub file<&[u8], File>,
        do_parse!(
            depot_file: depot_file >>
            client_file: client_file >>
            rev: rev >>
            action: action >>
            file_size: file_size >>
            _ignore: opt!(delimited!(ignore_info1, ignore_info1, change)) >>
            (
                File {
                    depot_file: depot_file.path.to_owned(),
                    client_file: path::PathBuf::from(client_file.path),
                    rev: rev.rev,
                    action: action.action.parse().expect("`Unknown` to capture all"),
                    file_size: file_size.size,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sync_single() {
        let output: &[u8] = br#"info1: depotFile //depot/dir/file
info1: clientFile /home/user/depot/dir/file
info1: rev 1
info1: action added
info1: fileSize 1016
info1: totalFileSize 865153
info1: totalFileCount 24
info1: change 25662947
exit: 0
"#;
        let (_remains, (items, exit)) = files_parser::files(output).unwrap();
        let first = items[0].as_data().unwrap();
        assert_eq!(first.depot_file, "//depot/dir/file");
        assert_eq!(exit.as_error(), Some(&error::OperationError::new(0)));
    }

    #[test]
    fn sync_multi() {
        let output: &[u8] = br#"info1: depotFile //depot/dir/file
info1: clientFile /home/user/depot/dir/file
info1: rev 1
info1: action added
info1: fileSize 1016
info1: totalFileSize 865153
info1: totalFileCount 24
info1: change 25662947
info1: depotFile //depot/dir/file1
info1: clientFile /home/user/depot/dir/file1
info1: rev 1
info1: action added
info1: fileSize 729154
exit: 0
"#;
        let (_remains, (items, exit)) = files_parser::files(output).unwrap();
        let first = items[0].as_data().unwrap();
        let last = items[1].as_data().unwrap();
        assert_eq!(first.depot_file, "//depot/dir/file");
        assert_eq!(last.depot_file, "//depot/dir/file1");
        assert_eq!(exit.as_error(), Some(&error::OperationError::new(0)));
    }
}
