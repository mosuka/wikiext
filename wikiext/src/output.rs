use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use bzip2::write::BzEncoder;
use bzip2::Compression;

use crate::error::Error;

/// Number of output files per directory.
const FILES_PER_DIR: u64 = 100;

/// Configuration for the output writer.
///
/// Controls where extracted articles are written, how large each output file
/// can grow before rotation, and whether output is compressed with bzip2.
pub struct OutputConfig {
    /// Output directory path, or `"-"` for stdout.
    pub path: PathBuf,
    /// Maximum bytes per output file. `0` means one article per file.
    pub max_file_size: u64,
    /// Whether to compress output files with bzip2.
    pub compress: bool,
}

/// Internal writer abstraction that handles stdout, plain files, and
/// bzip2-compressed files transparently.
enum Writer {
    /// Writes to standard output.
    Stdout(io::Stdout),
    /// Writes to an uncompressed buffered file.
    File(BufWriter<File>),
    /// Writes to a bzip2-compressed buffered file.
    CompressedFile(BzEncoder<BufWriter<File>>),
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Writer::Stdout(w) => w.write(buf),
            Writer::File(w) => w.write(buf),
            Writer::CompressedFile(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Writer::Stdout(w) => w.flush(),
            Writer::File(w) => w.flush(),
            Writer::CompressedFile(w) => w.flush(),
        }
    }
}

/// Manages splitting output across multiple files following the wikiextractor
/// directory/file naming convention.
///
/// Files are organized into directories named with two uppercase letters
/// (AA, AB, ..., AZ, BA, ..., ZZ) with up to 100 files per directory
/// (`wiki_00` through `wiki_99`). When a file exceeds the configured size
/// limit, it is closed and a new file is opened.
pub struct OutputSplitter {
    /// The output configuration.
    config: OutputConfig,
    /// The current writer, if open.
    writer: Option<Writer>,
    /// Number of bytes written to the current file.
    current_bytes: u64,
    /// Global file index (incremented across all directories).
    file_index: u64,
}

impl OutputSplitter {
    /// Creates a new `OutputSplitter` with the given configuration.
    ///
    /// If the path is `"-"`, output is written to stdout without file
    /// splitting. Otherwise, the first output file is opened immediately.
    ///
    /// # Arguments
    ///
    /// * `config` - The output configuration specifying path, file size limit,
    ///   and compression settings.
    ///
    /// # Returns
    ///
    /// A new `OutputSplitter` ready to accept writes, or an `Error` if the
    /// initial output file cannot be created.
    pub fn new(config: OutputConfig) -> Result<Self, Error> {
        let mut splitter = OutputSplitter {
            config,
            writer: None,
            current_bytes: 0,
            file_index: 0,
        };

        if splitter.is_stdout() {
            splitter.writer = Some(Writer::Stdout(io::stdout()));
        } else {
            splitter.open_next_file()?;
        }

        Ok(splitter)
    }

    /// Writes formatted page data to the current output.
    ///
    /// For file-based output, this method automatically handles rotation when
    /// the current file exceeds the configured `max_file_size`, or after every
    /// article when `max_file_size` is `0`.
    ///
    /// For stdout mode, data is written directly without splitting.
    ///
    /// # Arguments
    ///
    /// * `data` - The article text to write.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an `Error` if writing or file rotation fails.
    pub fn write(&mut self, data: &str) -> Result<(), Error> {
        if self.is_stdout() {
            if let Some(ref mut writer) = self.writer {
                writer.write_all(data.as_bytes())?;
            }
            return Ok(());
        }

        // Check if rotation is needed before writing.
        let needs_rotation = if self.config.max_file_size == 0 {
            // For zero max_file_size, rotate if we already wrote something.
            self.current_bytes > 0
        } else {
            self.current_bytes + data.len() as u64 > self.config.max_file_size
        };

        if needs_rotation {
            self.close()?;
            self.open_next_file()?;
        }

        if let Some(ref mut writer) = self.writer {
            writer.write_all(data.as_bytes())?;
            self.current_bytes += data.len() as u64;
        }

        Ok(())
    }

    /// Closes the current output file, flushing buffers and finishing
    /// compression if applicable.
    ///
    /// For stdout mode this is a no-op. For file-based output, the current
    /// writer is dropped after flushing.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an `Error` if flushing or finishing compression
    /// fails.
    pub fn close(&mut self) -> Result<(), Error> {
        if self.is_stdout() {
            return Ok(());
        }

        if let Some(writer) = self.writer.take() {
            match writer {
                Writer::CompressedFile(encoder) => {
                    encoder.finish()?;
                }
                Writer::File(mut buf_writer) => {
                    buf_writer.flush()?;
                }
                Writer::Stdout(_) => {}
            }
        }

        self.current_bytes = 0;
        Ok(())
    }

    /// Returns `true` if output is directed to stdout.
    fn is_stdout(&self) -> bool {
        self.config.path.as_os_str() == "-"
    }

    /// Computes the directory name for the given directory index.
    ///
    /// Directory names are two uppercase letters: the first is derived from
    /// `dir_index / 26` and the second from `dir_index % 26`, both mapped
    /// to `A`-`Z`.
    ///
    /// # Arguments
    ///
    /// * `dir_index` - The zero-based directory index.
    ///
    /// # Returns
    ///
    /// A two-character directory name (e.g., `"AA"`, `"AB"`, `"BA"`).
    fn dir_name(dir_index: u64) -> String {
        let first = (b'A' + ((dir_index / 26) % 26) as u8) as char;
        let second = (b'A' + (dir_index % 26) as u8) as char;
        format!("{}{}", first, second)
    }

    /// Computes the file name for the given file-within-directory index.
    ///
    /// File names follow the pattern `wiki_XX` where `XX` is zero-padded.
    /// If compression is enabled, `.bz2` is appended.
    ///
    /// # Arguments
    ///
    /// * `file_in_dir` - The zero-based file index within the directory (0-99).
    /// * `compress` - Whether to append the `.bz2` extension.
    ///
    /// # Returns
    ///
    /// The file name string (e.g., `"wiki_00"`, `"wiki_07.bz2"`).
    fn file_name(file_in_dir: u64, compress: bool) -> String {
        if compress {
            format!("wiki_{:02}.bz2", file_in_dir)
        } else {
            format!("wiki_{:02}", file_in_dir)
        }
    }

    /// Opens the next output file based on the current `file_index`.
    ///
    /// Creates the necessary directory structure and opens a new writer
    /// (compressed or plain) for writing.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an `Error` if directory creation or file
    /// opening fails.
    fn open_next_file(&mut self) -> Result<(), Error> {
        let dir_index = self.file_index / FILES_PER_DIR;
        let file_in_dir = self.file_index % FILES_PER_DIR;

        let dir_name = Self::dir_name(dir_index);
        let file_name = Self::file_name(file_in_dir, self.config.compress);

        let dir_path = self.config.path.join(&dir_name);
        fs::create_dir_all(&dir_path)?;

        let file_path = dir_path.join(&file_name);
        let file = File::create(&file_path)?;
        let buf_writer = BufWriter::new(file);

        self.writer = if self.config.compress {
            Some(Writer::CompressedFile(BzEncoder::new(
                buf_writer,
                Compression::default(),
            )))
        } else {
            Some(Writer::File(buf_writer))
        };

        self.current_bytes = 0;
        self.file_index += 1;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::path::Path;

    /// Helper to create a temporary directory for tests.
    fn test_dir(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("wikiext_test_{}", name));
        if dir.exists() {
            fs::remove_dir_all(&dir).unwrap();
        }
        dir
    }

    /// Helper to clean up a test directory.
    fn cleanup(dir: &Path) {
        if dir.exists() {
            fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn test_next_file_naming() {
        let dir = test_dir("file_naming");
        let config = OutputConfig {
            path: dir.clone(),
            max_file_size: 1024,
            compress: false,
        };

        let mut splitter = OutputSplitter::new(config).unwrap();

        // First file should be AA/wiki_00
        assert!(dir.join("AA").join("wiki_00").exists());

        // Write enough to trigger rotation to AA/wiki_01
        splitter.close().unwrap();
        splitter.open_next_file().unwrap();
        assert!(dir.join("AA").join("wiki_01").exists());

        cleanup(&dir);
    }

    #[test]
    fn test_dir_naming_sequence() {
        // Verify directory naming: AA, AB, ..., AZ, BA, ...
        assert_eq!(OutputSplitter::dir_name(0), "AA");
        assert_eq!(OutputSplitter::dir_name(1), "AB");
        assert_eq!(OutputSplitter::dir_name(25), "AZ");
        assert_eq!(OutputSplitter::dir_name(26), "BA");
        assert_eq!(OutputSplitter::dir_name(27), "BB");
        assert_eq!(OutputSplitter::dir_name(51), "BZ");
        assert_eq!(OutputSplitter::dir_name(52), "CA");
    }

    #[test]
    fn test_file_rotation_by_size() {
        let dir = test_dir("rotation_size");
        let config = OutputConfig {
            path: dir.clone(),
            max_file_size: 50,
            compress: false,
        };

        let mut splitter = OutputSplitter::new(config).unwrap();

        // Write data that fits in one file.
        let small_data = "Hello, World!\n";
        splitter.write(small_data).unwrap();
        assert!(dir.join("AA").join("wiki_00").exists());

        // Write data that triggers rotation (total would exceed 50 bytes).
        let large_data = "A".repeat(50);
        splitter.write(&large_data).unwrap();

        // The second write should have gone to a new file.
        assert!(dir.join("AA").join("wiki_01").exists());

        splitter.close().unwrap();
        cleanup(&dir);
    }

    #[test]
    fn test_zero_size_one_per_file() {
        let dir = test_dir("zero_size");
        let config = OutputConfig {
            path: dir.clone(),
            max_file_size: 0,
            compress: false,
        };

        let mut splitter = OutputSplitter::new(config).unwrap();

        // Each write should go to a separate file.
        splitter.write("Article 1\n").unwrap();
        splitter.write("Article 2\n").unwrap();
        splitter.write("Article 3\n").unwrap();
        splitter.close().unwrap();

        assert!(dir.join("AA").join("wiki_00").exists());
        assert!(dir.join("AA").join("wiki_01").exists());
        assert!(dir.join("AA").join("wiki_02").exists());

        // Verify content of each file.
        let content0 = fs::read_to_string(dir.join("AA").join("wiki_00")).unwrap();
        let content1 = fs::read_to_string(dir.join("AA").join("wiki_01")).unwrap();
        let content2 = fs::read_to_string(dir.join("AA").join("wiki_02")).unwrap();

        assert_eq!(content0, "Article 1\n");
        assert_eq!(content1, "Article 2\n");
        assert_eq!(content2, "Article 3\n");

        cleanup(&dir);
    }

    #[test]
    fn test_stdout_mode() {
        let config = OutputConfig {
            path: PathBuf::from("-"),
            max_file_size: 1024,
            compress: false,
        };

        let splitter = OutputSplitter::new(config).unwrap();

        // In stdout mode, no directories should be created.
        assert!(splitter.is_stdout());

        // close() should be a no-op for stdout.
        // (We don't actually write to stdout in tests to avoid noise.)
    }
}
