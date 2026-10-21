# floetask: Deutsch.
#
# Übersetzt aus locales/en/floetask.ftl. Glossar: todo = Aufgabe,
# project = Projekt, context = Kontext, threshold = Startdatum,
# status = Status, board = Board, done file = Erledigt-Datei.

## Navigation and files

add_todo = Aufgabe hinzufügen
toggle_drawer = Filter und Sortierung
archive = Erledigte Aufgaben archivieren
open_file = Aufgabendatei öffnen
create_file = Aufgabendatei erstellen
settings = Einstellungen
search = Suchen
no_file_title = Keine Aufgabendatei geöffnet
no_file_body = Öffne eine vorhandene todo.txt oder erstelle eine neue. Du kannst eine Datei auch auf das Fenster ziehen.
no_todos_title = Diese Datei enthält noch keine Aufgaben
no_todos_body = Füge deine erste Aufgabe mit Strg+N hinzu.
no_visible_title = Keine Aufgabe passt zu den aktuellen Filtern
reset_filters = Filter zurücksetzen
file_dialog_title = todo.txt-Dateien
reveal_todo_file = Im Dateimanager zeigen
remove_file = Aus der Liste entfernen
remove_file_confirm = { $file } aus der Liste entfernen? Die Datei selbst wird nicht gelöscht.
file_load_failed = { $file } konnte nicht geöffnet werden
done_file_needed = Wähle oder erstelle zuerst eine Erledigt-Datei zum Archivieren.
archived =
    { $count ->
        [one] 1 Aufgabe archiviert
       *[other] { $count } Aufgaben archiviert
    }
archived_one = Aufgabe archiviert
new = Neu
toggle_files = Dateien zeigen oder ausblenden
files = Dateien
file_actions = Dateiaktionen
set_archive_file = Archivdatei festlegen
open_archive_file = Archivdatei öffnen
close_file = Schließen
sort_files = Dateien sortieren
file_sort_custom = Eigene Reihenfolge
file_sort_custom_hint = Dateien zum Ordnen ziehen
file_sort_name = Name
file_sort_folder = Ordner
overview = Übersicht
overview_hint = Alle Dateien, nach Datei gruppiert
toggle_theme = Hell / dunkel umschalten

## Counts

counts =
    { $visible } von { $total ->
        [one] { $total } Aufgabe
       *[other] { $total } Aufgaben
    }, { $completed } erledigt
groups = Gruppen

## List

created_on = Erstellt { $date }
completed_on = Erledigt { $date }
no_value = Ohne { $attribute }
copy = Kopieren
copied = In die Zwischenablage kopiert
delete = Löschen
delete_confirm = Diese Aufgabe löschen?
edit = Bearbeiten
open_link = Link öffnen
archive_one = In die Erledigt-Datei verschieben
due_label = fällig
threshold_label = ab
details = Details

## Editor

new_todo = Neue Aufgabe
edit_todo = Aufgabe bearbeiten
editor_file = In { $file }
editor_add_to = Hinzufügen zu
editor_placeholder = (A) Mama anrufen +familie @telefon due:tomorrow
editor_hint = Strg+Enter zum Speichern, Esc zum Schließen
add = Hinzufügen
add_count = Hinzufügen ({ $count })
save = Speichern
cancel = Abbrechen
close = Schließen
confirm = Bestätigen
empty_todo = Gib zuerst etwas ein
priority = Priorität
due = Fällig
t = Startdatum
rec = Wiederholen
pm = Pomodoros
status = Status
status_todo = Offen
status_doing = In Arbeit
status_waiting = Wartet
status_someday = Irgendwann
status_done = Erledigt
set = Setzen
clear = Leeren
today = Heute
strict = Streng
note = Notiz
notes = Notizen
previous_month = Vorheriger Monat
next_month = Nächster Monat

## Notes

notes_placeholder = Notizen in Markdown, gespeichert in einer Datei neben deiner Aufgabendatei
note_saved_to = Gespeichert in { $path }
open_note = Notiz öffnen
link_note_file = Eine Markdown-Datei verknüpfen, statt eine neue anzulegen
link_note_dialog_title = Markdown-Datei verknüpfen
markdown_files = Markdown und Text
note_outside_folder = { $file } liegt nicht in { $folder }, wo diese Liste ihre Notizen ablegt.
note_move_into = Nach { $folder } verschieben
note_keep_in_place = Dort lassen, wo sie ist
unlink_note = Notiz lösen; die Datei bleibt erhalten
open_note_window = In eigenem Fenster öffnen
note_in_window = Diese Notiz ist in einem eigenen Fenster geöffnet.
show_note_window = Fenster zeigen
note_saved = Gespeichert
note_unsaved = Ungespeicherte Änderungen
open_in_default_app = Mit der Standard-App öffnen
note_missing = { $note } existiert noch nicht; öffne sie, um sie anzulegen
note_invalid = { $note } ist kein gültiger Notizname
note_orphaned = Keine Aufgabe verweist mehr auf { $note }; die Datei wurde behalten

## Status board

show_board = Als Board zeigen (Strg+Alt+B)
show_list = Als Liste zeigen (Strg+Alt+B)
board_columns = Spalten
board_group = Gruppieren
board_group_by = Nach { $attribute } gruppieren
board_group_hint = Ein Board pro Gruppe. Aus behält die Sortierung auf einem einzigen Board.
board_other = Andere Status
board_columns_title = Board-Spalten für { $file }
board_columns_hint = Jede Spalte ist ein Status; Erledigt enthält abgeschlossene Aufgaben. Zieh eine Karte in eine andere Spalte, um ihren Status zu ändern.
status_placeholder = Name, z. B. In Prüfung
remove_column = Spalte entfernen
reset_columns = Zurücksetzen

## Calendar view

show_calendar = Kalender (Strg+Alt+C)
calendar_day = Tag
calendar_week = Woche
calendar_month = Monat
calendar_today = Heute
calendar_previous = Zurück
calendar_next = Weiter
calendar_more = +{ $count } weitere
calendar_undated = { $count } ohne Fälligkeitsdatum
calendar_undated_title = Ohne Fälligkeitsdatum
calendar_undated_hint = Zieh eine Aufgabe auf einen Tag oder setze ihr Datum hier.
calendar_set_due = Fälligkeitsdatum setzen
calendar_empty_day = An diesem Tag ist nichts fällig.

## Dates

date_month_year = { $month } { $year }
date_long = { $weekday }, { $day }. { $month } { $year }
date_range_in_month = { $first_day }.–{ $last_day }. { $month } { $year }
date_range_in_year = { $first_day }. { $first_month } – { $last_day }. { $last_month } { $year }
date_range = { $first_day }. { $first_month } { $first_year } – { $last_day }. { $last_month } { $last_year }

month_1 = Januar
month_2 = Februar
month_3 = März
month_4 = April
month_5 = Mai
month_6 = Juni
month_7 = Juli
month_8 = August
month_9 = September
month_10 = Oktober
month_11 = November
month_12 = Dezember

month_short_1 = Jan.
month_short_2 = Feb.
month_short_3 = März
month_short_4 = Apr.
month_short_5 = Mai
month_short_6 = Juni
month_short_7 = Juli
month_short_8 = Aug.
month_short_9 = Sept.
month_short_10 = Okt.
month_short_11 = Nov.
month_short_12 = Dez.

weekday_mon = Montag
weekday_tue = Dienstag
weekday_wed = Mittwoch
weekday_thu = Donnerstag
weekday_fri = Freitag
weekday_sat = Samstag
weekday_sun = Sonntag

weekday_abbr_mon = Mo.
weekday_abbr_tue = Di.
weekday_abbr_wed = Mi.
weekday_abbr_thu = Do.
weekday_abbr_fri = Fr.
weekday_abbr_sat = Sa.
weekday_abbr_sun = So.

mon = Mo
tue = Di
wed = Mi
thu = Do
fri = Fr
sat = Sa
sun = So

## Human-friendly dates

bucket_overdue = überfällig
bucket_elapsed = vergangen
bucket_last_week = letzte Woche
bucket_today = heute
bucket_tomorrow = morgen
bucket_this_week = diese Woche
bucket_next_week = nächste Woche
bucket_this_month = diesen Monat
bucket_next_month = nächsten Monat

## Recurrence units

unit_d = Tage
unit_b = Werktage
unit_w = Wochen
unit_m = Monate
unit_y = Jahre

## Attributes and the drawer

projects = Projekte
contexts = Kontexte
created = Erstellt
completed = Erledigt
attributes = Attribute
filters = Filter
sorting = Sortierung
hide_category = Aufgaben mit einem Wert ausblenden
show_category = Aufgaben mit diesem Attribut wieder zeigen
rename = Umbenennen
rename_title = { $value } in dieser Datei überall umbenennen
rename_title_all = { $value } in allen Dateien überall umbenennen
remove_value = Entfernen
remove_value_confirm = { $value } aus jeder Aufgabe dieser Datei entfernen?
remove_value_confirm_all = { $value } aus jeder Aufgabe aller Dateien entfernen?
renamed =
    { $count ->
        [one] 1 Aufgabe aktualisiert
       *[other] { $count } Aufgaben aktualisiert
    }
include_hint = Klicken zum Einschließen, Alt+Klick zum Ausschließen. Rechtsklick auf ein Projekt oder einen Kontext zum Umbenennen oder Entfernen.
show_completed = Erledigte Aufgaben zeigen
show_hidden = Versteckte Aufgaben zeigen (h:1)
show_hidden_attributes = Attribute versteckter Aufgaben zeigen
show_hidden_statuses = Aufgaben mit Status { $statuses } zeigen
show_future_threshold = Aufgaben mit künftigem Startdatum zeigen
show_future_due = Aufgaben mit künftigem Fälligkeitsdatum zeigen
file_order = Dateireihenfolge (ohne Gruppierung)
completed_last = Erledigte Aufgaben zuletzt
move_up = Nach oben
move_down = Nach unten
invert = Umkehren

## Search

search_placeholder = Suchen oder filtern: +projekt @kontext due: < today
saved_filters = Gespeicherte Filter
save_filter = Aktuelle Suche speichern
filter_name = Name
no_saved_filters = Noch keine gespeicherten Filter
delete_filter_confirm = Gespeicherten Filter { $name } löschen?
suppress_notifications = Benachrichtigungen für passende Aufgaben stummschalten

## Settings

settings_title = Einstellungen
section_todos = Aufgaben
section_dates = Datum
section_statuses = Status
section_appearance = Darstellung
section_notifications = Benachrichtigungen
section_files = Dateien
section_about = Über
append_creation_date = Erstellungsdatum zu neuen Aufgaben hinzufügen
append_creation_date_desc = Neue Aufgaben beginnen mit dem heutigen Datum, wie todo.txt es erlaubt.
convert_relative_dates = Relative Daten (due:tomorrow) in Daten umwandeln
convert_relative_dates_desc = Wörter wie tomorrow oder next friday in due: und t: werden als echte Daten gespeichert.
human_friendly_dates = Daten als heute, nächste Woche, … zeigen
human_friendly_dates_desc = Daten lesen sich als heute, morgen oder nächste Woche statt 2026-10-05.
safe_writes = Sicheres Schreiben (temporäre Datei und Sicherung)
safe_writes_desc = Schreibt zuerst in eine temporäre Datei und behält eine Sicherung, damit ein Absturz deine Aufgabendatei nicht beschädigen kann.
bulk_creation = Eine Aufgabe pro Zeile erstellen
bulk_creation_desc = Im Dialog zum Hinzufügen wird jede Zeile zu einer eigenen Aufgabe.
disable_animations = Animationen ausschalten
disable_animations_desc = Schaltet Bewegung aus; floetask hat noch keine Animationen.
compact = Kompakter Modus
compact_desc = Engere Zeilen und Karten, damit mehr Aufgaben auf den Bildschirm passen.
month_overflow = Volle Monatszellen
month_overflow_desc = Wenn eine Monatszelle im Kalender mehr Aufgaben hat, als hineinpassen: +N weitere zeigen oder scrollen.
month_overflow_more = +N weitere zeigen
month_overflow_scroll = Scrollen
week_start = Woche beginnt am
week_start_desc = Erster Tag der Woche in Kalendern und für diese und nächste Woche.
monday = Montag
saturday = Samstag
sunday = Sonntag
theme = Design
theme_desc = Hell, dunkel oder wie im System.
theme_system = System
theme_light = Hell
theme_dark = Dunkel
language = Sprache
language_desc = Sprache der Oberfläche. System folgt deinem Betriebssystem.
language_system = System
zoom = Zoom { $percent } %
zoom_desc = Macht alles im Fenster größer oder kleiner.
notifications = Benachrichtigungen bei Fälligkeit
notifications_desc = Eine Systembenachrichtigung, wenn eine Aufgabe fällig wird.
notification_threshold =
    { $days ->
        [0] Am Fälligkeitstag benachrichtigen
        [one] 1 Tag vorher benachrichtigen
       *[other] { $days } Tage vorher benachrichtigen
    }
notification_threshold_desc = Wie viele Tage vor dem Fälligkeitsdatum du benachrichtigt wirst.
config_file = Einstellungsdatei
config_file_desc = Wo diese Einstellungen gespeichert werden. Board-Spalten und Status stehen auch dort.
config_location = Einstellungsdatei: { $path }
settings_save_failed = Einstellungen konnten nicht gespeichert werden
statuses_desc = Status für den status:-Tag, in der Reihenfolge, in der die Liste sie sortiert. Versteckte Status bleiben aus der Liste, bis du sie anforderst. Eine Board-Spalte kann einen Status nur für eine Datei hinzufügen.
status_todo_desc = Nicht begonnen. Dasselbe wie kein status:-Tag.
status_doing_desc = Wird gerade bearbeitet.
status_waiting_desc = Blockiert durch jemand oder etwas anderes.
status_someday_desc = Geparkt, nicht fest zugesagt.
status_custom_desc = Dein eigener Status.
status_hidden = Versteckt
remove_status = Status entfernen
add_status = Status hinzufügen

## About and updates

about_tagline = Ein todo.txt-Manager für den Desktop.
about_version = Version { $version }
check_for_updates = Beim Start nach Updates suchen
check_for_updates_desc = Sucht beim Start von floetask nach einer neuen Version. Gesendet wird nur die Versionsabfrage.
check_now = Jetzt suchen
update_checking = Suche nach Updates …
update_up_to_date = floetask ist aktuell.
update_available = floetask { $version } ist verfügbar.
update_available_toast = floetask { $version } ist verfügbar. Installiere es unter Einstellungen → Über.
update_install = Installieren und neu starten
update_installing = floetask { $version } wird heruntergeladen und installiert … floetask startet danach neu.
update_preview_version = { $version } (Vorschau)
update_preview_on_stable = Du verwendest eine Vorschau. Der stabile Kanal aktualisiert sie mit der nächsten Version; um sie jetzt zu verlassen, gehe unten zurück.
update_channel = Versionskanal
update_channel_desc = Vorschau bietet auch Release Candidates: die nächste Version einige Tage früher, um neue Funktionen auszuprobieren und Probleme zu melden, bevor alle sie bekommen. Sie sind weniger getestet. Du kannst jederzeit zu Stabil zurückwechseln.
update_channel_stable = Stabil
update_channel_preview = Vorschau
rollback = Zurückgehen
rollback_desc = Installiert eine frühere Version über diese, um eine Vorschau zu verlassen oder einer fehlerhaften Version auszuweichen. Deine Aufgabendateien und Einstellungen bleiben erhalten.
rollback_pick = Frühere Version
rollback_install = Installieren
rollback_check_first = Suche nach Updates, um frühere Versionen aufzulisten.
rollback_none = Keine frühere Version in diesem Kanal.
update_failed = Update fehlgeschlagen: { $error }
updates_not_configured = Dieser Build kann sich nicht selbst aktualisieren. Versionen von der Release-Seite des Projekts können es.

## Notifications

due_today = Heute fällig
due_tomorrow = Morgen fällig
due_in_days = In { $days } Tagen fällig

## Errors

error_io = Kein Zugriff auf { $path }: { $message }
error_stale_line = Zeile { $line } hat sich seit dem Lesen geändert; lade neu und versuche es noch einmal.
error_missing_line = Zeile { $line } existiert nicht.
error_note_empty = Der Notizname ist leer.
error_note_character = „{ $character }“ ist in einem Notiznamen nicht erlaubt; verwende Kleinbuchstaben a–z, 0–9, „.“, „_“ und „-“.
error_note_outside = Ein Notizname muss im Notizordner bleiben.
error_no_done_file = { $file } hat keine Erledigt-Datei zum Archivieren.
error_empty_todo = Die Aufgabe ist leer.
error_config = Ungültige Konfiguration: { $details }
