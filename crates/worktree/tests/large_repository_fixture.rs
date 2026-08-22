use std::path::{Path, PathBuf};

/// Describes a deterministic synthetic repository without materializing its
/// contents. Keeping the description separate from filesystem creation lets
/// benchmarks scale the same workload across real and fake filesystems.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LargeRepositoryFixture {
    pub directories: usize,
    pub files_per_directory: usize,
    pub ignored_directory_stride: Option<usize>,
}

impl LargeRepositoryFixture {
    pub const fn new(directories: usize, files_per_directory: usize) -> Self {
        Self {
            directories,
            files_per_directory,
            ignored_directory_stride: None,
        }
    }

    pub const fn with_ignored_directory_stride(mut self, stride: usize) -> Self {
        assert!(stride > 0, "ignored directory stride must be non-zero");
        self.ignored_directory_stride = Some(stride);
        self
    }

    pub fn file_count(self) -> usize {
        self.directories.saturating_mul(self.files_per_directory)
    }

    pub fn directory_path(self, index: usize) -> PathBuf {
        assert!(index < self.directories);
        PathBuf::from(format!("directory-{index:08}"))
    }

    pub fn file_path(self, directory: usize, file: usize) -> PathBuf {
        assert!(file < self.files_per_directory);
        self.directory_path(directory)
            .join(format!("file-{file:08}.rs"))
    }

    pub fn directory_is_ignored(self, index: usize) -> bool {
        self.ignored_directory_stride
            .is_some_and(|stride| index % stride == 0)
    }

    pub fn paths(self) -> impl Iterator<Item = PathBuf> {
        (0..self.directories).flat_map(move |directory| {
            (0..self.files_per_directory).map(move |file| self.file_path(directory, file))
        })
    }

    pub fn gitignore(self) -> String {
        let Some(stride) = self.ignored_directory_stride else {
            return String::new();
        };

        let mut output = String::new();
        for directory in (0..self.directories).step_by(stride) {
            output.push_str(&format!("/{}/\n", self.directory_path(directory).display()));
        }
        output
    }
}

pub fn path_depth(path: &Path) -> usize {
    path.components().count()
}

#[test]
fn fixture_is_deterministic_and_bounded() {
    let fixture = LargeRepositoryFixture::new(3, 4).with_ignored_directory_stride(2);
    assert_eq!(fixture.file_count(), 12);
    assert_eq!(fixture.paths().count(), 12);
    assert_eq!(fixture.file_path(2, 3), PathBuf::from("directory-00000002/file-00000003.rs"));
    assert!(fixture.directory_is_ignored(0));
    assert!(!fixture.directory_is_ignored(1));
    assert!(fixture.directory_is_ignored(2));
    assert_eq!(path_depth(&fixture.file_path(0, 0)), 2);
    assert_eq!(
        fixture.gitignore(),
        "/directory-00000000/\n/directory-00000002/\n"
    );
}

#[test]
fn fixture_path_iteration_does_not_require_a_path_corpus() {
    let fixture = LargeRepositoryFixture::new(1_000_000, 1);
    let mut paths = fixture.paths();
    assert_eq!(
        paths.next(),
        Some(PathBuf::from("directory-00000000/file-00000000.rs"))
    );
    assert_eq!(
        paths.nth(999_998),
        Some(PathBuf::from("directory-00999999/file-00000000.rs"))
    );
    assert_eq!(paths.next(), None);
}
