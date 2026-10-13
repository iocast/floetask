# floetask: español.
#
# Traducido de locales/en/floetask.ftl. Glosario: todo = tarea,
# project = proyecto, context = contexto, threshold = fecha de inicio,
# status = estado, board = tablero, done file = archivo de completadas.

## Navigation and files

add_todo = Añadir tarea
toggle_drawer = Filtros y orden
archive = Archivar tareas completadas
open_file = Abrir archivo de tareas
create_file = Crear archivo de tareas
settings = Ajustes
search = Buscar
no_file_title = No hay ningún archivo de tareas abierto
no_file_body = Abre un todo.txt existente o crea uno nuevo. También puedes soltar un archivo en la ventana.
no_todos_title = Este archivo aún no tiene tareas
no_todos_body = Añade tu primera tarea con Ctrl+N.
no_visible_title = Ninguna tarea coincide con los filtros actuales
reset_filters = Restablecer filtros
file_dialog_title = Archivos todo.txt
reveal_todo_file = Mostrar en el gestor de archivos
remove_file = Quitar de la lista
remove_file_confirm = ¿Quitar { $file } de la lista? El archivo en sí no se borra.
file_load_failed = No se pudo abrir { $file }
done_file_needed = Primero elige o crea un archivo de completadas para archivar.
archived =
    { $count ->
        [one] { $count } tarea archivada
       *[other] { $count } tareas archivadas
    }
archived_one = Tarea archivada
new = Nuevo
toggle_files = Mostrar u ocultar archivos
files = Archivos
file_actions = Acciones del archivo
set_archive_file = Definir archivo de archivo
open_archive_file = Abrir archivo de archivo
close_file = Cerrar
toggle_theme = Cambiar claro / oscuro

## Counts

counts =
    { $visible } de { $total ->
        [one] { $total } tarea
       *[other] { $total } tareas
    }, { $completed ->
        [one] { $completed } completada
       *[other] { $completed } completadas
    }
groups = grupos

## List

created_on = Creada el { $date }
completed_on = Completada el { $date }
no_value = Sin { $attribute }
copy = Copiar
copied = Copiado al portapapeles
delete = Borrar
delete_confirm = ¿Borrar esta tarea?
edit = Editar
open_link = Abrir enlace
archive_one = Mover al archivo de completadas
due_label = vence
threshold_label = inicio
details = Detalles

## Editor

new_todo = Nueva tarea
edit_todo = Editar tarea
editor_placeholder = (A) Llamar a mamá +familia @teléfono due:tomorrow
editor_hint = Ctrl+Intro para guardar, Esc para cerrar
add = Añadir
add_count = Añadir ({ $count })
save = Guardar
cancel = Cancelar
close = Cerrar
confirm = Confirmar
empty_todo = Escribe algo primero
priority = Prioridad
due = Vencimiento
t = Fecha de inicio
rec = Repetir
pm = Pomodoros
status = Estado
status_todo = Por hacer
status_doing = En curso
status_waiting = En espera
status_someday = Algún día
status_done = Hecho
set = Fijar
clear = Borrar
today = Hoy
strict = Estricto
note = Nota
notes = Notas
previous_month = Mes anterior
next_month = Mes siguiente

## Notes

notes_placeholder = Notas en Markdown, guardadas en un archivo junto a tu archivo de tareas
note_saved_to = Guardada en { $path }
open_note = Abrir nota
link_note_file = Enlazar un archivo Markdown en lugar de crear uno
link_note_dialog_title = Enlazar un archivo Markdown
markdown_files = Markdown y texto
note_outside_folder = { $file } no está en { $folder }, donde esta lista guarda sus notas.
note_move_into = Moverlo a { $folder }
note_keep_in_place = Dejarlo donde está
unlink_note = Desenlazar la nota; su archivo se conserva
open_note_window = Abrir en una ventana aparte
note_in_window = Esta nota está abierta en su propia ventana.
show_note_window = Mostrar ventana
note_saved = Guardada
note_unsaved = Cambios sin guardar
open_in_default_app = Abrir en la aplicación predeterminada
note_missing = { $note } aún no existe; ábrela para crearla
note_invalid = { $note } no es un nombre de nota válido
note_orphaned = Ninguna tarea enlaza ya con { $note }; el archivo se conservó

## Status board

show_board = Mostrar como tablero (Ctrl+Alt+B)
show_list = Mostrar como lista (Ctrl+Alt+B)
board_columns = Columnas
board_group = Agrupar
board_group_by = Agrupar por { $attribute }
board_group_hint = Un tablero por grupo. Desactivado mantiene el orden en un solo tablero.
board_other = Otros estados
board_columns_title = Columnas del tablero de { $file }
board_columns_hint = Cada columna es un estado; Hecho contiene las tareas completadas. Arrastra una tarjeta a otra columna para cambiar su estado.
column_placeholder = Estado, p. ej. in-review
remove_column = Quitar columna
reset_columns = Restablecer

## Calendar view

show_calendar = Calendario (Ctrl+Alt+C)
calendar_day = Día
calendar_week = Semana
calendar_month = Mes
calendar_today = Hoy
calendar_previous = Anterior
calendar_next = Siguiente
calendar_more = +{ $count } más
calendar_undated = { $count } sin fecha de vencimiento
calendar_undated_title = Sin fecha de vencimiento
calendar_undated_hint = Arrastra una tarea a un día o fija su fecha aquí.
calendar_set_due = Fijar vencimiento
calendar_empty_day = Nada vence este día.

## Dates

date_month_year = { $month } de { $year }
date_long = { $weekday }, { $day } de { $month } de { $year }
date_range_in_month = { $first_day }–{ $last_day } { $month } { $year }
date_range_in_year = { $first_day } { $first_month } – { $last_day } { $last_month } { $year }
date_range = { $first_day } { $first_month } { $first_year } – { $last_day } { $last_month } { $last_year }

month_1 = enero
month_2 = febrero
month_3 = marzo
month_4 = abril
month_5 = mayo
month_6 = junio
month_7 = julio
month_8 = agosto
month_9 = septiembre
month_10 = octubre
month_11 = noviembre
month_12 = diciembre

month_short_1 = ene
month_short_2 = feb
month_short_3 = mar
month_short_4 = abr
month_short_5 = may
month_short_6 = jun
month_short_7 = jul
month_short_8 = ago
month_short_9 = sept
month_short_10 = oct
month_short_11 = nov
month_short_12 = dic

weekday_mon = lunes
weekday_tue = martes
weekday_wed = miércoles
weekday_thu = jueves
weekday_fri = viernes
weekday_sat = sábado
weekday_sun = domingo

weekday_abbr_mon = lun
weekday_abbr_tue = mar
weekday_abbr_wed = mié
weekday_abbr_thu = jue
weekday_abbr_fri = vie
weekday_abbr_sat = sáb
weekday_abbr_sun = dom

mon = Lu
tue = Ma
wed = Mi
thu = Ju
fri = Vi
sat = Sá
sun = Do

## Human-friendly dates

bucket_overdue = vencida
bucket_elapsed = pasada
bucket_last_week = la semana pasada
bucket_today = hoy
bucket_tomorrow = mañana
bucket_this_week = esta semana
bucket_next_week = la próxima semana
bucket_this_month = este mes
bucket_next_month = el próximo mes

## Recurrence units

unit_d = días
unit_b = días laborables
unit_w = semanas
unit_m = meses
unit_y = años

## Attributes and the drawer

projects = Proyectos
contexts = Contextos
created = Creada
completed = Completada
attributes = Atributos
filters = Filtros
sorting = Orden
hide_category = Ocultar tareas con algún valor
show_category = Volver a mostrar tareas con este atributo
rename = Renombrar
rename_title = Renombrar { $value } en todo este archivo
remove_value = Quitar
remove_value_confirm = ¿Quitar { $value } de todas las tareas de este archivo?
renamed =
    { $count ->
        [one] { $count } tarea actualizada
       *[other] { $count } tareas actualizadas
    }
include_hint = Haz clic para incluir, Alt+clic para excluir. Clic derecho en un proyecto o contexto para renombrarlo o quitarlo.
show_completed = Mostrar tareas completadas
show_hidden = Mostrar tareas ocultas (h:1)
show_hidden_attributes = Mostrar atributos de tareas ocultas
show_hidden_statuses = Mostrar tareas { $statuses }
show_future_threshold = Mostrar tareas con fecha de inicio futura
show_future_due = Mostrar tareas con vencimiento futuro
file_order = Orden del archivo (sin agrupar)
completed_last = Ordenar las tareas completadas al final
move_up = Subir
move_down = Bajar
invert = Invertir

## Search

search_placeholder = Buscar, o filtrar: +proyecto @contexto due: < today
saved_filters = Filtros guardados
save_filter = Guardar la búsqueda actual
filter_name = Nombre
no_saved_filters = Aún no hay filtros guardados
delete_filter_confirm = ¿Borrar el filtro guardado { $name }?
suppress_notifications = Silenciar notificaciones de las tareas que coincidan

## Settings

settings_title = Ajustes
section_todos = Tareas
section_dates = Fechas
section_statuses = Estados
section_appearance = Apariencia
section_notifications = Notificaciones
section_files = Archivos
section_about = Acerca de
append_creation_date = Añadir la fecha de creación a las tareas nuevas
append_creation_date_desc = Las tareas nuevas empiezan con la fecha de hoy, como permite todo.txt.
convert_relative_dates = Convertir fechas relativas (due:tomorrow) en fechas
convert_relative_dates_desc = Palabras como tomorrow o next friday en due: y t: se guardan como fechas reales.
human_friendly_dates = Mostrar fechas como hoy, la próxima semana…
human_friendly_dates_desc = Las fechas se leen como hoy, mañana o la próxima semana en lugar de 2026-10-05.
safe_writes = Escritura segura (archivo temporal y copia de seguridad)
safe_writes_desc = Escribe primero en un archivo temporal y guarda una copia de seguridad, para que un fallo no pueda dañar tu archivo de tareas.
bulk_creation = Crear una tarea por línea
bulk_creation_desc = En el diálogo de añadir, cada línea se convierte en su propia tarea.
disable_animations = Desactivar animaciones
disable_animations_desc = Quita el movimiento; floetask aún no tiene animaciones.
compact = Modo compacto
compact_desc = Filas y tarjetas más ajustadas, para que quepan más tareas en pantalla.
month_overflow = Celdas del mes llenas
month_overflow_desc = Cuando una celda del mes tiene más tareas de las que caben: mostrar +N más, o desplazar.
month_overflow_more = Mostrar +N más
month_overflow_scroll = Desplazar
week_start = La semana empieza el
week_start_desc = Primer día de la semana en los calendarios y para esta semana y la próxima.
monday = Lunes
saturday = Sábado
sunday = Domingo
theme = Tema
theme_desc = Claro, oscuro o como el sistema.
theme_system = Sistema
theme_light = Claro
theme_dark = Oscuro
language = Idioma
language_desc = Idioma de la interfaz. Sistema sigue a tu sistema operativo.
language_system = Sistema
zoom = Zoom { $percent } %
zoom_desc = Hace todo lo de la ventana más grande o más pequeño.
notifications = Notificaciones de vencimiento
notifications_desc = Una notificación del sistema cuando vence una tarea.
notification_threshold =
    { $days ->
        [0] Avisar el día del vencimiento
        [one] Avisar { $days } día antes
       *[other] Avisar { $days } días antes
    }
notification_threshold_desc = Cuántos días antes del vencimiento recibes el aviso.
config_file = Archivo de ajustes
config_file_desc = Dónde se guardan estos ajustes. Las columnas del tablero y los estados también están ahí.
config_location = Archivo de ajustes: { $path }
settings_save_failed = No se pudieron guardar los ajustes
statuses_desc = Estados para la etiqueta status:, en el orden en que la lista los ordena. Los estados ocultos quedan fuera de la lista hasta que los pidas. Una columna del tablero puede añadir un estado solo para un archivo.
status_todo_desc = Sin empezar. Igual que no tener etiqueta status:.
status_doing_desc = En la que se está trabajando.
status_waiting_desc = Bloqueada por alguien o algo.
status_someday_desc = Aparcada, sin compromiso.
status_custom_desc = Tu propio estado.
status_hidden = Oculto
remove_status = Quitar estado
add_status = Añadir estado

## About and updates

about_tagline = Un gestor de todo.txt para el escritorio.
about_version = Versión { $version }
check_for_updates = Buscar actualizaciones al iniciar
check_for_updates_desc = Busca una versión nueva al iniciar floetask. Solo se envía la comprobación de versión.
check_now = Comprobar ahora
update_checking = Buscando actualizaciones…
update_up_to_date = floetask está actualizado.
update_available = floetask { $version } está disponible.
update_available_toast = floetask { $version } está disponible. Instálalo desde Ajustes → Acerca de.
update_install = Instalar y reiniciar
update_installing = Descargando e instalando floetask { $version }… floetask se reinicia al terminar.
update_preview_version = { $version } (vista previa)
update_preview_on_stable = Estás usando una vista previa. El canal estable la actualizará con la próxima versión; para dejarla ahora, vuelve atrás abajo.
update_channel = Canal de versiones
update_channel_desc = Vista previa también ofrece versiones candidatas: la próxima versión unos días antes, para probar novedades e informar de problemas antes que nadie. Están menos probadas. Vuelve a Estable cuando quieras.
update_channel_stable = Estable
update_channel_preview = Vista previa
rollback = Volver atrás
rollback_desc = Instala una versión anterior sobre esta, para dejar una vista previa o retroceder desde una versión con problemas. Tus archivos de tareas y ajustes se conservan.
rollback_pick = Versión anterior
rollback_install = Instalar
rollback_check_first = Busca actualizaciones para ver las versiones anteriores.
rollback_none = No hay versiones anteriores en este canal.
update_failed = La actualización falló: { $error }
updates_not_configured = Esta compilación no puede actualizarse sola. Las versiones de la página de publicaciones del proyecto sí pueden.

## Notifications

due_today = Vence hoy
due_tomorrow = Vence mañana
due_in_days = Vence en { $days } días

## Errors

error_io = No se pudo acceder a { $path }: { $message }
error_stale_line = La línea { $line } cambió desde que se leyó; recarga e inténtalo de nuevo.
error_missing_line = La línea { $line } no existe.
error_note_empty = El nombre de la nota está vacío.
error_note_character = «{ $character }» no está permitido en un nombre de nota; usa a-z en minúsculas, 0-9, «.», «_» y «-».
error_note_outside = Un nombre de nota debe quedar dentro de la carpeta de notas.
error_no_done_file = { $file } no tiene archivo de completadas en el que archivar.
error_empty_todo = La tarea está vacía.
error_config = Configuración no válida: { $details }
