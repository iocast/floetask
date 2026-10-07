+++
title = "floetask"
template = "index.html"

[extra]
tagline = "Your todo.txt, on the desktop."
lead = "floetask is a fast desktop todo.txt manager written in Rust. Your todos stay in one plain text file that you own, and floetask gives them a list, a status board, a calendar, powerful search and notes."

[extra.hero]
file = "board-grouped-dark.png"
caption = "The status board grouped by project, with the sorting drawer open."

[extra.plain]
title = "Just a text file. Readable, portable and yours."
text = "floetask reads and writes the todo.txt format. Every todo is one line in a plain text file, so your list stays portable, scriptable and yours."

[[extra.plain.pillars]]
title = "Portable"
text = "Open the same file with any todo.txt app on any platform, or with a plain text editor. floetask changes only the words you edit, so nothing else in the file moves."

[[extra.plain.pillars]]
title = "Scriptable"
text = "grep it, sed it, script it, keep it in git. A line of text is the easiest data format there is to automate."

[[extra.plain.pillars]]
title = "Yours"
text = "No account, no server, no subscription, no lock-in. If you stop using floetask tomorrow, your list is still right there."

[extra.sync]
title = "Put your list in the cloud you already use"
text = "floetask has no cloud of its own, and it does not need one. Keep todo.txt in any synced folder and the same list is on every computer and phone you own. When another device changes the file, floetask notices and reloads it."
places = ["OneDrive", "Google Drive", "Dropbox", "iCloud Drive", "Nextcloud", "Syncthing", "a git repository", "a network share"]

[[extra.showcases]]
id = "calendar"
icon = "month"
title = "Plan your weeks on a calendar"
text = "See your todos laid out by due date in a day, week or month view. The calendar shows exactly what the list shows, with the same search and filters."
points = [
  "Drag a todo to another day to move its due date; only the due: date changes in the file.",
  "A side panel lists the todos without a due date. Drag one onto a day, or pick a date for it.",
  "Today is highlighted, and the week starts on the day you choose.",
  "Click any todo to edit it.",
]
images = [
  { file = "calendar-undated.png", caption = "The month view with the panel of todos without a due date." },
  { file = "calendar-week.png", caption = "The week view: one column per day." },
  { file = "calendar-day.png", caption = "The day view with full todo cards." },
]

[[extra.showcases]]
id = "board"
icon = "board"
title = "Move work across a status board"
text = "Give open todos a workflow state with the status: extension, then drag them between columns such as To do, Doing and Waiting."
points = [
  "Group the board by project, context or priority, with one board per group.",
  "Choose the columns per file and add your own statuses, such as in-review.",
  "Completed todos land in Done, and dragging a card back reopens it.",
]
images = [
  { file = "board-grouped.png", caption = "One board per priority." },
  { file = "board-columns.png", caption = "Choose the board's columns per file." },
  { file = "list.png", caption = "The same todos as a list, grouped by status." },
]

[[extra.showcases]]
id = "search"
icon = "search"
title = "Find anything, then narrow it down"
text = "Type plain words or a filter expression like +work and due <= today and not @phone. The drawer shows every project, context, priority and date with its count."
points = [
  "Click a value to show only matching todos, or Alt+click to hide them.",
  "Show or hide completed, hidden, someday and future todos with one switch.",
  "Sort by any attribute and group by the first one.",
  "Save your favourite searches and pick them from the search bar.",
]
images = [
  { file = "drawer-attributes.png", caption = "The drawer: every attribute with its count, ready to filter." },
  { file = "drawer-filters.png", caption = "Filter switches for completed, hidden and future todos." },
  { file = "dark.png", caption = "The attribute drawer in the dark theme." },
]

[[extra.features]]
icon = "file"
title = "Plain todo.txt"
text = "Your data is one human-readable text file. floetask changes only the tokens you edit, so the file round-trips untouched and works with every other todo.txt tool."

[[extra.features]]
icon = "cloud"
title = "Synced everywhere"
text = "Keep the file in OneDrive, Google Drive, Dropbox or any synced folder. floetask reloads it when another device changes it."

[[extra.features]]
icon = "month"
title = "Calendar"
text = "Day, week and month views of your due dates. Drag todos between days and plan the ones without a date."

[[extra.features]]
icon = "board"
title = "Status board"
text = "Drag todos between columns such as To do, Doing and Waiting. Group the board into lanes by project or context."

[[extra.features]]
icon = "search"
title = "Search that speaks todo.txt"
text = "Filter with expressions like +work and due <= today and not @phone, use regular expressions, and save your favourite searches."

[[extra.features]]
icon = "calendar"
title = "Dates that think ahead"
text = "Due and threshold dates, recurring todos, and natural input like due:tomorrow or due:next tuesday, converted to real dates on save."

[[extra.features]]
icon = "note"
title = "Notes for every todo"
text = "Link a Markdown notes file to any todo with note:, kept in a folder next to your todo.txt."

[[extra.features]]
icon = "bell"
title = "Notifications"
text = "Get a desktop notification when something is due, and silence the ones that match a saved filter."

[[extra.features]]
icon = "files"
title = "Many files, one window"
text = "Keep several todo.txt files open and archive completed work to done.txt."

[[extra.features]]
icon = "palette"
title = "Light, dark, yours"
text = "Light and dark themes that follow your system, custom colours, zoom, a compact layout and keyboard shortcuts for every action."

[[extra.screenshots]]
file = "list.png"
caption = "The todo list, grouped by status."

[[extra.screenshots]]
file = "calendar-month.png"
caption = "The calendar's month view."

[[extra.screenshots]]
file = "editor.png"
caption = "The editor with autocomplete for projects, contexts and dates."

[[extra.screenshots]]
file = "editor-date-picker.png"
caption = "Pick due and threshold dates, or type due:tomorrow."

[[extra.screenshots]]
file = "editor-note.png"
caption = "Markdown notes, saved next to your todo.txt."

[[extra.screenshots]]
file = "settings.png"
caption = "Settings: theme, language, zoom and compact list."

[[extra.screenshots]]
file = "dark-editor.png"
caption = "The editor in the dark theme."

[[extra.screenshots]]
file = "dark-files.png"
caption = "Several todo.txt files open in one window."

[[extra.downloads]]
os = "Windows"
formats = "Setup .exe or .msi"
note = "Windows 10 and 11. The setup installs per user, no admin prompt."

[[extra.downloads]]
os = "macOS"
formats = ".dmg"
note = "Drag floetask into Applications."

[[extra.downloads]]
os = "Linux"
formats = "AppImage or .deb"
note = "The AppImage runs anywhere and updates itself."
+++
