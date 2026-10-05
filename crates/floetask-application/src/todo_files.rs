//! Use cases that read and change todo files.
//!
//! Every change reads the file fresh, checks that the targeted line still
//! holds what the user saw, applies the domain rule and writes the result.
//! The returned document is what the GUI shows next.

use std::path::Path;
use std::sync::Arc;

use floetask_domain::board::Column;
use floetask_domain::listing::Attribute;
use floetask_domain::natural_date::rewrite_relative_dates;
use floetask_domain::todo::Todo;
use floetask_domain::{Date, TodoDocument, WeekStart};

use crate::error::AppError;
use crate::notes::NoteTransfer;
use crate::ports::{Clock, FileSystem};
use crate::settings::Settings;

/// How files are read and written.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileOptions {
    pub exclude_prefixes: Vec<String>,
    pub safe_writes: bool,
}

impl From<&Settings> for FileOptions {
    fn from(settings: &Settings) -> Self {
        Self {
            exclude_prefixes: settings.exclude_lines_with_prefix.clone(),
            safe_writes: settings.safe_writes,
        }
    }
}

/// How typed text becomes todos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputOptions {
    pub append_creation_date: bool,
    pub convert_relative_dates: bool,
    pub week_start: WeekStart,
    /// Each line becomes its own todo instead of one multi-line todo.
    pub bulk: bool,
}

impl From<&Settings> for InputOptions {
    fn from(settings: &Settings) -> Self {
        Self {
            append_creation_date: settings.append_creation_date,
            convert_relative_dates: settings.convert_relative_dates,
            week_start: settings.week_start,
            bulk: settings.bulk_creation,
        }
    }
}

/// Identifies the todo the user acted on: its line and the text they saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoRef {
    pub line: usize,
    pub raw: String,
}

impl TodoRef {
    pub fn new(line: usize, todo: &Todo) -> Self {
        Self {
            line,
            raw: todo.raw().to_owned(),
        }
    }
}

#[derive(Clone)]
pub struct TodoFileService {
    files: Arc<dyn FileSystem>,
    clock: Arc<dyn Clock>,
}

impl TodoFileService {
    pub fn new(files: Arc<dyn FileSystem>, clock: Arc<dyn Clock>) -> Self {
        Self { files, clock }
    }

    pub fn today(&self) -> Date {
        self.clock.today()
    }

    pub fn load(&self, path: &Path, options: &FileOptions) -> Result<TodoDocument, AppError> {
        let content = self.files.read(path)?;
        Ok(TodoDocument::parse(&content, &options.exclude_prefixes))
    }

    /// Loads a file, creating it empty when it does not exist.
    pub fn open_or_create(&self, path: &Path, options: &FileOptions) -> Result<TodoDocument, AppError> {
        if !self.files.exists(path) {
            self.files.write(path, "", false)?;
        }
        self.load(path, options)
    }

    /// Turns typed text into todos without writing anything. Used for the
    /// "Add (3)" count as well as for saving.
    pub fn prepare_new_todos(&self, text: &str, input: &InputOptions) -> Vec<Todo> {
        let today = self.today();
        let text = self.convert_dates(text, input);
        let pieces: Vec<String> = if input.bulk {
            text.lines().map(str::to_owned).collect()
        } else {
            vec![text]
        };
        pieces
            .iter()
            .map(|piece| Todo::from_user_text(piece))
            .filter(|todo| !todo.is_empty())
            .map(|todo| {
                if input.append_creation_date && todo.created().is_none() && !todo.is_complete() {
                    todo.with_created(Some(today))
                } else {
                    todo
                }
            })
            .collect()
    }

    pub fn add(
        &self,
        path: &Path,
        text: &str,
        input: &InputOptions,
        options: &FileOptions,
    ) -> Result<TodoDocument, AppError> {
        let todos = self.prepare_new_todos(text, input);
        if todos.is_empty() {
            return Err(AppError::EmptyTodo);
        }
        self.modify(path, options, |document| {
            document.append(todos);
            Ok(())
        })
        .map(|(document, ())| document)
    }

    /// Replaces a todo with the edited text. Line breaks stay inside the one
    /// todo; bulk splitting only applies to new todos.
    pub fn update(
        &self,
        path: &Path,
        target: &TodoRef,
        text: &str,
        input: &InputOptions,
        options: &FileOptions,
    ) -> Result<TodoDocument, AppError> {
        let todo = Todo::from_user_text(&self.convert_dates(text, input));
        if todo.is_empty() {
            return Err(AppError::EmptyTodo);
        }
        self.replace(path, target, options, |_| vec![todo])
    }

    /// Completes or un-completes a todo. Completing a recurring todo appends
    /// its next occurrence right after it.
    pub fn toggle_complete(
        &self,
        path: &Path,
        target: &TodoRef,
        options: &FileOptions,
    ) -> Result<TodoDocument, AppError> {
        let today = self.today();
        self.replace(path, target, options, |todo| {
            let completion = todo.toggle_complete(today);
            std::iter::once(completion.completed).chain(completion.next).collect()
        })
    }

    /// Moves a todo to a board column: sets its status, or completes it for
    /// the `done` column (a recurring todo adds its next occurrence).
    pub fn move_to_column(
        &self,
        path: &Path,
        target: &TodoRef,
        column: &Column,
        options: &FileOptions,
    ) -> Result<TodoDocument, AppError> {
        let today = self.today();
        self.replace(path, target, options, |todo| todo.moved_to(column, today))
    }

    /// Sets or clears a date extension (`due`, `t`) from the inline picker.
    pub fn set_date(
        &self,
        path: &Path,
        target: &TodoRef,
        key: &str,
        date: Option<Date>,
        options: &FileOptions,
    ) -> Result<TodoDocument, AppError> {
        self.replace(path, target, options, |todo| vec![todo.with_date_extension(key, date)])
    }

    pub fn delete(&self, path: &Path, target: &TodoRef, options: &FileOptions) -> Result<TodoDocument, AppError> {
        self.modify(path, options, |document| {
            document.remove(target.line, &target.raw)?;
            Ok(())
        })
        .map(|(document, ())| document)
    }

    /// Renames a project or context in every todo of the file. Returns how
    /// many todos changed.
    pub fn rename_attribute(
        &self,
        path: &Path,
        attribute: Attribute,
        from: &str,
        to: &str,
        options: &FileOptions,
    ) -> Result<(TodoDocument, usize), AppError> {
        let to = to.trim();
        if to.is_empty() || to.contains(char::is_whitespace) {
            return Err(AppError::Other(format!("`{to}` is not a valid name")));
        }
        self.modify(path, options, |document| {
            Ok(match attribute {
                Attribute::Projects => document.map_todos(|todo| todo.with_project_renamed(from, to)),
                Attribute::Contexts => document.map_todos(|todo| todo.with_context_renamed(from, to)),
                _ => 0,
            })
        })
    }

    /// Removes a project or context from every todo of the file.
    pub fn remove_attribute(
        &self,
        path: &Path,
        attribute: Attribute,
        value: &str,
        options: &FileOptions,
    ) -> Result<(TodoDocument, usize), AppError> {
        self.modify(path, options, |document| {
            Ok(match attribute {
                Attribute::Projects => document.map_todos(|todo| todo.without_project(value)),
                Attribute::Contexts => document.map_todos(|todo| todo.without_context(value)),
                _ => 0,
            })
        })
    }

    /// Moves every completed todo to the done file. The done file is written
    /// first, so a crash can at worst duplicate todos, never lose them.
    pub fn archive_completed(
        &self,
        path: &Path,
        done_path: &Path,
        options: &FileOptions,
    ) -> Result<(TodoDocument, usize), AppError> {
        let mut document = self.load(path, options)?;
        let completed = document.take_completed();
        let count = completed.len();
        if count > 0 {
            self.move_to_done(path, done_path, completed, &document, options)?;
        }
        Ok((document, count))
    }

    /// Moves a single todo, completed or not, to the done file.
    pub fn archive_one(
        &self,
        path: &Path,
        done_path: &Path,
        target: &TodoRef,
        options: &FileOptions,
    ) -> Result<TodoDocument, AppError> {
        let mut document = self.load(path, options)?;
        let todo = document.remove(target.line, &target.raw)?;
        self.move_to_done(path, done_path, vec![todo], &document, options)?;
        Ok(document)
    }

    /// Moves todos to the done file, their notes into the done file's notes
    /// folder, and saves what stays. Notes are copied before either file is
    /// written and the originals removed last, so a crash can at worst leave
    /// a note in both folders.
    fn move_to_done(
        &self,
        path: &Path,
        done_path: &Path,
        todos: Vec<Todo>,
        staying: &TodoDocument,
        options: &FileOptions,
    ) -> Result<(), AppError> {
        let (todos, notes) = NoteTransfer::plan(self.files.as_ref(), path, done_path, todos, staying);
        notes.copy(self.files.as_ref())?;
        self.append_to_done(done_path, todos, options)?;
        self.files.write(path, &staying.to_content(), options.safe_writes)?;
        notes.remove_originals(self.files.as_ref());
        Ok(())
    }

    fn append_to_done(&self, done_path: &Path, todos: Vec<Todo>, options: &FileOptions) -> Result<(), AppError> {
        let mut done = if self.files.exists(done_path) {
            self.load(done_path, options)?
        } else {
            TodoDocument::parse("", &options.exclude_prefixes)
        };
        done.append(todos);
        self.files.write(done_path, &done.to_content(), options.safe_writes)
    }

    fn convert_dates(&self, text: &str, input: &InputOptions) -> String {
        if input.convert_relative_dates {
            rewrite_relative_dates(text, self.today(), input.week_start)
        } else {
            text.to_owned()
        }
    }

    fn replace(
        &self,
        path: &Path,
        target: &TodoRef,
        options: &FileOptions,
        change: impl FnOnce(&Todo) -> Vec<Todo>,
    ) -> Result<TodoDocument, AppError> {
        self.modify(path, options, |document| {
            let current = document
                .todo_at(target.line)
                .ok_or(floetask_domain::DocumentError::MissingLine { line: target.line })?
                .clone();
            document.replace(target.line, &target.raw, change(&current))?;
            Ok(())
        })
        .map(|(document, ())| document)
    }

    fn modify<T>(
        &self,
        path: &Path,
        options: &FileOptions,
        change: impl FnOnce(&mut TodoDocument) -> Result<T, AppError>,
    ) -> Result<(TodoDocument, T), AppError> {
        let mut document = self.load(path, options)?;
        let before = document.to_content();
        let result = change(&mut document)?;
        let after = document.to_content();
        if after != before {
            self.files.write(path, &after, options.safe_writes)?;
        }
        Ok((document, result))
    }
}

#[cfg(test)]
mod tests;
