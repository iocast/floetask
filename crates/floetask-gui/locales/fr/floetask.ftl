# floetask : français.
#
# Traduit de locales/en/floetask.ftl. Glossaire : todo = tâche,
# project = projet, context = contexte, threshold = date de début,
# status = statut, board = tableau, done file = fichier des tâches terminées.

## Navigation and files

add_todo = Ajouter une tâche
toggle_drawer = Filtres et tri
archive = Archiver les tâches terminées
open_file = Ouvrir un fichier de tâches
create_file = Créer un fichier de tâches
settings = Paramètres
search = Rechercher
no_file_title = Aucun fichier de tâches ouvert
no_file_body = Ouvrez un todo.txt existant ou créez-en un. Vous pouvez aussi déposer un fichier sur la fenêtre.
no_todos_title = Ce fichier ne contient encore aucune tâche
no_todos_body = Ajoutez votre première tâche avec Ctrl+N.
no_visible_title = Aucune tâche ne correspond aux filtres actuels
reset_filters = Réinitialiser les filtres
file_dialog_title = Fichiers todo.txt
reveal_todo_file = Afficher dans le gestionnaire de fichiers
remove_file = Retirer de la liste
remove_file_confirm = Retirer { $file } de la liste ? Le fichier lui-même n’est pas supprimé.
file_load_failed = Impossible d’ouvrir { $file }
done_file_needed = Choisissez ou créez d’abord un fichier des tâches terminées pour l’archivage.
archived =
    { $count ->
        [one] { $count } tâche archivée
       *[other] { $count } tâches archivées
    }
archived_one = Tâche archivée
new = Nouveau
toggle_files = Afficher ou masquer les fichiers
files = Fichiers
file_actions = Actions sur le fichier
set_archive_file = Définir le fichier d’archive
open_archive_file = Ouvrir le fichier d’archive
close_file = Fermer
sort_files = Trier les fichiers
file_sort_custom = Ordre personnalisé
file_sort_custom_hint = Glissez les fichiers pour les ranger
file_sort_name = Nom
file_sort_folder = Dossier
overview = Vue d’ensemble
overview_hint = Tous les fichiers, groupés par fichier
toggle_theme = Basculer clair / sombre

## Counts

counts =
    { $visible } sur { $total ->
        [one] { $total } tâche
       *[other] { $total } tâches
    }, { $completed ->
        [one] { $completed } terminée
       *[other] { $completed } terminées
    }
groups = groupes

## List

created_on = Créée le { $date }
completed_on = Terminée le { $date }
no_value = Sans { $attribute }
copy = Copier
copied = Copié dans le presse-papiers
delete = Supprimer
delete_confirm = Supprimer cette tâche ?
edit = Modifier
open_link = Ouvrir le lien
archive_one = Déplacer vers le fichier des tâches terminées
due_label = échéance
threshold_label = début
details = Détails

## Editor

new_todo = Nouvelle tâche
edit_todo = Modifier la tâche
editor_file = Dans { $file }
editor_add_to = Ajouter à
editor_placeholder = (A) Appeler maman +famille @téléphone due:tomorrow
editor_hint = Ctrl+Entrée pour enregistrer, Échap pour fermer
add = Ajouter
add_count = Ajouter ({ $count })
save = Enregistrer
cancel = Annuler
close = Fermer
confirm = Confirmer
empty_todo = Saisissez d’abord quelque chose
priority = Priorité
due = Échéance
t = Date de début
rec = Répéter
pm = Pomodoros
status = Statut
status_todo = À faire
status_doing = En cours
status_waiting = En attente
status_someday = Un jour
status_done = Terminé
set = Définir
clear = Effacer
today = Aujourd’hui
strict = Strict
note = Note
notes = Notes
previous_month = Mois précédent
next_month = Mois suivant

## Notes

notes_placeholder = Notes en Markdown, enregistrées dans un fichier à côté de votre fichier de tâches
note_saved_to = Enregistrée dans { $path }
open_note = Ouvrir la note
link_note_file = Lier un fichier Markdown au lieu d’en créer un
link_note_dialog_title = Lier un fichier Markdown
markdown_files = Markdown et texte
note_outside_folder = { $file } n’est pas dans { $folder }, où cette liste range ses notes.
note_move_into = Le déplacer vers { $folder }
note_keep_in_place = Le laisser où il est
unlink_note = Délier la note ; son fichier est conservé
open_note_window = Ouvrir dans une fenêtre séparée
note_in_window = Cette note est ouverte dans sa propre fenêtre.
show_note_window = Afficher la fenêtre
note_saved = Enregistrée
note_unsaved = Modifications non enregistrées
open_in_default_app = Ouvrir dans l’application par défaut
note_missing = { $note } n’existe pas encore ; ouvrez-la pour la créer
note_invalid = { $note } n’est pas un nom de note valide
note_orphaned = Plus aucune tâche ne renvoie à { $note } ; le fichier a été conservé

## Status board

show_board = Afficher en tableau (Ctrl+Alt+B)
show_list = Afficher en liste (Ctrl+Alt+B)
board_columns = Colonnes
board_group = Grouper
board_group_by = Grouper par { $attribute }
board_group_hint = Un tableau par groupe. Désactivé, l’ordre de tri est conservé sur un seul tableau.
board_other = Autres statuts
board_columns_title = Colonnes du tableau pour { $file }
board_columns_hint = Chaque colonne est un statut ; Terminé contient les tâches terminées. Faites glisser une carte vers une autre colonne pour changer son statut.
status_placeholder = Nom, p. ex. En revue
remove_column = Retirer la colonne
reset_columns = Réinitialiser

## Calendar view

show_calendar = Calendrier (Ctrl+Alt+C)
calendar_day = Jour
calendar_week = Semaine
calendar_month = Mois
calendar_today = Aujourd’hui
calendar_previous = Précédent
calendar_next = Suivant
calendar_more = +{ $count } de plus
calendar_undated = { $count } sans échéance
calendar_undated_title = Sans échéance
calendar_undated_hint = Faites glisser une tâche sur un jour, ou définissez sa date ici.
calendar_set_due = Définir l’échéance
calendar_empty_day = Rien n’est dû ce jour-là.

## Dates

date_month_year = { $month } { $year }
date_long = { $weekday } { $day } { $month } { $year }
date_range_in_month = { $first_day } – { $last_day } { $month } { $year }
date_range_in_year = { $first_day } { $first_month } – { $last_day } { $last_month } { $year }
date_range = { $first_day } { $first_month } { $first_year } – { $last_day } { $last_month } { $last_year }

month_1 = janvier
month_2 = février
month_3 = mars
month_4 = avril
month_5 = mai
month_6 = juin
month_7 = juillet
month_8 = août
month_9 = septembre
month_10 = octobre
month_11 = novembre
month_12 = décembre

month_short_1 = janv.
month_short_2 = févr.
month_short_3 = mars
month_short_4 = avr.
month_short_5 = mai
month_short_6 = juin
month_short_7 = juil.
month_short_8 = août
month_short_9 = sept.
month_short_10 = oct.
month_short_11 = nov.
month_short_12 = déc.

weekday_mon = lundi
weekday_tue = mardi
weekday_wed = mercredi
weekday_thu = jeudi
weekday_fri = vendredi
weekday_sat = samedi
weekday_sun = dimanche

weekday_abbr_mon = lun.
weekday_abbr_tue = mar.
weekday_abbr_wed = mer.
weekday_abbr_thu = jeu.
weekday_abbr_fri = ven.
weekday_abbr_sat = sam.
weekday_abbr_sun = dim.

mon = Lu
tue = Ma
wed = Me
thu = Je
fri = Ve
sat = Sa
sun = Di

## Human-friendly dates

bucket_overdue = en retard
bucket_elapsed = passée
bucket_last_week = la semaine dernière
bucket_today = aujourd’hui
bucket_tomorrow = demain
bucket_this_week = cette semaine
bucket_next_week = la semaine prochaine
bucket_this_month = ce mois-ci
bucket_next_month = le mois prochain

## Recurrence units

unit_d = jours
unit_b = jours ouvrés
unit_w = semaines
unit_m = mois
unit_y = ans

## Attributes and the drawer

projects = Projets
contexts = Contextes
created = Créée
completed = Terminée
attributes = Attributs
filters = Filtres
sorting = Tri
hide_category = Masquer les tâches ayant une valeur
show_category = Afficher à nouveau les tâches avec cet attribut
rename = Renommer
rename_title = Renommer { $value } partout dans ce fichier
rename_title_all = Renommer { $value } partout dans tous les fichiers
remove_value = Retirer
remove_value_confirm = Retirer { $value } de toutes les tâches de ce fichier ?
remove_value_confirm_all = Retirer { $value } de toutes les tâches de tous les fichiers ?
renamed =
    { $count ->
        [one] { $count } tâche mise à jour
       *[other] { $count } tâches mises à jour
    }
include_hint = Cliquez pour inclure, Alt+clic pour exclure. Clic droit sur un projet ou un contexte pour le renommer ou le retirer.
show_completed = Afficher les tâches terminées
show_hidden = Afficher les tâches masquées (h:1)
show_hidden_attributes = Afficher les attributs des tâches masquées
show_hidden_statuses = Afficher les tâches { $statuses }
show_future_threshold = Afficher les tâches avec une date de début future
show_future_due = Afficher les tâches avec une échéance future
file_order = Ordre du fichier (sans regroupement)
completed_last = Trier les tâches terminées en dernier
move_up = Monter
move_down = Descendre
invert = Inverser

## Search

search_placeholder = Rechercher, ou filtrer : +projet @contexte due: < today
saved_filters = Filtres enregistrés
save_filter = Enregistrer la recherche
filter_name = Nom
no_saved_filters = Aucun filtre enregistré
delete_filter_confirm = Supprimer le filtre enregistré { $name } ?
suppress_notifications = Couper les notifications des tâches correspondantes

## Settings

settings_title = Paramètres
section_todos = Tâches
section_dates = Dates
section_statuses = Statuts
section_appearance = Apparence
section_notifications = Notifications
section_files = Fichiers
section_about = À propos
append_creation_date = Ajouter la date de création aux nouvelles tâches
append_creation_date_desc = Les nouvelles tâches commencent par la date du jour, comme le permet todo.txt.
convert_relative_dates = Convertir les dates relatives (due:tomorrow) en dates
convert_relative_dates_desc = Des mots comme tomorrow ou next friday dans due: et t: sont enregistrés comme de vraies dates.
human_friendly_dates = Afficher les dates comme aujourd’hui, la semaine prochaine…
human_friendly_dates_desc = Les dates se lisent aujourd’hui, demain ou la semaine prochaine au lieu de 2026-10-05.
safe_writes = Écriture sûre (fichier temporaire et sauvegarde)
safe_writes_desc = Écrit d’abord dans un fichier temporaire et garde une sauvegarde, pour qu’un plantage ne puisse pas corrompre votre fichier de tâches.
bulk_creation = Créer une tâche par ligne
bulk_creation_desc = Dans la boîte d’ajout, chaque ligne devient une tâche.
disable_animations = Désactiver les animations
disable_animations_desc = Supprime le mouvement ; floetask n’a pas encore d’animations.
compact = Mode compact
compact_desc = Lignes et cartes plus serrées, pour afficher plus de tâches à l’écran.
month_overflow = Cellules du mois pleines
month_overflow_desc = Quand une cellule du mois contient plus de tâches qu’elle ne peut en afficher : afficher +N de plus, ou défiler.
month_overflow_more = Afficher +N de plus
month_overflow_scroll = Défiler
week_start = La semaine commence le
week_start_desc = Premier jour de la semaine dans les calendriers et pour cette semaine et la semaine prochaine.
monday = Lundi
saturday = Samedi
sunday = Dimanche
theme = Thème
theme_desc = Clair, sombre, ou comme le système.
theme_system = Système
theme_light = Clair
theme_dark = Sombre
language = Langue
language_desc = Langue de l’interface. Système suit votre système d’exploitation.
language_system = Système
notifications = Notifications d’échéance
notifications_desc = Une notification système quand une tâche arrive à échéance.
notification_threshold =
    { $days ->
        [0] Notifier le jour de l’échéance
        [one] Notifier { $days } jour avant
       *[other] Notifier { $days } jours avant
    }
notification_threshold_desc = Combien de jours avant l’échéance vous êtes notifié.
config_file = Fichier de paramètres
config_file_desc = Où ces paramètres sont enregistrés. Les colonnes du tableau et les statuts s’y trouvent aussi.
config_location = Fichier de paramètres : { $path }
settings_save_failed = Impossible d’enregistrer les paramètres
statuses_desc = Statuts de l’étiquette status:, dans l’ordre de tri de la liste. Les statuts masqués restent hors de la liste jusqu’à ce que vous les demandiez. Une colonne de tableau peut ajouter un statut pour un seul fichier.
status_todo_desc = Pas commencée. Comme l’absence d’étiquette status:.
status_doing_desc = En cours de traitement.
status_waiting_desc = Bloquée par quelqu’un ou quelque chose d’autre.
status_someday_desc = En suspens, sans engagement.
status_custom_desc = Votre propre statut.
status_hidden = Masqué
remove_status = Retirer le statut
add_status = Ajouter un statut

## About and updates

about_tagline = Un gestionnaire todo.txt pour le bureau.
about_version = Version { $version }
check_for_updates = Rechercher les mises à jour au démarrage
check_for_updates_desc = Cherche une nouvelle version au démarrage de floetask. Seule la vérification de version est envoyée.
check_now = Vérifier maintenant
update_checking = Recherche de mises à jour…
update_up_to_date = floetask est à jour.
update_available = floetask { $version } est disponible.
update_button = Mettre à jour
update_button_hint = floetask { $version } est disponible. Cliquez pour voir les nouveautés et l’installer.
update_install = Installer et redémarrer
update_installing = Téléchargement et installation de floetask { $version }… floetask redémarre une fois terminé.
update_preview_version = { $version } (préversion)
update_preview_on_stable = Vous utilisez une préversion. Le canal stable la mettra à jour avec la prochaine version ; pour la quitter maintenant, revenez en arrière ci-dessous.
update_channel = Canal de publication
update_channel_desc = Préversion propose aussi les versions candidates : la prochaine version quelques jours plus tôt, pour essayer les nouveautés et signaler les problèmes avant tout le monde. Elles sont moins testées. Revenez à Stable à tout moment.
update_channel_stable = Stable
update_channel_preview = Préversion
rollback = Revenir en arrière
rollback_desc = Installe une version antérieure par-dessus celle-ci, pour quitter une préversion ou s’éloigner d’une version problématique. Vos fichiers de tâches et vos paramètres sont conservés.
rollback_pick = Version antérieure
rollback_install = Installer
rollback_check_first = Recherchez les mises à jour pour lister les versions antérieures.
rollback_none = Aucune version antérieure sur ce canal.
update_failed = Échec de la mise à jour : { $error }
updates_not_configured = Cette version ne peut pas se mettre à jour elle-même. Les versions de la page de publication du projet le peuvent.

## Notifications

due_today = Échéance aujourd’hui
due_tomorrow = Échéance demain
due_in_days = Échéance dans { $days } jours

## Errors

error_io = Impossible d’accéder à { $path } : { $message }
error_stale_line = La ligne { $line } a changé depuis sa lecture ; rechargez et réessayez.
error_missing_line = La ligne { $line } n’existe pas.
error_note_empty = Le nom de la note est vide.
error_note_character = « { $character } » n’est pas autorisé dans un nom de note ; utilisez a-z en minuscules, 0-9, « . », « _ » et « - ».
error_note_outside = Un nom de note doit rester dans le dossier des notes.
error_no_done_file = { $file } n’a pas de fichier des tâches terminées pour l’archivage.
error_empty_todo = La tâche est vide.
error_config = Configuration invalide : { $details }
