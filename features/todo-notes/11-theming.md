# 11. Theming and colours (P1)

- Light and dark themes; "system" follows the OS (`Ctrl+Alt+D` toggles).
- `colors.json`-equivalent user file overriding semantic colours, theme colours, navigation colours and priority colours (A, B, C, others). Reloaded on restart (P1) or live (P2).

**iced notes:** iced 0.14 supports custom `Theme::custom(palette)` and per-widget style closures; build a floetask palette struct loaded from the colour file. Follow-system needs OS dark-mode detection (`dark-light` crate) polled or subscribed.

---

[Back to the overview](../FEATURES.md)
