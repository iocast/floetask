//! Sharing todo files: linking a file to a shared copy, joining one, the
//! sync status in the file drawer and the conflict dialog. The syncing
//! itself is `SyncService`'s; this feature decides when to sync and shows
//! how it went. See `features/todo-notes/19-sync.md`.

mod update;
mod view;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use floetask_application::{AppError, ItemConflicts, ShareOutcome, StoreLocation, SyncLink, SyncReport, SyncStatus};
use floetask_domain::merge::Choice;

pub(crate) use view::{conflicts_dialog, join_dialog, share_dialog, status_icon};

#[derive(Debug, Clone)]
pub enum Message {
    /// Opens the share dialog for the registered file at this index.
    Share(usize),
    /// Pick a folder for the shared copy, which gets the file's name with
    /// `-share` added.
    ShareChooseFolder,
    /// Pick an existing file to be the shared copy.
    ShareChooseFile,
    ShareFolder(Option<PathBuf>),
    ShareFile(Option<PathBuf>),
    /// The name of the shared copy typed in the share dialog.
    ShareName(String),
    /// Whether the shared copy in this folder under this name exists.
    ShareExists(PathBuf, String, bool),
    /// The result of sharing a file in a folder under a name.
    Shared(ShareDialog, Result<ShareOutcome, AppError>),
    /// Opens the join wizard.
    Join,
    /// Pick the shared copy to join.
    JoinChooseShared,
    JoinShared(Option<PathBuf>),
    /// Pick where a new local copy goes.
    JoinChooseNew,
    /// Pick an existing file to be the local copy.
    JoinChooseExisting,
    JoinLocal(Option<PathBuf>),
    /// Whether the local copy at this path exists.
    JoinLocalExists(PathBuf, bool),
    /// Back to the join wizard's previous step.
    JoinBack,
    Joined(SyncLink, Result<SyncReport, AppError>),
    /// Share the file at this index again under its old name, after the
    /// shared copy went missing.
    ShareAgain(usize),
    SyncNow(usize),
    /// Time for the periodic sync of every linked file.
    Interval,
    Synced(PathBuf, Result<SyncReport, AppError>),
    StopSharing(usize),
    Unlinked(PathBuf, Result<(), AppError>),
    ShowConflicts(PathBuf),
    ConflictsLoaded(PathBuf, Result<Vec<ItemConflicts>, AppError>),
    /// The choice for one conflict: file (or note) and conflict index.
    Choose(usize, usize, Choice),
    Resolved(PathBuf, Result<SyncReport, AppError>),
}

/// The linked files and how each one stands.
#[derive(Debug, Default)]
pub struct SyncState {
    pub links: Vec<SyncLink>,
    pub files: HashMap<PathBuf, FileSync>,
}

impl SyncState {
    pub fn link(&self, local: &Path) -> Option<&SyncLink> {
        self.links.iter().find(|link| link.local == local)
    }

    pub fn status(&self, local: &Path) -> Option<&FileSync> {
        self.files.get(local)
    }

    /// Adds or replaces the link of its local file.
    fn set_link(&mut self, link: SyncLink) {
        self.links.retain(|existing| existing.local != link.local);
        self.links.push(link);
    }
}

/// One linked file's sync.
#[derive(Debug, Clone, Default)]
pub struct FileSync {
    /// How the last sync ended; `None` before the first one.
    pub status: Option<SyncStatus>,
    pub running: bool,
    /// Another sync was asked for while one ran: run again after it.
    pub again: bool,
}

/// The share dialog: the file to share and where its shared copy goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareDialog {
    pub file: PathBuf,
    /// The folder of the shared copy; `None` until one is chosen.
    pub folder: Option<PathBuf>,
    pub name: String,
    /// The shared copy exists already: sharing joins it.
    pub exists: bool,
}

impl ShareDialog {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            folder: None,
            name: String::new(),
            exists: false,
        }
    }

    /// The shared copy's file name in a chosen folder: the file's name with
    /// `-share` added, `todo.txt` becoming `todo-share.txt`.
    pub fn suggested_name(&self) -> String {
        let stem = self
            .file
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        match self.file.extension() {
            Some(extension) => format!("{stem}-share.{}", extension.to_string_lossy()),
            None => format!("{stem}-share"),
        }
    }

    /// Where the shared copy goes, once a folder or file is chosen.
    pub fn shared_path(&self) -> Option<PathBuf> {
        Some(self.folder.as_ref()?.join(self.name.trim()))
    }

    /// The shared copy is the file itself.
    pub fn is_same_file(&self) -> bool {
        self.shared_path().is_some_and(|path| path == self.file)
    }

    /// A folder is chosen, the name is a plain file name and the shared
    /// copy is not the file itself.
    pub fn is_ready(&self) -> bool {
        let name = self.name.trim();
        self.folder.is_some()
            && !name.is_empty()
            && !name.contains(['/', '\\'])
            && name != "."
            && name != ".."
            && !self.is_same_file()
    }
}

/// The steps of the join wizard, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JoinStep {
    /// Pick the shared copy someone shared.
    #[default]
    Shared,
    /// Choose where your own copy goes.
    Local,
    /// Check both and join.
    Review,
}

impl JoinStep {
    pub const ALL: [JoinStep; 3] = [JoinStep::Shared, JoinStep::Local, JoinStep::Review];

    pub fn next(self) -> Option<JoinStep> {
        match self {
            JoinStep::Shared => Some(JoinStep::Local),
            JoinStep::Local => Some(JoinStep::Review),
            JoinStep::Review => None,
        }
    }

    pub fn previous(self) -> Option<JoinStep> {
        match self {
            JoinStep::Shared => None,
            JoinStep::Local => Some(JoinStep::Shared),
            JoinStep::Review => Some(JoinStep::Local),
        }
    }
}

/// The join wizard: one step at a time, the shared copy first, then the
/// user's own copy, then a summary of what joining does. Two bare file
/// pickers in a row left people guessing which file each one asked for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JoinDialog {
    pub step: JoinStep,
    pub shared: Option<PathBuf>,
    pub local: Option<PathBuf>,
    /// The local copy was suggested, not chosen: picking another shared
    /// copy suggests again.
    pub suggested: bool,
    /// Whether the local copy exists, so joining merges it; `None` until
    /// checked.
    pub local_exists: Option<bool>,
}

impl JoinDialog {
    /// The local copy is the shared copy itself.
    pub fn is_same_file(&self) -> bool {
        self.local.is_some() && self.local == self.shared
    }

    /// The current step has what it needs to go on.
    pub fn can_continue(&self) -> bool {
        match self.step {
            JoinStep::Shared => self.shared.is_some(),
            JoinStep::Local | JoinStep::Review => {
                self.shared.is_some() && self.local.is_some() && !self.is_same_file() && self.local_exists.is_some()
            }
        }
    }

    /// The link joining creates, once both copies are chosen.
    pub fn link(&self) -> Option<SyncLink> {
        let shared = self.shared.as_ref()?;
        let (folder, name) = (shared.parent()?, shared.file_name()?);
        Some(SyncLink {
            local: self.local.clone()?,
            store: StoreLocation::Folder {
                path: folder.to_path_buf(),
            },
            name: name.to_string_lossy().into_owned(),
        })
    }
}

/// The conflict dialog of a linked file.
#[derive(Debug, Clone)]
pub struct ConflictsDialog {
    pub file: PathBuf,
    pub items: Vec<ItemConflicts>,
    /// One choice per conflict of each item, the local version first.
    pub choices: Vec<Vec<Choice>>,
}

impl ConflictsDialog {
    pub fn new(file: PathBuf, items: Vec<ItemConflicts>) -> Self {
        let choices = items
            .iter()
            .map(|item| vec![Choice::Mine; item.conflicts.len()])
            .collect();
        Self { file, items, choices }
    }
}
