# floetask: italiano.
#
# Tradotto da locales/en/floetask.ftl. Glossario: todo = attività,
# project = progetto, context = contesto, threshold = data di inizio,
# status = stato, board = bacheca, done file = file delle completate.

## Navigation and files

add_todo = Aggiungi attività
toggle_drawer = Filtri e ordinamento
archive = Archivia le attività completate
open_file = Apri file di attività
create_file = Crea file di attività
settings = Impostazioni
search = Cerca
no_file_title = Nessun file di attività aperto
no_file_body = Apri un todo.txt esistente o creane uno nuovo. Puoi anche trascinare un file sulla finestra.
no_todos_title = Questo file non ha ancora attività
no_todos_body = Aggiungi la tua prima attività con Ctrl+N.
no_visible_title = Nessuna attività corrisponde ai filtri attuali
reset_filters = Reimposta filtri
file_dialog_title = File todo.txt
reveal_todo_file = Mostra nel file manager
remove_file = Rimuovi dall’elenco
remove_file_confirm = Rimuovere { $file } dall’elenco? Il file in sé non viene eliminato.
file_load_failed = Impossibile aprire { $file }
done_file_needed = Prima scegli o crea un file delle completate per l’archiviazione.
archived =
    { $count ->
        [one] { $count } attività archiviata
       *[other] { $count } attività archiviate
    }
archived_one = Attività archiviata
new = Nuovo
toggle_files = Mostra o nascondi i file
files = File
file_actions = Azioni sul file
set_archive_file = Imposta file di archivio
open_archive_file = Apri file di archivio
close_file = Chiudi
sort_files = Ordina file
file_sort_custom = Ordine personalizzato
file_sort_custom_hint = Trascina i file per ordinarli
file_sort_name = Nome
file_sort_folder = Cartella
overview = Panoramica
overview_hint = Tutti i file, raggruppati per file
toggle_theme = Passa a chiaro / scuro

## Counts

counts =
    { $visible } di { $total } attività, { $completed ->
        [one] { $completed } completata
       *[other] { $completed } completate
    }
groups = gruppi

## List

created_on = Creata il { $date }
completed_on = Completata il { $date }
no_value = Senza { $attribute }
copy = Copia
copied = Copiato negli appunti
delete = Elimina
delete_confirm = Eliminare questa attività?
edit = Modifica
open_link = Apri link
archive_one = Sposta nel file delle completate
due_label = scadenza
threshold_label = inizio
details = Dettagli

## Editor

new_todo = Nuova attività
edit_todo = Modifica attività
editor_file = In { $file }
editor_add_to = Aggiungi a
editor_placeholder = (A) Chiamare la mamma +famiglia @telefono due:tomorrow
editor_hint = Ctrl+Invio per salvare, Esc per chiudere
add = Aggiungi
add_count = Aggiungi ({ $count })
save = Salva
cancel = Annulla
close = Chiudi
confirm = Conferma
empty_todo = Scrivi prima qualcosa
priority = Priorità
due = Scadenza
t = Data di inizio
rec = Ripeti
pm = Pomodori
status = Stato
status_todo = Da fare
status_doing = In corso
status_waiting = In attesa
status_someday = Prima o poi
status_done = Fatto
set = Imposta
clear = Cancella
today = Oggi
strict = Rigoroso
note = Nota
notes = Note
previous_month = Mese precedente
next_month = Mese successivo

## Notes

notes_placeholder = Note in Markdown, salvate in un file accanto al tuo file di attività
note_saved_to = Salvata in { $path }
open_note = Apri nota
link_note_file = Collega un file Markdown invece di crearne uno
link_note_dialog_title = Collega un file Markdown
markdown_files = Markdown e testo
note_outside_folder = { $file } non è in { $folder }, dove questo elenco tiene le sue note.
note_move_into = Spostalo in { $folder }
note_keep_in_place = Lascialo dov’è
unlink_note = Scollega la nota; il suo file viene mantenuto
open_note_window = Apri in una finestra separata
note_in_window = Questa nota è aperta nella sua finestra.
show_note_window = Mostra finestra
note_saved = Salvata
note_unsaved = Modifiche non salvate
open_in_default_app = Apri nell’app predefinita
note_missing = { $note } non esiste ancora; aprila per crearla
note_invalid = { $note } non è un nome di nota valido
note_orphaned = Nessuna attività rimanda più a { $note }; il file è stato mantenuto

## Status board

show_board = Mostra come bacheca (Ctrl+Alt+B)
show_list = Mostra come elenco (Ctrl+Alt+B)
board_columns = Colonne
board_group = Raggruppa
board_group_by = Raggruppa per { $attribute }
board_group_hint = Una bacheca per gruppo. Disattivato mantiene l’ordinamento su una sola bacheca.
board_other = Altri stati
board_columns_title = Colonne della bacheca per { $file }
board_columns_hint = Ogni colonna è uno stato; Fatto contiene le attività completate. Trascina una scheda in un’altra colonna per cambiarne lo stato.
status_placeholder = Nome, ad es. In revisione
remove_column = Rimuovi colonna
reset_columns = Reimposta

## Calendar view

show_calendar = Calendario (Ctrl+Alt+C)
calendar_day = Giorno
calendar_week = Settimana
calendar_month = Mese
calendar_today = Oggi
calendar_previous = Precedente
calendar_next = Successivo
calendar_more = +{ $count } altre
calendar_undated = { $count } senza scadenza
calendar_undated_title = Senza scadenza
calendar_undated_hint = Trascina un’attività su un giorno o imposta qui la sua data.
calendar_set_due = Imposta scadenza
calendar_empty_day = Niente in scadenza in questo giorno.

## Dates

date_month_year = { $month } { $year }
date_long = { $weekday } { $day } { $month } { $year }
date_range_in_month = { $first_day }–{ $last_day } { $month } { $year }
date_range_in_year = { $first_day } { $first_month } – { $last_day } { $last_month } { $year }
date_range = { $first_day } { $first_month } { $first_year } – { $last_day } { $last_month } { $last_year }

month_1 = gennaio
month_2 = febbraio
month_3 = marzo
month_4 = aprile
month_5 = maggio
month_6 = giugno
month_7 = luglio
month_8 = agosto
month_9 = settembre
month_10 = ottobre
month_11 = novembre
month_12 = dicembre

month_short_1 = gen
month_short_2 = feb
month_short_3 = mar
month_short_4 = apr
month_short_5 = mag
month_short_6 = giu
month_short_7 = lug
month_short_8 = ago
month_short_9 = set
month_short_10 = ott
month_short_11 = nov
month_short_12 = dic

weekday_mon = lunedì
weekday_tue = martedì
weekday_wed = mercoledì
weekday_thu = giovedì
weekday_fri = venerdì
weekday_sat = sabato
weekday_sun = domenica

weekday_abbr_mon = lun
weekday_abbr_tue = mar
weekday_abbr_wed = mer
weekday_abbr_thu = gio
weekday_abbr_fri = ven
weekday_abbr_sat = sab
weekday_abbr_sun = dom

mon = Lu
tue = Ma
wed = Me
thu = Gi
fri = Ve
sat = Sa
sun = Do

## Human-friendly dates

bucket_overdue = scaduta
bucket_elapsed = passata
bucket_last_week = la settimana scorsa
bucket_today = oggi
bucket_tomorrow = domani
bucket_this_week = questa settimana
bucket_next_week = la settimana prossima
bucket_this_month = questo mese
bucket_next_month = il mese prossimo

## Recurrence units

unit_d = giorni
unit_b = giorni lavorativi
unit_w = settimane
unit_m = mesi
unit_y = anni

## Attributes and the drawer

projects = Progetti
contexts = Contesti
created = Creata
completed = Completata
attributes = Attributi
filters = Filtri
sorting = Ordinamento
hide_category = Nascondi le attività con un valore
show_category = Mostra di nuovo le attività con questo attributo
rename = Rinomina
rename_title = Rinomina { $value } in tutto questo file
rename_title_all = Rinomina { $value } in tutti i file
remove_value = Rimuovi
remove_value_confirm = Rimuovere { $value } da tutte le attività di questo file?
remove_value_confirm_all = Rimuovere { $value } da tutte le attività di tutti i file?
renamed =
    { $count ->
        [one] { $count } attività aggiornata
       *[other] { $count } attività aggiornate
    }
include_hint = Clic per includere, Alt+clic per escludere. Clic destro su un progetto o contesto per rinominarlo o rimuoverlo.
show_completed = Mostra le attività completate
show_hidden = Mostra le attività nascoste (h:1)
show_hidden_attributes = Mostra gli attributi delle attività nascoste
show_hidden_statuses = Mostra le attività { $statuses }
show_future_threshold = Mostra le attività con data di inizio futura
show_future_due = Mostra le attività con scadenza futura
file_order = Ordine del file (senza raggruppamento)
completed_last = Metti in fondo le attività completate
move_up = Sposta su
move_down = Sposta giù
invert = Inverti

## Search

search_placeholder = Cerca, o filtra: +progetto @contesto due: < today
saved_filters = Filtri salvati
save_filter = Salva la ricerca attuale
filter_name = Nome
no_saved_filters = Ancora nessun filtro salvato
delete_filter_confirm = Eliminare il filtro salvato { $name }?
suppress_notifications = Silenzia le notifiche per le attività corrispondenti

## Settings

settings_title = Impostazioni
section_todos = Attività
section_dates = Date
section_statuses = Stati
section_appearance = Aspetto
section_notifications = Notifiche
section_files = File
section_about = Informazioni
append_creation_date = Aggiungi la data di creazione alle nuove attività
append_creation_date_desc = Le nuove attività iniziano con la data di oggi, come consente todo.txt.
convert_relative_dates = Converti le date relative (due:tomorrow) in date
convert_relative_dates_desc = Parole come tomorrow o next friday in due: e t: vengono salvate come date vere.
human_friendly_dates = Mostra le date come oggi, la settimana prossima…
human_friendly_dates_desc = Le date si leggono come oggi, domani o la settimana prossima invece di 2026-10-05.
safe_writes = Scrittura sicura (file temporaneo e backup)
safe_writes_desc = Scrive prima in un file temporaneo e conserva un backup, così un crash non può danneggiare il tuo file di attività.
bulk_creation = Crea un’attività per riga
bulk_creation_desc = Nella finestra di aggiunta, ogni riga diventa un’attività a sé.
disable_animations = Disattiva le animazioni
disable_animations_desc = Elimina il movimento; floetask non ha ancora animazioni.
compact = Modalità compatta
compact_desc = Righe e schede più strette, così entrano più attività sullo schermo.
month_overflow = Celle del mese piene
month_overflow_desc = Quando una cella del mese ha più attività di quante ne entrino: mostra +N altre, oppure scorri.
month_overflow_more = Mostra +N altre
month_overflow_scroll = Scorri
week_start = La settimana inizia di
week_start_desc = Primo giorno della settimana nei calendari e per questa settimana e la prossima.
monday = Lunedì
saturday = Sabato
sunday = Domenica
theme = Tema
theme_desc = Chiaro, scuro o come il sistema.
theme_system = Sistema
theme_light = Chiaro
theme_dark = Scuro
language = Lingua
language_desc = Lingua dell’interfaccia. Sistema segue il tuo sistema operativo.
language_system = Sistema
zoom = Zoom { $percent }%
zoom_desc = Rende tutto nella finestra più grande o più piccolo.
notifications = Notifiche di scadenza
notifications_desc = Una notifica di sistema quando un’attività scade.
notification_threshold =
    { $days ->
        [0] Avvisa il giorno della scadenza
        [one] Avvisa { $days } giorno prima
       *[other] Avvisa { $days } giorni prima
    }
notification_threshold_desc = Quanti giorni prima della scadenza ricevi l’avviso.
config_file = File delle impostazioni
config_file_desc = Dove vengono salvate queste impostazioni. Anche le colonne della bacheca e gli stati si trovano qui.
config_location = File delle impostazioni: { $path }
settings_save_failed = Impossibile salvare le impostazioni
statuses_desc = Stati per l’etichetta status:, nell’ordine in cui l’elenco li ordina. Gli stati nascosti restano fuori dall’elenco finché non li chiedi. Una colonna della bacheca può aggiungere uno stato per un solo file.
status_todo_desc = Non iniziata. Come non avere l’etichetta status:.
status_doing_desc = Ci si sta lavorando.
status_waiting_desc = Bloccata da qualcuno o qualcos’altro.
status_someday_desc = Messa da parte, senza impegno.
status_custom_desc = Il tuo stato personale.
status_hidden = Nascosto
remove_status = Rimuovi stato
add_status = Aggiungi stato

## About and updates

about_tagline = Un gestore todo.txt per il desktop.
about_version = Versione { $version }
check_for_updates = Cerca aggiornamenti all’avvio
check_for_updates_desc = Cerca una nuova versione all’avvio di floetask. Viene inviato solo il controllo della versione.
check_now = Controlla ora
update_checking = Ricerca aggiornamenti…
update_up_to_date = floetask è aggiornato.
update_available = floetask { $version } è disponibile.
update_button = Aggiorna
update_button_hint = floetask { $version } è disponibile. Fai clic per vedere le novità e installarlo.
update_install = Installa e riavvia
update_installing = Download e installazione di floetask { $version }… floetask si riavvia al termine.
update_preview_version = { $version } (anteprima)
update_preview_on_stable = Stai usando un’anteprima. Il canale stabile la aggiornerà con la prossima versione; per lasciarla subito, torna indietro qui sotto.
update_channel = Canale di rilascio
update_channel_desc = Anteprima offre anche le release candidate: la prossima versione qualche giorno prima, per provare le novità e segnalare problemi prima di tutti. Sono meno testate. Puoi tornare a Stabile in qualsiasi momento.
update_channel_stable = Stabile
update_channel_preview = Anteprima
rollback = Torna indietro
rollback_desc = Installa una versione precedente sopra questa, per lasciare un’anteprima o tornare indietro da una versione con problemi. I tuoi file di attività e le impostazioni vengono mantenuti.
rollback_pick = Versione precedente
rollback_install = Installa
rollback_check_first = Cerca aggiornamenti per elencare le versioni precedenti.
rollback_none = Nessuna versione precedente su questo canale.
update_failed = Aggiornamento non riuscito: { $error }
updates_not_configured = Questa build non può aggiornarsi da sola. Le versioni dalla pagina dei rilasci del progetto sì.

## Notifications

due_today = Scade oggi
due_tomorrow = Scade domani
due_in_days = Scade tra { $days } giorni

## Errors

error_io = Impossibile accedere a { $path }: { $message }
error_stale_line = La riga { $line } è cambiata da quando è stata letta; ricarica e riprova.
error_missing_line = La riga { $line } non esiste.
error_note_empty = Il nome della nota è vuoto.
error_note_character = «{ $character }» non è consentito in un nome di nota; usa a-z minuscole, 0-9, «.», «_» e «-».
error_note_outside = Un nome di nota deve restare nella cartella delle note.
error_no_done_file = { $file } non ha un file delle completate in cui archiviare.
error_empty_todo = L’attività è vuota.
error_config = Configurazione non valida: { $details }
