use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct LargeRepositoryFixture {
    directories: usize,
    files_per_directory: usize,
    ignored_directory_stride: Option<usize>,
}

impl LargeRepositoryFixture {
    pub fn new(directories: usize, files_per_directory: usize) -> Self {
        Self {
            directories,
            files_per_directory,
            ignored_directory_stride: None,
        }
    }

    pub fn with_ignored_directory_stride(mut self, stride: usize) -> Self {
        self.ignored_directory_stride = Some(stride.max(1));
        self
    }

    pub fn file_count(&self) -> usize {
        self.directories.saturating_mul(self.files_per_directory)
    }

    pub fn file_path(&self, directory: usize, file: usize) -> PathBuf {
        self.directory_path(directory)
            .join(format!("file-{file:08}.rs"))
    }

    pub fn directory_path(&self, directory: usize) -> PathBuf {
        PathBuf::from(format!("directory-{directory:08}"))
    }

    pub fn directory_is_ignored(&self, directory: usize) -> bool {
        self.ignored_directory_stride
            .is_some_and(|stride| directory % stride == 0)
    }

    pub fn paths(&self) -> impl Iterator<Item = PathBuf> + '_ {
        (0..self.directories).flat_map(move |directory| {
            (0..self.files_per_directory).map(move |file| self.file_path(directory, file))
        })
    }

    pub fn gitignore(&self) -> String {
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
    assert_eq!(
        fixture.file_path(2, 3),
        PathBuf::from("directory-00000002/file-00000003.rs")
    );
    assert!(fixture.directory_is_ignored(0));
    assert!(!fixture.directory_is_ignored(1));
    assert!(fixture.directory_is_ignored(2));
    assert_eq!(path_depth(&fixture.file_path(0, 0)), 2);
    assert_eq!(
        fixture.gitignore(),
        "/directory-00000000/\n/directory-00000002/\n"
    );
}
