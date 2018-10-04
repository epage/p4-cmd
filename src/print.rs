use std::vec;

use error;
use p4;

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
#[derive(Debug, Clone)]
pub struct Print<'p, 'f> {
    connection: &'p p4::P4,
    file: Vec<&'f str>,

    all_revs: bool,
    keyword_expansion: bool,
    max_files: Option<usize>,
}

impl<'p, 'f> Print<'p, 'f> {
    pub fn new(connection: &'p p4::P4, file: &'f str) -> Self {
        Self {
            connection: connection,
            file: vec![file],
            all_revs: false,
            keyword_expansion: true,
            max_files: None,
        }
    }

    pub fn file(mut self, dir: &'f str) -> Self {
        self.file.push(dir);
        self
    }

    /// The -a flag prints all revisions within the specified range, rather
    /// than just the highest revision in the range.
    pub fn all_revs(mut self, all_revs: bool) -> Self {
        self.all_revs = all_revs;
        self
    }

    /// The -k flag suppresses keyword expansion.
    pub fn keyword_expansion(mut self, keyword_expansion: bool) -> Self {
        self.keyword_expansion = keyword_expansion;
        self
    }

    /// The -m flag limits print to the first 'max' number of files.
    pub fn max_files(mut self, max_files: usize) -> Self {
        self.max_files = Some(max_files);
        self
    }

    /// Run the `print` command.
    pub fn run(self) -> Result<PrintIter, error::P4Error> {
        let mut cmd = self.connection.connect();
        cmd.arg("print");
        if self.all_revs {
            cmd.arg("-s");
        }
        if !self.keyword_expansion {
            cmd.arg("-k");
        }
        if let Some(max_files) = self.max_files {
            let max_files = format!("{}", max_files);
            cmd.args(&["-m", &max_files]);
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
        Ok(PrintIter(items.into_iter()))
    }
}

pub type FileItem = error::Item<File>;

#[derive(Debug)]
pub struct PrintIter(vec::IntoIter<FileItem>);

impl Iterator for PrintIter {
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
    pub content: FileContent,
    pub depot_file: String,
    pub rev: usize,
    pub change: usize,
    pub action: p4::Action,
    pub file_type: p4::FileType,
    pub time: p4::Time,
    pub file_size: usize,
    non_exhaustive: (),
}

mod files_parser {
    use super::*;

    use super::super::parser::*;

    named!(pub file<&[u8], File>,
        do_parse!(
            depot_file: depot_file >>
            rev: rev >>
            change: change >>
            action: action >>
            file_type: file_type >>
            time: time >>
            file_size: file_size >>
            content: alt!(
                map!(many1!(text), texts_to_content) |
                map!(take!(file_size.size), slice_to_content)
            ) >>
            (
                File {
                    content: content,
                    depot_file: depot_file.path.to_owned(),
                    rev: rev.rev,
                    change: change.change,
                    action: action.action.parse().expect("`Unknown` to capture all"),
                    file_type: file_type.ft.parse().expect("`Unknown` to capture all"),
                    time: p4::from_timestamp(time.time),
                    file_size: file_size.size,
                    non_exhaustive: (),
                }
            )
        )
    );

    named!(item<&[u8], FileItem>,
        alt!(
            map!(file, data_to_item) |
            map!(error, error_to_item)
        )
    );

    named!(pub files<&[u8], (Vec<FileItem>, FileItem)>,
        pair!(
            many0!(item),
            map!(exit, exit_to_item)
        )
    );

    fn texts_to_content(texts: Vec<String>) -> FileContent {
        FileContent::Text(texts)
    }

    fn slice_to_content(s: &[u8]) -> FileContent {
        FileContent::Binary(s.to_vec())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn print_text_single() {
        let output: &[u8] = br#"info1: depotFile //depot/dir/file
info1: rev 3
info1: change 42
info1: action edit
info1: type text
info1: time 1527128624
info1: fileSize 494514
text: Hello
text: World
exit: 0
"#;
        let (_remains, (items, exit)) = files_parser::files(output).unwrap();
        let item = items[0].as_data().unwrap();
        assert_eq!(
            item.content,
            FileContent::Text(vec!["Hello".to_owned(), "World".to_owned()])
        );
        assert_eq!(exit.as_error(), Some(&error::OperationError::new(0)));
    }

    #[test]
    fn print_text_multi() {
        let output: &[u8] = br#"info1: depotFile //depot/dir/file
info1: rev 3
info1: change 42
info1: action edit
info1: type text
info1: time 1527128624
info1: fileSize 494514
text: Hello
text: World
info1: depotFile //depot/dir/file2
info1: rev 3
info1: change 42
info1: action edit
info1: type text
info1: time 1527128624
info1: fileSize 494514
text: Goodbye
text: World
exit: 0
"#;
        let (_remains, (items, exit)) = files_parser::files(output).unwrap();
        let first = items[0].as_data().unwrap();
        let last = items[1].as_data().unwrap();
        assert_eq!(
            first.content,
            FileContent::Text(vec!["Hello".to_owned(), "World".to_owned()])
        );
        assert_eq!(
            last.content,
            FileContent::Text(vec!["Goodbye".to_owned(), "World".to_owned()])
        );
        assert_eq!(exit.as_error(), Some(&error::OperationError::new(0)));
    }

    #[test]
    fn print_binary_single() {
        let output: &[u8] = b"info1: depotFile //depot/dir/file
info1: rev 3
info1: change 42
info1: action edit
info1: type binary
info1: time 1527128624
info1: fileSize 5
1\02\n3exit: 0
";
        let (_remains, (items, exit)) = files_parser::files(output).unwrap();
        assert_eq!(
            items[0].as_data().unwrap().content,
            FileContent::Binary(b"1\02\n3".to_vec())
        );
        assert_eq!(exit.as_error(), Some(&error::OperationError::new(0)));
    }

    #[test]
    fn file_binary() {
        let output: &[u8] = b"info1: depotFile //depot/dir/file
info1: rev 3
info1: change 42
info1: action edit
info1: type binary
info1: time 1527128624
info1: fileSize 5
1\02\n3
";
        let (_remains, item) = files_parser::file(output).unwrap();
        assert_eq!(item.content, FileContent::Binary(b"1\02\n3".to_vec()));
    }
}
